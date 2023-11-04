use interpreter::Interpreter;
use rustyline::{error::ReadlineError, DefaultEditor};

mod environment;
mod interpreter;
mod parser;
mod scanner;
mod types;

fn run_file(filename: &str) {
    let contents =
        std::fs::read_to_string(filename).expect("Something went wrong reading the file");
    let interpreter = Interpreter::new();
    match interpreter.parse_and_run(&contents) {
        Ok(_) => {}
        Err(error) => println!("{}", error),
    }
}

fn repl() {
    let mut rl = DefaultEditor::new().unwrap();
    let interpreter = Interpreter::new();

    if rl.load_history(".history").is_err() {
        println!("No previous history.");
    }
    loop {
        match rl.readline("rubrs> ") {
            Ok(line) => {
                rl.add_history_entry(line.as_str()).unwrap();

                match interpreter.parse_and_run(&line) {
                    Ok(_) => {}
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
