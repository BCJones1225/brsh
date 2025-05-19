mod byte_chars;

use std::io::{Read, stdin};

use lexer::{Token, lex};

use parser::parse;

fn main() -> miette::Result<()> {
    let mut buffer = String::new();
    stdin().read_to_string(&mut buffer).expect("Invalid UTF8!");
    // TODO: miette error here

    let tokens: miette::Result<Vec<Token>> = lex(&buffer).collect();
    for token in parse(tokens?.into_iter()) {
        println!("{:?}", token?);
    }

    Ok(())
}

mod parser {
    use miette::{Diagnostic, NamedSource, SourceSpan};

    use crate::lexer::Token;

    pub fn parse<I>(input: I) -> Parse<I>
    where
        I: Iterator<Item = Token>,
    {
        Parse { input }
    }

    pub struct Parse<I>
    where
        I: Iterator<Item = Token>,
    {
        input: I,
    }

    impl<I> Parse<I>
    where
        I: Iterator<Item = Token>,
    {
        fn parse_next(&mut self) -> Option<miette::Result<SyntaxTree>> {
            if let Some(token) = self.input.next() {
                Some(match token {
                    Token::Int(_) => self.leaf_or_opn(token),

                    // Anything else, we don't recognize yet.
                    _ => Err(UnexpectedToken {
                        // TODO: src: NamedSource::new("stdin", self.input.to_owned()),
                        src: NamedSource::new("stdin", "todo".to_owned()),
                        // TODO: bad_bit: token.span.into()
                        bad_bit: (1, 1).into(),
                    }
                    .into()),
                })
            } else {
                None
            }
        }

        fn leaf_or_opn(&mut self, token: Token) -> miette::Result<SyntaxTree> {
            if let Some(opr) = self.input.next() {
                match opr {
                    // e.g., 3 +
                    Token::Operator(_) => {
                        if let Some(rhs) = self.input.next() {
                            match rhs {
                                // e.g., 3 + 3
                                Token::Int(_) => Ok(SyntaxTree::Operation {
                                    operator: opr,
                                    left: Box::new(SyntaxTree::Leaf(token)),
                                    right: Box::new(SyntaxTree::Leaf(rhs)),
                                }),

                                // e.g., 3 + + is not valid.
                                Token::Operator(_) => Err(UnexpectedToken {
                                    // TODO: src: NamedSource::new("stdin", self.input.to_owned()),
                                    src: NamedSource::new(
                                        "stdin",
                                        "todo".to_owned(),
                                    ),
                                    // TODO: bad_bit: token.span.into()
                                    bad_bit: (1, 1).into(),
                                }
                                .into()),
                            }
                        } else {
                            // e.g., 3 + (EOF) is not valid syntax.
                            Err(UnexpectedEof {
                                // TODO: src: NamedSource::new("stdin", self.input.to_owned()),
                                src: NamedSource::new(
                                    "stdin",
                                    "todo".to_owned(),
                                ),
                                // TODO: bad_bit: token.span.into()
                                bad_bit: (1, 1).into(),
                            }
                            .into())
                        }
                    }

                    // 3 3 is not valid syntax.
                    Token::Int(_) => Err(UnexpectedToken {
                        // TODO: src: NamedSource::new("stdin", self.input.to_owned()),
                        src: NamedSource::new("stdin", "todo".to_owned()),
                        // TODO: bad_bit: token.span.into()
                        bad_bit: (1, 1).into(),
                    }
                    .into()),
                }
            } else {
                Ok(SyntaxTree::Leaf(token))
            }
        }
    }

    impl<I> Iterator for Parse<I>
    where
        I: Iterator<Item = Token>,
    {
        type Item = miette::Result<SyntaxTree>;

        fn next(&mut self) -> Option<Self::Item> {
            self.parse_next()
        }
    }

    #[derive(Debug, PartialEq)]
    pub enum SyntaxTree {
        Leaf(Token),
        Operation {
            operator: Token,
            left: Box<SyntaxTree>,
            right: Box<SyntaxTree>,
        },
    }

    #[derive(thiserror::Error, Debug, Diagnostic, PartialEq)]
    #[error("Unexpected token")]
    pub struct UnexpectedToken {
        #[source_code]
        src: NamedSource<String>,

        #[label("This token")]
        bad_bit: SourceSpan,
    }

    #[derive(thiserror::Error, Debug, Diagnostic, PartialEq)]
    #[error("Reached end of file mid-expression")]
    pub struct UnexpectedEof {
        #[source_code]
        src: NamedSource<String>,

        #[label("Here")]
        bad_bit: SourceSpan,
    }

    #[cfg(test)]
    mod test {
        use crate::lexer::lex;

        use super::*;

