use std::{fmt, ops::Deref};

use regex::Regex;
use rocket::form::{self, FromFormField};
use serde::Serialize;

pub mod datetime;
pub mod errors;
pub mod post;

pub struct SubmitType(bool);

impl SubmitType {
    pub fn pressed(&self) -> bool {
        self.0
    }
}

#[rocket::async_trait]
impl<'v> FromFormField<'v> for SubmitType {
    fn from_value(_: form::ValueField<'v>) -> form::Result<'v, Self> {
        Ok(Self(true))
    }

    fn default() -> Option<Self> {
        Some(Self(false))
    }
}

#[derive(sqlx::Type, Serialize, Clone, Copy)]
#[sqlx(transparent)]
#[serde(transparent)]
pub struct TrimmedStr<'v>(&'v str);

#[rocket::async_trait]
impl<'v> FromFormField<'v> for TrimmedStr<'v> {
    fn from_value(field: form::ValueField<'v>) -> form::Result<'v, Self> {
        Ok(Self(field.value.trim()))
    }
}

impl<'v> Deref for TrimmedStr<'v> {
    type Target = &'v str;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl<'v> From<&TrimmedStr<'v>> for &'v str {
    fn from(t: &TrimmedStr<'v>) -> Self {
        **t
    }
}

impl From<TrimmedStr<'_>> for serde_json::Value {
    fn from(t: TrimmedStr) -> Self {
        (*t).into()
    }
}

impl fmt::Display for TrimmedStr<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.0.fmt(f)
    }
}

impl form::validate::Len<usize> for TrimmedStr<'_> {
    fn len(&self) -> usize {
        self.0.len()
    }

    fn len_into_u64(len: usize) -> u64 {
        len as u64
    }

    fn zero_len() -> usize {
        0
    }
}

#[derive(sqlx::Type, Serialize, Clone, Copy)]
#[sqlx(transparent)]
#[serde(transparent)]
pub struct GroupKeyDto<'v>(pub &'v str);

#[rocket::async_trait]
impl<'v> FromFormField<'v> for GroupKeyDto<'v> {
    fn from_value(field: form::ValueField<'v>) -> form::Result<'v, Self> {
        valid_groupkey(field.value)?;
        Ok(Self(field.value.trim()))
    }
}

fn valid_groupkey<'v, T: Into<&'v str>>(s: T) -> form::Result<'v, ()> {
    let re = Regex::new("^[a-z0-9]+(-[a-z0-9]+)*@[-a-z0-9]+\\.[a-z]+$").unwrap();

    if re.is_match(s.into()) {
        Ok(())
    } else {
        Err(form::Error::validation("invalid groupkey").into())
    }
}
