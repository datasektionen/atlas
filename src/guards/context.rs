use std::{borrow::Cow, fmt};

use rocket::{
    Request,
    request::{FromRequest, Outcome},
};

use super::{lang::Language, nav::Nav, user::User};

use crate::{
    auth::hive::HivePermissionSet,
    errors::{AppError, AppResult},
    splash::Splashes,
};

pub struct PageContext {
    pub user: Option<User>,
    pub lang: Language,
    pub splashes: Splashes,
    pub nav: Nav,
}

impl PageContext {
    // Shorthand for i18n in templates
    pub fn t<'a>(&self, key: &'a str) -> Cow<'a, str> {
        self.lang.t(key)
    }

    // Templates don't support macros and therefore variable arguments, hence these functions
    pub fn t1<'a, T: fmt::Display>(&self, key: &'a str, x: T) -> Cow<'a, str> {
        self.lang.t1(key, x)
    }

    pub fn t2<'a, T, U>(&self, key: &'a str, x: T, y: U) -> Cow<'a, str>
    where
        T: fmt::Display,
        U: fmt::Display,
    {
        self.lang.t2(key, x, y)
    }

    // Shorthand to generate splash text
    pub fn splash(&self) -> &String {
        self.splashes.choose()
    }

    // Shorthand to get permission set
    pub fn perms(&self) -> AppResult<&HivePermissionSet> {
        if let Some(user) = &self.user {
            Ok(user.permissions())
        } else {
            Err(AppError::NotAuthenticated)
        }
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
        let nav = req.guard::<Nav>().await.unwrap();

        Outcome::Success(Self {
            user,
            lang,
            splashes,
            nav,
        })
    }
}
