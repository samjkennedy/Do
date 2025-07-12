use interpreter::Interpreter;
use lexer::{Lexer, Token};
use parser::{Op, Parser};
use typechecker::TypeChecker;
use std::{env, fs};
use anyhow::{Context, Result};

mod lexer;
mod parser;
mod typechecker;
mod interpreter;

fn main() -> Result<()>  {
    let mut args = env::args().skip(1).peekable();

    let input_path = args
        .next()
        .context("Usage: do <input>.do")?;

    let input = fs::read_to_string(&input_path)
        .with_context(|| format!("Failed to read input file `{}`", input_path))?;

    let mut lexer = Lexer::new(input);

    let mut tokens: Vec<Token> = vec![];
    while let Some(token) = lexer.next() {
        // println!("{:?}", token);
        tokens.push(token);
    }

    let mut parser = Parser::new(tokens);
    let mut ops: Vec<Op> = vec![];
    while let Some(op) = parser.parse_op() {
        // println!("{:?}", op);
        ops.push(op);
    }

    let mut type_checker = TypeChecker::new();
    type_checker.type_check(&ops);

    let mut interpreter = Interpreter::new();
    interpreter.interpret(&ops);

    Ok(())
}
