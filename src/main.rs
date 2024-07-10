use std::env;

use typecheck::TypeCheck;
use typing::TypeEnviroment;

mod ast;
mod constants;
mod interpreter;
mod lexer;
mod parser;
mod properties;
mod reduction;
mod run_tests;
mod state;
mod token;
mod typecheck;
mod typing;

fn main() {
    env::set_var("RUST_BACKTRACE", "1");
    // take first arg as file dir
    let args: Vec<String> = std::env::args().collect();

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
            }
            "-p" | "-parse" => {
                env::set_var("PARSE_OUT", "1");
            }
            "-t" | "-typecheck" => {
                env::set_var("TYPE_OUT", "1");
            }
            "-e" | "-eval" => {
                env::set_var("EVAL_OUT", "1");
            }
            _ => {}
        }
    }

    // get file contents
    let file_dir = &args[args.len() - 1];
    if let Ok(file_contents) = std::fs::read_to_string(file_dir) {
        // tokenize
        let mut lexer = lexer::Lexer::new(&file_contents);
        let tokens = lexer.tokenize();
        if env::var("LEX_OUT").is_ok() {
            println!("{:?}", tokens);
        }

        // parse
        let mut parser = parser::Parser::new(tokens);
        let mut ast = parser.parse();

        if env::var("PARSE_OUT").is_ok() {
            println!("{:#?}", ast);
        }

        // interpret
        let mut interpreter = interpreter::Interpreter::new();
        match ast.type_check(TypeEnviroment::new(), 0) {
            Ok(_) => {}
            Err(e) => {
                println!("TYPE ERROR: {}", e);
                return;
            }
        };

        // return;s
        let res = interpreter.run(ast);

        // print result
        match res {
            Ok(v) => match v {
                ast::Value::Epsilon => {}
                _ => println!("{:?}", v),
            },
            Err(e) => {
                println!("ERROR: {}", e);
            }
        }
    } else {
        println!("No file provided")
    }

    
}
