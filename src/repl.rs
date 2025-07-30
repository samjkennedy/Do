use crate::bytecode_interpreter::BytecodeInterpreter;
use crate::lexer::Lexer;
use crate::lowerer::Lowerer;
use crate::parser::Parser;
use crate::typechecker::{TypeChecker, TypeKind};
use std::io;
use std::io::Write;
use std::iter::zip;

const GREEN: &str = "\x1b[32m";
const GREY: &str = "\x1b[2m";
const RESET: &str = "\x1b[0m";

pub fn repl_mode() -> anyhow::Result<()> {
    let stdin = io::stdin();

    let mut lexer = Lexer::new();
    let mut parser = Parser::new();
    let mut type_checker = TypeChecker::new(false);
    let mut lowerer = Lowerer::new();
    let mut interpreter = BytecodeInterpreter::new();

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
                    let type_checker_checkpoint = type_checker.clone();
                    let typed_ops = type_checker.type_check(&ops);

                    if !&type_checker.diagnostics.is_empty() {
                        for diagnostic in &type_checker.diagnostics {
                            diagnostic.display_diagnostic("", &line);
                        }
                        //rewind
                        type_checker = type_checker_checkpoint;
                        print_input_symbol()?;
                        continue;
                    }

                    let stack_frames = lowerer.lower(&typed_ops);

                    interpreter.interpret(&stack_frames, &lowerer.constant_pool);

                    if !&interpreter.stack.is_empty() {
                        print!("{}", GREY);
                        for (value, (type_kind, _)) in
                            zip(&interpreter.stack, &type_checker.type_stack)
                        {
                            print_value(*value, type_kind, &interpreter, &type_checker);
                            print!(" ")
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

fn print_value(
    value: usize,
    type_kind: &TypeKind,
    interpreter: &BytecodeInterpreter,
    type_checker: &TypeChecker,
) {
    match type_kind {
        TypeKind::Bool => print!("{} ", if value > 0 { "true" } else { "false" }),
        TypeKind::Int => print!("{}", value),
        TypeKind::List(el_type) => {
            print!("[");
            let length = interpreter.heap[value];
            for i in 1..length + 1 {
                print_value(
                    interpreter.heap[value + i],
                    el_type,
                    interpreter,
                    type_checker,
                );
                if i < length {
                    print!(" ")
                }
            }
            print!("]");
        }
        TypeKind::Block { .. } => print!("fn"),
        TypeKind::Generic(_) => match type_checker.erase(type_kind) {
            None => print!("<?>"),
            Some(type_kind) => print_value(value, &type_kind, interpreter, type_checker),
        },
    }
}

fn print_input_symbol() -> anyhow::Result<()> {
    print!("{}(≡) {}", GREEN, RESET);
    io::stdout().flush()?;
    Ok(())
}
