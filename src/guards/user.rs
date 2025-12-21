use std::sync::Arc;

use rocket::{
    Request,
    http::Status,
    request::{FromRequest, Outcome},
};

use crate::auth;

pub struct User(Arc<auth::Session>);

impl User {
    pub fn username(&self) -> &str {
        &self.0.username
    }

    pub fn display_name(&self) -> &str {
        &self.0.display_name
    }

    pub fn permissions(&self) -> &auth::hive::HivePermissionSet {
        &self.0.permissions
    }
}

#[rocket::async_trait]
impl<'r> FromRequest<'r> for User {
    type Error = ();

    async fn from_request(req: &'r Request<'_>) -> Outcome<Self, Self::Error> {
        let result = req.local_cache(|| auth::get_current_session(req.cookies()).map(Arc::new));

        match result {
            Some(session) => Outcome::Success(User(session.clone())),
            None => Outcome::Forward(Status::Unauthorized),
        }
    }
}
