use crate::filters;
use chrono::{DateTime, Local};

use crate::templates::index;

#[derive(askama::Template, Debug, askama_web::WebTemplate)]
#[template(path = "page/news.html")]
pub struct NewsPage {
    pub ctx: index::PageContext,
    pub news: News,
}

#[derive(Debug)]
pub struct News {
    pub title: String,
    pub description: String,
    pub date: DateTime<Local>,
    pub tags: Vec<String>,
    pub owner: String,
}
