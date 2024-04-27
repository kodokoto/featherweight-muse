use std::fmt::Display;

use crate::{reduction::Evaluate, typecheck::TypeCheck, typing::Type};

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Reference {
    pub location: String,
    pub path: Vec<usize>,
    pub owned: bool, // owned references are dropped recursively
}

impl Display for Reference {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.location)
    }
}

pub trait AST: Evaluate + TypeCheck {}

#[derive(Debug, Clone, Eq, PartialEq, Hash)]
pub enum Value {
    NumericLiteral(i64),
    Reference(Reference),
    Epsilon,
    Undefined,
}

impl Display for Value {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Value::NumericLiteral(n) => write!(f, "{}", n),
            Value::Reference(r) => write!(f, "ref {}", r.location),
            Value::Epsilon => write!(f, "Epsilon"),
            Value::Undefined => write!(f, "Undefined"),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum LVal {
    Variable {
        name: String,
        copyable: Option<bool>,
    },
    Deref {
        var: Box<LVal>,
    },
}

impl LVal {
    pub fn get_name(&self) -> String {
        match self {
            LVal::Variable { name, .. } => name.clone(),
            LVal::Deref { var } => var.get_name().clone(),
        }
    }
    pub fn is_copyable(&self) -> Result<bool, String> {
        match self {
            LVal::Variable { copyable, .. } => match copyable {
                Some(c) => Ok(*c),
                None => Err(
                    "Tried to access copyable field of variable, however it was  before it was set"
                        .to_string(),
                ),
            },
            LVal::Deref { var } => var.is_copyable(),
        }
    }
    pub fn set_copyable(&mut self, copyable: bool) {
        match self {
            LVal::Variable { copyable: c, .. } => *c = Some(copyable),
            LVal::Deref { var } => var.set_copyable(copyable),
        }
    }
}

#[derive(Debug)]
pub struct Program {
    pub terms: Vec<Term>,
}

#[derive(Debug, Clone)]
pub enum Term {
    Variable(LVal),
    Value(Value),
    Box {
        term: Box<Term>,
    },
    Ref {
        mutable: bool,
        var: LVal,
    },
    Let {
        mutable: bool,
        variable: LVal,
        term: Box<Term>,
    },
    Assign {
        variable: LVal,
        term: Box<Term>,
    },
    FunctionCall {
        name: String,
        params: Vec<Term>,
    },
    FunctionDeclaration {
        name: String,
        args: Vec<Argument>,
        body: Vec<Term>,
        ty: Option<Type>,
    },
}

#[derive(Debug, Clone)]
pub struct Argument {
    pub name: String,
    pub ty: Type,
    pub mutable: bool,
    pub reference: bool,
}
