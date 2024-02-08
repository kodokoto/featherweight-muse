use std::{collections::HashMap, env};

mod lexer;
mod token;
mod parser;
mod ast;
mod state;
mod interpreter;

fn main() {
    env::set_var("RUST_BACKTRACE", "1");

    // take first arg as file dir
    let args: Vec<String> = std::env::args().collect();
    let file_dir = &args[1];
    let file_contents = std::fs::read_to_string(file_dir).expect("Error reading file");
    let mut lexer = lexer::Lexer::new(&file_contents);
    let tokens = lexer.tokenize();
    println!("{:?}", tokens);

    let mut parser = parser::Parser::new(tokens);
    let ast = parser.parse();
    println!("{:#?}", ast);

    // let mut reducer = reduction::Reducer::new();
    // reducer.r_program(&ast);
    let mut interpreter = interpreter::Interpreter::new();
    for t in ast.terms {
        let res = interpreter.run(t);
        println!("{:?}", res);
    }
}

