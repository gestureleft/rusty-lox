#[derive(Debug)]
pub enum Error {
    Io(std::io::Error),
    Usage,
}

impl From<std::io::Error> for Error {
    fn from(io_error: std::io::Error) -> Self {
        Error::Io(io_error)
    }
}
