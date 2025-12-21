use askama::Template;
use rocket::response::content::RawHtml;

use crate::{errors::AppResult, guards::context::PageContext, routing::RouteTree};

mod auth;

type RenderedTemplate = RawHtml<String>;

pub fn tree() -> RouteTree {
    RouteTree::Branch(vec![auth::routes(), rocket::routes![index].into()])
}

#[derive(Template)]
#[template(path = "index.html.j2")]
struct IndexView {
    ctx: PageContext,
}

#[rocket::get("/")]
fn index(ctx: PageContext) -> AppResult<RenderedTemplate> {
    let template = IndexView { ctx };

    Ok(RawHtml(template.render()?))
}
