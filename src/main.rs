use anyhow::{Context, Error, Result};
use interpreter::Interpreter;
use lexer::{Lexer, Token};
use parser::{Op, Parser};
use std::{env, fs, io};
use std::io::Write;
use typechecker::TypeChecker;

mod diagnostic;
mod interpreter;
mod lexer;
mod parser;
mod typechecker;

const CYAN: &str = "\x1b[36m";
const RESET: &str = "\x1b[0m";

fn main() -> Result<()> {
    let mut args = env::args().skip(1).peekable();

    match args.next() {
        Some(input_path) => {
            interpret_file(&input_path)
        },
        None => {
            let stdin = io::stdin();

            let mut lexer = Lexer::new();
            let mut parser = Parser::new();
            let mut type_checker = TypeChecker::new(false);
            let mut interpreter = Interpreter::new();

            print!("{}>>> {}", CYAN, RESET);
            io::stdout().flush()?;
            for line in stdin.lines() {
                //TODO: store all the lines so that the diagnostics are accurate
                if let Ok(line) = line {

                    match line.as_str() {
                        "" => continue,
                        "quit" => return Ok(()),
                        _ => {
                            let tokens = lexer.lex(&line);
                            if !lexer.diagnostics.is_empty() {
                                for diagnostic in &lexer.diagnostics {
                                    diagnostic.display_diagnostic(&"", &line);
                                }
                                print!("{}>>> {}", CYAN, RESET);
                                io::stdout().flush()?;
                                continue;
                            }
                            lexer = Lexer::new();

                            let ops = parser.parse(&tokens);

                            if !parser.diagnostics.is_empty() {
                                for diagnostic in &parser.diagnostics {
                                    diagnostic.display_diagnostic(&"", &line);
                                }
                                print!("{}>>> {}", CYAN, RESET);
                                io::stdout().flush()?;
                                continue;
                            }
                            parser = Parser::new();

                            //save the state of the stack before type checking, so we can rewind if there is an error
                            let checkpoint = type_checker.clone();
                            type_checker.type_check(&ops);

                            if !&type_checker.diagnostics.is_empty() {
                                for diagnostic in &type_checker.diagnostics {
                                    diagnostic.display_diagnostic(&"", &line);
                                }
                                //rewind
                                type_checker = checkpoint;
                                print!("{}>>> {}", CYAN, RESET);
                                io::stdout().flush()?;
                                continue;
                            }
                            interpreter.interpret(&ops);

                            print!("{}>>> {}", CYAN, RESET);
                            io::stdout().flush()?;
                        }
                    }
                } else {
                    //TODO: worry about Error case later
                    panic!()
                }
            }
            todo!()
        }
    }
}

fn interpret_file(input_path: &String) -> Result<(), Error> {
    let input = fs::read_to_string(&input_path)
        .with_context(|| format!("Failed to read input file `{}`", input_path))?;

    let mut lexer = Lexer::new();

    let tokens: Vec<Token> = lexer.lex(&input);

    if !lexer.diagnostics.is_empty() {
        for diagnostic in lexer.diagnostics {
            diagnostic.display_diagnostic(&input_path, &input);
        }
        return Ok(());
    }

    let mut parser = Parser::new();
    let mut ops = parser.parse(&tokens);

    if !parser.diagnostics.is_empty() {
        for diagnostic in parser.diagnostics {
            diagnostic.display_diagnostic(&input_path, &input);
        }
        return Ok(());
    }

    let mut type_checker = TypeChecker::new(true);
    type_checker.type_check(&ops);

    if !type_checker.diagnostics.is_empty() {
        for diagnostic in type_checker.diagnostics {
            diagnostic.display_diagnostic(&input_path, &input);
        }
        return Ok(());
    }

    let mut interpreter = Interpreter::new();
    interpreter.interpret(&ops);

    Ok(())
}
