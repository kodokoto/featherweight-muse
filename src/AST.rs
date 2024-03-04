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

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Path {
    pub selectors: Vec<PathSelector>
}

type PathSelector = i32;

// pub type Variable = String;
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Variable {
    pub name: String,
    pub path: Path
}

impl Variable {
    pub fn traverse(&self, p: Path, i: usize) -> Variable {
        let p1 = p.selectors;
        let p2 = &self.path.selectors;

        if p2.len() == i {
            return self.clone()
        } else {
            let n = p1.len();
            let m = p2.len();
            let mut nelements: Vec<PathSelector> = Vec::with_capacity(n + m);

// Extend nelements with elements from path_elements
            nelements.extend_from_slice(&p1[0..n]);

            // Extend nelements with elements from p_elements starting at index i
            nelements.extend_from_slice(&p2[i..i + m]);

            return Variable {
                name: self.name.clone(),
                path: Path {
                    selectors: nelements
                }
            }

        }
    }
}


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
        var: Variable,
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
    // Block {
    //     terms: Vec<Term>,
    //     lifetime: String
    // }
}




