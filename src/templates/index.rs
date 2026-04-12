use crate::filters;
use crate::templates::{calendar, misc};
use chrono::Datelike;

#[derive(askama::Template, Debug, askama_web::WebTemplate)]
#[template(path = "page/index.html")]
pub struct MainPage {
    pub ctx: PageContext,
    pub news_cards: Vec<misc::NewsCard>,
}

#[derive(askama::Template, Debug, askama_web::WebTemplate)]
#[template(path = "page/calendar.html")]
pub struct CalendarPage {
    pub ctx: PageContext,
    pub cal: calendar::Calendar,
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

#[derive(Debug)]
pub struct User {
    pub name: String,
    pub avatar_url: String,
}
