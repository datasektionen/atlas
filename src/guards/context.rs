use rocket::{
    Request,
    request::{FromRequest, Outcome},
};

use super::user::User;

pub struct PageContext {
    pub user: Option<User>,
}

#[rocket::async_trait]
impl<'r> FromRequest<'r> for PageContext {
    type Error = ();

    async fn from_request(req: &'r Request<'_>) -> Outcome<Self, Self::Error> {
        let user = req.guard::<User>().await.succeeded();

        Outcome::Success(Self { user })
    }
}
