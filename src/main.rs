use crate::emitter::FasmEmitter;
use anyhow::{Context, Error, Result};
use bytecode_interpreter::BytecodeInterpreter;
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

    //TODO: this is a stupid way to do args, use a lib to parse properly with usage
    match args.peek().map(|s| s.as_str()) {
        Some("-r") => {
            args.next(); // consume -r
            match args.peek().map(|s| s.as_str()) {
                Some(path) if path.ends_with(".do") => {
                    let input_path = args.next().unwrap();
                    compile_file(&input_path, true, &[])
                }
                Some(_) => Err(anyhow::anyhow!("Expected .do file path")),
                None => Err(anyhow::anyhow!("Unknown arguments")),
            }
        }
        Some("-i") => {
            args.next(); // consume -i
            match args.peek().map(|s| s.as_str()) {
                Some(path) if path.ends_with(".do") => {
                    let input_path = args.next().unwrap();
                    interpret_file(&input_path)
                }
                Some(_) => Err(anyhow::anyhow!("Expected .do file path")),
                None => Err(anyhow::anyhow!("Unknown arguments")),
            }
        }
        Some("-b") => {
            args.next(); // consume -i
            match args.peek().map(|s| s.as_str()) {
                Some(path) if path.ends_with(".dob") => {
                    todo!("interpreting raw .dob files")
                }
                Some(_) => Err(anyhow::anyhow!("Expected .dob file path")),
                None => Err(anyhow::anyhow!("Unknown arguments")),
            }
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
    let typed_ops = type_checker.type_check(&ops);

    if !type_checker.diagnostics.is_empty() {
        for diagnostic in type_checker.diagnostics {
            diagnostic.display_diagnostic(input_path, &input);
        }
        return Ok(());
    }

    let mut lowerer = Lowerer::new();
    let bytecode = lowerer.lower(&typed_ops);

    //TODO: allow saving and interpreting straight from dob files
    // // Derive output file names from input path
    // let input_stem = Path::new(input_path)
    //     .file_stem()
    //     .and_then(|s| s.to_str())
    //     .ok_or_else(|| anyhow::anyhow!("Invalid input file path"))?;
    //
    // let dob_file = format!("{}.dob", input_stem);
    // {
    //     let mut file = File::create(&dob_file)?;
    //     let mut i = 0;
    //     for (_, function) in &bytecode {
    //         for op in function {
    //             for word in op.to_binary() {
    //                 write!(file, "{:04x} ", word)?;
    //                 i += 1;
    //                 if i == 8 {
    //                     writeln!(file)?;
    //                     i = 0;
    //                 }
    //             }
    //         }
    //     }
    // }

    let mut bytecode_interpreter = BytecodeInterpreter::new();

    bytecode_interpreter.interpret(&bytecode, &lowerer.constant_pool);

    Ok(())
}

fn interpret_bytecode(input_path: &String) -> Result<(), Error> {
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

    //TODO: allow saving and interpreting straight from dob files
    // // Derive output file names from input path
    // let input_stem = Path::new(input_path)
    //     .file_stem()
    //     .and_then(|s| s.to_str())
    //     .ok_or_else(|| anyhow::anyhow!("Invalid input file path"))?;
    //
    // let dob_file = format!("{}.dob", input_stem);
    // {
    //     let mut file = File::create(&dob_file)?;
    //     let mut i = 0;
    //     for (_, function) in &bytecode {
    //         for op in function {
    //             for word in op.to_binary() {
    //                 write!(file, "{:04x} ", word)?;
    //                 i += 1;
    //                 if i == 8 {
    //                     writeln!(file)?;
    //                     i = 0;
    //                 }
    //             }
    //         }
    //     }
    // }

    let mut bytecode_interpreter = BytecodeInterpreter::new();

    bytecode_interpreter.interpret(&bytecode, &lowerer.constant_pool);

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
        let output = Command::new("fasm")
            .arg(&asm_file)
            .output()
            .expect("failed to execute fasm");
        print!("{}", String::from_utf8(output.stdout)?);
        eprint!("{}", String::from_utf8(output.stderr)?);
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
