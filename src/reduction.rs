use crate::{ast::{Program, Term, Value, Variable}, state::{loc, write, drop, insert, read, State}};

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
        return Ok((s, Program { terms }))
    }
}

impl Evaluate for Term {
    fn evaluate(&mut self, s: State) -> Result<(State, Term), String> {
        match self {
            Term::Let { variable, term, .. } => {
                let (mut s2, t) = match term.evaluate(s) {
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

            Term::Ref { mutable: _, term } => {
                // check that term is a variable
                let variable = match *term.clone() {
                    Term::Variable(v) => v,
                    _ => panic!("Invalid term")
                };
                // read(S, w) = ⟨v⟩
                let Some(reference) = loc(&s, &variable) else {
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

}
