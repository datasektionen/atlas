use crate::AceValue;
use std::collections::HashMap;

pub trait AceInstance {
    fn constants() -> HashMap<&'static str, AceValue>;
    const FIELDS: &'static [(&'static str, &'static str)];
}
