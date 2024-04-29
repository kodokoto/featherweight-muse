use std::collections::HashMap;

use crate::{
    ast::{Argument, LVal, Program, Term, Value},
    state::{add_function, bind, drop, drop_lifetime, insert, loc, read, write, StackFrame, State},
};

pub trait Evaluate {
    fn evaluate(&mut self, s: State, lifetime: usize) -> Result<(State, Self), String>
    where
        Self: Sized;
}

impl Evaluate for Value {
    fn evaluate(&mut self, s: State, _lifetime: usize) -> Result<(State, Value), String> {
        return Ok((s, self.clone()));
    }
}

impl Evaluate for LVal {
    fn evaluate(&mut self, s: State, _lifetime: usize) -> Result<(State, LVal), String> {
        return Ok((s, self.clone()));
    }
}

impl Evaluate for Program {
    fn evaluate(&mut self, s: State, lifetime: usize) -> Result<(State, Program), String> {
        let state = s;
        let (s, _) = match self.terms.remove(0).evaluate(state, lifetime) {
            Ok((s, t)) => (s, t),
            Err(e) => return Err(e),
        };
        let terms = self.terms.clone();
        return Ok((s, Program { terms }));
    }
}

impl Evaluate for Term {
    fn evaluate(&mut self, s: State, lifetime: usize) -> Result<(State, Term), String> {
        match self {
            Term::FunctionCall { name, params } => {
                println!("Reducing function call: {:?}()", name);
                let (args, body) = match s.top().functions.get(name) {
                    Some((args, body)) => (args.clone(), body.clone()),
                    None => return Err(format!("Function: {:?} not found", name)),
                };

                let mut outer_state = s.clone();

                // evaluate the parameters

                let mut values: Vec<Value> = vec![];

                for (param, arg) in params.iter().zip(args.iter()) {
                    let (s2, t) = match (param, arg) {
                        (
                            Term::Variable(var),
                            Argument {
                                mutable: false,
                                reference: true,
                                ..
                            },
                        ) => {
                            // create a reference
                            let Ok(mut reference) = loc(&outer_state, &var) else {
                                return Err(format!("Variable: {:?} not found", var));
                            };
                            reference.owned = false;
                            (outer_state, Term::Value(Value::Reference(reference)))
                        }
                        (
                            Term::Variable(var),
                            Argument {
                                mutable: true,
                                reference: true,
                                ..
                            },
                        ) => {
                            // read(S, w) = ⟨v⟩
                            let Ok(mut reference) = loc(&outer_state, &var) else {
                                return Err(format!("Variable: {:?} not found", var));
                            };
                            reference.owned = false;
                            (outer_state, Term::Value(Value::Reference(reference)))
                        }
                        (_, _) => param.clone().evaluate(outer_state, lifetime)?,
                    };
                    let value = match t {
                        Term::Value(v) => v,
                        Term::Variable(var) => {
                            // read(S, w) = ⟨v⟩
                            let value = read(&s, &var)?;
                            value
                        }
                        _ => panic!("expression {:?} does not return a value", param),
                    };

                    values.push(value);
                    outer_state = s2;
                }

                let mut new_state = outer_state.clone();

                new_state.stack.push(StackFrame {
                    locations: HashMap::new(),
                    functions: s.top().functions.clone(),
                });

                for (arg, value) in args.iter().zip(values) {
                    let (s4, r) = insert(new_state, lifetime + 1, &value);
                    new_state = bind(s4, &arg.name, r);
                }

                let mut t1: Term = Term::Value(Value::Epsilon);

                // evaluate the body of the function
                println!("Evaluating body of function: {:?}()", name);
                for term in body {
                    let (s3, t2) = term.clone().evaluate(new_state, lifetime + 1)?;
                    new_state = s3;
                    t1 = t2;
                }

                println!("Finished evaluating body of function: {:?}()", name);

                let v = match t1 {
                    Term::Value(v) => v,
                    Term::Variable(var) => {
                        // get the value of the variable
                        let value = read(&new_state, &var)?;
                        value
                    }
                    _ => Value::Epsilon,
                };

                let final_state = drop_lifetime(new_state, lifetime + 1);

                return Ok((final_state, Term::Value(v)));
            }
            Term::FunctionDeclaration {
                name, args, body, ..
            } => {
                println!("Reducing function declaration: {:?}()", name);
                let s2 = add_function(s, name.to_string(), args.clone(), body.clone());
                return Ok((s2, Term::Value(Value::Epsilon)));
            }
            Term::Let { variable, term, .. } => {
                let (s2, t) = match term.evaluate(s, lifetime) {
                    Ok((s2, t)) => (s2, t),
                    Err(e) => return Err(e),
                };

                let value = match t {
                    Term::Value(v) => v,
                    _ => {
                        panic!("Invalid term, this should not happen")
                    }
                };

                let (s3, r) = insert(s2, lifetime, &value);
                let s4 = bind(s3, &variable.get_name(), r);

                return Ok((s4, Term::Value(Value::Epsilon)));
            }

            Term::Assign { variable, term } => {
                let (s2, t) = match term.evaluate(s, lifetime) {
                    Ok((s2, t)) => (s2, t),
                    Err(e) => return Err(e),
                };

                // sanity check
                let value = match t {
                    Term::Value(v) => v,
                    _ => panic!("Invalid term, this should not happen"),
                };

                // v′ = read(S, w)
                let old_value = read(&s2, &variable)?;

                // S′ = drop(S, v′)
                let s3 = drop(s2, &old_value)?;

                // S′′ = write(S′, w, v)
                let s4 = write(s3, &variable, &value)?;

                return Ok((s4, Term::Value(Value::Epsilon)));
            }

            Term::Box { term } => {
                println!("Reducing box");
                // we need to evaluate the term to get the value before we can add to heap
                let (s3, value) = match term.evaluate(s, lifetime) {
                    Ok((s2, Term::Value(v))) => (s2, v),
                    Err(e) => return Err(e),
                    _ => panic!("Invalid term, this should not happen"),
                };

                // ℓn ∉ dom(S1)

                // S2 = S1 [ℓn ↦ → ⟨v⟩∗]
                let (s4, r) = insert(s3, 0, &value);
                return Ok((s4, Term::Value(Value::Reference(r))));
            }

            Term::Ref { mutable: _, var } => {
                println!("Reducing ref of variable: {:?}", var.get_name());
                // check that term is a variable
                // read(S, w) = ⟨v⟩
                let Ok(mut reference) = loc(&s, &var) else {
                    return Err(format!("Variable: {:?} not found", var));
                };
                reference.owned = false;
                return Ok((s, Term::Value(Value::Reference(reference))));
            }

            Term::Variable(var) => {
                if var.is_copyable()? == true {
                    println!("Reducing copy of variable: {:?}", var.get_name());
                    // read(S, w) = ⟨v⟩
                    let value = read(&s, var)?;
                    return Ok((s, Term::Value(value)));
                } else if var.is_copyable()? == false {
                    println!("Reducing move of variable: {:?}", var.get_name());
                    let value = read(&s, var)?;
                    let s2 = write(s, var, &Value::Undefined)?;

                    return Ok((s2, Term::Value(value)));
                } else {
                    panic!(
                        "Variable: {:?} has not been typechecked properly",
                        var.get_name()
                    )
                }
            }
            Term::Value(val) => {
                println!("Reducing value: {:?}", val);
                return Ok((s, Term::Value(val.clone())));
            }
        }
    }
}
