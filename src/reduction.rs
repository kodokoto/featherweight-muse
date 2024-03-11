use std::{collections::HashMap, os::macos::raw::stat};

use crate::{ast::{Argument, Declaration, Path, Program, Reference, Term, Value, Variable}, state::{add_function, drop, insert, loc, read, write, Enviroment, State}, typecheck::TypeCheck, typing::AtomicType};

pub trait Evaluate {
    fn evaluate(&mut self, s: State) -> Result<(State, Self), String> where Self: Sized;
}

impl Evaluate for Value {
    fn evaluate(&mut self, s: State) -> Result<(State, Value), String> {
        return Ok((s, self.clone()))
    }
}

impl Evaluate for Variable {
    fn evaluate(&mut self, s: State) -> Result<(State, Variable), String> {
        return Ok((s, self.clone()))
    }
}

impl Evaluate for Program {
    fn evaluate(&mut self, s: State) -> Result<(State, Program), String> {
        let state = s;
        let (s, _) = match self.terms.remove(0).evaluate(state) {
            Ok((s, t)) => (s, t),
            Err(e) => return Err(e)
        };
        let terms = self.terms.clone();
        return Ok((s, Program { 
            terms 
        }))
    }
}

impl Evaluate for Term {
    fn evaluate(&mut self, s: State) -> Result<(State, Term), String> {
        match self {
            Term::FunctionCall { name, params } => {
                let (args, body, ty) = match s.functions.get(name) {
                    Some((args, body, ty)) => (args, body, ty),
                    None => return Err(format!("Function: {:?} not found", name))
                };
                
                let mut s2 = s.clone();

                println!("Reducing function call");

                let mut state_block = s2.clone();

                state_block.push(Enviroment {
                        locations: HashMap::new(),
                        state: HashMap::new(),
                });
                // evaluate the parameters

                // for each argument in the function, 
                for (arg, param) in args.iter().zip(params.iter()) {
                    let (mut s3, t) = param.clone().evaluate(s2)?;
                    let value = match arg {
                        // copy the value into the state
                        Argument { name, ty, mutable: false, reference: false} => {
                            match t {
                                Term::Value(v) => {
                                    println!("BOOOO");
                                    v
                                },
                                Term::Variable(var) => {
                                    // read(S, w) = ⟨v⟩
                                    let value = read(&s3, &var)?;
                                    value
                                },
                                _ => panic!("expression {:?} does not return a value", param)
                            }
                        },
                        Argument { name, ty, mutable: true, reference: false} => {
                            // move the value into the state
                            match t {
                                Term::Value(v) => v,
                                Term::Variable(var) => {
                                    // read(S, w) = ⟨v⟩
                                    let value = read(&s3, &var)?;
                                    value
                                },
                                _ => panic!("expression {:?} does not return a value", param)
                            }
                        },
                        // Argument { name, ty, mutable: false, reference: true} => {
                        //     let reference = match t {
                        //         Term::Value(v) => Value::Reference(
                        //             v
                        //         ),
                        //         _ => panic!("expression {:?} does not return a reference", param)
                        //     };
                        // }
                        _ => panic!("OTHER BRANCHES NOT IMPLEMENTED")
                    };

                    let reference = state_block.create_variable_reference(&Variable { name: arg.name.clone(), path: Path { selectors: vec![] }});
                    // ugly i know but hey, time is of the essence
                    
                    state_block = insert(state_block, reference, &value);
                    s2 = s3;
                }

                println!(" outer state: {:#?}", s2);
                println!(" inner state: {:#?}", state_block);

                let mut t1: Term = Term::Value(Value::Epsilon);

                // evaluate the body of the function
                for term in body {
                    let (s3, t2) = term.clone().evaluate(state_block)?;
                    state_block = s3;
                    t1 = t2;
                }

                // let (mut s3, t2) = t1.evaluate(s2)?;

                let t3 = match t1 {
                    Term::Variable(v) => {
                        // get the value of the variable
                        let value = read(&state_block, &v)?;
                        Term::Value(value)
                    },
                    _ => t1
                };

                state_block.pop();

                println!("state after function call: {:?}", s2);
                println!("returning: {:?}", t3);
                match t3 {
                    Term::Value(v) => {
                        return Ok((s2, Term::Value(v)))
                    },
                    _ => {
                        return Ok((s2, Term::Value(Value::Epsilon)))
                    }
                }
            }
            Term::FunctionDeclaration { name, args, body, ty } => {
                let s2 = add_function(s, name.to_string(), args.clone(), body.clone(), ty.clone());
                return Ok((s2, Term::Value(Value::Epsilon)));
            }
            Term::Let { variable, term, .. } => {
                println!("Reducing let {:?} = {:?}", variable, term);

                let (mut s2, t) = match term.evaluate(s) {
                    Ok((s2, t)) => (s2, t),
                    Err(e) => return Err(e)
                };



                let value = match t {
                    Term::Value(v) => v,
                    _ => {
                        println!("Oh no!");
                        println!("{:?}", t);
                        panic!("Invalid term, this should not happen")
                    }
                };

                let reference = s2.create_variable_reference(&variable);
                let s3 = insert(s2, reference, &value);
                return Ok((s3, Term::Value(Value::Epsilon)))
            }
            Term::Assign { variable, term } => {
                let (s2, t) = match term.evaluate(s) {
                    Ok((s2, t)) => (s2, t),
                    Err(e) => return Err(e)
                };

                let value = match t {
                    Term::Value(v) => v,
                    _ => panic!("Invalid term, this should not happen")
                };
                let old_value = read(&s2, &variable)?;
                let s3 = drop(s2, &old_value);
                let s4 = write(s3, &variable, &value)?;
                
                return Ok((s4, Term::Value(Value::Epsilon)))
            }
            Term::Move { variable } => {
                let value = read(&s, variable)?;
                let s2 = write(s, variable, &Value::Undefined)?;
                println!("Reducing move of variable: {:?} with value: {:?}", variable, value);
                println!("State after move: {:#?}", s2);
                return Ok((s2, Term::Value(value)))
            }
            Term::Copy { variable } => {
                // read(S, w) = ⟨v⟩
                let value = read(&s, variable)?;
                return Ok((s, Term::Value(value)))
            }

            Term::Box { term } => {
                // we need to evaluate the term to get the value before we can add to heap
                let (mut s3, value) = match term.evaluate(s) {
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

            Term::Ref { mutable: _, var } => {
                // check that term is a variable
                // read(S, w) = ⟨v⟩
                let Some(reference) = loc(&s, &var) else {
                    return Err(format!("Variable: {:?} not found", var));
                };
                return Ok((s, Term::Value(Value::Reference(reference))))
            }

            Term::Variable(var) => {
                // let value = read(&s, &var)?;
                panic!("This should not happen I think");
                return Ok((s, Term::Variable(var.clone())))
            }
            Term::Value(val) => {
                return Ok((s, Term::Value(val.clone())))
            }
        }
    }

}

// fn main() {
//     fn t(x: String) -> String {
//         return x;
//     }

//     let mut s = String::from("Hello");
//     let mut s2 = t(s);
//     println!("{:?}", s);
// }