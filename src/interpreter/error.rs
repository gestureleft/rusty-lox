use crate::{
    lexer::{self, Token},
    span::Span,
};

#[derive(Debug)]
pub enum Error {
    Type(TypeError),
    VariableDoesntExist(Token),
    NotCallable(Span),
    Arity {
        got: usize,
        expected: usize,
        call_span: Span,
    },
}

#[derive(Debug)]
pub struct TypeError {
    expected: String,
    got: String,
    source_token_span: Span,
}

impl Error {
    pub fn type_error(expected: String, got: String, source_token_span: Span) -> Self {
        Error::Type(TypeError {
            expected,
            got,
            source_token_span,
        })
    }
}

impl Error {
    pub fn display(&self, source: &str) {
        match self {
            Error::Type(TypeError {
                expected,
                got,
                source_token_span,
            }) => lexer::Error::display_error(
                source,
                source_token_span,
                &format!("Type Error: expected {}, got {}", expected, got),
            ),
            Error::VariableDoesntExist(token) => {
                lexer::Error::display_error(source, &token.span, "Variable doesn't exist")
            }
            Error::NotCallable(name_span) => {
                lexer::Error::display_error(source, name_span, "Value is not callable")
            }
            Error::Arity {
                got,
                expected,
                call_span,
            } => lexer::Error::display_error(
                source,
                call_span,
                &format!(
                    "Wrong number of call arguments. Expected {}, but got {}",
                    expected, got
                ),
            ),
        }
    }
}
