use askama_macros::filter_fn;
use chrono::{DateTime, Local};
use rand::seq::IteratorRandom;
use regex::Regex;
use std::fmt::Display;

#[filter_fn]
pub fn time_since(value: &DateTime<Local>, _env: &dyn askama::Values) -> askama::Result<String> {
    let now = Local::now();
    let seconds = (now.signed_duration_since(*value)).num_seconds();
    let (future, s) = if seconds < 0 {
        (true, -seconds)
    } else {
        (false, seconds)
    };

    let human = if s < 5 {
        "just now".to_string()
    } else if s < 60 {
        format!("{} second{}", s, if s == 1 { "" } else { "s" })
    } else if s < 3600 {
        let m = s / 60;
        format!("{} minute{}", m, if m == 1 { "" } else { "s" })
    } else if s < 86400 {
        let h = s / 3600;
        format!("{} hour{}", h, if h == 1 { "" } else { "s" })
    } else if s < 7 * 86400 {
        let d = s / 86400;
        format!("{} day{}", d, if d == 1 { "" } else { "s" })
    } else if s < 30 * 86400 {
        let w = s / (7 * 86400);
        format!("{} week{}", w, if w == 1 { "" } else { "s" })
    } else if s < 365 * 86400 {
        let mo = s / (30 * 86400);
        format!("{} month{}", mo, if mo == 1 { "" } else { "s" })
    } else {
        let y = s / (365 * 86400);
        format!("{} year{}", y, if y == 1 { "" } else { "s" })
    };

    // "just now" stands on its own; otherwise attach "ago" or "in"
    if human == "just now" {
        Ok(human)
    } else if future {
        Ok(format!("in {}", human))
    } else {
        Ok(format!("{} ago", human))
    }
}

#[filter_fn]
pub fn format_time(value: &DateTime<Local>, _env: &dyn askama::Values) -> askama::Result<String> {
    Ok(value.format("%H:%M").to_string())
}

#[filter_fn]
pub fn format_date(value: &DateTime<Local>, _env: &dyn askama::Values) -> askama::Result<String> {
    Ok(value.format("%Y-%m-%d").to_string())
}

#[filter_fn]
pub fn format_date_time(
    value: &DateTime<Local>,
    _env: &dyn askama::Values,
) -> askama::Result<String> {
    Ok(value.format("%Y-%m-%d %H:%M").to_string())
}

#[filter_fn]
pub fn format_date_query(
    value: &DateTime<Local>,
    _env: &dyn askama::Values,
) -> askama::Result<String> {
    Ok(value.to_rfc3339())
}

#[filter_fn]
pub fn urlencode(value: &str, _env: &dyn askama::Values) -> askama::Result<String> {
    Ok(urlencoding::encode(value).to_string())
}

#[filter_fn]
pub fn urldecode(value: &str, _env: &dyn askama::Values) -> askama::Result<String> {
    Ok(urlencoding::decode(value)
        .map_err(|_| askama::Error::Fmt)?
        .to_string())
}

#[filter_fn]
pub fn format_month(value: &DateTime<Local>, _env: &dyn askama::Values) -> askama::Result<String> {
    Ok(format!("{} {}", value.format("%B"), value.format("%Y")))
}

#[filter_fn]
pub fn markdown(value: impl Display, _env: &dyn askama::Values) -> askama::Result<String> {
    // Accept any Display (String, &str, etc.), convert to String and render Markdown.
    let s = value.to_string();
    let ext = comrak::options::Extension::builder().autolink(true).build();
    let html = comrak::markdown_to_html(
        &s,
        &comrak::Options {
            extension: ext,
            ..Default::default()
        },
    );
    Ok(html)
}

#[filter_fn]
pub fn strip_headers(value: &str, _env: &dyn askama::Values) -> askama::Result<String> {
    // Remove markdown headers (lines that start with one or more #) while keeping the header text.
    // This turns lines like "# Title" into "Title".
    let re = Regex::new(r"(?m)^\s*#+\s*(.*)$").unwrap();
    let result = re.replace_all(value, "$1");

    Ok(result.trim_matches('\n').to_string())
}

#[filter_fn]
pub fn owner_icon_url(value: &str, _env: &dyn askama::Values) -> askama::Result<String> {
    let icon_url =
        "https://dsekt-assets.s3.eu-west-1.amazonaws.com/shield-color-white-delta.png".to_string();

    Ok(icon_url)
}

#[filter_fn]
pub fn choose_rand(value: &'static str, _env: &dyn askama::Values) -> askama::Result<String> {
    let val = if value.is_empty() {
        String::new()
    } else {
        let mut rng = rand::rng();
        let choice = value.trim().split('\n').choose(&mut rng).unwrap_or("");
        choice.to_string()
    };

    Ok(val)
}
