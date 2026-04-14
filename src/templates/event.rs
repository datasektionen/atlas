use crate::filters;
use chrono::{DateTime, Local};

use crate::templates::index;

#[derive(askama::Template, Debug, askama_web::WebTemplate)]
#[template(path = "page/event.html")]
pub struct EventPage {
    pub ctx: index::PageContext,
    pub event: Event,
}

#[derive(Debug)]
pub struct Event {
    pub title: String,
    pub description: String,
    pub from: DateTime<Local>,
    pub to: DateTime<Local>,
    pub tags: Vec<String>,
    pub owner: String,
    pub place: Option<String>,
    pub link: Option<String>,
}
