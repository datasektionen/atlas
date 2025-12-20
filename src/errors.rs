use std::io::Cursor;

use log::*;
use rocket::{
    Request, Response,
    http::Status,
    response::{self, Responder},
};

pub type AppResult<T> = Result<T, AppError>;

#[derive(thiserror::Error, Debug)]
pub enum AppError {
    #[error("template render error: {0}")]
    RenderError(#[from] askama::Error),
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
