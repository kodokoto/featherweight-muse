use std::fmt::Display;

use crate::state::{State, insert, read, drop, write};

pub type Reference = String;

pub trait AST {
    fn execute(&self, s: State) -> Result<(State, Term), String>;
}

#[derive(Debug, Clone)]
pub enum Value {
    NumericLiteral(i64),
    Reference(Reference),
    Epsilon,
}

impl Display for Value {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Value::NumericLiteral(n) => write!(f, "{}", n),
            Value::Reference(r) => write!(f, "{}", r),
            Value::Epsilon => write!(f, "Epsilon"),
        }
    }
}

#[derive(Debug)]
pub struct Variable {
    pub name: String,
}

#[derive(Debug)]
pub struct Program {
    pub terms: Vec<Term>,
}

#[derive(Debug)]
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

impl AST for Term {
    fn execute<'a>(&self, mut s: State) -> Result<(State, Term), String> {
        match self {
            Term::Let { mutable, variable, term } => {
                let (mut s2, t) = match term.execute(s) {
                    Ok((s2, t)) => (s2, t),
                    Err(e) => return Err(e)
                };

                let value = match t {
                    Term::Value(v) => v,
                    _ => panic!("Invalid term, this should not happen")
                };

                let reference = s2.create_reference_from_variable(&variable);
                let s3 = insert(s2, reference, &value);
                return Ok((s3, Term::Value(Value::Epsilon)))
            }
            Term::Assign { variable, term } => {
                let (mut s2, t) = match term.execute(s) {
                    Ok((s2, t)) => (s2, t),
                    Err(e) => return Err(e)
                };

                let value = match t {
                    Term::Value(v) => v,
                    _ => panic!("Invalid term, this should not happen")
                };
                let Some(s3) = drop(s2, &variable) else {
                    return Err(format!("Variable: {} not found", variable.name));
                };
                let Some(s4) = write(s3, &variable, &value) else {
                    return Err(format!("Variable: {} not found", variable.name));
                };
                return Ok((s4, Term::Value(Value::Epsilon)))
            }
            Term::Move { variable } => {
                let Some(value) = read(&s, &variable) else {
                    return Err(format!("Variable: {} not found", variable.name));
                };

                let Some(s2) = drop(s, &variable) else {
                    return Err(format!("Variable: {} not found", variable.name));
                };
                return Ok((s2, Term::Value(value)))
            }
            Term::Copy { variable } => {
                let Some(value) = read(&s, &variable) else {
                    return Err(format!("Variable: {} not found", variable.name));
                };

                return Ok((s, Term::Value(value)))
            }
            Term::Box { term } => {
                let (mut s2, t) = match term.execute(s) {
                    Ok((s2, t)) => (s2, t),
                    Err(e) => return Err(e)
                };

                let value = match t {
                    Term::Value(v) => v,
                    _ => panic!("Invalid term")
                };
                let reference = s2.create_reference_from_value(&value);
                let s3 = insert(s2, reference.clone(), &value);
                return Ok((s3, Term::Value(Value::Reference(reference))))
            }
            Term::Ref { mutable, term } => {
                let (mut s2, t) = match term.execute(s) {
                    Ok((s2, t)) => (s2, t),
                    Err(e) => return Err(e)
                };

                let value = match t {
                    Term::Value(v) => v,
                    _ => panic!("Invalid term")
                };
                let reference = s2.create_reference_from_value(&value);
                let s3 = insert(s2, reference.clone(), &value);
                return Ok((s3, Term::Value(Value::Reference(reference))))
            }
            Term::Variable(var) => {
                let Some(value) = read(&s, &var) else {
                    panic!("Variable: {} not found", var.name)
                };
                return Ok((s, Term::Value(value)))
            }
            Term::Value(val) => {
                return Ok((s, Term::Value(val.clone())))
            }
        }
    }
}



