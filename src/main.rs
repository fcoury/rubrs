use std::{cell::RefCell, fmt, rc::Rc};

use environment::Environment;
use rustyline::{error::ReadlineError, DefaultEditor};

mod environment;
mod parser;

#[derive(Debug, Clone)]
struct Scanner {
    source: String,
    tokens: Vec<Token>,
    start: usize,
    current: usize,
    line: usize,
}

impl Scanner {
    fn new(source: String) -> Self {
        Self {
            source,
            tokens: Vec::new(),
            start: 0,
            current: 0,
            line: 1,
        }
    }

    fn scan_tokens(&mut self) -> Vec<Token> {
        while !self.is_at_end() {
            self.start = self.current;
            self.scan_token();
        }

        self.tokens
            .push(Token::new(TokenType::Eof, String::from(""), self.line));
        self.tokens.clone()
    }

    fn is_at_end(&self) -> bool {
        self.current >= self.source.len()
    }

    fn advance(&mut self) -> char {
        self.current += 1;
        self.source.chars().nth(self.current - 1).unwrap()
    }

    fn add_token(&mut self, token_type: TokenType) {
        let text = self.source[self.start..self.current].to_string();
        self.tokens.push(Token::new(token_type, text, self.line));
    }

    fn match_char(&mut self, expected: char) -> bool {
        if self.is_at_end() {
            return false;
        }

        if self.source.chars().nth(self.current).unwrap() != expected {
            return false;
        }

        self.current += 1;
        true
    }

    fn peek(&self) -> char {
        if self.is_at_end() {
            return '\0';
        }

        self.source.chars().nth(self.current).unwrap()
    }

    fn peek_next(&self) -> char {
        if self.current + 1 >= self.source.len() {
            return '\0';
        }

        self.source.chars().nth(self.current + 1).unwrap()
    }

    fn string(&mut self) {
        while self.peek() != '"' && !self.is_at_end() {
            if self.peek() == '\n' {
                self.line += 1;
            }
            self.advance();
        }

        if self.is_at_end() {
            panic!("Unterminated string at line {}", self.line);
        }

        self.advance();

        let value = self.source[self.start + 1..self.current - 1].to_string();
        self.add_token(TokenType::String(value));
    }

    fn is_digit(&self, c: char) -> bool {
        c.is_ascii_digit()
    }

    fn number(&mut self) {
        while self.is_digit(self.peek()) {
            self.advance();
        }

        if self.peek() == '.' && self.is_digit(self.peek_next()) {
            self.advance();

            while self.is_digit(self.peek()) {
                self.advance();
            }
        }

        let value = self.source[self.start..self.current].parse().unwrap();
        self.add_token(TokenType::Number(value));
    }

    fn identifier(&mut self) {
        while self.peek().is_alphanumeric() {
            self.advance();
        }

        let text = self.source[self.start..self.current].to_string();
        let token_type = match text.as_str() {
            "and" => TokenType::And,
            "class" => TokenType::Class,
            "else" => TokenType::Else,
            "false" => TokenType::False,
            "for" => TokenType::For,
            "fun" => TokenType::Fun,
            "if" => TokenType::If,
            "nil" => TokenType::Nil,
            "or" => TokenType::Or,
            "print" => TokenType::Print,
            "return" => TokenType::Return,
            "super" => TokenType::Super,
            "this" => TokenType::This,
            "true" => TokenType::True,
            "var" => TokenType::Var,
            "while" => TokenType::While,
            _ => TokenType::Identifier(text),
        };

        self.add_token(token_type);
    }

    fn scan_token(&mut self) {
        let c = self.advance();
        match c {
            '(' => self.add_token(TokenType::LeftParen),
            ')' => self.add_token(TokenType::RightParen),
            '{' => self.add_token(TokenType::LeftBrace),
            '}' => self.add_token(TokenType::RightBrace),
            ',' => self.add_token(TokenType::Comma),
            '.' => self.add_token(TokenType::Dot),
            '-' => self.add_token(TokenType::Minus),
            '+' => self.add_token(TokenType::Plus),
            ';' => self.add_token(TokenType::Semicolon),
            '*' => self.add_token(TokenType::Star),
            '!' => {
                let token_type = if self.match_char('=') {
                    TokenType::BangEqual
                } else {
                    TokenType::Bang
                };
                self.add_token(token_type);
            }
            '=' => {
                let token_type = if self.match_char('=') {
                    TokenType::EqualEqual
                } else {
                    TokenType::Equal
                };
                self.add_token(token_type);
            }
            '<' => {
                let token_type = if self.match_char('=') {
                    TokenType::LessEqual
                } else {
                    TokenType::Less
                };
                self.add_token(token_type);
            }
            '>' => {
                let token_type = if self.match_char('=') {
                    TokenType::GreaterEqual
                } else {
                    TokenType::Greater
                };
                self.add_token(token_type);
            }
            '/' => {
                if self.match_char('/') {
                    while self.peek() != '\n' && !self.is_at_end() {
                        self.advance();
                    }
                } else {
                    self.add_token(TokenType::Slash);
                }
            }
            ' ' | '\r' | '\t' => {}
            '\n' => self.line += 1,
            '"' => self.string(),
            _ => {
                if c.is_ascii_digit() {
                    self.number();
                } else if c.is_alphabetic() {
                    self.identifier();
                } else {
                    panic!("Unexpected character at line {}", self.line);
                }
            }
        }
    }
}

