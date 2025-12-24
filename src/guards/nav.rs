use rocket::{
    Request,
    request::{FromRequest, Outcome},
};

use super::user::User;
use crate::auth::hive::HivePermission;

pub struct Nav {
    pub links: Vec<NavLink>,
}

pub struct NavLink {
    pub key: &'static str,
    pub href: &'static str,
}

impl NavLink {
    fn new(key: &'static str, href: &'static str) -> Self {
        Self { key, href }
    }
}

#[rocket::async_trait]
impl<'r> FromRequest<'r> for Nav {
    type Error = ();

    async fn from_request(req: &'r Request<'_>) -> Outcome<Self, Self::Error> {
        let mut links = vec![NavLink::new("subscribe", "/subscribe")];

        if let Outcome::Success(user) = req.guard::<User>().await {
            let perms = user.permissions();

            if perms.has(&HivePermission::Post) {
                links.push(NavLink::new("post", "/post"));
            }
        }

        Outcome::Success(Self { links })
    }
}
