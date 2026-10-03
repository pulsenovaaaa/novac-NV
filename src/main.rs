#![allow(unused)]
use colored::Colorize;
use std::{
    path::PathBuf,
    process::{self, exit},
    thread,
    time::Duration,
};

use indicatif::{ProgressBar, ProgressStyle};

use crate::{
    codegens::asm::{AsmCodegen, write_asm},
    enums::{Expression, NType, Program, Statement},
    lexer::{Lexer, read_to_str},
    macros::{print_ast, run_with_spinner, show_parsed_nicely},
    parser::ParserNNV,
};
use clap::{Parser, Subcommand, ValueEnum};

mod builder;
mod codegen;
mod codegens;
mod enums;
mod lexer;

#[macro_use]
mod macros;

mod parser;
mod semantic;

#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
struct Cli {
    #[command(subcommand)]
    cmd: Option<Commands>,

    /// Open github repository in browser
    #[arg(short = 'g', long = "github", action = clap::ArgAction::SetTrue)]
    open_repo: bool,

    /// Verbose output
    #[arg(short = 'v', long = "verbose", action = clap::ArgAction::SetTrue)]
    verbose: bool,
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

    #[command(name = "tester")]
    Tester {
        #[arg(short, long)]
        id: i32,
    },

    /// Show AST without building
    #[command(name = "tree")]
    AST {
        input_file: String,

        #[arg(short = 'd', long = "display", default_value = "new")]
        format_type: String,
    },

    /// Show lexed vector
    #[command(name = "lexed")]
    Lexed { input_file: String },

    /// Generate C code from NNV source
    #[command(name = "generate")]
    OnlyGenerateC { input_file: String },
}

fn main() -> Result<(), String> {
    let cli = Cli::parse();

    if cli.open_repo {
        let repo_url = "https://github.com/pulsenovaaaa/novac-NV";

        success!("Opening GitHub repository in browser: {}", repo_url);

        if let Err(err) = open::that(repo_url) {
            error!("Failed to open browser: {}", err);
        }

        process::exit(0);
    } else if cli.verbose {
        println!("Verbosing.......");
    }

    match &cli.cmd {
        Some(Commands::Build {
            input_file,
            output_file,
            mode,
        }) => {
            let path = PathBuf::from(input_file);
            let opened = read_to_str(path);

            let lexer = Lexer::new(&opened);
            let tokens = run_with_spinner(" [1/3] Tokenizing", || lexer.tokenize_all());

            let mut parser = ParserNNV::new(&tokens);
            let program = run_with_spinner(" [2/3] Parsing", || parser.parse());

            let mut asm = AsmCodegen::new(program.unwrap());

            run_with_spinner("[3/3] Assembiing...", || match mode {
                Some(BuildMode::Assembly) => {
                    asm.generate();
                    let emmited = asm.emitate_asm();
                    write_asm(output_file.as_ref().unwrap(), emmited).unwrap();
                }
                _ => {}
            });

            success!("built successfully");
            return Ok(());
        }

        Some(Commands::Lexed { input_file }) => {
            let path = PathBuf::from(input_file);
            let opened = read_to_str(path);

            let lexer = Lexer::new(&opened);
            let tokens = lexer.tokenize_all();

            println!("{:?}", tokens);

            success!("lexed successfully");
            return Ok(());
        }

        Some(Commands::AST {
            input_file,
            format_type,
        }) => {
            println!(
                "{}: Selected mode: AST Display\n",
                format!("Compiling `{}`", input_file).bold().cyan()
            );

            let path = PathBuf::from(input_file);
            let opened = read_to_str(path);

            let lexer = Lexer::new(&opened);
            let tokens = lexer.tokenize_all();

            let mut parser = ParserNNV::new(&tokens);
            let program = parser.parse();

            let np = match &program {
                Ok(prg) => prg,
                Err(err) => {
                    println!("{}", "Compilation failed".bold().red());
                    exit(1);
                }
            };

            match format_type.as_str() {
                "new" => print_ast(np),
                "raw" => println!("{:#?}", np),
                _ => {
                    eprintln!("Incorrect formatting mode. Available: `raw`, `new`");
                    exit(1);
                }
            }

            success!("tree displayed successfully. Mode: {}", format_type);
            return Ok(());
        }

        Some(Commands::Tester { id }) => {
            match id {
                // Тест No.0 -- Прогресс-бары
                0 => {
                    let pb = ProgressBar::new(100);
                    pb.set_style(
                        ProgressStyle::default_bar()
                            .template("{spinner:.green} [{elapsed_precise}] [{bar:40.cyan/blue}] {pos}/{len} ({eta})")
                            .unwrap()
                            .progress_chars("#>-"),
                    );

                    fn work_02() {
                        thread::sleep(Duration::from_millis(20 * 10));
                    }

                    println!("Testing Progress-Bar...");
                    for _ in 0..100 {
                        work_02();
                        pb.inc(1);
                    }

                    pb.finish_with_message("All Done!");
                }

                _ => {
                    error!("Not done yet! id: {}", id);
                    std::process::exit(1);
                }
            }
        }

        Some(_) => {
            error!("Not implemented")
        }

        None => {
            error!("No subcommand provided. Use `--help` for details.");
        }
    }
    Ok(())
}
