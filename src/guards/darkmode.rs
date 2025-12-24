use log::*;
use rocket::{
    Request,
    http::Status,
    request::{FromRequest, Outcome},
};

use crate::{
    config::Config,
    errors::{AppError, AppResult},
};

pub struct Darkmode(bool);

impl Darkmode {
    pub fn on(&self) -> bool {
        self.0
    }
}

async fn darkmode_ask<U>(url: &U) -> AppResult<bool>
where
    for<'u> &'u U: reqwest::IntoUrl,
{
    let answer = reqwest::get(url)
        .await
        .inspect_err(|e| error!("Darkmode connection error: {e:?}"))
        .map_err(|_| AppError::ExternalConnectionError)?
        .json::<bool>()
        .await
        .inspect_err(|e| error!("Darkmode response error: {e:?}"))
        .map_err(|_| AppError::ExternalConnectionError)?;
    Ok(answer)
}

#[rocket::async_trait]
impl<'r> FromRequest<'r> for Darkmode {
    type Error = AppError;

    async fn from_request(req: &'r Request<'_>) -> Outcome<Self, Self::Error> {
        let config = req.guard::<&Config>().await.expect("infallible");
        match config.darkmode_url.as_str() {
            "yes" => Outcome::Success(Self(true)),
            "no" => Outcome::Success(Self(false)),
            _ => match darkmode_ask(&config.darkmode_url).await {
                Ok(answer) => Outcome::Success(Self(answer)),
                Err(e) => Outcome::Error((Status::InternalServerError, e)),
            },
        }
    }
}
