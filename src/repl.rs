use std::io::{self, stdout, Write};

use crate::error::Error;
use crate::parser::Parser;

pub fn run_repl() -> Result<(), Error> {
    let mut buffer = String::new();
    let stdin = io::stdin();
    loop {
        print!("> ");
        stdout().flush()?;
        stdin.read_line(&mut buffer)?;
        let errors = Parser::parse(&buffer);

        if !errors.is_empty() {
            let error = &errors[0];
            error.display(&buffer);
        } else {
        }

        if buffer == *"\n" {
            break;
        };

        buffer.clear();
    }
    Ok(())
}
