use std::{cell::Ref, collections::HashMap, os::macos::raw::stat};

use crate::{ast::{Argument, Declaration, Path, Program, Reference, Term, Value, Variable}, state::{add_function, bind, drop, drop_lifetime, insert, loc, read, write, StackFrame, State}, typecheck::TypeCheck, typing::AtomicType};

pub trait Evaluate {
    fn evaluate(&mut self, s: State, lifetime: usize) -> Result<(State, Self), String> where Self: Sized;
}

impl Evaluate for Value {
    fn evaluate(&mut self, s: State, lifetime: usize) -> Result<(State, Value), String> {
        return Ok((s, self.clone()))
    }
}

impl Evaluate for Variable {
    fn evaluate(&mut self, s: State, lifetime: usize) -> Result<(State, Variable), String> {
        return Ok((s, self.clone()))
    }
}

impl Evaluate for Program {
    fn evaluate(&mut self, s: State, lifetime: usize) -> Result<(State, Program), String> {
        let state = s;
        let (s, _) = match self.terms.remove(0).evaluate(state, lifetime) {
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
    fn evaluate(&mut self, s: State, lifetime: usize) -> Result<(State, Term), String> {
        match self {
            Term::FunctionCall { name, params } => {
                let (args, body, ty) = match s.top().functions.get(name) {
                    Some((args, body, ty)) => (args.clone(), body.clone(), ty.clone()),
                    None => return Err(format!("Function: {:?} not found", name))
                };
                

                println!("Reducing function call");

                let mut outer_state = s.clone();

                
                // evaluate the parameters

                let mut values: Vec<Value> = vec![];

                for (param, arg) in params.iter().zip(args.iter()) {
                    let (s2, t) = match (param, arg) {
                        (Term::Variable(var), Argument { name, ty, mutable: false, reference: true }) => {
                            // create a reference
                            let Ok(reference) = loc(&outer_state, &var) else {
                                return Err(format!("Variable: {:?} not found", var));
                            };
                            (outer_state, Term::Value(Value::Reference(reference)))
                        },
                        (_, _) => param.clone().evaluate(outer_state, lifetime)?
                    };
                    let value = match t {
                        Term::Value(v) => {
                            v
                        },
                        Term::Variable(var) => {
                            // read(S, w) = ⟨v⟩
                            let value = read(&s, &var)?;
                            value
                        },
                        _ => panic!("expression {:?} does not return a value", param)
                    };

                    values.push(value);
                    outer_state = s2;
                }

                let mut new_state = outer_state.clone();

                new_state.stack.push(
                    StackFrame {
                        locations: HashMap::new(),
                        functions: s.top().functions.clone()
                    }
                );

                for (arg, value) in args.iter().zip(values) {
                    let (s4, r) = insert(new_state, lifetime + 1, &value);
                    new_state = bind(s4, &arg.name, r);
                }
                println!("State after evaluating parameters: {:#?}", new_state);

                let mut t1: Term = Term::Value(Value::Epsilon);

                // evaluate the body of the function
                for term in body {
                    let (s3, t2) = term.clone().evaluate(new_state, lifetime + 1)?;
                    new_state = s3;
                    t1 = t2;
                }

                println!("State after evaluating body: {:#?}", new_state);
                println!("Returning: {:?}", t1);

                let v = match t1 {
                    Term::Value(v) => {
                        v
                    },
                    Term::Variable(var) => {
                        // get the value of the variable
                        let value = read(&new_state, &var)?;
                        value
                    },
                    _ => Value::Epsilon
                };

                println!("lifetime: {:?}", lifetime + 1);

                let final_state = drop_lifetime(new_state, lifetime + 1);

                return Ok((final_state, Term::Value(v)));

                // let (mut s3, t2) = t1.evaluate(s2)?;


                // // for each argument in the function, 
                // for (arg, param) in args.iter().zip(params.iter()) {
                //     let (mut s3, t) = param.clone().evaluate(s2, lifetime)?;
                //     let value = match arg {
                //         // copy the value into the state
                //         Argument { name, ty, mutable: false, reference: false} => {
                //             match t {
                //                 Term::Value(v) => {
                //                     println!("BOOOO");
                //                     v
                //                 },
                //                 Term::Variable(var) => {
                //                     // read(S, w) = ⟨v⟩
                //                     let value = read(&s3, &var)?;
                //                     value
                //                 },
                //                 _ => panic!("expression {:?} does not return a value", param)
                //             }
                //         },
                //         Argument { name, ty, mutable: true, reference: false} => {
                //             // move the value into the state
                //             match t {
                //                 Term::Value(v) => v,
                //                 Term::Variable(var) => {
                //                     // read(S, w) = ⟨v⟩
                //                     let value = read(&s3, &var)?;
                //                     value
                //                 },
                //                 _ => panic!("expression {:?} does not return a value", param)
                //             }
                //         },
                //         // Argument { name, ty, mutable: false, reference: true} => {
                //         //     let reference = match t {
                //         //         Term::Value(v) => Value::Reference(
                //         //             v
                //         //         ),
                //         //         _ => panic!("expression {:?} does not return a reference", param)
                //         //     };
                //         // }
                //         _ => panic!("OTHER BRANCHES NOT IMPLEMENTED")
                //     };

                //     // ugly i know but hey, time is of the essence
                    
                //     let (s4, r) = insert(new_state, lifetime, &value);
                //     new_state = bind(s4, &arg.name, r);
                //     s2 = s3;
                // }

                // println!(" outer state: {:#?}", s2);
                // println!(" inner state: {:#?}", new_state);

                // let mut t1: Term = Term::Value(Value::Epsilon);

                // // evaluate the body of the function
                // for term in body {
                //     let (s3, t2) = term.clone().evaluate(new_state, lifetime)?;
                //     new_state = s3;
                //     t1 = t2;
                // }

                // // let (mut s3, t2) = t1.evaluate(s2)?;

                // let t3 = match t1 {
                //     Term::Variable(v) => {
                //         // get the value of the variable
                //         let value = read(&new_state, &v)?;
                //         Term::Value(value)
                //     },
                //     _ => t1
                // };

                // new_state.pop();

                // println!("state after function call: {:?}", s2);
                // println!("returning: {:?}", t3);
                // match t3 {
                //     Term::Value(v) => {
                //         return Ok((s2, Term::Value(v)))
                //     },
                //     _ => {
                //         return Ok((s2, Term::Value(Value::Epsilon)))
                //     }
                // }
                panic!("Not implemented");
            }
            Term::FunctionDeclaration { name, args, body, ty } => {
                let s2 = add_function(s, name.to_string(), args.clone(), body.clone(), ty.clone());
                return Ok((s2, Term::Value(Value::Epsilon)));
            }
            Term::Let { variable, term, .. } => {
                println!("Reducing let {:?} = {:?}", variable, term);

                let (mut s2, t) = match term.evaluate(s, lifetime) {
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

                let (s3, r) = insert(s2, lifetime, &value);
                println!("Binding variable: {:?} to reference: {:?}", variable.name, r);
                // println!("{:#?}", s3.top());
                let s4 = bind(s3, &variable.name, r);

                println!("Heap after let: {:#?}", s4.heap);
                return Ok((s4, Term::Value(Value::Epsilon)))
            }
            Term::Assign { variable, term } => {
                println!("Reducing assignment of variable: {:?} with term: {:?}", variable.name, term);
                let (s2, t) = match term.evaluate(s, lifetime) {
                    Ok((s2, t)) => (s2, t),
                    Err(e) => return Err(e)
                };

                let value = match t {
                    Term::Value(v) => v,
                    _ => panic!("Invalid term, this should not happen")
                };
                let old_value = read(&s2, &variable)?;


                println!("Dropping: {:?}", old_value);
                println!("Heap before drop: {:#?}", s2.heap);

                let s3 = drop(s2, &old_value);

                println!("Heap after drop: {:#?}", s3.heap);
                let s4 = write(s3, &variable, &value)?;
                
                return Ok((s4, Term::Value(Value::Epsilon)))
            }
            // Term::Move { variable } => {
            //     let value = read(&s, variable)?;
            //     let s2 = write(s, variable, &Value::Undefined)?;
            //     println!("Reducing move of variable: {:?} with value: {:?}", variable, value);
            //     println!("State after move: {:#?}", s2);
            //     return Ok((s2, Term::Value(value)))
            // }
            // Term::Copy { variable } => {
            //     // read(S, w) = ⟨v⟩
            //     let value = read(&s, variable)?;
            //     return Ok((s, Term::Value(value)))
            // }

            Term::Box { term } => {
                // we need to evaluate the term to get the value before we can add to heap
                let (s3, value) = match term.evaluate(s, lifetime) {
                    Ok((s2, Term::Value(v))) => (s2, v),
                    Err(e) => return Err(e),
                    _ => panic!("Invalid term, this should not happen")
                };

                // ℓn ∉ dom(S1)

                // S2 = S1 [ℓn ↦ → ⟨v⟩∗]
                let (s4, r) = insert(s3, 0, &value);
                println!("Reference for boxed value: {:?}", r);
                // println!("{:#?}", s4.top());
                return Ok((s4, Term::Value(Value::Reference(r))))
            }

            Term::Ref { mutable: _, var } => {
                // check that term is a variable
                // read(S, w) = ⟨v⟩
                let Ok(mut reference) = loc(&s, &var) else {
                    return Err(format!("Variable: {:?} not found", var));
                };
                reference.owned = false;
                return Ok((s, Term::Value(Value::Reference(reference))))
            }

            Term::Variable(var) => {
                if var.copyable == Some(true) {
                    // read(S, w) = ⟨v⟩
                    let value = read(&s, var)?;
                    return Ok((s, Term::Value(value)))

                } else if var.copyable == Some(false) {
                    let value = read(&s, var)?;
                    let s2 = write(s, var, &Value::Undefined)?;
                    // println!("Reducing move of variable: {:?} with value: {:?}", variable, value);
                    // println!("State after move: {:#?}", s2);
                    return Ok((s2, Term::Value(value)))
                } else {
                    panic!("Variable: {:?} has not been typechecked properly", var.name)

                }
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