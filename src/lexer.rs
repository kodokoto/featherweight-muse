use crate::token::TokenKind;
pub struct Lexer {
    input: Vec<String>,
    current_position: usize,
}

impl Lexer {
    pub fn new(input: &str) -> Lexer {
        Lexer {
            input: input.split_whitespace().map(|s| s.to_string()).collect(),
            current_position: 0
        }
    }

    fn next_token(&mut self) -> TokenKind{
        // if token is valid, return token, else return error
        match self.input.get(self.current_position) {
            Some(token) => {
                match token.as_str() {
                    // "move" => Token::Move,
                    // "copy" => Token::Copy,
                    "deref" => TokenKind::Deref,
                    "box" => TokenKind::Box,
                    "ref" => TokenKind::Ref,
                    "let" => TokenKind::Let,
                    "mut" => TokenKind::Mut,
                    "fn" => TokenKind::Fn,
                    ":" => TokenKind::Colon,
                    "," => TokenKind::Comma,
                    "(" => TokenKind::LParen,
                    ")" => TokenKind::RParen,
                    "{" => TokenKind::LCurl,
                    "}" => TokenKind::RCurl,
                    "=" => TokenKind::Assign,
                    // "{" => Token::LBrace,
                    // "}" => Token::RBrace,
                    _ => {
                        if token.chars().all(char::is_numeric) {
                            TokenKind::NumericLiteral(token.parse::<i64>().unwrap())
                        } else if token.chars().all(char::is_alphabetic) {
                            TokenKind::Identifier(token.to_string())
                        } else {
                            // panic
                            panic!("Invalid token: {}", token)
                        }
                    }
                }
            },
            None => TokenKind::EOF
        }
    }
    
    pub fn tokenize(&mut self) -> Vec<TokenKind> {
        let mut tokens = Vec::new();
        loop {
            let token = self.next_token();
            self.current_position += 1;
            if token == TokenKind::EOF {
                break;
            }
            tokens.push(token);
        }
        tokens
    }
}