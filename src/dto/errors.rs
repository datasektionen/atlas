use std::borrow::Cow;

use serde::{Deserialize, Serialize};

use crate::{errors::AppError, guards::lang::Language};

#[derive(Serialize, Deserialize)]
#[serde(tag = "key", content = "context")]
enum InnerAppErrorDto {
    // Anything related to handling requests/responses (500)
    #[serde(rename = "pipeline")]
    PipelineError,
    #[serde(rename = "forbidden")]
    NotAllowed,
    #[serde(rename = "auth.login.flow.expired")]
    AuthenticationFlowExpired,
}

impl From<AppError> for InnerAppErrorDto {
    fn from(err: AppError) -> Self {
        match err {
            AppError::RenderError(..) => Self::PipelineError,
            AppError::OidcAuthenticationError(..) => Self::PipelineError,
            AppError::StateSerializationError(..) => Self::PipelineError,
            AppError::StateDeserializationError(..) => Self::PipelineError,
            AppError::ErrorDecodeFailure => Self::PipelineError,
            AppError::ExternalConnectionError => Self::PipelineError,
            AppError::AuthenticationFlowExpired => Self::AuthenticationFlowExpired,
            AppError::NotAllowed(..) => Self::NotAllowed,
            AppError::NotAuthenticated => Self::NotAllowed,
        }
    }
}

// This provides an exhaustiveness check so you never forget to translate a newly added error.
impl InnerAppErrorDto {
    fn title<'a>(&'a self, lang: &Language) -> Cow<'a, str> {
        match self {
            Self::PipelineError => lang.t("errors.dto.pipeline.title"),
            Self::NotAllowed => lang.t("errors.dto.forbidden.title"),
            Self::AuthenticationFlowExpired => lang.t("errors.dto.auth.login.flow.expired.title"),
        }
    }

    fn description(&self, lang: &Language) -> String {
        match self {
            Self::PipelineError => lang.t("errors.dto.pipeline.description").to_string(),
            Self::NotAllowed => lang.t("errors.dto.forbidden.description").to_string(),
            Self::AuthenticationFlowExpired => lang
                .t("errors.dto.auth.login.flow.expired.description")
                .to_string(),
        }
    }
}

#[derive(Serialize, Deserialize)]
pub struct AppErrorDto {
    error: bool,
    info: InnerAppErrorDto,
}

impl From<AppError> for AppErrorDto {
    fn from(err: AppError) -> Self {
        Self {
            error: true,
            info: err.into(),
        }
    }
}

impl AppErrorDto {
    pub fn title<'a>(&'a self, lang: &Language) -> Cow<'a, str> {
        self.info.title(lang)
    }

    pub fn description(&self, lang: &Language) -> String {
        self.info.description(lang)
    }
}
