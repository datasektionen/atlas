mod language;
mod types;

pub use ace_derive::AceInstance;
pub use language::instance::AceInstance;
pub use types::*;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn proc_macro() {
        use crate as ace;

        #[derive(AceInstance)]
        #[constants(fippel = 5)]
        struct FippelInstance {
            #[db = "post.fippel"]
            _fippel: i64,
            #[db = "post.string"]
            _s: String,
        }

        assert_eq!(
            FippelInstance::FIELDS,
            &[("_fippel", "post.fippel"), ("_s", "post.string")]
        );
        assert_eq!(
            FippelInstance::constants().get("fippel").unwrap(),
            &AceValue::AceInt(5)
        );
    }

    #[test]
    fn lex() {
        use crate::language::lexer::lex;
        let tokens: Vec<_> = lex("(fippel > -65 + 4w3d) && (date == [2026-09-28]) + \"hej\"")
            .map(|t| t.unwrap().kind)
            .collect();

        use AceTokenKind::*;
        assert_eq!(
            tokens,
            vec![
                ParenL,
                Ident("fippel".into()),
                Gt,
                Minus,
                Literal(65.into()),
                Plus,
                Duration("4w3d".into()),
                ParenR,
                And,
                ParenL,
                Ident("date".into()),
                AceTokenKind::Eq,
                Date("2026-09-28".into()),
                ParenR,
                Plus,
                Literal("hej".to_string().into()),
            ],
        );
    }

    #[test]
    fn lex_fail() {
        use crate::language::lexer::lex;
        let date_fail = lex("[2026").map(|t| t.err().unwrap()).next().unwrap();
        let str_fail = lex("\"2026").map(|t| t.err().unwrap()).next().unwrap();

        assert!(match date_fail {
            LexError::ReachedEof(c) => c == ']',
            //_ => false,
        });
        assert!(match str_fail {
            LexError::ReachedEof(c) => c == '"',
            //_ => false,
        });
    }
}
