mod filters;
mod templates;

use crate::templates::{
    calendar,
    index::{CalendarPage, Crumb, MainPage, PageContext},
    misc,
};

use std::env;

use actix_web::{
    App, HttpServer, Responder, get, middleware,
    web::{self, Html},
};
use askama::Template;
use chrono::{Datelike, Local, Month};
use sqlx::postgres::PgPoolOptions;
use tracing_subscriber::EnvFilter;

#[get("/calendar")]
async fn calendarPage() -> impl Responder {
    let crumbs = vec![
        Crumb {
            name: "Home".to_string(),
            url: "/".to_string(),
        },
        Crumb {
            name: "Calendar".to_string(),
            url: "/calendar".to_string(),
        },
    ];
    let user = Some(templates::index::User {
        name: "Oskar".to_string(),
        avatar_url: "https://dsekt-assets.s3.eu-west-1.amazonaws.com/shield-color-white-delta.png"
            .to_string(),
    });

    let ctx = PageContext { user, crumbs };
    let now = Local::now().to_utc();
    Html::new(
        CalendarPage {
            ctx,
            cal: calendar::Calendar::new(Month::January, now.year()),
        }
        .render()
        .unwrap(),
    )
}

#[get("/")]
async fn index() -> impl Responder {
    let crumb = Crumb {
        name: "Home".to_string(),
        url: "/".to_string(),
    };
    let user = Some(templates::index::User {
        name: "Oskar".to_string(),
        avatar_url: "https://dsekt-assets.s3.eu-west-1.amazonaws.com/shield-color-white-delta.png"
            .to_string(),
    });
    let ctx = PageContext {
        user,
        crumbs: vec![crumb],
    };
    let now = Local::now();
    let cards = vec![
        misc::NewsCard {
            title: "First news".to_string(),
            summary: "This is the summary of the first news".to_string(),
            date: now - chrono::Duration::seconds(3600),
            tags: vec!["tag1".to_string(), "tag2".to_string()],
            owner: "Oskar".to_string(),
        },
        misc::NewsCard {
            title: "Second news".to_string(),
            summary: "This is the summary of the second news".to_string(),
            date: now - chrono::Duration::days(35),
            tags: vec!["tag3".to_string(), "tag4".to_string()],
            owner: "Oskar".to_string(),
        },
    ];
    // render index template
    Html::new(
        MainPage {
            ctx,
            news_cards: cards,
        }
        .render()
        .unwrap(),
    )
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    tracing_subscriber::fmt()
        .with_env_filter(EnvFilter::default())
        .init();

    let pool = web::Data::new(
        PgPoolOptions::new()
            .connect(&env::var("DATABASE_URL").expect("DATABASE_URL to exist"))
            .await
            .expect("Expected to connect to database"),
    );

    sqlx::migrate!("./migrations")
        .run(pool.get_ref())
        .await
        .expect("migrations to run");

    HttpServer::new(|| {
        App::new()
            .wrap(middleware::Compress::default())
            .wrap(middleware::Logger::default())
            .service(index)
            .service(calendarPage)
            .service(actix_files::Files::new("/static", "./static"))
    })
    .bind(("0.0.0.0", 8080))?
    .run()
    .await
}
