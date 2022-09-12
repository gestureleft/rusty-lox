mod error;
mod repl;

use std::fs;

use error::Error;

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

    println!("Got file_contents: {file_contents}");

    Ok(())
}

fn load_file_from_args() -> Result<String, Error> {
    let file_path = std::env::args().nth(1).ok_or(Error::Usage)?;

    fs::read_to_string(file_path).map_err(Error::Io)
}

fn print_usage() {
    println!("Usage: rusty-lox [file]");
}
