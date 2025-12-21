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

use crate::logging::Verbosity;
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

    pub bruh: String,

    pub db_url: String,
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
        let ident = rocket::config::Ident::try_new("atlas").unwrap();

        rocket::Config {
            address: self.listen_addr,
            port: self.port,
            ident,
            ..Default::default()
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
