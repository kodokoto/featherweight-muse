#[derive(Debug, PartialEq, Clone)]
pub enum Token {
    NumericLiteral(i64),
    Identifier(String),
    // Move,
    // Copy,
    Box,
    Ref,
    Let,
    Mut,
    Assign,
    Fn,
    LParen,
    RParen,
    LCurl,
    RCurl,
    Comma,
    Colon,
    // LBrace,
    // RBrace,
    EOF
}
