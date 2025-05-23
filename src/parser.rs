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
        self.input.next().map(|token| match token {
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
                            src: NamedSource::new("stdin", "todo".to_owned()),
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
