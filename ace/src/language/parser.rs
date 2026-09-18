use crate::{
    language::lexer::{AceLexer, Lexer},
    AceAst, AceBinop, AceResult, AceToken, AceTokenKind, AceUnop, ParseError,
};

pub(crate) struct AceParser<'a> {
    lexer: AceLexer<'a>,
}

impl<'a> AceParser<'a> {
    const BINOP_PRECEDENCE: &'static [&'static [(AceTokenKind, AceBinop)]] = &[
        &[(AceTokenKind::Or, AceBinop::Or)],
        &[(AceTokenKind::And, AceBinop::And)],
        &[
            (AceTokenKind::Eq, AceBinop::Eq),
            (AceTokenKind::Neq, AceBinop::Neq),
            (AceTokenKind::Glob, AceBinop::Glob),
            (AceTokenKind::Nglob, AceBinop::Nglob),
        ],
        &[
            (AceTokenKind::Lt, AceBinop::Lt),
            (AceTokenKind::Gt, AceBinop::Gt),
            (AceTokenKind::Leq, AceBinop::Leq),
            (AceTokenKind::Geq, AceBinop::Geq),
        ],
        &[
            (AceTokenKind::Plus, AceBinop::Add),
            (AceTokenKind::Minus, AceBinop::Sub),
        ],
    ];

    pub fn new(lexer: AceLexer<'a>) -> AceParser<'a> {
        AceParser { lexer }
    }

    pub fn parse(&mut self) -> AceResult<AceAst> {
        self.parse_binop(0)
    }

    // Parse a generic binary expression in reverse order of precedence. You can probably make this
    // more efficient by generating different functions using match statements with a macro.
    fn parse_binop(&mut self, idx: usize) -> AceResult<AceAst> {
        if idx >= AceParser::BINOP_PRECEDENCE.len() {
            return self.parse_unop();
        }

        let ops = AceParser::BINOP_PRECEDENCE[idx];
        let mut left = self.parse_binop(idx + 1)?;

        while let Ok(tok) = self.lexer.peek_tok() {
            let op = ops
                .iter()
                .filter_map(|op| if op.0 == tok.kind { Some(op.1) } else { None })
                .next();

            if let Some(op) = op {
                self.lexer.discard_tok()?;
                let right = self.parse_binop(idx + 1)?;
                left = AceAst::binop(op, left, right);
            } else {
                break;
            }
        }
        Ok(left)
    }

    fn parse_unop(&mut self) -> AceResult<AceAst> {
        if let Ok(tok) = self.lexer.peek_tok() {
            let unary_type = match tok.kind {
                AceTokenKind::Not => AceUnop::Not,
                AceTokenKind::Minus => AceUnop::Neg,
                _ => return self.parse_primary(),
            };
            self.lexer.discard_tok()?;
            let body = self.parse_unop()?;
            Ok(AceAst::unop(unary_type, body))
        } else {
            self.parse_primary()
        }
    }

    fn parse_primary(&mut self) -> AceResult<AceAst> {
        let tok = self.lexer.peek_tok()?;
        let ok = match tok.kind {
            AceTokenKind::Ident(s) => AceAst::Identifier(s),
            AceTokenKind::Literal(v) => AceAst::Literal(v.into()),
            // TODO: lös
            AceTokenKind::Date(s) => AceAst::Identifier(s.into()),
            AceTokenKind::Duration(s) => AceAst::Identifier(s.into()),

            AceTokenKind::ParenL => {
                self.lexer.discard_tok()?;
                let out = self.parse();
                self.lexer.eat_tok(AceTokenKind::ParenR)?;
                out?
            }

            _ => return Err(ParseError::Primary(tok).into()),
        };
        self.lexer.discard_tok()?;
        Ok(ok)
    }
}
