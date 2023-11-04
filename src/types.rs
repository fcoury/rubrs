use std::{cell::RefCell, fmt, rc::Rc};

use crate::{environment::Environment, interpreter::Interpreter};

#[derive(Debug, Clone)]
pub enum TokenType {
    LeftParen,
    RightParen,
    LeftBrace,
    RightBrace,
    Comma,
    Dot,
    Minus,
    Plus,
    Semicolon,
    Slash,
    Star,

    Bang,
    BangEqual,
    Equal,
    EqualEqual,
    Greater,
    GreaterEqual,
    Less,
    LessEqual,

    Identifier(String),
    String(String),
    Number(f64),

    And,
    Class,
    Else,
    False,
    Fun,
    For,
    If,
    Nil,
    Or,
    Print,
    Return,
    Super,
    This,
    True,
    Var,
    While,

    Eof,
}

#[derive(Debug, Clone)]
pub struct Token {
    pub token_type: TokenType,
    pub lexeme: String,
    #[allow(unused)]
    pub line: usize,
}

impl Token {
    pub fn new(token_type: TokenType, lexeme: String, line: usize) -> Self {
        Self {
            token_type,
            lexeme,
            line,
        }
    }
}

#[derive(Debug, Clone)]
pub enum Stmt {
    Block(Vec<Stmt>),
    Expression(Expr),
    Function {
        name: Token,
        parameters: Vec<Token>,
        body: Vec<Stmt>,
    },
    If {
        condition: Expr,
        then_branch: Box<Stmt>,
        else_branch: Option<Box<Stmt>>,
    },
    Print(Expr),
    Return {
        keyword: Token,
        value: Option<Expr>,
    },
    Var {
        name: Token,
        initializer: Option<Expr>,
    },
    While {
        condition: Expr,
        body: Box<Stmt>,
    },
}

impl Stmt {
    pub fn evaluate(&self, env: &Rc<RefCell<Environment>>) -> Result<(), String> {
        match self {
            Stmt::Expression(expr) => {
                expr.evaluate(env)?;
            }
            Stmt::If {
                condition,
                then_branch,
                else_branch,
            } => {
                if condition.evaluate(env)?.to_boolean() {
                    then_branch.evaluate(env)?;
                } else if let Some(else_branch) = else_branch {
                    else_branch.evaluate(env)?;
                }
            }
            Stmt::Function {
                name,
                parameters,
                body,
            } => {
                let function = Value::Function(Function {
                    name: name.clone(),
                    parameters: parameters.clone(),
                    body: body.clone(),
                    closure: env.clone(),
                });
                env.borrow_mut().define(name.lexeme.clone(), function);
            }
            Stmt::Print(expr) => {
                println!("{}", expr.evaluate(env)?);
            }
            Stmt::Return { value, .. } => {
                let value = match value {
                    Some(value) => value.evaluate(env)?,
                    None => Value::Nil,
                };
                return Err(format!("{}", value));
            }
            Stmt::Block(statements) => {
                let environment = Environment::new_enclosed(env);
                for statement in statements {
                    statement.evaluate(&environment)?;
                }
            }
            Stmt::Var { name, initializer } => {
                let value = match initializer {
                    Some(initializer) => initializer.evaluate(env)?,
                    None => Value::Nil,
                };
                env.borrow_mut().define(name.lexeme.clone(), value);
            }
            Stmt::While { condition, body } => {
                while condition.evaluate(env)?.to_boolean() {
                    body.evaluate(env)?;
                }
            }
        }

        Ok(())
    }
}

#[derive(Debug, Clone)]
pub enum Expr {
    Assign {
        name: Token,
        value: Box<Expr>,
    },
    Binary {
        left: Box<Expr>,
        operator: Token,
        right: Box<Expr>,
    },
    Call {
        callee: Box<Expr>,
        paren: Token,
        arguments: Vec<Expr>,
    },
    Grouping(Box<Expr>),
    Literal(Value),
    Logical {
        left: Box<Expr>,
        operator: Token,
        right: Box<Expr>,
    },
    Unary {
        operator: Token,
        right: Box<Expr>,
    },
    Variable(Token),
}

