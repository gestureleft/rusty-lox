use std::{cell::RefCell, collections::HashMap, rc::Rc};

use crate::lexer::Token;

use super::value::Value;

#[derive(Debug)]
pub struct Environment {
    parent: Option<Rc<RefCell<Environment>>>,
    values: HashMap<String, Rc<Value>>,
}

impl Environment {
    pub(crate) fn new() -> Self {
        Self {
            parent: None,
            values: HashMap::new(),
        }
    }

    pub(crate) fn close_over(parent: Rc<RefCell<Environment>>) -> Self {
        Self {
            parent: Some(parent),
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
        (*(self.parent.as_ref()?)).borrow().get(source, token)
    }

    pub(crate) fn assign(&mut self, name: &String, new_value: &Rc<Value>) -> Result<(), ()> {
        let value = self.values.get_mut(name);
        if let Some(value) = value {
            *value = new_value.clone();
            return Ok(());
        };
        if self.parent.is_none() {
            return Err(());
        };

        (*self.parent.clone().unwrap())
            .borrow_mut()
            .assign(name, new_value)
    }
}
