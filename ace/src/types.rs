use chrono::{DateTime, Local};

// Modeled after INTERVAL in Postgres
#[derive(Debug, PartialEq, Eq, Clone, Default, derive_more::Display)]
#[display("{}M{}d{}ms", months, days, ms)]
pub struct Duration {
    pub months: i32,
    pub days: i32,
    pub ms: i64,
}

#[derive(Debug, PartialEq, Eq, Clone, derive_more::From, derive_more::Display)]
pub enum AceValue {
    #[display("\"{_0}\"")]
    AceString(String),
    AceInt(i64),
    AceDuration(Duration),
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
    Dot,

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

#[derive(Debug, Clone, Copy, PartialEq, Eq, derive_more::Display)]
pub(crate) enum AceUnop {
    #[display("!")]
    Not,
    #[display("-")]
    Neg,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, derive_more::Display)]
pub(crate) enum AceBinop {
    #[display(".")]
    Dot,
    #[display("+")]
    Add,
    #[display("-")]
    Sub,
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
    #[display("&&")]
    And,
    #[display("||")]
    Or,
}

#[derive(Debug, derive_more::Display)]
pub(crate) enum AceAst {
    #[display("{_0}")]
    Identifier(String),
    #[display("{_0}")]
    Literal(AceValue),
    #[display("({_0}{_1})")]
    UnaryExpr(AceUnop, Box<AceAst>),
    #[display("({_1} {_0} {_2})")]
    BinaryExpr(AceBinop, Box<AceAst>, Box<AceAst>),
}

impl AceAst {
    pub(crate) fn unop(op: AceUnop, operand: AceAst) -> AceAst {
        AceAst::UnaryExpr(op, Box::new(operand))
    }

    pub(crate) fn binop(op: AceBinop, left: AceAst, right: AceAst) -> AceAst {
        AceAst::BinaryExpr(op, Box::new(left), Box::new(right))
    }

    #[allow(unused)]
    pub fn pretty_print(&self) -> String {
        let mut buf = String::new();
        self.pp_impl(&mut buf, 0);
        buf
    }

    fn pp_impl(&self, buf: &mut String, nindent: usize) {
        const INDENT_STR: &str = "  ";
        let indent = INDENT_STR.repeat(nindent);

        match self {
            AceAst::Identifier(s) => buf.push_str(format!("{indent}{s}\n").as_str()),
            AceAst::Literal(s) => buf.push_str(format!("{indent}{s}\n").as_str()),
            AceAst::UnaryExpr(op, body) => {
                buf.push_str(format!("{indent}un:{op}\n").as_str());
                body.pp_impl(buf, nindent + 1);
            }
            AceAst::BinaryExpr(op, left, right) => {
                buf.push_str(format!("{indent}bin:{op}\n").as_str());
                left.pp_impl(buf, nindent + 1);
                right.pp_impl(buf, nindent + 1);
            }
        }
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
    #[error("Failed to parse the duration at {1}: {0}")]
    Duration(DurationError, Span),
}

#[derive(Debug, Clone, thiserror::Error)]
pub enum DurationError {
    #[error("Missing last unit")]
    MissingUnit,
    #[error("No such unit '{0}'")]
    InvalidUnit(String),
}
