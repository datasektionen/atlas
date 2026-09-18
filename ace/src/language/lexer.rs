use crate::{AceError, AceResult, AceToken, AceTokenKind, LexError, ParseError};
use std::{iter::Peekable, str::Chars};

pub(crate) struct Cursor<'a> {
    chars: Peekable<Chars<'a>>,
    pos: usize,
}

impl<'a> Cursor<'a> {
    pub fn new(input: &'a str) -> Cursor<'a> {
        Cursor {
            chars: input.chars().peekable(),
            pos: 0,
        }
    }

    fn nextc(&mut self) -> Option<char> {
        self.pos += 1;
        self.chars.next()
    }

    fn eatc(&mut self) {
        let _ = self.nextc();
    }

    fn peekc(&mut self) -> Option<char> {
        self.chars.peek().copied()
    }

    fn read_until(&mut self, target: char) -> AceResult<String> {
        let mut buf = String::new();
        while let Some(c) = self.nextc() {
            if c == target {
                return Ok(buf);
            }
            buf.push(c);
        }
        Err(LexError::ReachedEof(target).into())
    }

    fn one_or_two_token(
        &mut self,
        one_kind: AceTokenKind,
        two_kinds: &[(char, AceTokenKind)],
    ) -> AceTokenKind {
        let long_kind = if let Some(n) = self.peekc() {
            two_kinds
                .iter()
                .filter_map(|(c, kind)| if *c == n { Some(kind.clone()) } else { None })
                .next()
        } else {
            None
        };
        if let Some(long_kind) = long_kind {
            self.eatc();
            long_kind
        } else {
            one_kind
        }
    }

    pub fn next_token(&mut self) -> AceResult<AceToken> {
        // Ignore all whitespace
        while let Some(c) = self.peekc() {
            if c.is_whitespace() {
                self.eatc();
            } else {
                break;
            }
        }

        self.next_token_impl()
    }

    fn next_token_impl(&mut self) -> AceResult<AceToken> {
        let pos_before = self.pos;
        let kind = if let Some(c) = self.nextc() {
            match c {
                '(' => AceTokenKind::ParenL,
                ')' => AceTokenKind::ParenR,
                '+' => AceTokenKind::Plus,
                '-' => AceTokenKind::Minus,
                '~' => AceTokenKind::Glob,
                '.' => AceTokenKind::Dot,

                '!' => self.one_or_two_token(
                    AceTokenKind::Not,
                    &[('=', AceTokenKind::Neq), ('~', AceTokenKind::Nglob)],
                ),
                '=' => self.one_or_two_token(AceTokenKind::Unknown, &[('=', AceTokenKind::Eq)]),
                '&' => self.one_or_two_token(AceTokenKind::Unknown, &[('&', AceTokenKind::And)]),
                '|' => self.one_or_two_token(AceTokenKind::Unknown, &[('|', AceTokenKind::Or)]),
                '<' => self.one_or_two_token(AceTokenKind::Lt, &[('=', AceTokenKind::Leq)]),
                '>' => self.one_or_two_token(AceTokenKind::Gt, &[('=', AceTokenKind::Geq)]),

                '[' => AceTokenKind::Date(self.read_until(']')?),
                '"' => AceTokenKind::Literal(self.read_until('"')?.into()),
                'a'..='z' | 'A'..='Z' => {
                    let mut buf = String::from(c);
                    while let Some(n) = self.peekc() {
                        if !n.is_ascii_alphabetic() && !n.is_ascii_digit() {
                            break;
                        }
                        buf.push(n);
                        self.eatc();
                    }
                    AceTokenKind::Ident(buf)
                }
                '1'..='9' => {
                    let mut duration = false;
                    let mut buf = String::from(c);
                    while let Some(n) = self.peekc() {
                        if !n.is_ascii_alphabetic() && !n.is_ascii_digit() {
                            break;
                        }
                        duration = duration || n.is_ascii_alphabetic();
                        buf.push(n);
                        self.eatc();
                    }
                    if duration {
                        AceTokenKind::Duration(buf)
                    } else {
                        AceTokenKind::Literal(
                            buf.parse::<i64>()
                                .expect("If this fails something is wrong")
                                .into(),
                        )
                    }
                }

                _ => AceTokenKind::Unknown,
            }
        } else {
            AceTokenKind::Eof
        };
        Ok(AceToken::new(kind, pos_before, self.pos))
    }
}

impl<'a> Iterator for Cursor<'a> {
    type Item = AceResult<AceToken>;

    fn next(&mut self) -> Option<Self::Item> {
        let token = self.next_token();
        if token.is_err() {
            Some(token)
        } else {
            let t = token.expect("infallible");
            if t.kind == AceTokenKind::Eof {
                None
            } else {
                Some(Ok(t))
            }
        }
    }
}

// This feels weird. Can probably be done in a nicer way.
pub(crate) type AceLexer<'a> = Peekable<Cursor<'a>>;

impl<'a, 'b> Lexer<'b> for AceLexer<'a>
where
    'b: 'a,
{
    fn lex(input: &'b str) -> AceLexer<'a> {
        Cursor::new(input).peekable()
    }

    fn next_tok(&mut self) -> AceResult<AceToken> {
        if let Some(t) = Iterator::next(self) {
            t.clone()
        } else {
            Err(ParseError::Eof.into())
        }
    }

    fn peek_tok(&mut self) -> AceResult<AceToken> {
        if let Some(t) = self.peek() {
            t.clone()
        } else {
            Err(ParseError::Eof.into())
        }
    }
}

pub(crate) trait Lexer<'a> {
    fn lex(input: &'a str) -> Self;

    fn next_tok(&mut self) -> AceResult<AceToken>;
    fn peek_tok(&mut self) -> AceResult<AceToken>;

    fn expect_tok(&mut self, target: AceTokenKind) -> AceResult<bool> {
        self.peek_tok().map(|t| t.kind == target)
    }

    fn eat_tok(&mut self, target: AceTokenKind) -> AceResult<()> {
        let token = self.peek_tok();
        if let Ok(t) = token {
            if t.kind == target {
                Ok(())
            } else {
                Err(ParseError::Expected(target, t.kind, t.span).into())
            }
        } else {
            token.map(|_| ())
        }
    }

    fn discard_tok(&mut self) -> AceResult<()> {
        let _ = self.next_tok()?;
        Ok(())
    }
}
