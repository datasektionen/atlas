use askama::Template;
pub use catchers::catchers;
use log::*;
use rocket::{Responder, response::content::RawHtml};

use crate::{errors::AppResult, guards::context::PageContext, routing::RouteTree};

mod auth;
mod catchers;
mod posts;

type RenderedTemplate = RawHtml<String>;

#[derive(Responder)]
enum Either<L, R> {
    Left(L),
    Right(R),
}

pub fn tree() -> RouteTree {
    RouteTree::Branch(vec![
        auth::routes(),
        posts::routes(),
        rocket::routes![index, subscribe].into(),
    ])
}

#[derive(Template)]
#[template(path = "index.html.j2")]
struct IndexView {
    ctx: PageContext,
}

#[rocket::get("/")]
fn index(ctx: PageContext) -> AppResult<RenderedTemplate> {
    if let Some(ref user) = ctx.user {
        debug!("Permissions: {:?}", user.permissions());
        debug!("Groups: {:?}", user.groups());
    }

    let template = IndexView { ctx };

    Ok(RawHtml(template.render()?))
}

#[derive(Template)]
#[template(path = "subscribe.html.j2")]
struct SubscribeView {
    ctx: PageContext,
}

#[rocket::get("/subscribe")]
fn subscribe(
    ctx: PageContext,
    darkmode: crate::guards::darkmode::Darkmode,
) -> AppResult<RenderedTemplate> {
    let template = SubscribeView { ctx };

    info!("darkmode: {}", darkmode.on());

    Ok(RawHtml(template.render()?))
}
