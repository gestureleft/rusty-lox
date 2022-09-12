use std::io::{self, stdout, Write};

use crate::error::Error;

pub fn run_repl() -> Result<(), Error> {
    let mut buffer = String::new();
    let stdin = io::stdin();
    loop {
        print!("> ");
        stdout().flush()?;
        stdin.read_line(&mut buffer)?;
        if buffer == *"\n" {
            break;
        };
        buffer.clear();
    }
    Ok(())
}
