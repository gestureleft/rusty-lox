#![feature(let_chains)]
#![feature(try_trait_v2)]

mod error;
mod expression;
mod interpreter;
mod lexer;
mod parser;
mod repl;
mod span;
mod statement;

use std::fs;

use error::Error;

use crate::{interpreter::Interpreter, lexer::Lexer, parser::Parser};

fn main() -> Result<(), Error> {
    let argc = std::env::args().count();

    if argc > 2 {
        print_usage();
        return Err(Error::Usage);
    };

    if argc == 1 {
        return repl::run_repl();
    }

    let file_contents = load_file_from_args()?;

    let lexer_result = Lexer::lex(&file_contents);

    if !lexer_result.errors.is_empty() {
        println!("Got lexing errors");
        lexer_result
            .errors
            .iter()
            .for_each(|e| e.display(&file_contents));
        return Ok(());
    }

    let parse_result = Parser::parse(&lexer_result.tokens);

    if !parse_result.errors.is_empty() {
        parse_result
            .errors
            .iter()
            .for_each(|e| e.display(&file_contents));
        return Ok(());
    }

    let result = Interpreter::new().interpret(&file_contents, parse_result.declarations);

    if let Err(error) = result {
        error.display(&file_contents);
    }

    Ok(())
}

fn load_file_from_args() -> Result<String, Error> {
    let file_path = std::env::args().nth(1).ok_or(Error::Usage)?;

    Ok(fs::read_to_string(file_path)?)
}

fn print_usage() {
    println!("Usage: rusty-lox [file]");
}
