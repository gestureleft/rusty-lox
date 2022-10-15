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
        if let Some(expression) = parse_result.expression {
            let value = Interpreter::interpret(&buffer, expression);
            println!("{:?}", value);
        }

        if buffer == *"\n" {
            break;
        };

        buffer.clear();
    }
    Ok(())
}
