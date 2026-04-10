use actix_web::{App, HttpResponse, HttpServer, Responder, get, middleware, web::Html};
use askama::Template;
use chrono::Local;
use tracing_subscriber::EnvFilter;

#[derive(askama::Template, Debug, askama_web::WebTemplate)]
#[template(path = "index.html")]
struct MainPage {
    name: String,
}

#[get("/")]
async fn index() -> impl Responder {
    // render index template
    Html::new(
        MainPage {
            name: Local::now().to_string(),
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
    HttpServer::new(|| {
        App::new()
            .wrap(middleware::Compress::default())
            .wrap(middleware::Logger::default())
            .service(index)
    })
    .bind(("0.0.0.0", 8080))?
    .run()
    .await
}
