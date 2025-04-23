use std::io::stdin;

use lexer::lex;

fn main() {
    for token in lex(stdin()) {
        println!("{:?}", token);
    };
}

mod lexer {
    use std::{char, io::{BufReader, Read}};

    pub fn lex(input: impl Read) -> impl Iterator<Item = Token> {
        let mut input = BufReader::new(input);
        let mut buffer = String::new();
        input.read_to_string(&mut buffer).unwrap();
        
        let mut chars = buffer.chars();
        
        let mut ret = Vec::new();
        while let Some(ch) = chars.next() {
            let tok = match ch {
                first_digit @ '0'..='9' => int(&mut chars, first_digit),
                _ => panic!("Unexpected input: {ch}")
            };

            ret.push(tok);
        }

        ret.into_iter()
    }

    fn int(chars: &mut dyn Iterator<Item = char>, first_digit: char) -> Token {
        let mut ret = String::from(first_digit);
        
        while let Some(ch) = chars.next() {
            match ch {
                next_digit @ '0'..='9' => ret.push(next_digit),
                _ => break
            }
        }

        Token::Int(ret)
    }

    #[derive(Debug, PartialEq)]
    pub enum Token {
        Int(String) 
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

        // #[test]
        // fn lexing_non_utf8_is_error() {
        //     assert_eq!(to_vec("33 6"), [Token::int("33"), Token::int("6")]);
        //     assert_eq!(to_vec("7 16"), [Token::int("7"), Token::int("16")]);
        // }

        impl Token {
            fn int(literal: &str) -> Self {
                Token::Int(literal.to_owned())
            }
        }

        fn to_vec(input: &str) -> Vec<Token> {
           lex(input.as_bytes()).collect()
        }
    }
}
