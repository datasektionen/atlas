use chrono::{Datelike, Local, Month, TimeZone};

#[derive(Debug)]
pub struct Calendar {
    pub days: Vec<CalendarDay>,
}

#[derive(Debug)]
pub struct CalendarDay {
    pub date: chrono::DateTime<Local>,
    pub events: Vec<CalendarEvent>,
    pub is_current_month: bool,
}

#[derive(Debug)]
pub struct CalendarEvent {
    pub title: String,
    pub from: chrono::DateTime<Local>,
    pub to: chrono::DateTime<Local>,
}

impl Calendar {
    pub fn new(month: Month, year: i32) -> Self {
        let num_days = month.num_days(year).unwrap();
        let mut days = Vec::new();
        let weekday_offset = Local
            .with_ymd_and_hms(year, month.number_from_month(), 1, 0, 0, 0)
            .unwrap()
            .date_naive()
            .weekday()
            .num_days_from_monday();
        let last_month = month.pred();
        let last_month_num_days = last_month.num_days(year).unwrap() as u32;
        let year_offset = if month == Month::January { -1 } else { 0 };

        for i in (last_month_num_days - weekday_offset + 1)..=last_month_num_days {
            let date = Local
                .with_ymd_and_hms(
                    year + year_offset,
                    last_month.number_from_month(),
                    i,
                    0,
                    0,
                    0,
                )
                .unwrap();
            days.push(CalendarDay {
                date,
                events: vec![
                    CalendarEvent {
                        title: format!("Event on {}-{}-{}", year, month.number_from_month(), i),
                        from: date,
                        to: date + chrono::Duration::hours(4),
                    },
                    CalendarEvent {
                        title: format!("Event on {}-{}-{}", year, month.number_from_month(), i),
                        from: date,
                        to: date + chrono::Duration::hours(4),
                    },
                ],
                is_current_month: false,
            });
        }

        for day in 1..=num_days {
            let date = Local
                .with_ymd_and_hms(year, month.number_from_month(), day as u32, 0, 0, 0)
                .unwrap();
            days.push(CalendarDay {
                date,
                events: vec![
                    CalendarEvent {
                        title: format!("Event on {}-{}-{}", year, month.number_from_month(), day),
                        from: date,
                        to: date + chrono::Duration::hours(4),
                    },
                    CalendarEvent {
                        title: format!("Event on {}-{}-{}", year, month.number_from_month(), day),
                        from: date,
                        to: date + chrono::Duration::hours(4),
                    },
                ],
                is_current_month: true,
            });
        }

        let next_month_offset = 6 * 7 - (weekday_offset + num_days as u32);
        let next_month = month.succ();
        let next_year_offset = if month == Month::December { 1 } else { 0 };

        for i in 1..=next_month_offset {
            let date = Local
                .with_ymd_and_hms(
                    year + next_year_offset,
                    next_month.number_from_month(),
                    i,
                    0,
                    0,
                    0,
                )
                .unwrap();
            days.push(CalendarDay {
                date,
                events: vec![
                    CalendarEvent {
                        title: format!(
                            "Event on {}-{}-{}",
                            year,
                            next_month.number_from_month(),
                            i
                        ),
                        from: date,
                        to: date + chrono::Duration::hours(4),
                    },
                    CalendarEvent {
                        title: format!("Event on {}-{}-{}", year, month.number_from_month(), i),
                        from: date,
                        to: date + chrono::Duration::hours(4),
                    },
                ],
                is_current_month: false,
            });
        }

        Calendar { days }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::Month;

    #[test]
    fn test_calendar_jan_2026() {
        let calendar = Calendar::new(Month::January, 2026);
        assert_eq!(calendar.days.len(), 42); // 6 weeks * 7 days
        println!("Calendar days:");
        for day in &calendar.days {
            println!(
                "{}-{}-{}",
                day.date.year(),
                day.date.month(),
                day.date.day()
            );
        }
        assert_eq!(calendar.days[0].date.day(), 29); // Last days of December 2025
        assert_eq!(calendar.days[0].date.weekday(), chrono::Weekday::Mon);
        assert_eq!(calendar.days[0].date.month(), 12);
        assert_eq!(calendar.days[0].date.year(), 2025);
        assert_eq!(calendar.days[2].date.day(), 31); // Last day of December 2025
        assert_eq!(calendar.days[2].date.weekday(), chrono::Weekday::Wed);
        assert_eq!(calendar.days[2].date.month(), 12);
        assert_eq!(calendar.days[2].date.year(), 2025);
        assert_eq!(calendar.days[3].date.day(), 1); // First day of January 2026
        assert_eq!(calendar.days[3].date.weekday(), chrono::Weekday::Thu);
        assert_eq!(calendar.days[3].date.month(), 1);
        assert_eq!(calendar.days[3].date.year(), 2026);
    }

    #[test]
    fn test_calendar_feb_2026() {
        let calendar = Calendar::new(Month::February, 2026);
        assert_eq!(calendar.days.len(), 42); // 6 weeks * 7 days
        println!("Calendar days:");
        for day in &calendar.days {
            println!(
                "{}-{}-{}",
                day.date.year(),
                day.date.month(),
                day.date.day()
            );
        }
        assert_eq!(calendar.days[0].date.day(), 26); // Last days of January 2026
        assert_eq!(calendar.days[0].date.weekday(), chrono::Weekday::Mon);
        assert_eq!(calendar.days[0].date.month(), 1);
        assert_eq!(calendar.days[0].date.year(), 2026);
        assert_eq!(calendar.days[5].date.day(), 31); // Last day of January 2026
        assert_eq!(calendar.days[5].date.weekday(), chrono::Weekday::Sat);
        assert_eq!(calendar.days[5].date.month(), 1);
        assert_eq!(calendar.days[5].date.year(), 2026);
        assert_eq!(calendar.days[6].date.day(), 1); // First day of February 2026
        assert_eq!(calendar.days[6].date.weekday(), chrono::Weekday::Sun);
        assert_eq!(calendar.days[6].date.month(), 2);
        assert_eq!(calendar.days[6].date.year(), 2026);
        assert_eq!(calendar.days.last().unwrap().date.day(), 8); // Beginning of March 2026
        assert_eq!(
            calendar.days.last().unwrap().date.weekday(),
            chrono::Weekday::Sun
        );
        assert_eq!(calendar.days.last().unwrap().date.month(), 3);
        assert_eq!(calendar.days.last().unwrap().date.year(), 2026);
    }
}
