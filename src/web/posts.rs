use askama::Template;
use log::*;
use rocket::{
    State,
    form::{self, Contextual, Form},
    response::{Redirect, content::RawHtml},
    uri,
};
use sqlx::PgPool;

use crate::{
    auth::hive::HivePermission,
    dto::posts::EditPostDto,
    errors::AppResult,
    guards::{context::PageContext, user::User},
    models::Post,
    routing::RouteTree,
    services::posts,
};

use super::{Either, RenderedTemplate};

pub fn routes() -> RouteTree {
    rocket::routes![
        posts_list,
        post_edit_new,
        post_create_new,
        post_view,
        post_edit,
        post_update
    ]
    .into()
}

#[derive(Template)]
#[template(path = "posts/edit.html.j2")]
struct PostEditView<'f, 'v> {
    ctx: PageContext,
    post_form: &'f form::Context<'v>,
    post: Option<Post>,
}

#[rocket::get("/posts")]
fn posts_list(ctx: PageContext) -> AppResult<RenderedTemplate> {
    Ok(RawHtml("temp".to_string()))
}

#[rocket::get("/posts?edit")]
fn post_edit_new(ctx: PageContext) -> AppResult<RenderedTemplate> {
    ctx.perms()?.require(HivePermission::Post)?;

    let template = PostEditView {
        ctx,
        post_form: &form::Context::default(),
        post: None,
    };

    Ok(RawHtml(template.render()?))
}

#[rocket::post("/posts", data = "<form>")]
async fn post_create_new<'v>(
    form: Form<Contextual<'v, EditPostDto<'v>>>,
    ctx: PageContext,
    db: &State<PgPool>,
    user: User,
) -> AppResult<Either<Redirect, RenderedTemplate>> {
    ctx.perms()?.require(HivePermission::Post)?;

    if let Some(dto) = &form.value {
        // validation passed

        // TODO: validate mandate

        let post_id = posts::create(dto, db.inner(), &user).await?;

        // TODO: redirect to edit page only on draft save, otherwise view page
        Ok(Either::Left(Redirect::to(uri!(post_edit(id = post_id)))))
    } else {
        // validation failed, so show the form again
        debug!("Create post form errors: {:?}", &form.context);

        let template = PostEditView {
            ctx,
            post_form: &form.context,
            post: None,
        };

        Ok(Either::Right(RawHtml(template.render()?)))
    }
}

#[rocket::get("/posts/<id>")]
fn post_view(id: i64, ctx: PageContext) -> AppResult<RenderedTemplate> {
    Ok(RawHtml("temp".to_string()))
}

#[rocket::get("/posts/<id>?edit")]
async fn post_edit(id: i64, ctx: PageContext, db: &State<PgPool>) -> AppResult<RenderedTemplate> {
    ctx.perms()?.require(HivePermission::Post)?;
    // TODO: check mandate and such for permission

    let post = posts::require_one(id, db.inner()).await?;

    let template = PostEditView {
        ctx,
        post_form: &form::Context::default(),
        post: Some(post),
    };

    Ok(RawHtml(template.render()?))
}

#[rocket::patch("/posts/<id>", data = "<form>")]
fn post_update<'v>(
    id: i64,
    form: Form<Contextual<'v, EditPostDto<'v>>>,
    ctx: PageContext,
    db: &State<PgPool>,
) -> AppResult<RenderedTemplate> {
    // TODO: check permission to edit for mandate

    Ok(RawHtml("temp".to_string()))
}
