use std::collections::HashMap;

use crate::ast::{Reference, Value, Variable};

#[derive(Debug, Clone)]
pub struct State {
    state: HashMap<Reference, Value>,
    locations: HashMap<String, Reference>,
    heap_ref_counter: u64
}

impl State {
    pub fn new() -> State {
        return State {
            state: HashMap::new(),
            locations: HashMap::new(),
            heap_ref_counter: 1
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
        let location = format!("l-{}", variable);
        let reference = Reference {
            location: location.clone(),
            owned: true
        };
        assert!(self.state.get(&reference).is_none());
        assert!(self.locations.get(&variable.to_string()).is_none());
        self.locations.insert(variable.to_string(), reference.clone());
        return reference
    }

    // pub fn create_reference_from_variable(&mut self, variable: &Variable, owned: bool) -> Reference {
    //     // assert that reference does not already exist in program state
    //     // let reference  = variable.clone();
    //     // assert!(self.state.get(&reference).is_none());
    //     // assert!(self.locations.get(variable).is_none());
    //     // self.locations.insert(variable.clone(), reference.location.clone());
    //     // return reference
    //     let location = format!("l-{}", variable);
    //     let reference = Reference {
    //         location: location.clone(),
    //         owned
    //     };
    //     assert!(self.state.get(&reference).is_none());
    //     assert!(self.locations.get(&variable.to_string()).is_none());
    //     self.locations.insert(variable.to_string(), reference.clone());

    // }

    // pub fn create_reference_from_value(&mut self, value: &Value, owned: bool) -> Reference {
    //     // assert that reference does not already exist in program state
    //     if let 

    //     let location = format!("l-{}", self.ref_counter);
    //     let reference = Reference {
    //         location: location.clone(),
    //         owned: true
    //     };
    //     assert!(self.state.get(&reference).is_none());
    //     assert!(self.locations.get(&value.to_string()).is_none());
    //     self.locations.insert(value.to_string(), reference.clone());
    //     return reference
    // }
}

// Helper functions

pub fn loc(s: &State, variable: &Variable) -> Option<Reference> {
    // loc(S, x) = ℓ
    match  s.locations.get(variable) {
        Some(reference) => Some(reference.clone()),
        None => None
    }
}

pub fn read(s: &State, variable: &Variable) -> Result<Value, String>
{
    // where loc(S, w) = ℓw
    let Some(reference) = loc(&s, variable) else {
        return Err(format!("Error reading from program state: Variable {} does not exist in state", variable))
    };
    
    // S(ℓw)
    match s.state.get(&reference) {
        Some(value) => Ok(value.clone()),
        None => Err(format!("Error reading from program state: Variable {} does not exist in state", variable))
    }
}

pub fn write(s: State, variable: &Variable, value: &Value) -> Result<State, String> {
    
    // where loc(S, w) = ℓw and S(ℓw) = ⟨·⟩m
    let Some(r) = loc(&s, variable) else {
        return Err(format!("Error writing to program state: Variable {} does not exist in state", variable))
    }; 

    // and S(ℓw) = ⟨·⟩m
    if s.state.get(&r).is_none() {
        return Err(format!("Error writing to program state: Variable {} does not exist in state", variable))
    }

    // S [ℓw ↦ → ⟨v⊥⟩m] 
    return Ok(insert(s, r, value));    
}

pub fn insert(mut s: State, reference: Reference, value: &Value) -> State {
    // S [ℓw ↦ → ⟨v⊥⟩m]
    s.state.insert(reference, value.clone());
    return s
}

// pub fn drop<'a>(mut s: State, variable: &Variable) -> Option<State> {
//     let reference = loc(&s, variable);
//     match reference {
//         Some(reference) => {
//             s.state.remove(&reference);
//             s.locations.remove(variable);
//             return Some(s)
//         },
//         None => None
//     }
// }

pub fn drop(mut s: State, value: &Value) -> State {
    match value {
        Value::Reference(r @ Reference { location, owned }) => {
            // if owned, recursively drop the value
            if *owned {
                let v = s.state.remove(r).unwrap();
                drop(s, &v)
            } else {
                s.state.remove(r);
                s.locations.remove(location);
                s
            }
        },
        _ => s
    }
}