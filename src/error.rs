use crate::lexer;

#[derive(Debug)]
pub enum Error {
    Io(std::io::Error),
    Usage,
    Lexer(lexer::Error),
}

impl From<std::io::Error> for Error {
    fn from(io_error: std::io::Error) -> Self {
        Error::Io(io_error)
    }
}

impl From<lexer::Error> for Error {
    fn from(lexer_error: lexer::Error) -> Self {
        Error::Lexer(lexer_error)
    }
}
