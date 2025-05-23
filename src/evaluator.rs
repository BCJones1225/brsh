use miette::{Diagnostic, NamedSource, SourceSpan};

use crate::parser::{LeafToken, OperatorToken, SyntaxTree};

pub fn eval<I>(input: I) -> Values<I>
where
    I: Iterator<Item = SyntaxTree>,
{
    Values { input }
}

pub struct Values<I>
where
    I: Iterator<Item = SyntaxTree>,
{
    input: I,
}

impl<I> Values<I>
where
    I: Iterator<Item = SyntaxTree>,
{
    fn eval_next(&mut self) -> Option<miette::Result<Value>> {
        let tree = self.input.next()?;
        Some(self.eval_tree(tree))
    }

    fn eval_tree(&mut self, tree: SyntaxTree) -> miette::Result<Value> {
        match tree {
            SyntaxTree::Leaf(leaf) => self.leaf(leaf),
            SyntaxTree::Operation {
                operator,
                left,
                right,
            } => self.operation(operator, *left, *right),
        }
    }

    fn leaf(&mut self, leaf: LeafToken) -> miette::Result<Value> {
        match leaf {
            LeafToken::Int(literal) => literal
                .parse::<i32>()
                .map_err(|_e| {
                    InvalidNumber {
                        // TODO: src: NamedSource::new("stdin", self.input.to_owned()),
                        src: NamedSource::new("stdin", "todo".to_owned()),
                        // TODO: bad_bit: token.span.into()
                        bad_bit: (1, 1).into(),
                    }
                    .into()
                })
                .map(|num| -> Value { Value::I32(num) }),
        }
    }

    fn operation(
        &mut self,
        operator: OperatorToken,
        left: SyntaxTree,
        right: SyntaxTree,
    ) -> miette::Result<Value> {
        let left = self.eval_tree(left)?;
        let right = self.eval_tree(right)?;

        // TODO: operator should not be a token.
        match operator {
            OperatorToken::Plus(_) => match (left, right) {
                (Value::I32(left), Value::I32(right)) => {
                    Ok(Value::I32(left + right))
                }
            },
        }
    }
}

impl<I> Iterator for Values<I>
where
    I: Iterator<Item = SyntaxTree>,
{
    type Item = miette::Result<Value>;

    fn next(&mut self) -> Option<Self::Item> {
        self.eval_next()
    }
}

#[derive(Debug, PartialEq)]
pub enum Value {
    I32(i32),
}

// TODO: Better error description -- copy rustc
#[derive(thiserror::Error, Debug, Diagnostic, PartialEq)]
#[error("Number could not be evaluated")]
pub struct InvalidNumber {
    #[source_code]
    src: NamedSource<String>,

    #[label("Here")]
    bad_bit: SourceSpan,
}

#[cfg(test)]
mod test {
    use crate::{
        lexer::{Token, lex},
        parser::parse,
    };

    use super::*;

    #[test]
    fn evaluating_no_trees_gives_no_results() {
        assert_eq!(to_vec(""), &[]);
    }

    #[test]
    fn evaluating_integer_gives_i32() {
        assert_eq!(to_vec("35"), &[Value::I32(35)]);
    }

    #[test]
    fn evaluating_addition_gives_sum() {
        assert_eq!(to_vec("81 + 2"), &[Value::I32(83)]);
    }

    fn to_vec(input: &str) -> Vec<Value> {
        let tokens: miette::Result<Vec<Token>> = lex(input).collect();

        let syntax_trees: miette::Result<Vec<SyntaxTree>> =
            parse(tokens.expect("Lexing error").into_iter()).collect();

        let values: miette::Result<Vec<Value>> =
            eval(syntax_trees.expect("Parsing error").into_iter()).collect();

        values.expect("Evaluation error")
    }
}
