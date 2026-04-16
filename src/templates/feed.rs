use crate::filters;
use crate::templates::{event, index, misc};

#[derive(askama::Template, Debug, askama_web::WebTemplate)]
#[template(path = "page/feed.html")]
pub struct FeedPage {
    pub ctx: index::PageContext,
    pub feed: Vec<event::Event>,
    pub tags: Vec<String>,
}
