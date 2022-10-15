use crate::span::Span;

#[derive(Debug, Clone)]
pub enum Value {
    String(Span, String),
    Number(Span, f64),
    Boolean(Span, bool),
    Nil(Span),
}

impl Value {
    pub(crate) fn span(&self) -> Span {
        match self {
            Value::String(span, _) => span,
            Value::Number(span, _) => span,
            Value::Boolean(span, _) => span,
            Value::Nil(span) => span,
        }
        .clone()
    }

    pub(crate) fn pretty_print(&self) {
        match self {
            Value::String(_, string) => println!("{string}"),
            Value::Number(_, number) => println!("{number}"),
            Value::Boolean(_, boolean) => println!("{boolean}"),
            Value::Nil(_) => println!("nil"),
        }
    }
}
