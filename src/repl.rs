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
        println!("Got tokens: {:?}", tokens);
        if buffer == *"\n" {
            break;
        };

        buffer.clear();
    }
    Ok(())
}
