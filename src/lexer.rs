use regex::Regex;

use crate::token::Token;
pub struct Lexer {
    input: Vec<String>,
    current_position: usize,
}

impl Lexer {
    pub fn new(input: &str) -> Lexer {
        Lexer {
            input: Regex::new(r#"[^\W_]+|\S"#)
                .expect("regex")
                .captures_iter(input)
                .map(|c| c.get(0).unwrap().as_str().to_string())
                .collect::<Vec<String>>(),
            current_position: 0,
        }
    }

    fn next_token(&mut self) -> Token {
        // if token is valid, return token, else return error
        match self.input.get(self.current_position) {
            Some(token) => {
                match token.as_str() {
                    "*" => Token::Deref,
                    "box" => Token::Box,
                    "ref" => Token::Ref,
                    "let" => Token::Let,
                    "mut" => Token::Mut,
                    "fn" => Token::Fn,
                    ":" => Token::Colon,
                    "," => Token::Comma,
                    "(" => Token::LParen,
                    ")" => Token::RParen,
                    "{" => Token::LCurl,
                    "}" => Token::RCurl,
                    "=" => Token::Assign,
                    _ => {
                        if token.chars().all(char::is_numeric) {
                            Token::NumericLiteral(token.parse::<i64>().unwrap())
                        } else if token.chars().all(char::is_alphabetic) {
                            Token::Identifier(token.to_string())
                        } else {
                            // panic
                            panic!("Invalid token: {}", token)
                        }
                    }
                }
            }
            None => Token::EOF,
        }
    }

    pub fn tokenize(&mut self) -> Vec<Token> {
        let mut tokens = Vec::new();
        loop {
            let token = self.next_token();
            self.current_position += 1;
            if token == Token::EOF {
                break;
            }
            tokens.push(token);
        }
        tokens
    }
}
