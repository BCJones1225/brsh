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
        let token = self.input.next()?;
        Some(match LeafToken::try_from(token) {
            Ok(leaf_token) => match leaf_token {
                LeafToken::Int(_) => self.leaf_or_operation(leaf_token),
            },
            // Anything else, we don't recognize yet.
            Err(_) => Err(UnexpectedToken {
                // TODO: src: NamedSource::new("stdin", self.input.to_owned()),
                src: NamedSource::new("stdin", "todo".to_owned()),
                // TODO: bad_bit: token.span.into()
                bad_bit: (1, 1).into(),
            }
            .into()),
        })
    }

    fn leaf_or_operation(
        &mut self,
        leaf_token: LeafToken,
    ) -> miette::Result<SyntaxTree> {
        if let Some(token) = self.input.next() {
            match OperatorToken::try_from(token) {
                Ok(operator_token) => {
                    self.operation_right_operand(leaf_token, operator_token)
                }

                // e.g., 3 3 is not valid.
                Err(_) => Err(UnexpectedToken {
                    // TODO: src: NamedSource::new("stdin", self.input.to_owned()),
                    src: NamedSource::new("stdin", "todo".to_owned()),
                    // TODO: bad_bit: token.span.into()
                    bad_bit: (1, 1).into(),
                }
                .into()),
            }
        } else {
            Ok(SyntaxTree::Leaf(leaf_token))
        }
    }

    fn operation_right_operand(
        &mut self,
        left: LeafToken,
        operator: OperatorToken,
    ) -> miette::Result<SyntaxTree> {
        if let Some(right) = self.input.next() {
            let right_leaf = LeafToken::try_from(right);
            match right_leaf {
                // e.g., 3 + 3
                Ok(right_leaf) => Ok(SyntaxTree::Operation {
                    operator,
                    left: Box::new(SyntaxTree::Leaf(left)),
                    right: Box::new(SyntaxTree::Leaf(right_leaf)),
                }),

                // e.g., 3 + + is not valid.
                Err(_) => Err(UnexpectedToken {
                    // TODO: src: NamedSource::new("stdin", self.input.to_owned()),
                    src: NamedSource::new("stdin", "todo".to_owned()),
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
    Leaf(LeafToken),
    Operation {
        operator: OperatorToken,
        left: Box<SyntaxTree>,
        right: Box<SyntaxTree>,
    },
}

#[derive(Debug, PartialEq)]
pub enum LeafToken {
    Int(String),
}

impl LeafToken {
    #[cfg(test)]
    pub fn int(literal: &str) -> Self {
        Self::Int(literal.to_owned())
    }
}

impl TryFrom<Token> for LeafToken {
    type Error = NotLeafToken;

    fn try_from(value: Token) -> Result<Self, Self::Error> {
        match value {
            Token::Int(i) => Ok(Self::Int(i)),
            Token::Operator(_) => Err(NotLeafToken),
        }
    }
}

pub struct NotLeafToken;

#[derive(Debug, PartialEq)]
pub enum OperatorToken {
    Plus(String),
}

impl OperatorToken {
    #[cfg(test)]
    pub fn operation(literal: &str) -> Self {
        Self::Plus(literal.to_owned())
    }
}

impl TryFrom<Token> for OperatorToken {
    type Error = NotOperatorToken;

    fn try_from(value: Token) -> Result<Self, Self::Error> {
        match value {
            Token::Int(_) => Err(NotOperatorToken),
            Token::Operator(literal) => Ok(OperatorToken::Plus(literal)),
        }
    }
}

pub struct NotOperatorToken;

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
        assert_eq!(to_vec("51"), &[SyntaxTree::Leaf(LeafToken::int("51"))]);
    }

    #[test]
    fn parsing_sum_produces_opr_expression() {
        assert_eq!(
            to_vec("82 + 1"),
            &[SyntaxTree::Operation {
                operator: OperatorToken::operation("+"),
                left: Box::new(SyntaxTree::Leaf(LeafToken::int("82"))),
                right: Box::new(SyntaxTree::Leaf(LeafToken::int("1")))
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
