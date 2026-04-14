use num_traits::cast::FromPrimitive;
mod filters;
mod templates;

use crate::templates::{
    calendar,
    index::{MainPage, PageContext},
    misc,
};

use std::env;

use actix_web::{
    App, HttpRequest, HttpServer, Responder, get, middleware,
    web::{self, Html},
};
use askama::Template;
use chrono::{Datelike, Local, Month};
use sqlx::postgres::PgPoolOptions;
use tracing_subscriber::EnvFilter;

#[get("/calendar")]
async fn calendar_page(req: HttpRequest) -> impl Responder {
    let user = Some(templates::index::User {
        name: "Oskar".to_string(),
        avatar_url: "https://dsekt-assets.s3.eu-west-1.amazonaws.com/shield-color-white-delta.png"
            .to_string(),
    });

    let ctx = PageContext::new(req.path(), user);
    let now = Local::now();
    let month = Month::from_u32(now.month()).unwrap();
    Html::new(
        calendar::CalendarPage {
            ctx,
            cal: calendar::Calendar::new(month, now.year()),
        }
        .render()
        .unwrap(),
    )
}

#[get("/event/{id}")]
async fn event_page(id: web::Path<u64>, req: HttpRequest) -> impl Responder {
    let user = Some(templates::index::User {
        name: "Oskar".to_string(),
        avatar_url: "https://dsekt-assets.s3.eu-west-1.amazonaws.com/shield-color-white-delta.png"
            .to_string(),
    });

    let ctx = PageContext::new(req.path(), user);
    let now = Local::now();
    Html::new(
        templates::event::EventPage {
            ctx,
            event: templates::event::Event {
                title: format!("Event {}", id),
                description: "This `is` **the** description of the _news_. Go here for more: https://datasektionen.se".to_string(),
                from: now,
                to: now + chrono::Duration::seconds(3600),
                tags: vec!["tag1".to_string(), "tag2".to_string()],
                owner: "Oskar".to_string(),
                place: Some("META".to_string()),
                link: Some("https://datasektionen.se".to_string()),
            },
        }
        .render()
        .unwrap(),
    )
}

#[get("/news/{id}")]
async fn news_page(id: web::Path<u64>, req: HttpRequest) -> impl Responder {
    let user = Some(templates::index::User {
        name: "Oskar".to_string(),
        avatar_url: "https://dsekt-assets.s3.eu-west-1.amazonaws.com/shield-color-white-delta.png"
            .to_string(),
    });

    let ctx = PageContext::new(req.path(), user);
    let now = Local::now();
    Html::new(
        templates::news::NewsPage {
            ctx,
            news: templates::news::News {
                title: format!("News {}", id),
                description: "This `is` **the** description of the _news_. Go here for more: https://datasektionen.se".to_string(),
                date: now - chrono::Duration::seconds(3600),
                tags: vec!["tag1".to_string(), "tag2".to_string()],
                owner: "Oskar".to_string(),
            },
        }
        .render()
        .unwrap(),
    )
}

#[get("/")]
async fn index(req: HttpRequest) -> impl Responder {
    let user = Some(templates::index::User {
        name: "Oskar".to_string(),
        avatar_url: "https://dsekt-assets.s3.eu-west-1.amazonaws.com/shield-color-white-delta.png"
            .to_string(),
    });
    let ctx = PageContext::new(req.path(), user);
    let now = Local::now();
    let cards = vec![
        misc::NewsCard {
            id: 1,
            title: "First news".to_string(),
            summary: "## This \n this is the **summary** of the first news. Hello, world!\n ### hello \n *hello* fwdijn awdnjwdl andla iwdajwnda wljdn alwdkjn awdjn awldkjnwa ldkjn.()(aa".to_string(),
            date: now - chrono::Duration::seconds(3600),
            tags: vec!["tag1".to_string(), "tag2".to_string()],
            owner: "Oskar".to_string(),
        },
        misc::NewsCard {
            id: 2,
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
            .service(calendar_page)
            .service(news_page)
            .service(event_page)
            .service(actix_files::Files::new("/static", "./static"))
    })
    .bind(("0.0.0.0", 8080))?
    .run()
    .await
}
