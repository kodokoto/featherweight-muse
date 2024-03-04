#[derive(Debug, PartialEq, Clone)]
pub enum Token {
    NumericLiteral(i64),
    Identifier(String),
    Move,
    Copy,
    Box,
    Ref,
    Let,
    Mut,
    Assign,
    // LBrace,
    // RBrace,
    EOF
}
