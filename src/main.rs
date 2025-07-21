use crate::emitter::FasmEmitter;
use anyhow::{Context, Error, Result};
use interpreter::Interpreter;
use lexer::{Lexer, Token};
use lowerer::Lowerer;
use parser::Parser;
use std::fs::File;
use std::path::Path;
use std::process::Command;
use std::{env, fs};
use typechecker::TypeChecker;

mod bytecode_interpreter;
mod diagnostic;
mod emitter;
mod interpreter;
mod lexer;
mod lowerer;
mod parser;
mod repl;
mod typechecker;

fn main() -> Result<()> {
    let mut args = env::args().skip(1).peekable();

    //TODO: this is an ass way to do args
    match args.peek().map(|s| s.as_str()) {
        Some("-r") => {
            args.next(); // consume -r
            let input_path = args
                .next()
                .ok_or_else(|| anyhow::anyhow!("Expected file path after -r"))?;
            let remaining_args: Vec<String> = args.collect();
            compile_file(&input_path, true, &remaining_args)
        }
        Some("-i") => {
            args.next(); // consume -i
            let input_path = args
                .next()
                .ok_or_else(|| anyhow::anyhow!("Expected file path after -i"))?;
            interpret_file(&input_path)
        }
        Some(path) if path.ends_with(".do") => {
            let input_path = args.next().unwrap();
            compile_file(&input_path, false, &[])
        }
        None => repl::repl_mode(),
        _ => Err(anyhow::anyhow!("Unknown arguments")),
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

fn compile_file(input_path: &String, run: bool, args: &[String]) -> Result<(), Error> {
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
    let typed_ops = type_checker.type_check(&ops);

    if !type_checker.diagnostics.is_empty() {
        for diagnostic in type_checker.diagnostics {
            diagnostic.display_diagnostic(input_path, &input);
        }
        return Ok(());
    }

    let mut lowerer = Lowerer::new();
    let bytecode = lowerer.lower(&typed_ops);

    // Derive output file names from input path
    let input_stem = Path::new(input_path)
        .file_stem()
        .and_then(|s| s.to_str())
        .ok_or_else(|| anyhow::anyhow!("Invalid input file path"))?;

    let asm_file = format!("{}.asm", input_stem);
    let exe_file = format!("{}.exe", input_stem);

    //perform emitting in a block to close the asm file
    {
        let file = File::create(&asm_file)?;
        let mut emitter = FasmEmitter::new(file);

        emitter.emit(&bytecode, &lowerer.constant_pool)?;
    }

    {
        Command::new("fasm")
            .arg(&asm_file)
            .output()
            .expect("failed to execute fasm");
    }

    if run {
        let output = Command::new(format!("./{}", exe_file))
            .args(args)
            .output()
            .expect("failed to execute compiled program");

        print!("{}", String::from_utf8(output.stdout)?);
    }

    Ok(())
}
