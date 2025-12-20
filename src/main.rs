use log::*;
use sqlx::PgPool;

mod config;
mod errors;
mod logging;
mod routing;
mod web;

#[rocket::launch]
async fn rocket() -> _ {
    let config = config::Config::get();

    logging::init_logger(config.verbosity, &config.log_file).expect("Failed to initialize logging");

    debug!("{config:?}");

    let db = PgPool::connect(&config.db_url)
        .await
        .expect("Failed to connect to database");

    rocket::custom(config.get_rocket_config())
        .manage(db)
        .manage(config)
        .mount("/", &web::tree())
}
