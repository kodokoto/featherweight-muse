
// macro_rules! enum_str {
//     ($name:ident { $($variant:ident = $str:expr,)* }) => {
//         #[derive(Debug)]
//         pub enum $name {
//             $($variant,)*
//         }

//         impl $name {
//             pub fn as_str(&self) -> &str {
//                 match self {
//                     $($name::$variant => $str,)*
//                 }
//             }
//         }
//     };
// }

// enum_str!(
//     Errors {
//         ASSIGN_BORROWED = "Type error: Cannot assign to borrowed reference",
//         INCOMPATABLE_TYPES = "Type error: Incompatible types",
//     }
// );

use crate::{ast::Term, typing::Type};
pub enum TypeError {
    FUNCTION_CALL_INCOMPATABLE_ARGUMENT_COUNT(usize, usize),
    FUNCTION_CALL_INCOMPATABLE_ARGUMENT_TYPE(Type, Type),
    FUNCTION_NOT_DEFINED(String),
    FUNCTION_DECL_DUP_ARG(String),
    FUNCTION_UNEXPECTED_RETURN(Type, Type),
    COPY_NOT_READABLE(String),
    MOVE_NOT_WRITABLE(String),
    MUTREF_ALREADY_BORROWED_IMMUT(String),
    MUTREF_IMMUT(String),
    REF_ALREADY_BORROWED_MUT(String),
    LET_ALREADY_DEFINED(String),
    LET_EXPR_NO_RETURN(Term),
    ASSIGN_BORROWED(String),
    INCOMPATABLE_TYPES(Type, Type),
    NOT_WITHIN_SCOPE(String),
    TYPE_MOVED(Type)
}

impl TypeError {
    pub fn to_string(&self) -> String {
        match self {
            TypeError::FUNCTION_CALL_INCOMPATABLE_ARGUMENT_COUNT(a1, a2) => format!("Type error: Incompatible argument count: expected {}, got {}", a1, a2),
            TypeError::FUNCTION_CALL_INCOMPATABLE_ARGUMENT_TYPE(t1, t2) => format!("Type error: Incompatible argument type: expected {}, got {}", t1.to_string(), t2.to_string()),
            TypeError::FUNCTION_NOT_DEFINED(s) => format!("Type error: Function not defined: {}", s),
            TypeError::FUNCTION_DECL_DUP_ARG(s) => format!("Type error: Duplicate argument in function declaration: {}", s),
            TypeError::FUNCTION_UNEXPECTED_RETURN(t1, t2) => format!("Type error: Unexpected return type: expected {}, got {}", t1.to_string(), t2.to_string()),
            TypeError::COPY_NOT_READABLE(s) => format!("Type error: Cannot copy variable that is mutually borrowed: {}", s),
            TypeError::MOVE_NOT_WRITABLE(s) => format!("Type error: Cannot move variable that is borrowd: {}", s),
            TypeError::MUTREF_ALREADY_BORROWED_IMMUT(s) => format!("Type error: Mutable reference already borrowed immutably: {}", s),
            TypeError::MUTREF_IMMUT(s) => format!("Type error: Mutable reference cannot be created from immutable reference: {}", s),
            TypeError::REF_ALREADY_BORROWED_MUT(s) => format!("Type error: Immutable reference already borrowed mutably: {}", s),
            TypeError::LET_ALREADY_DEFINED(s) => format!("Type error: Variable already defined: {}", s),
            TypeError::LET_EXPR_NO_RETURN(s) => format!("Type error: Let expression does not return a value: {:?}", s),
            TypeError::INCOMPATABLE_TYPES(t1, t2) => format!("Type error: Incompatible types: {} and {}", t1.to_string(), t2.to_string()),
            TypeError::NOT_WITHIN_SCOPE(s) => format!("Type error: Type is not within scope: {}", s),
            TypeError::ASSIGN_BORROWED(s) => format!("Type error: Cannot assign to borrowed reference: {}", s),
            TypeError::TYPE_MOVED(t1) => format!("Type error: Type of {} is undefined, indicating that it was moved", t1.to_string())
        }
    }
}

