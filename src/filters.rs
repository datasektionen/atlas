use askama_macros::filter_fn;
use chrono::{DateTime, Local};

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
