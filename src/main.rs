use std::env;

use ast::AST;
use typing::TypeEnviroment;

mod lexer;
mod token;
mod parser;
mod ast;
mod state;
mod interpreter;
mod typing;

fn main() {

    // take first arg as file dir
    let args: Vec<String> = std::env::args().collect();

    if args.len() < 2 {
        panic!("No file provided");
    }

    if args[1] == "-h" || args[1] == "--help" {
        println!("Usage: ./main [options] <file>");
        println!("Options:");
        println!("\t-v, -verbose\t\tEnable verbose mode");
        println!("\t-d, -debug\t\tEnable debug mode");
        return;
    }

    for arg in &args {
        match arg.as_str() {
            "-v" | "-verbose" => {
                env::set_var("RUST_BACKTRACE", "1");
            },
            "-d" | "-debug" => {
                env::set_var("MUSE_DEBUG", "1");
            },
            _ => {}
        }
    }

    // get file contents
    let file_dir = &args[args.len() -1];
    let file_contents = std::fs::read_to_string(file_dir).expect("Error reading file");

    // tokenize
    let mut lexer = lexer::Lexer::new(&file_contents);
    let tokens = lexer.tokenize();
    if env::var("MUSE_DEBUG").is_ok() {
        println!("{:?}", tokens);
    }

    // parse
    let mut parser = parser::Parser::new(tokens);
    let ast = parser.parse();

    if env::var("MUSE_DEBUG").is_ok() {
        println!("{:#?}", ast);
    }

    // interpret
    let mut interpreter = interpreter::Interpreter::new();
    match ast.type_check(TypeEnviroment::new()){
        Ok(_) => {},
        Err(e) => {
            println!("TYPE ERROR: {}", e);
            // return;
        }
    };
    let res = interpreter.run(ast);

    // print result
    match res {
        Ok(v) => match v {
            ast::Value::Epsilon => {},
            _ => println!("{:?}", v)
            
        },
        Err(e) => {
            println!("ERROR: {}", e);
        }
    }
}

