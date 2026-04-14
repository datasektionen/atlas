use crate::filters;
use crate::templates::{calendar, misc};
use chrono::Datelike;

#[derive(askama::Template, Debug, askama_web::WebTemplate)]
#[template(path = "page/index.html")]
pub struct MainPage {
    pub ctx: PageContext,
    pub news_cards: Vec<misc::NewsCard>,
}

#[derive(Debug)]
pub struct Crumb {
    pub name: String,
    pub url: String,
}

#[derive(Debug)]
pub struct PageContext {
    pub user: Option<User>,
    pub crumbs: Vec<Crumb>,
}

impl PageContext {
    pub fn new(url: &str, user: Option<User>) -> Self {
        let mut crumbs = vec![Crumb {
            name: "home".to_string(),
            url: "/".to_string(),
        }];
        url.trim().split("/").fold(String::new(), |mut s, v| {
            if !v.is_empty() {
                s += format!("/{}", v).as_str();
                let c = Crumb {
                    name: v.to_string(),
                    url: s.clone(),
                };
                crumbs.push(c);
            }
            s
        });
        Self { user, crumbs }
    }
}

#[derive(Debug)]
pub struct User {
    pub name: String,
    pub avatar_url: String,
}
