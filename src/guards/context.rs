use std::{borrow::Cow, fmt};

use rocket::{
    Request,
    request::{FromRequest, Outcome},
};

use super::{lang::Language, user::User};

use crate::splash::Splashes;

pub struct PageContext {
    pub user: Option<User>,
    pub lang: Language,
    pub splashes: Splashes,
}

impl PageContext {
    // Shorthand for i18n in templates
    pub fn t<'a>(&self, key: &'a str) -> Cow<'a, str> {
        self.lang.t(key)
    }

    // Templates don't support macros and therefore variable arguments, hence this function
    pub fn t1<'a, T: fmt::Display>(&self, key: &'a str, x: T) -> Cow<'a, str> {
        self.lang.t1(key, x)
    }

    // Shorthand to generate splash text
    pub fn splash(&self) -> &String {
        self.splashes.choose()
    }
}

#[rocket::async_trait]
impl<'r> FromRequest<'r> for PageContext {
    type Error = ();

    async fn from_request(req: &'r Request<'_>) -> Outcome<Self, Self::Error> {
        let user = req.guard::<User>().await.succeeded();
        let lang = req.guard::<Language>().await.unwrap();
        // Cloning an Arc is cheap
        let splashes = req.rocket().state::<Splashes>().unwrap().clone();

        Outcome::Success(Self {
            user,
            lang,
            splashes,
        })
    }
}
