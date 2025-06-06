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

        for ch in self.chars.by_ref() {
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
    #[cfg(test)]
    pub fn int(literal: &str) -> Self {
        Token::Int(literal.to_owned())
    }

    #[cfg(test)]
    pub fn operation(literal: &str) -> Self {
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
            [Token::int("34"), Token::operation("+"), Token::int("61")]
        );
        assert_eq!(
            to_vec("8 + 160"),
            [Token::int("8"), Token::operation("+"), Token::int("160")]
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
