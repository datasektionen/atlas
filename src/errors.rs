use std::io::Cursor;

use askama::Template;
use log::*;
use rocket::{
    Request, Response,
    http::Status,
    response::{self, Responder},
};

use crate::{
    auth::{hive::HivePermission, oidc::OidcAuthenticationError},
    guards::context::PageContext,
};

pub type AppResult<T> = Result<T, AppError>;

#[derive(thiserror::Error, Debug)]
pub enum AppError {
    #[error("template render error: {0}")]
    RenderError(#[from] askama::Error),
    // not for login failures! just 500
    #[error("internal OIDC authentication error: {0}")]
    OidcAuthenticationError(#[from] OidcAuthenticationError),
    #[error("failed to serialize internal state for storage: {0}")]
    StateSerializationError(#[source] serde_json::Error),
    #[error("failed to deserialize internal state from secure storage: {0}")]
    StateDeserializationError(#[source] serde_json::Error),
    #[error("user lacks permissions to perform action (minimum needed: {0})")]
    NotAllowed(HivePermission),
    #[error("user was not authenticated when required")]
    NotAuthenticated,
    #[error("authentication flow expired and can no longer be completed")]
    AuthenticationFlowExpired,
}

impl AppError {
    fn status(&self) -> Status {
        match self {
            Self::RenderError(..) => Status::InternalServerError,
            Self::OidcAuthenticationError(..) => Status::InternalServerError,
            Self::StateSerializationError(..) => Status::InternalServerError,
            Self::StateDeserializationError(..) => Status::InternalServerError,
            Self::NotAllowed(..) => Status::Forbidden,
            Self::NotAuthenticated => Status::Unauthorized,
            Self::AuthenticationFlowExpired => Status::Gone,
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

        // TODO: return error information using JSON
        let body = "An error occured";
        Ok(Response::build()
            .sized_body(body.len(), Cursor::new(body))
            .status(status)
            .finalize())
    }
}

#[derive(Template)]
#[template(path = "errors/error.html.j2")]
struct ErrorPageView {
    ctx: PageContext,
    title: String,
    description: String,
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
        title,
        description,
    };

    template.render().unwrap_or_else(|e| {
        error!("Failed to render error page: {e}");

        status.reason_lossy().to_owned()
    })
}
