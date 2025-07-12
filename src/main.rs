use anyhow::{Context, Result, anyhow};
use interpreter::Interpreter;
use lexer::{Lexer, Token};
use parser::{Op, Parser};
use std::{env, fs};
use typechecker::TypeChecker;

mod diagnostic;
mod interpreter;
mod lexer;
mod parser;
mod typechecker;

fn main() -> Result<()> {
    let mut args = env::args().skip(1).peekable();

    let input_path = args.next().context("Usage: do <input>.do")?;

    let input = fs::read_to_string(&input_path)
        .with_context(|| format!("Failed to read input file `{}`", input_path))?;

    let mut lexer = Lexer::new(input);

    let mut tokens: Vec<Token> = vec![];
    while let Some(token) = lexer.next() {
        // println!("{:?}", token);
        tokens.push(token);
    }

    if !lexer.diagnostics.is_empty() {
        for diagnostic in lexer.diagnostics {
            diagnostic.display_diagnostic(&input_path);
        }
        return Err(anyhow!("Failed to tokenize input"));
    }

    let mut parser = Parser::new(tokens);
    let mut ops: Vec<Op> = vec![];
    while let Some(op) = parser.parse_op() {
        // println!("{:?}", op);
        ops.push(op);
    }

    let mut type_checker = TypeChecker::new();
    type_checker.type_check(&ops);

    if !type_checker.diagnostics.is_empty() {
        for diagnostic in type_checker.diagnostics {
            diagnostic.display_diagnostic(&input_path);
        }
        return Err(anyhow!("Failed to type check input"));
    }

    let mut interpreter = Interpreter::new();
    interpreter.interpret(&ops);

    Ok(())
}
