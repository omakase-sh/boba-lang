mod lexer;
mod parser;
mod ast;
mod interpreter;
mod error;
mod types;
mod type_checker;

use clap::{Parser, Subcommand};
use std::fs;
use std::path::PathBuf;
use std::env;

#[derive(Parser)]
#[command(name = "boba")]
#[command(about = "Boba programming language interpreter", long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Option<Commands>,

    /// Path to the Boba source file (.bb)
    #[arg(value_name = "FILE")]
    file: Option<PathBuf>,
}

#[derive(Subcommand)]
enum Commands {
    /// Run a Boba program
    Run {
        /// Path to the Boba source file (.bb)
        #[arg(value_name = "FILE")]
        file: PathBuf,
    },
}

fn main() {
    let cli = Cli::parse();

    // Handle both formats: "boba run file.bb" and "boba file.bb"
    let file_path = match cli.command {
        Some(Commands::Run { file }) => file,
        None => {
            if let Some(file) = cli.file {
                file
            } else {
                eprintln!("Error: No file specified");
                eprintln!("Usage: boba run <FILE> or boba <FILE>");
                std::process::exit(1);
            }
        }
    };

    if !file_path.exists() {
        eprintln!("Error: File '{}' not found", file_path.display());
        std::process::exit(1);
    }

    if file_path.extension().unwrap_or_default() != "bb" {
        eprintln!("Warning: File does not have .bb extension");
    }

    match fs::read_to_string(&file_path) {
        Ok(source) => {
            println!("Running Boba program: {}", file_path.display());
            match run_program(&source) {
                Ok(_) => println!("Program executed successfully"),
                Err(e) => {
                    eprintln!("Error: {}", e);
                    std::process::exit(1);
                }
            }
        }
        Err(e) => {
            eprintln!("Error reading file: {}", e);
            std::process::exit(1);
        }
    }
}

fn run_program(source: &str) -> Result<(), String> {
    // Lexical analysis
    let tokens = match lexer::tokenize(source) {
        Ok(tokens) => tokens,
        Err(e) => return Err(format!("Lexer error: {}", e)),
    };

    // Parsing
    let ast = match parser::parse(tokens) {
        Ok(ast) => ast,
        Err(e) => return Err(format!("Parser error: {}", e)),
    };

    // Type checking
    let type_errors = type_checker::check_types(&ast);
    if !type_errors.is_empty() {
        return Err(format!("Type error: {}", type_errors[0]));
    }

    // Interpretation
    match interpreter::interpret(ast) {
        Ok(_) => Ok(()),
        Err(e) => Err(format!("Runtime error: {}", e)),
    }
}
