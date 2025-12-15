use clap::Parser;
use onion::context::eval;
use onion::parser::parse_expr;
use onion::stdlib::stdlib;
use rustyline::error::ReadlineError;
use rustyline::DefaultEditor;
use std::fs;
use std::process;

#[derive(Parser)]
#[command(name = "onion")]
#[command(version = "0.1.0")]
#[command(about = "Onion language interpreter", long_about = None)]
struct Cli {
    /// File to execute
    file: Option<String>,

    /// Evaluate expression directly
    #[arg(short, long, value_name = "EXPR")]
    eval: Option<String>,

    /// Enable debug output
    #[arg(short, long)]
    debug: bool,

    /// Arguments for the script
    #[arg(allow_hyphen_values = true)]
    args: Vec<String>,
}

fn main() {
    let cli = Cli::parse();

    if let Some(expr_str) = cli.eval {
        // Eval mode: execute expression from command line
        run_eval(&expr_str, cli.debug);
    } else if let Some(file_path) = cli.file {
        // File mode: execute file
        run_file(&file_path, cli.debug);
    } else {
        // REPL mode: interactive shell
        run_repl(cli.debug);
    }
}

fn run_eval(expr_str: &str, debug: bool) {
    let mut ctx = stdlib();
    
    match parse_expr(expr_str, &ctx) {
        Ok((_, expr)) => {
            if debug {
                eprintln!("Parsed: {:?}", expr);
            }
            let result = eval(expr, &mut ctx);
            println!("{}", result);
        }
        Err(e) => {
            eprintln!("Parse error: {:?}", e);
            process::exit(1);
        }
    }
}

fn run_file(file_path: &str, debug: bool) {
    let content = match fs::read_to_string(file_path) {
        Ok(c) => c,
        Err(e) => {
            eprintln!("Error reading file '{}': {}", file_path, e);
            process::exit(1);
        }
    };

    let mut ctx = stdlib();
    let mut input = content.trim();
    let mut last_result = onion::expr::Expr::Nil;

    while !input.is_empty() {
        match parse_expr(input, &ctx) {
            Ok((rest, expr)) => {
                if debug {
                    eprintln!("Parsed: {:?}", expr);
                }
                last_result = eval(expr, &mut ctx);
                input = rest.trim();
            }
            Err(e) => {
                eprintln!("Parse error: {:?}", e);
                eprintln!("Remaining input: {}", input);
                process::exit(1);
            }
        }
    }

    if debug {
        println!("Final result: {}", last_result);
    }
}

fn run_repl(debug: bool) {
    println!("Onion REPL v0.1.0");
    println!("Type expressions to evaluate. Press Ctrl+D to exit.");
    println!();

    let mut rl = match DefaultEditor::new() {
        Ok(editor) => editor,
        Err(e) => {
            eprintln!("Error initializing REPL: {}", e);
            process::exit(1);
        }
    };

    let mut ctx = stdlib();

    loop {
        let readline = rl.readline("λ> ");
        match readline {
            Ok(line) => {
                if line.trim().is_empty() {
                    continue;
                }

                let _ = rl.add_history_entry(line.as_str());

                match parse_expr(&line, &ctx) {
                    Ok((_, expr)) => {
                        if debug {
                            eprintln!("Parsed: {:?}", expr);
                        }
                        let result = eval(expr, &mut ctx);
                        println!("{}", result);
                    }
                    Err(e) => {
                        eprintln!("Parse error: {:?}", e);
                    }
                }
            }
            Err(ReadlineError::Interrupted) => {
                println!("^C");
                continue;
            }
            Err(ReadlineError::Eof) => {
                println!("Bye!");
                break;
            }
            Err(err) => {
                eprintln!("Error: {:?}", err);
                break;
            }
        }
    }
}
