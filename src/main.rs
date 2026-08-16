#![allow(unused)]

use std::{
    env::{self, args},
    fs,
    process::exit,
};

use crate::{ast_utils::show_ast, codegen::Codegen, lexer::Lexer, parser::Parser};

mod ast_utils;
mod codegen;
mod lexer;
mod macros;
mod parser;

#[derive(Debug, PartialEq, Eq)]
pub enum Token {
    KeywordExit,
    KeywordSet,
    KeywordPutChar,
    CRuntimeKeywordPrint,

    BinPlus,
    BinMinus,
    BinMul,
    BinDiv,
    Ampersand,

    LogicalAND,
    LogicalOR,
    LogicalNot,

    PlusEq,
    SubEq,
    MulEq,
    DivEq,
    BOrEq,
    MaskEq,

    BitOR,
    BitAND,
    BitMask,
    BitNot,

    Identifier(String),
    IntLiteral(i64),
    Char(u8),
    StringConstLiteral(String),
    UnterminatedString,
    Comment,

    Equals,
    EqualsEquals,
    HashTag,
    Dollar,
    Slash,
    Colon,
    SemiColon,
    OpenParen,
    CloseParen,
    Unknown,

    Scrap,
}

#[derive(Debug)]
pub enum Op {
    Add,
    Sub,
    Mul,
    Div,
}

#[derive(Debug)]
pub enum Expression {
    Int(i64),
    Var(String),
    Char(u8),
    ConstChar(String),

    AddrOf(Box<Expression>),
    Deref(Box<Expression>),

    Cast {
        target_type: NType,
        expr: Box<Expression>,
    },

    BinaryOp {
        left: Box<Expression>,
        op: Op,
        right: Box<Expression>,
    },
}

#[derive(Debug, PartialEq, Eq)]
pub enum NType {
    U8,
    I64,
    Int,
    Char,
    String,
    Non,
    Ptr(Box<NType>),
}

#[derive(Debug)]
pub enum Statement {
    Exit(Expression),
    PutChar(Expression),
    Set {
        name: String,
        ty: NType,
        val: Option<Expression>,
    },
    CRtmPrint(Expression),
}

#[derive(Debug)]
pub struct Program {
    pub statements: Vec<Statement>,
}

fn help() {
    eprintln!("* SubCommands: ");
    eprintln!("- build - Build NNV code into C");
    eprintln!("- ast - Show AST without building");
    eprintln!("- tokens - Show vector of tokens");
    eprintln!("- version - Show compilator version");
}

pub enum State {
    Build,
    Ast,
    Tokens,
    Version,
}

fn main() {
    let args: Vec<String> = env::args().collect();
    if args.len() < 2 {
        eprintln!("Usage: {} <subcommand>", args[0]);
        help();
        exit(1);
    }

    let subcommand = args[1].to_lowercase();
    let state = match subcommand.as_str() {
        "build" => State::Build,
        "ast" => State::Ast,
        "tokens" => State::Tokens,
        "version" => State::Version,
        _ => {
            eprintln!("Unknown subcommand: {}", subcommand);
            exit(1);
        }
    };
    match state {
        State::Build => {
            let buf = fs::read_to_string("main.nnv").expect("Read error!");

            let lexer = Lexer::new(buf.as_str());
            let tokens = lexer.tokenize_all();

            let mut parser = Parser::new(&tokens);
            let ast = parser.parse().unwrap();

            let mut codegen = Codegen::new(ast);
            codegen.generate();

            codegen.to_c_file("main.c");
            if let Err(err) = codegen.compile("main.c", "./runtime", "./runtime") {
                eprintln!("Error: {err}");
            }
        }
        State::Ast => {
            let buf = fs::read_to_string("main.nnv").expect("Read error!");

            let lexer = Lexer::new(buf.as_str());
            let tokens = lexer.tokenize_all();

            let mut parser = Parser::new(&tokens);
            let ast = parser.parse().unwrap();
            show_ast(&ast);
        }
        State::Tokens => {
            let buf = fs::read_to_string("main.nnv").expect("Read error!");

            let lexer = Lexer::new(buf.as_str());
            let tokens = lexer.tokenize_all();
            println!("{:?}", tokens);
        }
        State::Version => {
            println!("[GNU Novac Never-Value] gnnv compiler version: b.01-2026-rc01");
        }
    }

    // println!("{:#?}", tokens);
}
