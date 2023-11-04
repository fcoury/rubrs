use std::collections::HashMap;

use crate::{Literal, Token};

pub struct Environment {
    values: HashMap<String, Literal>,
}

impl Environment {
    pub fn new() -> Self {
        Self {
            values: HashMap::new(),
        }
    }

    pub fn define(&mut self, name: String, value: Literal) {
        self.values.insert(name, value);
    }

    pub fn get(&self, name: &Token) -> Result<&Literal, String> {
        self.values
            .get(&name.lexeme)
            .ok_or_else(|| format!("Undefined variable '{}'.", name.lexeme))
    }

    pub fn assign(&mut self, name: &Token, value: Literal) -> Result<(), String> {
        if self.values.contains_key(&name.lexeme) {
            self.values.insert(name.lexeme.clone(), value);
            Ok(())
        } else {
            Err(format!("Undefined variable '{}'.", name.lexeme))
        }
    }
}
