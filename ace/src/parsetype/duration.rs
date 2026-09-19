use crate::{Duration, DurationError};
use std::ops::{Add, Mul};

#[derive(Clone, Copy)]
struct DurImpl(i32, i32, i64);

impl Add for DurImpl {
    type Output = Self;

    fn add(self, other: Self) -> Self {
        // TODO: look up if milliseconds should add up to make days
        Self(self.0 + other.0, self.1 + other.1, self.2 + other.2)
    }
}

impl Mul<i32> for DurImpl {
    type Output = Self;

    fn mul(self, other: i32) -> Self {
        Self(self.0 * other, self.1 * other, self.2 * (other as i64))
    }
}

impl From<DurImpl> for Duration {
    fn from(d: DurImpl) -> Duration {
        Duration {
            months: d.0,
            days: d.1,
            ms: d.2,
        }
    }
}

struct UnitValue<'a>(i32, &'a str);

struct UnitParser<'a> {
    s: &'a str,
    pos: usize,
}

impl<'a> UnitParser<'a> {
    fn new(s: &'a str) -> UnitParser<'a> {
        UnitParser { s, pos: 0 }
    }
}

impl<'a> Iterator for UnitParser<'a> {
    type Item = Result<UnitValue<'a>, DurationError>;

    // XXX: This is really bad but I didn't get it to work at all using Chars or anything
    // reasonable...
    fn next(&mut self) -> Option<Result<UnitValue<'a>, DurationError>> {
        if self.pos >= self.s.len() {
            return None;
        }

        let start = self.pos;

        while self.s[start..=self.pos].parse::<i32>().is_ok() {
            self.pos += 1;
            if self.pos >= self.s.len() {
                return Some(Err(DurationError::MissingUnit));
            }
        }
        let i: i32 = self.s[start..self.pos].parse::<i32>().expect("infallible");
        let start = self.pos;

        while self.s[self.pos..self.pos + 1].parse::<i32>().is_err() {
            self.pos += 1;
            if self.pos >= self.s.len() {
                break;
            }
        }
        let u = &self.s[start..self.pos];
        if u == "" {
            Some(Err(DurationError::MissingUnit))
        } else {
            Some(Ok(UnitValue(i, u)))
        }
    }
}

const UNITS: &'static [(&'static str, DurImpl)] = &[
    ("Y", DurImpl(0, 365, 0)),
    ("M", DurImpl(1, 0, 0)),
    ("w", DurImpl(0, 7, 0)),
    ("d", DurImpl(0, 1, 0)),
    ("h", DurImpl(0, 0, 3600000)),
    ("m", DurImpl(0, 0, 60000)),
    ("s", DurImpl(0, 0, 1000)),
    ("ms", DurImpl(0, 0, 1)),
];

fn find_unit(unit: &str) -> Result<DurImpl, DurationError> {
    UNITS
        .iter()
        .filter_map(|(u, v)| if unit == *u { Some(*v) } else { None })
        .next()
        .ok_or(DurationError::InvalidUnit(unit.to_string()))
}

pub fn parse_duration(s: &str) -> Result<Duration, DurationError> {
    let mut sum = DurImpl(0, 0, 0);
    for uv in UnitParser::new(s) {
        let uv = uv?;
        sum = sum + find_unit(uv.1)? * uv.0;
    }
    Ok(sum.into())
}
