use clap::Parser;
use onion::context::eval;
use onion::parser::{convert_error_to_string, parse_expr};
use onion::stdlib::stdlib;
use std::fs;
use std::path::PathBuf;

#[derive(Parser)]
#[command(author, version, about, long_about = None)]
struct Cli {
    /// Optional file to run
    file: Option<PathBuf>,
}

fn main() {
    let cli = Cli::parse();

    let mut ctx = stdlib();

    if let Some(file_path) = cli.file {
        let content = fs::read_to_string(&file_path).expect("Failed to read file");
        let mut input = content.as_str();

        while !input.trim().is_empty() {
            match parse_expr(input, &ctx) {
                Ok((rest, expr)) => {
                    eval(expr, &mut ctx);
                    input = rest;
                }
                Err(nom::Err::Error(e)) | Err(nom::Err::Failure(e)) => {
                    println!("Parse error:\n{}", convert_error_to_string(input, e));
                    return;
                }
                Err(nom::Err::Incomplete(_)) => {
                    println!("Parse incomplete");
                    return;
                }
            }
        }
    } else {
        println!("Onion REPL");
        println!("(Press Ctrl+C to exit)");

        let mut rl = rustyline::DefaultEditor::new().unwrap();

        loop {
            match rl.readline(">> ") {
                Ok(line) => {
                    let _ = rl.add_history_entry(line.as_str());
                    let input = line.as_str();
                    match parse_expr(input, &ctx) {
                        Ok((_, expr)) => {
                            let res = eval(expr, &mut ctx);
                            println!("{}", res);
                        }
                        Err(nom::Err::Error(e)) | Err(nom::Err::Failure(e)) => {
                            println!("Parse error:\n{}", convert_error_to_string(input, e));
                        }
                        Err(nom::Err::Incomplete(_)) => {
                            println!("Parse incomplete");
                        }
                    }
                }
                Err(rustyline::error::ReadlineError::Interrupted) => {
                    println!("CTRL-C");
                    break;
                }
                Err(rustyline::error::ReadlineError::Eof) => {
                    println!("CTRL-D");
                    break;
                }
                Err(err) => {
                    println!("Error: {:?}", err);
                    break;
                }
            }
        }
    }
}
