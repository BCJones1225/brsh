use std::io::{Read, stdin};

use lexer::lex;

fn main() {
    let mut buffer = String::new();
    stdin().read_to_string(&mut buffer).expect("Invalid UTF8!");
    // TODO: miette error here

    for token in lex(&buffer) {
        println!("{:?}", token);
    }
}

mod lexer {
    pub fn lex(
        input: &str,
    ) -> impl Iterator<Item = Result<Token, LexingError>> {
let mut chars = input.chars();
        let mut line = 1;
        let mut character = 1;

        let mut ret = Vec::new();

        while let Some(ch) = chars.next() {
            let tok = match ch {
                '0'..='9' => Ok(int(&mut chars, &mut line, &mut character, ch)),
                _ => Err(LexingError::from(format!(
                    "\
                        Unexpected character: \"{ch}\". \
                        Line: {line}, character: {character}.\
                    "
                ))),
            };

            let is_err = if let Err(_) = &tok { true } else { false };

            ret.push(tok);

            if is_err {
                break;
            }
        }

        ret.into_iter()
    }

    fn int(
        chars: &mut dyn Iterator<Item = char>,
        line: &mut usize,
        character: &mut usize,
        first_digit: char,
    ) -> Token {
        *character += 1;
        let mut ret = String::from(first_digit);

        while let Some(ch) = chars.next() {
            match ch {
                '0'..='9' => ret.push(ch),
                _ => { 
                    if ch == '\n' {
                        *line += 1;
                        *character = 1;
                    } else {
                        *character += 1;
                    }
                    break;
                }
            }
            *character += 1;
        }

        Token::Int(ret)
    }

    #[derive(Debug, PartialEq)]
    pub enum Token {
        Int(String),
    }

    #[derive(Debug, PartialEq)]
    pub struct LexingError {
        message: String,
    }

    impl From<&str> for LexingError {
        fn from(value: &str) -> Self {
            Self {
                message: value.to_owned(),
            }
        }
    }
    impl From<String> for LexingError {
        fn from(value: String) -> Self {
            Self { message: value }
        }
    }

    #[cfg(test)]
    mod tests {
        use super::*;

        #[test]
        fn lex_int() {
            assert_eq!(to_vec("32"), [Token::int("32")]);
            assert_eq!(to_vec("5"), [Token::int("5")]);
        }

        #[test]
        fn lex_multiple_tokens() {
            assert_eq!(to_vec("33 6"), [Token::int("33"), Token::int("6")]);
            assert_eq!(to_vec("7 16"), [Token::int("7"), Token::int("16")]);
        }

        #[test]
        fn lexing_invalid_is_error() {
            let tokens: Vec<_> = lex("3\n4\n87 'sd").collect();
            assert_eq!(
                tokens,
                &[
                    Ok(Token::int("3")),
                    Ok(Token::int("4")),
                    Ok(Token::int("87")),
                    Err(LexingError::from(
                        "\
                            Unexpected character: \"'\". \
                            Line: 3, character: 4.\
                        "
                    ))
                ]
            );

            let tokens: Vec<_> = lex("`sd").collect();
            assert_eq!(
                tokens,
                &[
                    Err(LexingError::from(
                        "\
                            Unexpected character: \"`\". \
                            Line: 1, character: 1.\
                        "
                    ))
                ]
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
            let ret: Result<Vec<Token>, LexingError> = lex(input).collect();
            ret.expect("Lexing failed")
        }
    }
}
