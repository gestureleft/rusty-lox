use std::io::{self, stdout, Write};

use crate::error::Error;
use crate::interpreter::Interpreter;
use crate::lexer::Lexer;
use crate::parser::Parser;

pub fn run_repl() -> Result<(), Error> {
    let mut buffer = String::new();
    let stdin = io::stdin();
    loop {
        print!("> ");
        stdout().flush()?;
        stdin.read_line(&mut buffer)?;
        let lexer_result = Lexer::lex(&buffer);
        let parse_result = Parser::parse(&buffer, &lexer_result.tokens);

        if !parse_result.errors.is_empty() {
            let error = &parse_result.errors[0];
            error.display(&buffer);
        }
        let result = Interpreter::interpret(&buffer, &parse_result.statements);

        if let Err(error) = result {
            error.display(&buffer);
        } else {
            let values = result.unwrap();
            for value in values {
                value.pretty_print();
            }
        }

        if buffer == *"\n" {
            break;
        };

        buffer.clear();
    }
    Ok(())
}
