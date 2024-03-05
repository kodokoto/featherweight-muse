use std::{collections::HashMap, hash::Hash};

use crate::{ast::{Reference, Term, Value, Variable, Argument}, typing::AtomicType};

#[derive(Debug, Clone)]
pub struct Enviroment {
    pub state: HashMap<Reference, Value>,
    pub locations: HashMap<String, Reference>
}

#[derive(Debug, Clone)]
pub struct State {
    stack: Vec<Enviroment>,
    heap_ref_counter: u64,
    pub functions: HashMap<String, (Vec<Argument>, Vec<Term>, Option<AtomicType>)>
}

impl State {
    pub fn new() -> State {
        return State {
            stack: vec![Enviroment {
                state: HashMap::new(),
                locations: HashMap::new()
            }],
            heap_ref_counter: 1,
            functions: HashMap::new()
        }
    }

    pub fn create_heap_reference(&mut self) -> Reference {
        let location = format!("l-{}", self.heap_ref_counter);
        let reference = Reference {
            location: location.clone(),
            owned: false
        };
        self.heap_ref_counter += 1;
        return reference
    }

    pub fn create_variable_reference(&mut self, variable: &Variable) -> Reference {
        let location = format!("l-{:?}", variable.name);
        let reference = Reference {
            location: location.clone(),
            owned: true
        };
        assert!(self.top().state.get(&reference).is_none());
        assert!(self.top().locations.get(&variable.name.to_string()).is_none());
        self.top_mut().locations.insert(variable.name.to_string(), reference.clone());
        return reference
    }

    pub fn top(&self) -> &Enviroment {
        let length = self.stack.len();
        let env = &self.stack.get(length - 1).unwrap();
        return env
    }

    pub fn top_mut(&mut self) -> &mut Enviroment {
        let length = self.stack.len();
        return &mut self.stack[length - 1]
    }

    pub fn push(&mut self) {
        self.stack.push(Enviroment {
            state: HashMap::new(),
            locations: HashMap::new()
        });
    }

    pub fn pop(&mut self) {
        self.stack.pop();
    }
}


pub fn push(mut s: State) -> State {
    s.push();
    return s
}
// Helper functions

pub fn add_function(mut s: State, name: String, args: Vec<Argument>, body: Vec<Term>, ty: Option<AtomicType>) -> State {
    s.functions.insert(name, (args, body, ty));
    return s
}

pub fn loc(s: &State, variable: &Variable) -> Option<Reference> {
    // loc(S, x) = ℓ
    match  s.top().locations.get(variable.name.as_str()) {
        Some(reference) => Some(reference.clone()),
        None => None
    }
}

pub fn read(mut s: &State, variable: &Variable) -> Result<Value, String>
{
    // where loc(S, w) = ℓw
    let Some(reference) = loc(&mut s, variable) else {
        return Err(format!("Error reading from program state: Variable {:?} does not exist in state", variable))
    };
    
    // S(ℓw)
    match s.top().state.get(&reference) {
        Some(value) => Ok(value.clone()),
        None => Err(format!("Error reading from program state: Variable {:?} does not exist in state", variable))
    }
}

pub fn write(mut s: State, variable: &Variable, value: &Value) -> Result<State, String> {
    
    // where loc(S, w) = ℓw and S(ℓw) = ⟨·⟩m
    let Some(r) = loc(&mut s, variable) else {
        return Err(format!("Error writing to program state: Variable {:?} does not exist in state", variable))
    }; 

    // and S(ℓw) = ⟨·⟩m
    if s.top().state.get(&r).is_none() {
        return Err(format!("Error writing to program state: Variable {:?} does not exist in state", variable))
    }

    // S [ℓw ↦ → ⟨v⊥⟩m] 
    return Ok(insert(s, r, value));    
}

pub fn insert(mut s: State, reference: Reference, value: &Value) -> State {
    // S [ℓw ↦ → ⟨v⊥⟩m]
    s.top_mut().state.insert(reference, value.clone());
    return s
}

pub fn drop(mut s: State, value: &Value) -> State {
    match value {
        Value::Reference(r @ Reference { location, owned }) => {
            // if owned, recursively drop the value
            if *owned {
                let v = s.top_mut().state.remove(r).unwrap();
                drop(s, &v)
            } else {
                s.top_mut().state.remove(r);
                s.top_mut().locations.remove(location);
                s
            }
        },
        _ => s
    }
}