        #[test]
        fn parsing_nothing_returns_empty_list() {
            assert_eq!(to_vec(""), &[]);
        }

        #[test]
        fn parsing_number_produces_leaf_expression() {
            assert_eq!(
                to_vec("51"),
                &[SyntaxTree::Leaf(Token::Int("51".to_owned()))]
            );
        }

        #[test]
        fn parsing_sum_produces_opr_expression() {
            assert_eq!(
                to_vec("82 + 1"),
                &[SyntaxTree::Operation {
                    operator: Token::opn("+"),
                    left: Box::new(SyntaxTree::Leaf(Token::int("82"))),
                    right: Box::new(SyntaxTree::Leaf(Token::int("1")))
                }]
            );
        }

        fn to_vec(input: &str) -> Vec<SyntaxTree> {
            let tokens: miette::Result<Vec<_>> = lex(input).collect();
            let ret: miette::Result<Vec<SyntaxTree>> =
                parse(tokens.expect("Lexing failed!").into_iter()).collect();
            ret.expect("Parsing failed")
        }
    }
}

mod lexer {

    use miette::{Diagnostic, NamedSource, SourceSpan};

    use crate::byte_chars::ByteChars;

    pub fn lex<'a>(input: &'a str) -> Lex<'a> {
        Lex::new(input)
    }

    pub struct Lex<'a> {
        input: &'a str,
        chars: ByteChars<'a>,
        hit_error: bool,
    }

    impl<'a> Lex<'a> {
        fn new(input: &'a str) -> Self {
            Self {
                input,
                chars: ByteChars::new(input),
                hit_error: false,
            }
        }

        fn lex_next(&mut self) -> Option<miette::Result<Token>> {
            if self.hit_error {
                return None;
            }

            if let Some(ch) = self.chars.next() {
                let tok = match ch {
                    // Integers start with a digit.
                    '0'..='9' => Ok(self.int(ch)),

                    // Operators.
                    op @ '+' => Ok(Token::Operator(op.to_string())),

                    // Skip spaces -- just recurse into this function for the next character.
                    ' ' => return self.lex_next(),

                    // Anything else, we don't recognize yet.
                    _ => Err(UnexpectedCharacter {
                        // TODO:not convert input to String?
                        src: NamedSource::new("stdin", self.input.to_owned()),
                        bad_bit: (self.chars.bytes() - 1, 1).into(),
                    }
                    .into()),
                };

                if tok.is_err() {
                    self.hit_error = true;
                }

                Some(tok)
            } else {
                None
            }
        }

        fn int(&mut self, first_digit: char) -> Token {
            let mut ret = String::from(first_digit);

            while let Some(ch) = self.chars.next() {
                match ch {
                    '0'..='9' => ret.push(ch),
                    _ => break,
                }
            }

            Token::Int(ret)
        }
    }

    impl<'a> Iterator for Lex<'a> {
        type Item = miette::Result<Token>;

        fn next(&mut self) -> Option<Self::Item> {
            self.lex_next()
        }
    }

    #[derive(Debug, PartialEq)]
    pub enum Token {
        Int(String),
        Operator(String),
    }

    impl Token {
        pub fn int(literal: &str) -> Self {
            Token::Int(literal.to_owned())
        }

        pub fn opn(literal: &str) -> Self {
            Token::Operator(literal.to_owned())
        }
    }

    #[derive(thiserror::Error, Debug, Diagnostic, PartialEq)]
    #[error("Unexpected character")]
    pub struct UnexpectedCharacter {
        #[source_code]
        src: NamedSource<String>,

        #[label("This character")]
        bad_bit: SourceSpan,
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
        fn lex_add_expr() {
            assert_eq!(
                to_vec("34 + 61"),
                [Token::int("34"), Token::opn("+"), Token::int("61")]
            );
            assert_eq!(
                to_vec("8 + 160"),
                [Token::int("8"), Token::opn("+"), Token::int("160")]
            );
        }

        #[test]
        fn stop_producing_tokens_when_is_error() {
            assert_eq!(lex("33 6 ' 5 5").count(), 3);
            assert_eq!(lex("7 16 52 ' 9 8").count(), 4);
        }

        #[test]
        fn lexing_invalid_is_error() {
            miette::set_hook(Box::new(|_| {
                Box::new(miette::MietteHandlerOpts::new().color(false).build())
            }))
            .unwrap();

            let token = lex("`").next().unwrap().unwrap_err();
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

            let token = lex("3\n4\n87 'sd").nth(3).unwrap().unwrap_err();
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

        /// Lex the supplied str, collect into a Vec, and panic if any of the
        /// tokens causes an error.
        fn to_vec(input: &str) -> Vec<Token> {
            let ret: miette::Result<Vec<Token>> = lex(input).collect();
            ret.expect("Lexing failed")
        }
    }
}
