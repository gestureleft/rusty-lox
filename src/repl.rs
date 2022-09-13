use std::io::{self, stdout, Write};

use crate::error::Error;
use crate::lexer::Lexer;

pub fn run_repl() -> Result<(), Error> {
    let mut buffer = String::new();
    let stdin = io::stdin();
    loop {
        print!("> ");
        stdout().flush()?;
        stdin.read_line(&mut buffer)?;
        let tokens = Lexer::lex(&buffer);

        if tokens.errors.len() > 0 {
            let error = &tokens.errors[0];
            error.display(&buffer);
        }

        if buffer == *"\n" {
            break;
        };

        buffer.clear();
    }
    Ok(())
}
