use askama::Template;
pub use catchers::catchers;
use log::*;
use rocket::response::content::RawHtml;

use crate::{
    auth::hive::HivePermission, errors::AppResult, guards::context::PageContext, routing::RouteTree,
};

mod auth;
mod catchers;

type RenderedTemplate = RawHtml<String>;

pub fn tree() -> RouteTree {
    RouteTree::Branch(vec![
        auth::routes(),
        rocket::routes![index, subscribe, post].into(),
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
fn subscribe(ctx: PageContext) -> AppResult<RenderedTemplate> {
    let template = SubscribeView { ctx };

    Ok(RawHtml(template.render()?))
}

#[rocket::get("/post")]
fn post(ctx: PageContext) -> AppResult<RenderedTemplate> {
    ctx.perms()?.require(HivePermission::Post)?;

    let template = IndexView { ctx };

    Ok(RawHtml(template.render()?))
}
