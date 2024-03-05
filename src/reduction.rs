use crate::{ast::{Declaration, Path, Program, Term, Value, Variable}, state::{add_function, drop, insert, loc, push, read, write, State}};

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
                let s2 = push(s.clone());

                if args.len() != params.len() {
                    return Err(format!("Function: {:?} expected {:?} arguments, got {:?} arguments", name, args.len(), params.len()))
                }

                let mut s3 = s2.clone();

                // for each argument in the function, 
                for (arg, param) in args.iter().zip(params.iter()) {
                    let (mut s2, t) = match param.clone().evaluate(s3) {
                        Ok((s2, t)) => (s2, t),
                        Err(e) => return Err(e)
                    };
                    let value = match t {
                        Term::Value(v) => v,
                        _ => panic!("Invalid term, this should not happen")
                    };
                    let reference = s2.create_variable_reference(&Variable { name: arg.name.clone(), path: Path { selectors: vec![] }});
                    s3 = insert(s2, reference, &value);
                }

                let mut t1: Term = Term::Value(Value::Epsilon);

                // evaluate the body of the function
                for term in body {
                    let (s4, t2) = match term.clone().evaluate(s3) {
                        Ok((s4, t2)) => (s4, t2),
                        Err(e) => return Err(e)
                    };
                    s3 = s4;
                    t1 = t2;
                }
                s3.pop();

                match t1 {
                    Term::Value(v) => {
                        return Ok((s3, Term::Value(v)))
                    },
                    _ => {
                        return Ok((s3, Term::Value(Value::Epsilon)))
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
                return Ok((s, Term::Variable(var.clone())))
            }
            Term::Value(val) => {
                return Ok((s, Term::Value(val.clone())))
            }
        }
    }

}
