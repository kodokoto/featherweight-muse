#[derive(Debug, PartialEq, Clone)]
pub enum TokenKind {
    NumericLiteral(i64),
    Identifier(String),
    // Move,
    // Copy,
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
    // LBrace,
    // RBrace,
    EOF
}

#[derive(Debug, PartialEq, Clone)]
pub struct Location {
    pub line: usize,
    pub column: usize
}

#[derive(Debug, PartialEq, Clone)]
pub struct Token {
    pub token_type: TokenKind,
    pub location: Location
}
