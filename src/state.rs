use std::collections::HashMap;

use crate::ast::{Reference, Value, Variable};

#[derive(Debug, Clone)]
pub struct State {
    state: HashMap<Reference, Value>,
    locations: HashMap<String, Reference>,
}

impl State {
    pub fn new() -> State {
        return State {
            state: HashMap::new(),
            locations: HashMap::new()
        }
    }

    pub fn create_reference_from_variable(&mut self, variable: &Variable) -> Reference {
        // assert that reference does not already exist in program state
        let reference  = variable.name.clone();
        assert!(self.state.get(&reference).is_none());
        assert!(self.locations.get(&variable.name).is_none());
        self.locations.insert(variable.name.clone(), reference.clone());
        return reference
    }

    pub fn create_reference_from_value(&mut self, value: &Value) -> Reference {
        // assert that reference does not already exist in program state
        let reference = value.to_string();
        assert!(self.state.get(&reference).is_none());
        assert!(self.locations.get(&value.to_string()).is_none());
        self.locations.insert(value.to_string(), reference.clone());
        return reference
    }
}

// Helper functions

pub fn loc(s: &State, variable: &Variable) -> Option<Reference> {
    match  s.locations.get(&variable.name) {
        Some(reference) => Some(reference.clone()),
        None => None
    }
}

pub fn read(s: &State, variable: &Variable) -> Option<Value>
{
    let r = loc(&s, variable);
    match r {
        Some(reference) => {
            match s.state.get(&reference) {
                Some(value) => Some(value.clone()),
                None => None
            }
        },
        None => None
    }
}

pub fn write(mut s: State, variable: &Variable, value: &Value) -> Option<State> {
    let r = loc(&s, variable);
    match r {
        Some(reference) => {
            s.state.insert(reference, value.clone());
            Some(s)
        },
        None => None
    }
}

pub fn insert(mut s: State, reference: Reference, value: &Value) -> State {
    s.state.insert(reference, value.clone());
    return s
}

pub fn drop<'a>(mut s: State, variable: &Variable) -> State {
    let reference = loc(&s, variable);
    match reference {
        Some(reference) => {
            s.state.remove(&reference);
            s.locations.remove(&variable.name);
            return s
        },
        None => panic!("Variable: {} not found", variable.name)
    }
}