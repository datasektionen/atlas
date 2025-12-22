use auth::oidc::OidcClient;
use log::*;
use rocket::fs::FileServer;
use sqlx::PgPool;

mod auth;
mod config;
mod errors;
mod guards;
mod logging;
mod routing;
mod web;

rust_i18n::i18n!("./locales");

#[rocket::launch]
async fn rocket() -> _ {
    let config = config::Config::get();

    logging::init_logger(config.verbosity, &config.log_file).expect("Failed to initialize logging");

    debug!("{config:?}");

    let db = PgPool::connect(&config.db_url)
        .await
        .expect("Failed to connect to database");

    debug!("Initialized database connection pool");

    debug!(
        "Available i18n locales: {:?}",
        rust_i18n::available_locales!()
    );

    let oidc_client = OidcClient::new(config.get_oidc_config())
        .await
        .expect("Failed to initialize OIDC");

    rocket::custom(config.get_rocket_config())
        .manage(db)
        .manage(config)
        .manage(oidc_client)
        .mount("/static", FileServer::from("./static"))
        .mount("/", &web::tree())
}
