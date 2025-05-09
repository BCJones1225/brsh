use std::io::{Read, stdin};

use lexer::lex;

fn main() -> miette::Result<()> {
    let mut buffer = String::new();
    stdin().read_to_string(&mut buffer)
        .expect("Invalid UTF8!");
    // TODO: miette error here

    for token in lex(&buffer) {
        println!("{:?}", token?);
    }

    Ok(())
}

mod lexer {
    use miette::{Diagnostic, NamedSource, SourceSpan};

    pub fn lex(input: &str) -> 
    impl Iterator<
        Item = miette::Result<Token>
    > {
        let mut chars = input.chars();
        let mut character = 0;

        let mut ret = Vec::new();

        let mut cho = chars.next();        
        character += 1;

        while let Some(ch) = cho {
            dbg!((&ch, &character));
            let tok = match ch {
                '0'..='9' => Ok(
                    int(
                        &mut chars, 
                        &mut character, 
                        ch
                    )
                ),
                _ => Err(UnexpectedCharacter {
                    // TODO:not convert input to String?
                    src: NamedSource::new(
                        "stdin", 
                        input.to_owned()), 
                    bad_bit: (character - 1, 1).into()
                }.into())
            };

            let is_err = tok.is_err();

            ret.push(tok);

            if is_err {
                break;
            }

            cho = chars.next();
            character += 1;
        }

        ret.into_iter()
    }

    fn int(
        chars: &mut dyn Iterator<Item = char>,
        character: &mut usize,
        first_digit: char,
    ) -> Token {
        let mut ret = String::from(first_digit);

        let mut cho = chars.next();
        *character += 1;

        while let Some(ch) = cho {
            match ch {
                '0'..='9' => ret.push(ch),
                _ => break
            }

            cho = chars.next();
            *character += 1;
        }

        Token::Int(ret)
    }

    #[derive(Debug, PartialEq)]
    pub enum Token {
        Int(String),
    }

    #[derive(thiserror::Error, Debug, Diagnostic, PartialEq)]
    #[error("Unexpected character")]
    pub struct UnexpectedCharacter {
        #[source_code]
        src: NamedSource<String>,

        #[label("This character")]
        bad_bit: SourceSpan
    }

    #[cfg(test)]
    mod tests {
        use super::*;

        #[test]
        fn lex_int() {
            assert_eq!(
                to_vec("32"), 
                [Token::int("32")]
            );
            assert_eq!(
                to_vec("5"), 
                [Token::int("5")]
            );
        }

        #[test]
        fn lex_multiple_tokens() {
            assert_eq!(
                to_vec("33 6"), 
                [
                    Token::int("33"), 
                    Token::int("6")
                ]
            );
            assert_eq!(
                to_vec("7 16"), 
                [
                    Token::int("7"), 
                    Token::int("16")
                ]
            );
        }

        #[test]
        fn lexing_invalid_is_error() {
            miette::set_hook(Box::new(|_| {
                Box::new(
                    miette::MietteHandlerOpts::new()
                        .color(false)
                        .build()
                )
            }))
                .unwrap();

            let token = lex("`")
                .next()
                .unwrap()
                .unwrap_err();
            assert_eq!(
                format!("{token:?}"), 
                "  \
                     × Unexpected character\n   \
                      ╭─[stdin:1:1]\n \
                    1 │ `\n   \
                      · ┬\n   \
                      · ╰── This character\n   \
                      ╰────\n\
                "
            );

            let token = lex("3\n4\n87 'sd")
                .nth(3)
                .unwrap()
                .unwrap_err();
            assert_eq!(
                format!("{token:?}"), 
                "  \
                     × Unexpected character\n   \
                      ╭─[stdin:3:4]\n \
                    2 │ 4\n \
                    3 │ 87 'sd\n   \
                      ·    ┬\n   \
                      ·    ╰── This character\n   \
                      ╰────\n\
                "
            );
        }

        impl Token {
            fn int(literal: &str) -> Self {
                Token::Int(literal.to_owned())
            }
        }

        /// Lex the supplied str, collect into a Vec, and panic if any of the
        /// tokens causes an error.
        fn to_vec(input: &str) -> Vec<Token> {
            let ret: miette::Result<Vec<Token>> =
                lex(input).collect();
            ret.expect("Lexing failed")
        }
    }
}