#[derive(Debug, Clone)]
enum TokenType {
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
    token_type: TokenType,
    lexeme: String,
    #[allow(unused)]
    line: usize,
}

impl Token {
    fn new(token_type: TokenType, lexeme: String, line: usize) -> Self {
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
    If {
        condition: Expr,
        then_branch: Box<Stmt>,
        else_branch: Option<Box<Stmt>>,
    },
    Print(Expr),
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
    fn evaluate(&self, env: &Rc<RefCell<Environment>>) -> Result<(), String> {
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
            Stmt::Print(expr) => {
                println!("{}", expr.evaluate(env)?);
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
                    None => Literal::Nil,
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
    Grouping(Box<Expr>),
    Literal(Literal),
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
    fn evaluate(&self, env: &Rc<RefCell<Environment>>) -> Result<Literal, String> {
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
                    TokenType::Minus => Ok(Literal::Number(left.to_number() - right.to_number())),
                    TokenType::Plus => Ok(Literal::Number(left.to_number() + right.to_number())),
                    TokenType::Slash => Ok(Literal::Number(left.to_number() / right.to_number())),
                    TokenType::Star => Ok(Literal::Number(left.to_number() * right.to_number())),
                    TokenType::Greater => {
                        Ok(Literal::Boolean(left.to_number() > right.to_number()))
                    }
                    TokenType::GreaterEqual => {
                        Ok(Literal::Boolean(left.to_number() >= right.to_number()))
                    }
                    TokenType::Less => Ok(Literal::Boolean(left.to_number() < right.to_number())),
                    TokenType::LessEqual => {
                        Ok(Literal::Boolean(left.to_number() <= right.to_number()))
                    }
                    TokenType::BangEqual => Ok(Literal::Boolean(left != right)),
                    _ => panic!("Unexpected operator {:?}", operator),
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
                    TokenType::Minus => Ok(Literal::Number(-right.to_number())),
                    TokenType::Bang => Ok(Literal::Boolean(!right.to_boolean())),
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
pub enum Literal {
    Boolean(bool),
    Nil,
    Number(f64),
    String(String),
}

impl Literal {
    fn to_boolean(&self) -> bool {
        match self {
            Literal::Boolean(boolean) => *boolean,
            Literal::Nil => false,
            Literal::Number(number) => *number != 0.0,
            Literal::String(string) => !string.is_empty(),
        }
    }

    fn to_number(&self) -> f64 {
        match self {
            Literal::Boolean(boolean) => {
                if *boolean {
                    1.0
                } else {
                    0.0
                }
            }
            Literal::Nil => 0.0,
            Literal::Number(number) => *number,
            Literal::String(string) => string.parse().unwrap(),
        }
    }
}

impl fmt::Display for Literal {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Literal::Boolean(boolean) => write!(f, "{}", boolean),
            Literal::Nil => write!(f, "nil"),
            Literal::Number(number) => write!(f, "{}", number),
            Literal::String(string) => write!(f, "{}", string),
        }
    }
}

fn run_file(filename: &str) {
    let contents =
        std::fs::read_to_string(filename).expect("Something went wrong reading the file");
    let mut scanner = Scanner::new(contents);
    let tokens = scanner.scan_tokens();
    let mut parser = parser::Parser::new(tokens);
    let environment = environment::Environment::new();

    match parser.parse() {
        Ok(statements) => {
            for statement in statements {
                match statement.evaluate(&environment) {
                    Ok(_) => {}
                    Err(error) => println!("{}", error),
                }
            }
        }
        Err(error) => println!("{}", error),
    }
}

fn repl() {
    let mut rl = DefaultEditor::new().unwrap();
    let environment = environment::Environment::new();

    if rl.load_history(".history").is_err() {
        println!("No previous history.");
    }
    loop {
        match rl.readline("rubrs> ") {
            Ok(line) => {
                rl.add_history_entry(line.as_str()).unwrap();

                let mut scanner = Scanner::new(line);
                let tokens = scanner.scan_tokens();
                let mut parser = parser::Parser::new(tokens);
                match parser.parse() {
                    Ok(statements) => {
                        for statement in statements {
                            match statement.evaluate(&environment) {
                                Ok(_) => {}
                                Err(error) => println!("{}", error),
                            }
                        }
                    }
                    Err(error) => println!("{}", error),
                }
            }
            Err(ReadlineError::Interrupted) => {
                // User pressed Ctrl+C
                // println!("CTRL+C");
            }
            Err(ReadlineError::Eof) => {
                // User pressed Ctrl+D
                break;
            }
            Err(error) => println!("error: {}", error),
        }
        rl.save_history(".history").unwrap();
    }
}

fn main() {
    let args = std::env::args().collect::<Vec<String>>();

    match args.len() {
        3.. => println!("Usage: rubrs [script]"),
        2 => run_file(&args[1]),
        _ => repl(),
    }
}