impl Expr {
    fn evaluate(&self, env: &Rc<RefCell<Environment>>) -> Result<Value, String> {
        match self {
            Expr::Assign { name, value } => {
                let value = value.evaluate(env)?;
                env.borrow_mut().assign(name, value.clone())?;
                Ok(value)
            }
            Expr::Binary {
                left,
                operator,
                right,
            } => {
                let left = left.evaluate(env)?;
                let right = right.evaluate(env)?;
                match operator.token_type {
                    TokenType::Minus => Ok(Value::Number(left.to_number()? - right.to_number()?)),
                    TokenType::Plus => Ok(Value::Number(left.to_number()? + right.to_number()?)),
                    TokenType::Slash => Ok(Value::Number(left.to_number()? / right.to_number()?)),
                    TokenType::Star => Ok(Value::Number(left.to_number()? * right.to_number()?)),
                    TokenType::Greater => {
                        Ok(Value::Boolean(left.to_number()? > right.to_number()?))
                    }
                    TokenType::GreaterEqual => {
                        Ok(Value::Boolean(left.to_number()? >= right.to_number()?))
                    }
                    TokenType::Less => Ok(Value::Boolean(left.to_number()? < right.to_number()?)),
                    TokenType::LessEqual => {
                        Ok(Value::Boolean(left.to_number()? <= right.to_number()?))
                    }
                    TokenType::BangEqual => Ok(Value::Boolean(left != right)),
                    _ => panic!("Unexpected operator {:?}", operator),
                }
            }
            Expr::Call {
                callee, arguments, ..
            } => {
                let calee = callee.evaluate(env)?;
                let mut evaluated_arguments = Vec::new();
                for argument in arguments {
                    evaluated_arguments.push(argument.evaluate(env)?);
                }

                match calee {
                    Value::Function(function) => {
                        if function.parameters.len() != evaluated_arguments.len() {
                            return Err(format!(
                                "Expected {} arguments but got {}.",
                                function.parameters.len(),
                                evaluated_arguments.len()
                            ));
                        }

                        let environment = Environment::new_enclosed(&function.closure);
                        for (parameter, argument) in
                            function.parameters.iter().zip(evaluated_arguments.iter())
                        {
                            environment
                                .borrow_mut()
                                .define(parameter.lexeme.clone(), argument.clone());
                        }

                        for statement in function.body {
                            match statement {
                                Stmt::Return { value, .. } => {
                                    return if let Some(value) = value {
                                        Ok(value.evaluate(&environment).unwrap())
                                    } else {
                                        Ok(Value::Nil)
                                    }
                                }
                                _ => statement.evaluate(&environment).unwrap(),
                            }
                        }

                        Ok(Value::Nil)
                    }
                    Value::NativeFunction(function) => {
                        if function.arity() != evaluated_arguments.len() {
                            return Err(format!(
                                "Expected {} arguments but got {}.",
                                function.arity(),
                                evaluated_arguments.len()
                            ));
                        }

                        function.call(&Interpreter::new(), evaluated_arguments)
                    }
                    _ => Err(String::from("Can only call functions and classes.")),
                }
            }
            Expr::Grouping(expr) => expr.evaluate(env),
            Expr::Literal(literal) => Ok(literal.clone()),
            Expr::Logical {
                left,
                operator,
                right,
            } => {
                let left = left.evaluate(env)?;
                match operator.token_type {
                    TokenType::And => {
                        if !left.to_boolean() {
                            return Ok(left);
                        }
                    }
                    TokenType::Or => {
                        if left.to_boolean() {
                            return Ok(left);
                        }
                    }
                    _ => panic!("Unexpected operator {:?}", operator),
                }
                right.evaluate(env)
            }
            Expr::Unary { operator, right } => {
                let right = right.evaluate(env)?;
                match operator.token_type {
                    TokenType::Minus => Ok(Value::Number(-right.to_number()?)),
                    TokenType::Bang => Ok(Value::Boolean(!right.to_boolean())),
                    _ => panic!("Unexpected operator {:?}", operator),
                }
            }
            Expr::Variable(name) => Ok(env.borrow().get(name)?.clone()),
        }
    }
}

