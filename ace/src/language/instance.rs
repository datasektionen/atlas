use crate::{AceError, AceValue};
use std::collections::HashMap;

pub trait AceInstance {
    const FIELDS: &'static [(&'static str, &'static str)];
    fn constants() -> HashMap<&'static str, AceValue>;

    fn parse_expression(expr: String) -> Result<String, AceError> {
        todo!()
    }
}
