use std::{cell::RefCell, rc::Rc};

use crate::{
    environment::Environment,
    types::{NativeFunction, Stmt, Value},
};

#[derive(Debug, Clone)]
pub struct Interpreter {
    #[allow(unused)]
    globals: Rc<RefCell<Environment>>,
    environment: Rc<RefCell<Environment>>,
}

impl Interpreter {
    pub fn new() -> Self {
        let globals = Environment::new();
        let environment = Environment::new_enclosed(&globals);

        globals.borrow_mut().define(
            "clock".to_string(),
            Value::NativeFunction(NativeFunction::Clock),
        );

        Self {
            globals,
            environment,
        }
    }

    pub fn parse_and_run(&self, code: &str) -> Result<(), String> {
        let mut scanner = crate::scanner::Scanner::new(code.to_string());
        let tokens = scanner.scan_tokens();
        let mut parser = crate::parser::Parser::new(tokens);

        self.run(parser.parse()?)?;
        Ok(())
    }

    pub fn run(&self, statements: Vec<Stmt>) -> Result<(), String> {
        for statement in statements {
            statement.evaluate(&self.environment)?
        }

        Ok(())
    }
}
