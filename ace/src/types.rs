use chrono::{DateTime, Local, TimeDelta};

#[derive(Debug, PartialEq, Eq, Clone, derive_more::From, derive_more::Display)]
pub enum AceValue {
    #[display("\"{_0}\"")]
    AceString(String),
    AceInt(i64),
    AceDuration(TimeDelta),
    #[display("[{_0}]")]
    AceDate(DateTime<Local>),
}

#[derive(Debug, PartialEq, Eq, Clone, derive_more::Display)]
pub enum AceTokenKind {
    #[display("{_0}")]
    Ident(String),
    // For literals that cannot fail. I.e. strings and ints.
    Literal(AceValue),
    // Defer parsing these to the parsing step
    #[display("[{_0}]")]
    Date(String), // [2026-09-28 17:00]
    Duration(String), // 1w2d3h5m => 1 week, 2 days, 3 hours, 5 minutes

    #[display("(")]
    ParenL,
    #[display(")")]
    ParenR,
    #[display("!")]
    Not,
    #[display("+")]
    Plus,
    #[display("-")]
    Minus,
    #[display("&&")]
    And,
    #[display("||")]
    Or,
    #[display("<")]
    Lt,
    #[display(">")]
    Gt,
    #[display("<=")]
    Leq,
    #[display(">=")]
    Geq,
    #[display("==")]
    Eq,
    #[display("!=")]
    Neq,
    #[display("~")]
    Glob,
    #[display("!~")]
    Nglob,
    #[display(".")]
    Period,

    #[display("(unknown)")]
    Unknown,
    #[display("(eof)")]
    Eof,
}

#[derive(Debug, Clone, derive_more::Display)]
#[display("{}--{}", start, end)]
pub struct Span {
    pub start: usize,
    pub end: usize,
}

#[derive(Debug, Clone)]
pub struct AceToken {
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

pub type AceResult<T> = Result<T, AceError>;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum AceUnop {
    Not,
    Neg,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum AceBinop {
    Add,
    Sub,
    Lt,
    Gt,
    Leq,
    Geq,
    Eq,
    Neq,
    Glob,
    Nglob,
    And,
    Or,
}

#[derive(Debug)]
pub(crate) enum AceAst {
    Identifier(String),
    Literal(AceValue),
    PrimaryExpr(Box<AceAst>),
    UnaryExpr(AceUnop, Box<AceAst>),
    BinaryExpr(AceBinop, Box<AceAst>, Box<AceAst>),
}

impl AceAst {
    pub(crate) fn primary(expr: AceAst) -> AceAst {
        AceAst::PrimaryExpr(Box::new(expr))
    }

    pub(crate) fn unop(op: AceUnop, operand: AceAst) -> AceAst {
        AceAst::UnaryExpr(op, Box::new(operand))
    }

    pub(crate) fn binop(op: AceBinop, left: AceAst, right: AceAst) -> AceAst {
        AceAst::BinaryExpr(op, Box::new(left), Box::new(right))
    }
}

#[derive(Debug, Clone, thiserror::Error, derive_more::From)]
pub enum AceError {
    #[error("Error during the lexing process: {0}")]
    Lex(LexError),
    #[error("Error during the parsing process: {0}")]
    Parse(ParseError),
}

#[derive(Debug, Clone, thiserror::Error)]
pub enum LexError {
    #[error("Reached EOF while searching for '{0}'")]
    ReachedEof(char),
}

#[derive(Debug, Clone, thiserror::Error)]
pub enum ParseError {
    #[error("Premature EOF")]
    Eof,
    #[error("Expected '{0}', but found '{1}' at {2}")]
    Expected(AceTokenKind, AceTokenKind, Span),
    #[error("Expected primary expression, but found '{0}' at {1}")]
    Primary(AceTokenKind, Span),
    #[error("Failed to parse the date '{0}' at {1}")]
    Date(String, Span),
    #[error("Failed to parse the duration '{0}' at {1}")]
    Duration(String, Span),
}
