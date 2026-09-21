use crate::{
    language::lexer::{AceLexer, Lexer},
    parsetype, AceAst, AceBinop, AceResult, AceTokenKind, AceUnop, ParseError,
};

pub(crate) struct AceParser<'a> {
    lexer: AceLexer<'a>,
}

macro_rules! add_binop {
    ($name:ident, $next:ident, $($token_kind:pat => $op:expr),*$(,)*) => {
        fn $name(&mut self) -> AceResult<AceAst> {
            let mut left = self.$next()?;

            while let Ok(tok) = self.lexer.peek_tok() {
                let op = match tok.kind {
                    $($token_kind => $op,)*
                    _ => break,
                };

                self.lexer.discard_tok()?;
                let right = self.$name()?;
                left = AceAst::binop(op, left, right);
            }

            Ok(left)
        }
    };
}

macro_rules! add_unop {
    ($name:ident, $next:ident, $($token_kind:pat => $op:expr),*$(,)*) => {
        fn $name(&mut self) -> AceResult<AceAst> {
            if let Ok(tok) = self.lexer.peek_tok() {
                let op = match tok.kind {
                    $($token_kind => $op,)*
                    _ => return self.$next(),
                };
                self.lexer.discard_tok()?;
                Ok(AceAst::unop(op, self.$name()?))
            } else {
                self.$next()
            }
        }
    }
}

impl<'a> AceParser<'a> {
    pub fn new(lexer: AceLexer<'a>) -> AceParser<'a> {
        AceParser { lexer }
    }

    // Parse an expression which also happens to be the "root" node.
    pub fn parse(&mut self) -> AceResult<AceAst> {
        // Logical or has the lowest precedence. See the macro below.
        self._p_or()
    }

    add_binop!(_p_or, _p_and,
        AceTokenKind::Or => AceBinop::Or,
    );
    add_binop!(_p_and, _p_eq,
        AceTokenKind::And => AceBinop::And,
    );
    add_binop!(_p_eq, _p_rel,
        AceTokenKind::Eq => AceBinop::Eq,
        AceTokenKind::Neq => AceBinop::Neq,
        AceTokenKind::Glob => AceBinop::Glob,
        AceTokenKind::Nglob =>  AceBinop::Nglob,
    );
    add_binop!(_p_rel, _p_add,
        AceTokenKind::Lt => AceBinop::Lt,
        AceTokenKind::Gt => AceBinop::Gt,
        AceTokenKind::Leq => AceBinop::Leq,
        AceTokenKind::Geq => AceBinop::Geq,
    );
    add_binop!(_p_add, _p_un,
        AceTokenKind::Plus => AceBinop::Add,
        AceTokenKind::Minus => AceBinop::Sub,
    );
    add_unop!(_p_un, _p_dot,
        AceTokenKind::Not => AceUnop::Not,
        AceTokenKind::Minus => AceUnop::Neg,
    );
    add_binop!(_p_dot, _p_primary,
        AceTokenKind::Dot => AceBinop::Dot,
    );

    // Parse a primary expression
    fn _p_primary(&mut self) -> AceResult<AceAst> {
        // Fail on peek since this is the "deepest" we can defer
        let tok = self.lexer.peek_tok()?;
        let out = match tok.kind {
            AceTokenKind::Ident(s) => Ok(AceAst::Identifier(s)),
            AceTokenKind::Literal(v) => Ok(AceAst::Literal(v.into())),
            AceTokenKind::Date(s) => match parsetype::parse_date(s.as_str()) {
                Some(date) => Ok(AceAst::Literal(date.into())),
                None => Err(ParseError::Date(s, tok.span).into()),
            },
            AceTokenKind::Duration(s) => match parsetype::parse_duration(s.as_str()) {
                Ok(dur) => Ok(AceAst::Literal(dur.into())),
                Err(err) => Err(ParseError::Duration(err, tok.span).into()),
            },

            AceTokenKind::ParenL => {
                self.lexer.discard_tok()?;
                let out = self.parse();
                self.lexer.eat_tok(AceTokenKind::ParenR)?;
                out
            }

            _ => Err(ParseError::Primary(tok.kind, tok.span).into()),
        };
        self.lexer.discard_tok()?;
        out
    }
}
