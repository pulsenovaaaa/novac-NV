#![allow(unused)]
use std::{path::PathBuf, process::exit};

use crate::{
    codegens::asm::{AsmCodegen, write_asm},
    enums::{Expression, NType, Program, Statement},
    lexer::{Lexer, read_to_str},
    macros::show_parsed_nicely,
    parser::ParserNNV,
};
use clap::{Parser, Subcommand, ValueEnum};

mod codegen;
mod codegens;
mod enums;
mod lexer;
mod macros;
mod parser;
mod semantic;

#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
struct Cli {
    #[command(subcommand)]
    cmd: Commands,
}

#[derive(Debug, Clone, Copy, ValueEnum)]
enum BuildMode {
    Unoptimized,
    Standard,
    Ultra,
    Assembly,
}

#[derive(Debug, Subcommand)]
enum Commands {
    /// Build NNV code into executable (machine code)
    #[command(name = "build")]
    Build {
        input_file: String,

        #[arg(short, long, default_value = "a.asm")]
        output_file: Option<String>,

        #[arg(short, long)]
        mode: Option<BuildMode>,
    },

    /// Show AST without building
    #[command(name = "tree")]
    AST {
        input_file: String,

        #[arg(short, long, default_value = "new")]
        format_type: String,
    },

    #[command(name = "lexed")]
    Lexed { input_file: String },

    /// Generate C code from NNV source
    #[command(name = "generate")]
    OnlyGenerateC { input_file: String },
}

fn main() {
    let cli = Cli::parse();

    match &cli.cmd {
        Commands::Build {
            input_file,
            output_file,
            mode,
        } => {
            let path = PathBuf::from(input_file);
            let opened = read_to_str(path);

            let lexer = Lexer::new(&opened);
            let tokens = lexer.tokenize_all();

            let mut parser = ParserNNV::new(&tokens);
            let program = parser.parse();

            let mut asm = AsmCodegen::new(program.unwrap());

            match mode {
                Some(BuildMode::Assembly) => {
                    asm.generate();
                    let emmited = asm.emitate_asm();
                    write_asm(output_file.as_ref().unwrap(), emmited).unwrap();
                }
                _ => {}
            }
        }

        Commands::Lexed { input_file } => {
            let path = PathBuf::from(input_file);
            let opened = read_to_str(path);

            let lexer = Lexer::new(&opened);
            let tokens = lexer.tokenize_all();

            println!("{:?}", tokens);
        }

        Commands::AST {
            input_file,
            format_type,
        } => {
            let path = PathBuf::from(input_file);
            let opened = read_to_str(path);

            let lexer = Lexer::new(&opened);
            let tokens = lexer.tokenize_all();

            let mut parser = ParserNNV::new(&tokens);
            let program = parser.parse();

            let np = &program.unwrap();

            match format_type.as_str() {
                "new" => show_parsed_nicely(np),
                "raw" => println!("{:#?}", np),
                _ => {
                    eprintln!("Incorrect formatting mode. Available: `raw`, `new`");
                    exit(1);
                }
            }
        }

        _ => {
            todo!("Нет команды");
        }
    }
}
