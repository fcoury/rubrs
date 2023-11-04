use std::{cell::RefCell, collections::HashMap, rc::Rc};

use crate::types::{Token, Value};

#[derive(Debug, Clone)]
pub struct Environment {
    enclosing: Option<Rc<RefCell<Environment>>>,
    values: HashMap<String, Value>,
}

impl Environment {
    pub fn new() -> Rc<RefCell<Self>> {
        Rc::new(RefCell::new(Self {
            enclosing: None,
            values: HashMap::new(),
        }))
    }

    pub fn new_enclosed(enclosing: &Rc<RefCell<Environment>>) -> Rc<RefCell<Self>> {
        Rc::new(RefCell::new(Self {
            enclosing: Some(enclosing.clone()),
            values: HashMap::new(),
        }))
    }

    pub fn define(&mut self, name: String, value: Value) {
        self.values.insert(name, value);
    }

    pub fn get(&self, name: &Token) -> Result<Value, String> {
        if let Some(value) = self.values.get(&name.lexeme) {
            return Ok(value.clone());
        }

        if let Some(enclosing) = &self.enclosing {
            return enclosing.borrow().get(name).clone();
        }

        Err(format!("Undefined variable '{}'.", name.lexeme))
    }

    pub fn assign(&mut self, name: &Token, value: Value) -> Result<(), String> {
        if self.values.contains_key(&name.lexeme) {
            self.values.insert(name.lexeme.clone(), value);
            return Ok(());
        }

        if let Some(enclosing) = &mut self.enclosing {
            return enclosing.borrow_mut().assign(name, value);
        }

        Err(format!("Undefined variable '{}'.", name.lexeme))
    }
}
