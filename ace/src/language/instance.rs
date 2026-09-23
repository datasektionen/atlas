use crate::{AceError, AceValue};
use std::collections::HashMap;

// TODO: Expose type of fields. Also maybe make their declarations into tags to keep types
// implementing AceInstance zero-sized?
pub trait AceInstance {
    const FIELDS: &'static [(&'static str, &'static str)];
    fn constants() -> HashMap<&'static str, AceValue>;

    // TODO: change bool to AceType so fields can expose their types
    fn get_value(ident: &str) -> (bool, Option<AceValue>) {
        if let Some(val) = Self::constants().get(ident) {
            // Maybe try to get out of cloning here somehow?
            (true, Some(val.clone()))
        } else {
            (Self::FIELDS.iter().any(|(n, _)| *n == ident), None)
        }
    }

    fn compile_expression(expr: String) -> Result<String, AceError> {
        todo!()
    }
}
