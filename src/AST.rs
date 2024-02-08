use std::fmt::Display;

use crate::state::{State, insert, read, drop, write};

pub type Reference = String;

pub trait AST {
    fn execute(&mut self, s: State) -> Result<(State, Self), String> where Self: Sized;
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

impl AST for Value {
    fn execute(&mut self, s: State) -> Result<(State, Value), String> {
        return Ok((s, self.clone()))
    }
}

#[derive(Debug, Clone)]
pub struct Variable {
    pub name: String,
}

impl AST for Variable {
    fn execute(&mut self, s: State) -> Result<(State, Variable), String> {
        return Ok((s, self.clone()))
    }
}

#[derive(Debug)]
pub struct Program {
    pub terms: Vec<Term>,
}

impl AST for Program {
    fn execute(&mut self, s: State) -> Result<(State, Program), String> {
        let state = s;
        let (s, t) = match self.terms.remove(0).execute(state) {
            Ok((s, t)) => (s, t),
            Err(e) => return Err(e)
        };
        let mut terms = self.terms.clone();
        return Ok((s, Program { terms }))
    }
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

impl AST for Term {
    fn execute(&mut self, s: State) -> Result<(State, Term), String> {
        match self {
            Term::Let { variable, term, .. } => {
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
                let (s2, t) = match term.execute(s) {
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
            Term::Ref { mutable: _, term } => {
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



