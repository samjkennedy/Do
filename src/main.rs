use anyhow::{Context, Error, Result};
use interpreter::Interpreter;
use lexer::{Lexer, Token};
use parser::Parser;
use std::{env, fs};
use typechecker::TypeChecker;

mod diagnostic;
mod interpreter;
mod lexer;
mod parser;
mod repl;
mod typechecker;

fn main() -> Result<()> {
    let mut args = env::args().skip(1);

    match args.next() {
        Some(input_path) => interpret_file(&input_path),
        None => repl::repl_mode(),
    }
}

fn interpret_file(input_path: &String) -> Result<(), Error> {
    let input = fs::read_to_string(input_path)
        .with_context(|| format!("Failed to read input file `{}`", input_path))?;

    let mut lexer = Lexer::new();

    let tokens: Vec<Token> = lexer.lex(&input);

    if !lexer.diagnostics.is_empty() {
        for diagnostic in lexer.diagnostics {
            diagnostic.display_diagnostic(input_path, &input);
        }
        return Ok(());
    }

    let mut parser = Parser::new();
    let ops = parser.parse(&tokens);

    if !parser.diagnostics.is_empty() {
        for diagnostic in parser.diagnostics {
            diagnostic.display_diagnostic(input_path, &input);
        }
        return Ok(());
    }

    let mut type_checker = TypeChecker::new(true);
    type_checker.type_check(&ops);

    if !type_checker.diagnostics.is_empty() {
        for diagnostic in type_checker.diagnostics {
            diagnostic.display_diagnostic(input_path, &input);
        }
        return Ok(());
    }

    let mut interpreter = Interpreter::new();
    interpreter.interpret(&ops);

    Ok(())
}
