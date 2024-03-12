use std::env;

use typecheck::TypeCheck;
use typing::TypeEnviroment;

mod lexer;
mod token;
mod parser;
mod ast;
mod state;
mod interpreter;
mod typing;
mod reduction;
mod typecheck;

fn t() -> Box<i32> {
    let x = Box::new(2);
    return x
}

fn t2() {
    let p = t();
}

fn main() {

    // let x = Box::new(1);
    // let p = t(x);
    // println!("{:?}", p);

    // return;

    env::set_var("RUST_BACKTRACE", "1");
    // take first arg as file dir
    let args: Vec<String> = std::env::args().collect();

    if args.len() < 2 {
        panic!("No file provided");
    }

    if args[1] == "-h" || args[1] == "--help" {
        println!("Usage: ./main [options] <file>");
        println!("Options:");
        println!("\t-h, -help\tDisplay this message");
        println!("\t-l, -lex\tEnable lexer output");
        println!("\t-p, -parse\tEnable parser output");
        println!("\t-t, -typecheck\tEnable typecheck output");
        println!("\t-e, -eval\tEnable eval output");
        return;
    }

    for arg in &args {
        match arg.as_str() {
            "-l" | "-lex" => {
                env::set_var("LEX_OUT", "1");
            },
            "-p" | "-parse" => {
                env::set_var("PARSE_OUT", "1");
            },
            "-t" | "-typecheck" => {
                env::set_var("TYPE_OUT", "1");
            },
            "-e" | "-eval" => {
                env::set_var("EVAL_OUT", "1");
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
    if env::var("LEX_OUT").is_ok() {
        println!("{:?}", tokens);
    }

    // parse
    let mut parser = parser::Parser::new(tokens);
    let ast = parser.parse();

    if env::var("PARSE_OUT").is_ok() {
        println!("{:#?}", ast);
    }

    // interpret
    let mut interpreter = interpreter::Interpreter::new();
    match ast.type_check(TypeEnviroment::new()){
        Ok(_) => {},
        Err(e) => {
            print!("TYPE ERROR: {}", e);
            return;
            // panic!("Type error")
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

