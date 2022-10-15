use std::collections::HashMap;

use crate::lexer::Token;

use super::value::Value;

#[derive(Debug)]
pub struct Environment {
    values: HashMap<String, Value>,
}

impl Environment {
    pub(crate) fn new() -> Self {
        Self {
            values: HashMap::new(),
        }
    }
    pub(crate) fn define(&mut self, name: String, value: Value) {
        self.values.insert(name, value);
    }

    pub(crate) fn get(&self, source: &str, token: &Token) -> Option<Value> {
        self.values.get(token.span.slice(source)).cloned()
    }
}
