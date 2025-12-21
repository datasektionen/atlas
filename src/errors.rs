use std::io::Cursor;

use log::*;
use rocket::{
    Request, Response,
    http::Status,
    response::{self, Responder},
};

use crate::auth::oidc::OidcAuthenticationError;

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

    #[error("authentication flow expired and can no longer be completed")]
    AuthenticationFlowExpired,
}

impl AppError {
    fn status(&self) -> Status {
        match self {
            _ => Status::InternalServerError,
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
