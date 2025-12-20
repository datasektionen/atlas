use askama::Template;
use rocket::{State, response::content::RawHtml};

use crate::{config::Config, errors::AppResult, routing::RouteTree};

type RenderedTemplate = RawHtml<String>;

pub fn tree() -> RouteTree {
    RouteTree::Branch(vec![rocket::routes![index].into()])
}

#[derive(Template)]
#[template(path = "index.html.j2")]
struct IndexView<'a> {
    name: &'a str,
}

#[rocket::get("/")]
fn index(config: &State<Config>) -> AppResult<RenderedTemplate> {
    let template = IndexView { name: &config.bruh };

    Ok(RawHtml(template.render()?))
}
