use rocket::{
    Catcher, Request, Responder, catchers,
    http::{Method, Status},
    response::{Redirect, content::RawHtml},
    uri,
};

use super::RenderedTemplate;
use crate::{errors::render_error_page, guards::context::PageContext};

pub fn catchers() -> Vec<Catcher> {
    catchers![not_found, unauthenticated, unknown]
}

#[derive(Responder)]
pub enum Caught {
    Error(RenderedTemplate),
}

macro_rules! show_error_page {
    ($name:ident, $num:expr, $status:expr, $i18n_key:expr) => {
        #[rocket::catch($num)]
        async fn $name(req: &Request<'_>) -> Caught {
            let ctx = req.guard::<PageContext>().await.succeeded().unwrap();

            let title = ctx.t(concat!("errors.caught.", $i18n_key, ".title"));
            let description = ctx.t(concat!("errors.caught.", $i18n_key, ".description"));
            let html = render_error_page(title, description, $status, ctx);

            Caught::Error(RawHtml(html))
        }
    };
}

show_error_page!(not_found, 404, Status::NotFound, "not-found");
show_error_page!(unknown, default, Status::InternalServerError, "unknown");

#[rocket::catch(401)]
fn unauthenticated(req: &Request<'_>) -> Redirect {
    let next = if req.method() == Method::Get {
        // Redirect back after login
        Some(req.uri().to_string())
    } else {
        None
    };

    Redirect::to(uri!(super::auth::login(next)))
}
