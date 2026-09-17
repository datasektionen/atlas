#[derive(Debug, PartialEq, Eq, derive_more::From)]
pub enum AceValue {
    AceString(String),
    AceInt(i64),
    //AceDate(),
}
