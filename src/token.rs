#[derive(Debug, PartialEq, Clone)]
pub enum Token {
    NumericLiteral(i64),
    Identifier(String),
    Box,
    Ref,
    Deref,
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
    EOF
}
