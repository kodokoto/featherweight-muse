use crate::{ast::Term, typing::Type};
pub enum TypeError {
    FunctionCallIncompatableArgumentCount(usize, usize),
    FunctionCallIncompatableArgumentType(Type, Type),
    FunctionNotDefined(String),
    FunctionDeclDupArg(String),
    FunctionUnexpectedReturn(Type, Type),
    CopyNotReadable(String),
    MoveNotWritable(String),
    MutrefAlreadyBorrowedImmut(String),
    MutrefImmut(String),
    RefAlreadyBorrowedMut(String),
    LetAlreadyDefined(String),
    LetExprNoReturn(Term),
    AssignBorrowed(String),
    IncompatibleTypes(Type, Type),
    NotWithinScope(String),
    TypeMoved(Type),
}

impl TypeError {
    pub fn to_string(&self) -> String {
        match self {
            TypeError::FunctionCallIncompatableArgumentCount(a1, a2) => format!("Type error: Incompatible argument count: expected {}, got {}", a1, a2),
            TypeError::FunctionCallIncompatableArgumentType(t1, t2) => format!("Type error: Incompatible argument type: expected {}, got {}", t1.to_string(), t2.to_string()),
            TypeError::FunctionNotDefined(s) => format!("Type error: Function not defined: {}()", s),
            TypeError::FunctionDeclDupArg(s) => format!("Type error: Duplicate argument in function declaration: {}", s),
            TypeError::FunctionUnexpectedReturn(t1, t2) => format!("Type error: Unexpected return type: expected {}, got {}", t1.to_string(), t2.to_string()),
            TypeError::CopyNotReadable(s) => format!("Type error: Cannot copy variable that is mutually borrowed: {}", s),
            TypeError::MoveNotWritable(s) => format!("Type error: Cannot move variable that is borrowd: {}", s),
            TypeError::MutrefAlreadyBorrowedImmut(s) => format!("Type error: Cannot create a mutable reference to {} as it's already borrowed immutably", s),
            TypeError::MutrefImmut(s) => format!("Type error: Mutable reference cannot be created from immutable reference: {}", s),
            TypeError::RefAlreadyBorrowedMut(s) => format!("Type error: Immutable reference already borrowed mutably: {}", s),
            TypeError::LetAlreadyDefined(s) => format!("Type error: Variable already defined: {}", s),
            TypeError::LetExprNoReturn(s) => format!("Type error: Let expression does not return a value: {:?}", s),
            TypeError::IncompatibleTypes(t1, t2) => format!("Type error: Incompatible types: {} and {}", t1.to_string(), t2.to_string()),
            TypeError::NotWithinScope(s) => format!("Type error: Type is not within scope: {}", s),
            TypeError::AssignBorrowed(s) => format!("Type error: Cannot assign to borrowed reference: {}", s),
            TypeError::TypeMoved(t1) => format!("Type error: Type of {} is undefined, indicating that it was moved", t1.to_string())
        }
    }
}
