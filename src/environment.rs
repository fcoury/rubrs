use std::collections::HashMap;

use crate::{Literal, Token};

#[derive(Debug, Clone)]
pub struct Environment {
    enclosing: Option<Box<Environment>>,
    values: HashMap<String, Literal>,
}

impl Environment {
    pub fn new() -> Self {
        Self {
            enclosing: None,
            values: HashMap::new(),
        }
    }

    pub fn new_enclosed(enclosing: Environment) -> Self {
        Self {
            enclosing: Some(Box::new(enclosing)),
            values: HashMap::new(),
        }
    }

    pub fn define(&mut self, name: String, value: Literal) {
        self.values.insert(name, value);
    }

    pub fn get(&self, name: &Token) -> Result<&Literal, String> {
        if let Some(value) = self.values.get(&name.lexeme) {
            return Ok(value);
        }

        if let Some(enclosing) = &self.enclosing {
            return enclosing.get(name);
        }

        Err(format!("Undefined variable '{}'.", name.lexeme))
    }

    pub fn assign(&mut self, name: &Token, value: Literal) -> Result<(), String> {
        if self.values.contains_key(&name.lexeme) {
            self.values.insert(name.lexeme.clone(), value);
            return Ok(());
        }

        if let Some(enclosing) = &mut self.enclosing {
            return enclosing.assign(name, value);
        }

        Err(format!("Undefined variable '{}'.", name.lexeme))
    }
}
