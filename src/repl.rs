use crate::interpreter::Interpreter;
use crate::lexer::Lexer;
use crate::parser::Parser;
use crate::typechecker::TypeChecker;
use std::io;
use std::io::Write;

const GREEN: &str = "\x1b[32m";
const GREY: &str = "\x1b[2m";
const RESET: &str = "\x1b[0m";

pub fn repl_mode() -> anyhow::Result<()> {
    let stdin = io::stdin();

    let mut lexer = Lexer::new();
    let mut parser = Parser::new();
    let mut type_checker = TypeChecker::new(false);
    let mut interpreter = Interpreter::new();

    print_input_symbol()?;
    for line in stdin.lines() {
        //TODO: store all the lines so that the diagnostics are accurate
        if let Ok(line) = line {
            match line.as_str() {
                "" => {
                    print_input_symbol()?;
                    continue;
                }
                "quit" => return Ok(()),
                _ => {
                    let tokens = lexer.lex(&line);
                    if !lexer.diagnostics.is_empty() {
                        for diagnostic in &lexer.diagnostics {
                            diagnostic.display_diagnostic("", &line);
                        }
                        lexer = Lexer::new();
                        print_input_symbol()?;
                        continue;
                    }
                    lexer = Lexer::new();

                    let ops = parser.parse(&tokens);

                    if !parser.diagnostics.is_empty() {
                        for diagnostic in &parser.diagnostics {
                            diagnostic.display_diagnostic("", &line);
                        }
                        parser = Parser::new();
                        print_input_symbol()?;
                        continue;
                    }
                    parser = Parser::new();

                    //save the state of the stack before type checking, so we can rewind if there is an error
                    let checkpoint = type_checker.clone();
                    type_checker.type_check(&ops);

                    if !&type_checker.diagnostics.is_empty() {
                        for diagnostic in &type_checker.diagnostics {
                            diagnostic.display_diagnostic("", &line);
                        }
                        //rewind
                        type_checker = checkpoint;
                        print_input_symbol()?;
                        continue;
                    }

                    interpreter.interpret(&ops);

                    if !&interpreter.stack.is_empty() {
                        print!("{}", GREY);
                        for value in &interpreter.stack {
                            print!("{} ", value);
                        }
                        println!("{}", RESET);
                    }

                    print_input_symbol()?;
                }
            }
        } else {
            panic!()
        }
    }
    Ok(())
}

fn print_input_symbol() -> anyhow::Result<()> {
    print!("{}(≡) {}", GREEN, RESET);
    io::stdout().flush()?;
    Ok(())
}
