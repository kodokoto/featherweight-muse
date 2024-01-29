use std::env;

mod lexer;
mod token;
mod parser;
mod ast;

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
}
