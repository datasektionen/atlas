use chrono::{DateTime, Local, MappedLocalTime, NaiveDate, NaiveDateTime, NaiveTime};

fn unwrap_local_time(mlt: MappedLocalTime<DateTime<Local>>) -> Option<DateTime<Local>> {
    match mlt {
        MappedLocalTime::Single(dt) => Some(dt),
        MappedLocalTime::Ambiguous(dt1, _dt2) => Some(dt1),
        MappedLocalTime::None => None,
    }
}

const DATE_FMTS: &'static [&'static str] = &["%Y-%m-%d"];
const TIME_FMTS: &'static [&'static str] = &["%H:%M", "%H:%M:%S"];
const DATETIME_FMTS: &'static [&'static str] = &["%Y-%m-%d %H:%M", "%Y-%m-%d %H:%M:%S"];

pub fn parse_date(s: &str) -> Option<DateTime<Local>> {
    if let Some(date) = DATE_FMTS
        .iter()
        .filter_map(|fmt| NaiveDate::parse_from_str(s, fmt).ok())
        .next()
    {
        return unwrap_local_time(
            date.and_time(NaiveTime::default())
                .and_local_timezone(Local),
        );
    }

    if let Some(time) = TIME_FMTS
        .iter()
        .filter_map(|fmt| NaiveTime::parse_from_str(s, fmt).ok())
        .next()
    {
        return unwrap_local_time(
            NaiveDate::default()
                .and_time(time)
                .and_local_timezone(Local),
        );
    }

    if let Some(dt) = DATETIME_FMTS
        .iter()
        .filter_map(|fmt| NaiveDateTime::parse_from_str(s, fmt).ok())
        .next()
    {
        return unwrap_local_time(dt.and_local_timezone(Local));
    }

    None
}
