use std::collections::HashMap;

use crate::lexer::Token;

use super::value::Value;

#[derive(Debug)]
pub struct Environment<'a> {
    enclosing_environment: Option<&'a mut Environment<'a>>,
    values: HashMap<String, Value>,
}

impl<'a> Environment<'a> {
    pub(crate) fn new() -> Self {
        Self {
            values: HashMap::new(),
            enclosing_environment: None,
        }
    }
    pub(crate) fn define(&mut self, name: String, value: Value) {
        self.values.insert(name, value);
    }

    pub(crate) fn get(&self, source: &str, token: &Token) -> Option<Value> {
        let value = self.values.get(token.span.slice(source)).cloned();
        if value.is_some() {
            return value;
        };

        // Search enclosing scope
        if let Some(enclosing_environment) = &self.enclosing_environment {
            return enclosing_environment.get(source, token);
        };

        None
    }

    pub(crate) fn assign(&mut self, name: String, new_value: Value) -> Result<(), ()> {
        let value = self.values.get_mut(&name);
        if let Some(value) = value {
            *value = new_value;
            return Ok(());
        };

        if let Some(enclosing_environment) = &mut self.enclosing_environment {
            return enclosing_environment.assign(name, new_value);
        }

        Err(())
    }
}
