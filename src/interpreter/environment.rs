use std::{collections::HashMap, rc::Rc};

use crate::lexer::Token;

use super::value::Value;

#[derive(Debug)]
pub struct Environment {
    values: HashMap<String, Rc<Value>>,
}

impl Environment {
    pub(crate) fn new() -> Self {
        Self {
            values: HashMap::new(),
        }
    }

    pub(crate) fn define(&mut self, name: String, value: Rc<Value>) {
        self.values.insert(name, value);
    }

    pub(crate) fn get(&self, source: &str, token: &Token) -> Option<Rc<Value>> {
        let value = self.values.get(token.span.slice(source)).cloned();
        if value.is_some() {
            return value;
        };
        None
    }

    pub(crate) fn assign(&mut self, name: &String, new_value: &Rc<Value>) -> Result<(), ()> {
        let value = self.values.get_mut(name);
        if let Some(value) = value {
            *value = new_value.clone();
            return Ok(());
        };
        Err(())
    }
}
