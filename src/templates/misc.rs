use chrono::{DateTime, Local};

#[derive(askama::Template, Debug, askama_web::WebTemplate)]
#[template(path = "pagination.html")]
pub struct Pagination {
    total_pages: usize,
    current_page: usize,
}

// #[derive(askama::Template, Debug, askama_web::WebTemplate)]
// #[template(path = "news_card.html")]
#[derive(Debug)]
pub struct NewsCard {
    pub id: u64,
    pub title: String,
    pub summary: String,
    pub date: DateTime<Local>,
    pub tags: Vec<String>,
    pub owner: String,
}
