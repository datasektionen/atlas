use std::io::Cursor;

use askama::Template;
use log::*;
use rocket::{
    Request, Response,
    fairing::{self, Fairing},
    http::{ContentType, Status},
    response::{self, Responder},
    serde::json::Json,
};

use crate::{
    auth::{hive::HivePermission, oidc::OidcAuthenticationError},
    dto::errors::AppErrorDto,
    guards::context::PageContext,
};

pub type AppResult<T> = Result<T, AppError>;

#[derive(thiserror::Error, Debug)]
pub enum AppError {
    #[error("database error: {0}")]
    DbError(#[from] sqlx::Error),
    #[error("query building error: {0}")]
    QueryBuildError(#[source] sqlx::error::BoxDynError),
    #[error("template render error: {0}")]
    RenderError(#[from] askama::Error),

    // not for login failures! just 500
    #[error("internal OIDC authentication error: {0}")]
    OidcAuthenticationError(#[from] OidcAuthenticationError),
    #[error("failed to serialize internal state for storage: {0}")]
    StateSerializationError(#[source] serde_json::Error),
    #[error("failed to deserialize internal state from secure storage: {0}")]
    StateDeserializationError(#[source] serde_json::Error),
    #[error("authentication flow expired and can no longer be completed")]
    AuthenticationFlowExpired,

    #[error("user lacks permissions to perform action (minimum needed: {0})")]
    NotAllowed(HivePermission),
    #[error("user was not authenticated when required")]
    NotAuthenticated,

    #[error("could not find post with id '{0}'")]
    NoSuchPost(i64),

    #[error("failed to decode error while generating error page")]
    ErrorDecodeFailure,
    #[error("error while connecting to an external service")]
    ExternalConnectionError,
}

impl AppError {
    fn status(&self) -> Status {
        match self {
            Self::DbError(..) => Status::InternalServerError,
            Self::QueryBuildError(..) => Status::InternalServerError,
            Self::RenderError(..) => Status::InternalServerError,
            Self::OidcAuthenticationError(..) => Status::InternalServerError,
            Self::StateSerializationError(..) => Status::InternalServerError,
            Self::StateDeserializationError(..) => Status::InternalServerError,
            Self::AuthenticationFlowExpired => Status::Gone,
            Self::NotAllowed(..) => Status::Forbidden,
            Self::NotAuthenticated => Status::Forbidden,
            Self::NoSuchPost(..) => Status::NotFound,
            Self::ErrorDecodeFailure => Status::InternalServerError,
            Self::ExternalConnectionError => Status::InternalServerError,
        }
    }
}

impl<'r> Responder<'r, 'static> for AppError {
    fn respond_to(self, req: &'r Request<'_>) -> response::Result<'static> {
        let status = self.status();
        if status.code >= 500 {
            error!("While handling [{req}], encountered {self:?}: {self}");
        } else {
            debug!("While handling [{req}], encountered {self:?}: {self}");
        }

        let base = Json(AppErrorDto::from(self)).respond_to(req)?;

        Ok(Response::build_from(base).status(status).finalize())
    }
}

#[derive(Template)]
#[template(path = "errors/error.html.j2")]
struct ErrorPageView {
    ctx: PageContext,
    status: u16,
    title: String,
    description: String,
}

// Generates error pages for AppErrors, as the Responder trait does not allow async and can
// therefore not render an HTML page with PageContext. It instead generates a JSON response which
// gets intercepted by this fairing and, when relevant, renders it as a full page.
pub struct ErrorPageGenerator;

#[rocket::async_trait]
impl Fairing for ErrorPageGenerator {
    fn info(&self) -> fairing::Info {
        fairing::Info {
            name: "Error Page Generator",
            kind: fairing::Kind::Response,
        }
    }

    async fn on_response<'r>(&self, req: &'r Request<'_>, res: &mut Response<'r>) {
        let status_class = res.status().class();
        if !status_class.is_client_error() && !status_class.is_server_error() {
            // Ignore non-errors
            return;
        }

        if req.uri().path().starts_with("/api/") {
            // API errors should be in JSON format
            return;
        }

        if res.content_type().map(|t| t.is_html()).unwrap_or(false) {
            // Already rendered! Probably by a catcher.
            return;
        }

        let mut error = AppErrorDto::from(AppError::ErrorDecodeFailure);

        if let Ok(body) = res.body_mut().to_string().await {
            if let Ok(dto) = serde_json::from_str(&body) {
                error = dto;
            }
        }

        let ctx = req.guard::<PageContext>().await.expect("infallible");

        let title = error.title(&ctx.lang);
        let description = error.description(&ctx.lang);

        res.set_header(ContentType::HTML);

        let html = render_error_page(title, description, res.status(), ctx);
        res.set_sized_body(html.len(), Cursor::new(html));
    }
}

pub fn render_error_page<T, D>(title: T, description: D, status: Status, ctx: PageContext) -> String
where
    T: ToString,
    D: ToString,
{
    let title = title.to_string();
    let description = description.to_string();

    let template = ErrorPageView {
        ctx,
        status: status.code,
        title,
        description,
    };

    template.render().unwrap_or_else(|e| {
        error!("Failed to render error page: {e}");

        status.reason_lossy().to_owned()
    })
}
