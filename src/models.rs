//use chrono::{DateTime, Local};
use sqlx::FromRow;

use crate::{dto::datetime::BrowserDateTimeDto, guards::lang::Language};

#[derive(FromRow)]
pub struct PostId {
    pub id: i64,
}

#[derive(FromRow)]
pub struct Post {
    pub id: i64,
    pub darkmode_hide: bool,
    pub published: bool,
    pub publish_time: BrowserDateTimeDto,
    pub edit_time: BrowserDateTimeDto,
    pub author: String,
    pub mandate: Option<String>,
    pub title_sv: String,
    pub title_en: String,
    pub content_sv: String,
    pub content_en: String,
    pub banner: Option<String>,
}

impl Post {
    pub fn localized_title(&self, lang: &Language) -> &str {
        match lang {
            Language::Swedish => &self.title_sv,
            Language::English => &self.title_en,
        }
    }

    pub fn localized_content(&self, lang: &Language) -> &str {
        match lang {
            Language::Swedish => &self.content_sv,
            Language::English => &self.content_en,
        }
    }
}

// For when loading only displayable info from a post. Other data is meant to have already been
// taken into occasion when querying.
#[derive(FromRow)]
pub struct PostInfo {
    pub publish_time: BrowserDateTimeDto,
    pub author: String,
    pub mandate: Option<String>,
    pub title_sv: String,
    pub title_en: String,
    pub content_sv: String,
    pub content_en: String,
    pub banner: Option<String>,
}

impl PostInfo {
    pub fn localized_title(&self, lang: &Language) -> &str {
        match lang {
            Language::Swedish => &self.title_sv,
            Language::English => &self.title_en,
        }
    }

    pub fn localized_content(&self, lang: &Language) -> &str {
        match lang {
            Language::Swedish => &self.content_sv,
            Language::English => &self.content_en,
        }
    }
}

pub trait PostModel: for<'r> FromRow<'r, sqlx::postgres::PgRow> + Send + Unpin {}

impl PostModel for Post {}
impl PostModel for PostInfo {}
