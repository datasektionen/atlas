use std::{borrow::Cow, fmt};

use rocket::{
    Request,
    request::{FromRequest, Outcome},
};

use super::{lang::Language, user::User};

pub struct PageContext {
    pub user: Option<User>,
    pub lang: Language,
}

// Shorthand for i18n in templates
impl PageContext {
    pub fn t<'a>(&self, key: &'a str) -> Cow<'a, str> {
        self.lang.t(key)
    }

    pub fn t1<'a, T: fmt::Display>(&self, key: &'a str, x: T) -> Cow<'a, str> {
        self.lang.t1(key, x)
    }
}

#[rocket::async_trait]
impl<'r> FromRequest<'r> for PageContext {
    type Error = ();

    async fn from_request(req: &'r Request<'_>) -> Outcome<Self, Self::Error> {
        let user = req.guard::<User>().await.succeeded();
        let lang = req.guard::<Language>().await.unwrap();

        Outcome::Success(Self { user, lang })
    }
}
