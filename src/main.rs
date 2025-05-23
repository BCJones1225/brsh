mod byte_chars;
mod evaluator;
mod lexer;
mod parser;

use std::io::{Read, stdin};

use lexer::{Token, lex};

use parser::{SyntaxTree, parse};

use evaluator::eval;

fn main() -> miette::Result<()> {
    let mut buffer = String::new();
    stdin().read_to_string(&mut buffer).expect("Invalid UTF8!");
    // TODO: miette error here

    let tokens: miette::Result<Vec<Token>> = lex(&buffer).collect();

    let syntax_trees: miette::Result<Vec<SyntaxTree>> =
        parse(tokens?.into_iter()).collect();

    for value in eval(syntax_trees?.into_iter()) {
        // TODO: Nicer output format
        println!("{:?}", value?);
    }

    Ok(())
}
