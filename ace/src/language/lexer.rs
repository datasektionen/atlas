use crate::{AceToken, AceTokenKind, LexError};
use std::{iter::Peekable, str::Chars};

struct Cursor<'a> {
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

    fn next(&mut self) -> Option<char> {
        self.pos += 1;
        self.chars.next()
    }

    fn eat(&mut self) {
        let _ = self.next();
    }

    fn peek(&mut self) -> Option<char> {
        self.chars.peek().copied()
    }

    fn read_until(&mut self, target: char) -> Result<String, LexError> {
        let mut buf = String::new();
        while let Some(c) = self.next() {
            if c == target {
                return Ok(buf);
            }
            buf.push(c);
        }
        Err(LexError::ReachedEof(target))
    }

    fn one_or_two_token(
        &mut self,
        one_kind: AceTokenKind,
        two_kinds: &[(char, AceTokenKind)],
    ) -> AceTokenKind {
        let long_kind = if let Some(n) = self.peek() {
            two_kinds
                .iter()
                .filter_map(|(c, kind)| if *c == n { Some(kind.clone()) } else { None })
                .next()
        } else {
            None
        };
        if let Some(long_kind) = long_kind {
            self.eat();
            long_kind
        } else {
            one_kind
        }
    }

    pub fn next_token(&mut self) -> Result<AceToken, LexError> {
        // Ignore all whitespace
        while let Some(c) = self.peek() {
            if c.is_whitespace() {
                self.eat();
            } else {
                break;
            }
        }

        self.next_token_impl()
    }

    fn next_token_impl(&mut self) -> Result<AceToken, LexError> {
        let pos_before = self.pos;
        let kind = if let Some(c) = self.next() {
            match c {
                '(' => AceTokenKind::ParenL,
                ')' => AceTokenKind::ParenR,
                '+' => AceTokenKind::Plus,
                '-' => AceTokenKind::Minus,
                '~' => AceTokenKind::Glob,
                '.' => AceTokenKind::Period,

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
                    while let Some(n) = self.peek() {
                        if !n.is_ascii_alphabetic() && !n.is_ascii_digit() {
                            break;
                        }
                        buf.push(n);
                        self.eat();
                    }
                    AceTokenKind::Ident(buf)
                }
                '1'..='9' => {
                    let mut duration = false;
                    let mut buf = String::from(c);
                    while let Some(n) = self.peek() {
                        if !n.is_ascii_alphabetic() && !n.is_ascii_digit() {
                            break;
                        }
                        duration = duration || n.is_ascii_alphabetic();
                        buf.push(n);
                        self.eat();
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

pub(crate) fn lex(input: &str) -> impl Iterator<Item = Result<AceToken, LexError>> {
    let mut cursor = Cursor::new(input);
    std::iter::from_fn(move || {
        let token = cursor.next_token();
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
    })
}
