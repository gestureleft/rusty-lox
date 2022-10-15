use std::collections::HashMap;

use crate::lexer::Token;

use super::value::Value;

#[derive(Debug)]
pub struct Environment<'a> {
    values: HashMap<&'a str, Value>,
}

impl<'a> Environment<'a> {
    pub(crate) fn new() -> Self {
        Self {
            values: HashMap::new(),
        }
    }
    pub(crate) fn define(&mut self, name: &'a str, value: Value) {
        self.values.insert(name, value);
    }

    pub(crate) fn get(&'a self, source: &str, token: &Token) -> Option<Value> {
        self.values.get(token.span.slice(source)).cloned()
    }
}
