use chrono::{DateTime, Local};

// Modeled after INTERVAL in Postgres
#[derive(Debug, PartialEq, Eq, Clone, Default, derive_more::Display)]
#[display("{}M{}d{}ms", months, days, ms)]
pub struct Duration {
    pub months: i32,
    pub days: i32,
    pub ms: i64,
}

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub enum AceType {
    Unknown,
    AceString,
    AceInt,
    AceDuration,
    AceDate,
}

impl Default for AceType {
    fn default() -> Self {
        Self::Unknown
    }
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

impl AceValue {
    pub fn typ(&self) -> AceType {
        match self {
            AceValue::AceString(_) => AceType::AceString,
            AceValue::AceInt(_) => AceType::AceInt,
            AceValue::AceDuration(_) => AceType::AceDuration,
            AceValue::AceDate(_) => AceType::AceDate,
        }
    }
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

#[derive(Debug, Clone, Copy, derive_more::Display)]
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
#[display("{span}, {typ:?}")]
pub(crate) struct AceAstMeta {
    pub span: Span,
    pub typ: AceType,
}

impl AceAstMeta {
    pub fn new(span: Span) -> AceAstMeta {
        AceAstMeta {
            span,
            // Will be assigned during the semantic analysis phase
            typ: AceType::Unknown,
        }
    }
}

#[derive(Debug, derive_more::Display)]
pub(crate) enum AceAst {
    #[display("{_1}")]
    Identifier(AceAstMeta, String),
    #[display("{_1}")]
    Literal(AceAstMeta, AceValue),
    #[display("({_1}{_2})")]
    UnaryExpr(AceAstMeta, AceUnop, Box<AceAst>),
    #[display("({_2} {_1} {_3})")]
    BinaryExpr(AceAstMeta, AceBinop, Box<AceAst>, Box<AceAst>),
}

impl AceAst {
    pub(crate) fn meta(&self) -> &AceAstMeta {
        match self {
            AceAst::Identifier(m, _) => &m,
            AceAst::Literal(m, _) => &m,
            AceAst::UnaryExpr(m, _, _) => &m,
            AceAst::BinaryExpr(m, _, _, _) => &m,
        }
    }

    pub(crate) fn ident(val: String, span: Span) -> AceAst {
        AceAst::Identifier(AceAstMeta::new(span), val)
    }

    pub(crate) fn lit(val: AceValue, span: Span) -> AceAst {
        AceAst::Literal(AceAstMeta::new(span), val)
    }

    pub(crate) fn unop(op: AceUnop, operand: AceAst) -> AceAst {
        let span = operand.meta().span;
        AceAst::UnaryExpr(AceAstMeta::new(span), op, Box::new(operand))
    }

    pub(crate) fn binop(op: AceBinop, left: AceAst, right: AceAst) -> AceAst {
        let lspan = &left.meta().span;
        let rspan = &right.meta().span;
        let span = Span {
            start: lspan.start,
            end: rspan.end,
        };
        AceAst::BinaryExpr(AceAstMeta::new(span), op, Box::new(left), Box::new(right))
    }

    pub(crate) fn analyze<T>(self) -> AceResult<AceAst>
    where
        T: crate::language::instance::AceInstance,
    {
        let out = match self {
            AceAst::Identifier(mut m, s) => {
                let (exists, val) = T::get_value(s.as_str());
                if !exists {
                    return Err(ParseError::Identifier(s.to_string(), m.span).into());
                }
                m.typ = match val {
                    Some(v) => v.typ(),
                    None => AceType::default(),
                };

                AceAst::Identifier(m, s)
            }
            AceAst::Literal(mut m, v) => {
                m.typ = v.typ();

                AceAst::Literal(m, v)
            }
            AceAst::UnaryExpr(mut m, op, body) => {
                // TODO: calculate based on the operation
                let body = body.analyze::<T>()?;
                m.typ = AceType::default();

                AceAst::UnaryExpr(m, op, Box::new(body))
            }
            AceAst::BinaryExpr(mut m, op, left, right) => {
                // TODO: calculate based on the operation
                let left = left.analyze::<T>()?;
                let right = right.analyze::<T>()?;
                m.typ = AceType::default();

                AceAst::BinaryExpr(m, op, Box::new(left), Box::new(right))
            }
        };

        Ok(out)
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
            AceAst::Identifier(m, s) => buf.push_str(format!("{indent}{s} ({m})\n").as_str()),
            AceAst::Literal(m, s) => buf.push_str(format!("{indent}{s} ({m})\n").as_str()),
            AceAst::UnaryExpr(m, op, body) => {
                buf.push_str(format!("{indent}un:{op} ({m})\n").as_str());
                body.pp_impl(buf, nindent + 1);
            }
            AceAst::BinaryExpr(m, op, left, right) => {
                buf.push_str(format!("{indent}bin:{op} ({m})\n").as_str());
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
    #[error("Unknown identifier '{0}' at {1}")]
    Identifier(String, Span),
}

#[derive(Debug, Clone, thiserror::Error)]
pub enum DurationError {
    #[error("Missing last unit")]
    MissingUnit,
    #[error("No such unit '{0}'")]
    InvalidUnit(String),
}
