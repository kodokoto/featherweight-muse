use std::{collections::HashMap, fmt::Display};

use crate::{state::{drop, insert, loc, read, write, State}, typing::{dom, move_type, read_prohibited, shape_compatible, write_prohibited, Type, TypeEnviroment}};

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

pub trait AST {
    fn execute(&mut self, s: State) -> Result<(State, Self), String> where Self: Sized;
    fn type_check(&self, gamma: TypeEnviroment) -> Result<(TypeEnviroment, Type), String>;
}

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

impl AST for Value {
    fn execute(&mut self, s: State) -> Result<(State, Value), String> {
        return Ok((s, self.clone()))
    }

    fn type_check(&self, gamma: TypeEnviroment) -> Result<(TypeEnviroment, Type), String> {
        match self {
            Value::NumericLiteral(_) => return Ok((gamma, Type::Numeric)),
            // Value::Reference() => return Ok((gamma, Type::Reference)),
            Value::Epsilon => return Ok((gamma, Type::Epsilon)),
            _ => return Err("Invalid value".to_string())
        }
    }
}

pub type Variable = String;

impl AST for Variable {
    fn execute(&mut self, s: State) -> Result<(State, Variable), String> {
        return Ok((s, self.clone()))
    }

    fn type_check(&self, gamma: TypeEnviroment) -> Result<(TypeEnviroment, Type), String> {
        let t = match gamma.get(&self) {
            Ok(t) => t,
            Err(e) => return Err(e)
        };
        return Ok((gamma, t))
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

    fn type_check(&self, gamma: TypeEnviroment) -> Result<(TypeEnviroment, Type), String> {
        let mut gamma = gamma;
        let mut t = Type::Epsilon;
        for term in &self.terms {
            let (g, ty) = term.type_check(gamma)?;
            gamma = g;
            t = ty;
        }
        return Ok((gamma, t))
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

                let reference = s2.create_variable_reference(&variable);
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
                let old_value = read(&s2, &variable)?;
                println!("Old value: {:?}", old_value);
                let s3 = drop(s2, &old_value);
                let s4 = write(s3, &variable, &value)?;
                
                return Ok((s4, Term::Value(Value::Epsilon)))
            }
            Term::Move { variable } => {
                let value = read(&s, variable)?;
                let s2 = write(s, variable, &Value::Undefined)?;
                return Ok((s2, Term::Value(value)))
            }
            Term::Copy { variable } => {
                // read(S, w) = ⟨v⟩
                let value = read(&s, variable)?;
                return Ok((s, Term::Value(value)))
            }

            Term::Box { term } => {
                // we need to execute the term to get the value before we can add to heap
                let (mut s3, value) = match term.execute(s) {
                    Ok((s2, Term::Value(v))) => (s2, v),
                    Err(e) => return Err(e),
                    _ => panic!("Invalid term, this should not happen")
                };

                // ℓn ∉ dom(S1)
                let reference = s3.create_heap_reference();

                // S2 = S1 [ℓn ↦ → ⟨v⟩∗]
                let s3 = insert(s3, reference.clone(), &value);
                return Ok((s3, Term::Value(Value::Reference(reference))))
            }

            Term::Ref { mutable: _, term } => {
                println!("Term: {:?}", term);
                // check that term is a variable
                let variable = match *term.clone() {
                    Term::Variable(v) => v,
                    _ => panic!("Invalid term")
                };
                // read(S, w) = ⟨v⟩
                let Some(reference) = loc(&s, &variable) else {
                    panic!("Variable: {} not found", variable);
                    return Err(format!("Variable: {} not found", variable));
                };
                return Ok((s, Term::Value(Value::Reference(reference))))
            }

            Term::Variable(var) => {
                // let value = read(&s, &var)?;
                return Ok((s, Term::Variable(var.clone())))
            }
            Term::Value(val) => {
                return Ok((s, Term::Value(val.clone())))
            }
        }
    }

    fn type_check(&self, gamma: TypeEnviroment) -> Result<(TypeEnviroment, Type), String> {
        match self {
            Term::Variable(var) => {
                let (g, t) = var.type_check(gamma)?;
                return Ok((g, t))
            },
            Term::Value(val) => {
                let (g, t) = val.type_check(gamma)?;
                return Ok((g, t))
            },
            Term::Move { variable } => {
                let (g, t) = variable.type_check(gamma)?;
                if write_prohibited(&g, &variable)? {
                    return Err(format!("Error type-checking Move, variable {} is not writable", variable))
                };
                let gamma2 = move_type(g, variable)?;
                return Ok((gamma2, t))
            },
            Term::Copy { variable } => {
                let t = gamma.get(&variable).unwrap();
                if !t.is_copy_type() {
                    return Err(format!("Variable: {} is not copyable", variable))
                };
                if read_prohibited(&gamma, &variable)? {
                    return Err(format!("Variable: {} is not readable", variable))
                };
                return Ok((gamma, t.clone()))
            },
            Term::Box { term } => {
                let (g, t) = term.type_check(gamma)?;
                return Ok((g, Type::Box(Box::new(t))))
            },
            Term::Ref { mutable, term } => {
                let (g, t) = term.type_check(gamma)?;

                return Ok((g, Type::Reference { 
                    mutable: *mutable, 
                    kind: Box::new(t)
                }))
            },
            Term::Let { mutable, variable, term } => {
                if dom(&gamma).contains(&variable) {
                    return Err(format!("Error type-checking let declaration:  variable{} already exists", variable))
                };
                let (mut g, t) = term.type_check(gamma)?;
                g.insert(variable.clone(), t.clone());
                return Ok((g, Type::Epsilon))
            },
            Term::Assign { variable, term } => {
                let t1 = gamma.get_partial(&variable)?;
                let (g2, t2) = term.type_check(gamma)?;
                if !shape_compatible(&t1, &t2) {
                    return Err(format!("Error type-checking assignment:  types {:?} {:?} are not compatible", t1, t2))
                };
                if write_prohibited(&g2, &variable)? {
                    return Err(format!("Error type-checking assignment:  variable {} is not writable", variable))
                };
                return Ok((g2, Type::Epsilon))
            },
        }
    }
}



