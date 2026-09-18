use chrono::{DateTime, Local, TimeDelta};

#[derive(Debug, PartialEq, Eq, Clone, derive_more::From)]
pub enum AceValue {
    AceString(String),
    AceInt(i64),
    AceDuration(TimeDelta),
    AceDate(DateTime<Local>),
}

#[derive(Debug, PartialEq, Eq, Clone)]
pub(crate) enum AceTokenKind {
    Ident(String),
    // For literals that cannot fail. I.e. strings and ints.
    Literal(AceValue),
    // Defer parsing these to the parsing step
    Date(String),     // [2026-09-28 17:00]
    Duration(String), // 1w2d3h5m => 1 week, 2 days, 3 hours, 5 minutes

    ParenL, // (
    ParenR, // )
    Not,    // !
    Plus,   // +
    Minus,  // -
    And,    // &&
    Or,     // ||
    Lt,     // <
    Gt,     // >
    Leq,    // <=
    Geq,    // >=
    Eq,     // ==
    Neq,    // !=
    Glob,   // ~
    Nglob,  // !~
    Period, // .

    Unknown,
    Eof,
}

#[derive(Debug)]
pub(crate) struct Span {
    pub start: usize,
    pub end: usize,
}

#[derive(Debug)]
pub(crate) struct AceToken {
    pub kind: AceTokenKind,
    pub span: Span,
}

impl AceToken {
    pub fn new(kind: AceTokenKind, start: usize, end: usize) -> AceToken {
        AceToken {
            kind,
            span: Span { start, end },
        }
    }
}

#[derive(Debug, thiserror::Error, derive_more::From)]
pub enum AceError {
    #[error("Error during the lexing process: {0}")]
    Lex(LexError),
}

#[derive(Debug, thiserror::Error)]
pub enum LexError {
    #[error("Reached EOF while searching for '{0}'")]
    ReachedEof(char),
}
