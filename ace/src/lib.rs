mod language;
mod parsetype;
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
        use crate::language::lexer::{AceLexer, Lexer};
        let lexer = AceLexer::lex("(fippel > -65 + 4w3d) && (date == [2026-09-28]) + \"hej\"");
        let tokens: Vec<_> = lexer.map(|t| t.unwrap().kind).collect();

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
        use crate::language::lexer::{AceLexer, Lexer};
        let date_fail = AceLexer::lex("[2026")
            .map(|t| t.err().unwrap())
            .next()
            .unwrap();
        let str_fail = AceLexer::lex("\"2026")
            .map(|t| t.err().unwrap())
            .next()
            .unwrap();

        assert!(match date_fail {
            AceError::Lex(LexError::ReachedEof(c)) => c == ']',
            _ => false,
        });
        assert!(match str_fail {
            AceError::Lex(LexError::ReachedEof(c)) => c == '"',
            _ => false,
        });
    }

    #[test]
    fn parse() {
        use crate::language::{
            lexer::{AceLexer, Lexer},
            parser::AceParser,
        };
        let out = AceParser::new(AceLexer::lex(
            "(fippel > -65 + 4w3d) && (date.time == [2026-07-04 17:32].time) + \"hej\"",
        ))
        .parse()
        .unwrap();

        // The test is that it doesn't crash...
        //panic!("{}", out.pretty_print());
    }

    #[test]
    fn analyze() {
        use crate::language::{
            lexer::{AceLexer, Lexer},
            parser::AceParser,
        };
        use crate::parsetype::parse_date;

        use crate as ace;
        #[derive(AceInstance)]
        #[constants(date = parse_date("17:32").unwrap())]
        struct FippelInstance {
            #[db = "post.fippel"]
            fippel: i64,
            // TODO: Remove. Only here temporarily to prevent analysis from failing on dot
            // expressions, since they contain "special" identifiers.
            #[db = "post.time"]
            time: String,
        }

        let out = AceParser::new(AceLexer::lex(
            "(fippel > -65 + 4w3d) && (date.time == [2026-07-04 17:32].time) + \"hej\"",
        ))
        .parse()
        .unwrap()
        .analyze::<FippelInstance>()
        .unwrap();

        // The test is that it doesn't crash...
        //panic!("{}", out.pretty_print());
    }
}
