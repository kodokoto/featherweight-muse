use std::fmt::Display;

use crate::{reduction::Evaluate, typecheck::TypeCheck};

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Reference {
    pub location: String,
    pub owned: bool  // owned references are dropped recursively
}

impl Display for Reference {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.location)
    }
}

pub trait AST : Evaluate + TypeCheck {}

#[derive(Debug, Clone)]
pub enum Value {
    NumericLiteral(i64),
    Reference(Reference),
    Epsilon,
    Undefined
}

impl Display for Value {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Value::NumericLiteral(n) => write!(f, "{}", n),
            Value::Reference(r) => write!(f, "{}", r),
            Value::Epsilon => write!(f, "Epsilon"),
            Value::Undefined => write!(f, "Undefined")
        }
    }
}

pub type Variable = String;



#[derive(Debug)]
pub struct Program {
    pub terms: Vec<Term>,
}



#[derive(Debug, Clone)]
pub enum Term {
    Variable(Variable),
    Value(Value),
    Move {
        variable: Variable,
    },
    Copy {
        variable: Variable,
    },
    Box {
        term: Box<Term>,
    },
    Ref {
        mutable: bool, 
        term: Box<Term>,
    },
    Let {
        mutable: bool,
        variable: Variable,
        term: Box<Term>,
    },
    Assign {
        variable: Variable,
        term: Box<Term>,
    },
}




