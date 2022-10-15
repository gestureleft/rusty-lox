#[derive(Debug)]
pub enum Error {
    Type(TypeError),
}

#[derive(Debug)]
pub struct TypeError {
    expected: String,
    got: String,
}

impl Error {
    pub fn type_error(expected: String, got: String) -> Self {
        Error::Type(TypeError { expected, got })
    }
}
