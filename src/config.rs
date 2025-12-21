use std::{net::IpAddr, path::PathBuf};

use rocket::{
    Request,
    http::Status,
    request::{FromRequest, Outcome},
};

use figment::{
    Figment,
    providers::{Env, Format, Toml},
};
use serde::Deserialize;

use crate::{auth::oidc::OidcConfig, logging::Verbosity};
use log::*;

#[derive(Deserialize, Debug)]
pub struct Config {
    #[serde(default = "defaults::listen_addr")]
    pub listen_addr: IpAddr,

    #[serde(default = "defaults::port")]
    pub port: u16,

    #[serde(default = "defaults::verbosity")]
    pub verbosity: Verbosity,

    #[serde(default = "defaults::log_file")]
    pub log_file: PathBuf,

    // No defaults
    pub db_url: String,
    pub secret_key: String,
    pub oidc_issuer_url: String,
    pub oidc_client_id: String,
    pub oidc_client_secret: String,
}

impl Config {
    pub fn get() -> Self {
        let result = Figment::new()
            .merge(Toml::file("atlas.toml"))
            .merge(Env::prefixed("ATLAS_"))
            .extract();

        match result {
            Ok(config) => config,
            Err(errors) => {
                for error in errors {
                    eprintln!("Fatal configuration error: {error}");
                }
                panic!("Failed to load a valid configuration");
            }
        }
    }

    pub fn get_rocket_config(&self) -> rocket::Config {
        let secret_key =
            hex::decode(&self.secret_key).expect("Fatal error: secret key is invalid hex sequence");

        if secret_key.len() != 64 {
            panic!(
                "Fatal error: secret key has incorrect length. Use, e.g., `openssl rand -hex 64` \
                 to generate"
            )
        }

        let ident = rocket::config::Ident::try_new("atlas").unwrap();

        rocket::Config {
            address: self.listen_addr,
            port: self.port,
            secret_key: rocket::config::SecretKey::from(&secret_key),
            ident,
            ..Default::default()
        }
    }

    pub fn get_oidc_config(&self) -> OidcConfig {
        OidcConfig {
            issuer_url: self.oidc_issuer_url.clone(),
            client_id: self.oidc_client_id.clone(),
            client_secret: self.oidc_client_secret.clone(),
        }
    }
}

#[rocket::async_trait]
impl<'r> FromRequest<'r> for &'r Config {
    type Error = ();

    async fn from_request(req: &'r Request<'_>) -> Outcome<Self, Self::Error> {
        match req.rocket().state::<Config>() {
            Some(config) => Outcome::Success(config),
            None => {
                error!("trying to retrieve config as rocket state without managing it");
                Outcome::Error((Status::InternalServerError, ()))
            }
        }
    }
}

mod defaults {
    use std::{
        net::{IpAddr, Ipv4Addr},
        path::PathBuf,
    };

    use crate::logging::Verbosity;

    pub const fn listen_addr() -> IpAddr {
        // 0.0.0.0
        IpAddr::V4(Ipv4Addr::UNSPECIFIED)
    }

    pub const fn port() -> u16 {
        6767
    }

    pub const fn verbosity() -> Verbosity {
        Verbosity::Normal
    }

    pub fn log_file() -> PathBuf {
        PathBuf::from("/tmp/atlas.log")
    }
}