impl fmt::Display for Expr {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Expr::Assign { name, value } => write!(f, "({} = {})", name.lexeme, value),
            Expr::Binary {
                left,
                operator,
                right,
            } => write!(f, "({} {} {})", operator.lexeme, left, right),
            Expr::Call {
                callee, arguments, ..
            } => {
                let mut arguments_string = String::new();
                for argument in arguments {
                    arguments_string.push_str(&format!("{} ", argument));
                }
                write!(f, "({} {})", callee, arguments_string)
            }
            Expr::Grouping(expr) => write!(f, "(group {})", expr),
            Expr::Logical {
                left,
                operator,
                right,
            } => write!(f, "({} {} {})", operator.lexeme, left, right),
            Expr::Literal(literal) => write!(f, "{}", literal),
            Expr::Unary { operator, right } => write!(f, "({} {})", operator.lexeme, right),
            Expr::Variable(name) => write!(f, "{}", name.lexeme),
        }
    }
}

#[derive(Debug, Clone, PartialEq, PartialOrd)]
pub enum Value {
    Boolean(bool),
    Nil,
    Number(f64),
    String(String),
    Function(Function),
    NativeFunction(NativeFunction),
}

trait Callable {
    fn arity(&self) -> usize;
    fn call(&self, interpreter: &Interpreter, arguments: Vec<Value>) -> Result<Value, String>;
}

#[derive(Debug, Clone)]
pub struct Function {
    name: Token,
    parameters: Vec<Token>,
    body: Vec<Stmt>,
    closure: Rc<RefCell<Environment>>,
}

impl Callable for Function {
    fn arity(&self) -> usize {
        self.parameters.len()
    }

    fn call(&self, interpreter: &Interpreter, arguments: Vec<Value>) -> Result<Value, String> {
        let environment = Environment::new_enclosed(&self.closure);
        for (parameter, argument) in self.parameters.iter().zip(arguments.iter()) {
            environment
                .borrow_mut()
                .define(parameter.lexeme.clone(), argument.clone());
        }

        interpreter.run(self.body.clone())?;

        Ok(Value::Nil)
    }
}

impl PartialEq for Function {
    fn eq(&self, other: &Self) -> bool {
        self.name.lexeme == other.name.lexeme
    }
}

impl PartialOrd for Function {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        self.name.lexeme.partial_cmp(&other.name.lexeme)
    }
}

#[derive(Debug, Clone, PartialEq, PartialOrd)]
pub enum NativeFunction {
    Clock,
}

impl Callable for NativeFunction {
    fn arity(&self) -> usize {
        match self {
            NativeFunction::Clock => 0,
        }
    }

    fn call(&self, _interpreter: &Interpreter, _arguments: Vec<Value>) -> Result<Value, String> {
        match self {
            NativeFunction::Clock => Ok(Value::Number(
                std::time::SystemTime::now()
                    .duration_since(std::time::UNIX_EPOCH)
                    .unwrap()
                    .as_secs_f64(),
            )),
        }
    }
}

impl Value {
    fn to_boolean(&self) -> bool {
        match self {
            Value::Boolean(boolean) => *boolean,
            Value::Nil => false,
            Value::Number(number) => *number != 0.0,
            Value::String(string) => !string.is_empty(),
            Value::Function(_) => true,
            Value::NativeFunction(_) => true,
        }
    }

    fn to_number(&self) -> Result<f64, String> {
        match self {
            Value::Boolean(boolean) => Ok(*boolean as i32 as f64),
            Value::Nil => Ok(0.0),
            Value::Number(number) => Ok(*number),
            Value::String(string) => Ok(string.parse::<f64>().unwrap()),
            Value::Function(_) => Err(String::from("Cannot convert function to number.")),
            Value::NativeFunction(_) => {
                Err(String::from("Cannot convert native function to number."))
            }
        }
    }
}

impl fmt::Display for Value {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Value::Boolean(boolean) => write!(f, "{}", boolean),
            Value::Nil => write!(f, "nil"),
            Value::Number(number) => write!(f, "{}", number),
            Value::String(string) => write!(f, "{}", string),
            Value::Function(function) => write!(f, "<fn {}>", function.name.lexeme),
            Value::NativeFunction(function) => write!(f, "<native fn {:?}>", function),
        }
    }
}
