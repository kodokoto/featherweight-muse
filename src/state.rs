use std::{collections::HashMap, sync::atomic::{AtomicUsize, Ordering}};

use crate::{ast::{Argument, LVal, Reference, Term, Value}, typing::Slot};
type Location = String;
#[derive(Debug, Clone)]
pub struct StackFrame {
    pub locations: HashMap<String, Reference>,
    pub functions: HashMap<String, (Vec<Argument>, Vec<Term>)>
}

#[derive(Debug, Clone)]
pub struct Store {
    pub cells: HashMap<Location, Slot<Value>>,
}

static COUNTER : AtomicUsize = AtomicUsize::new(1);
fn get_id() -> usize { COUNTER.fetch_add(1, Ordering::Relaxed) }

impl Store {

    pub fn new() -> Store {
        Store {
            cells: HashMap::new()
        }
    }

    pub fn allocate(&mut self, value: Value, lifetime: usize) -> Reference {
        let location = format!("l-{}", get_id());
        let reference = Reference {
            location: location.clone(),
            owned: true,
            path: vec![]
        };
        let slot = Slot {
            value: value.clone(),
            lifetime: lifetime
        };
        self.cells.insert(reference.location.clone(), slot);
        return reference;
    }

    pub fn read(&self, reference: Reference) -> Result<Value, String> {
        let location = reference.location;
        let value: Value = self.cells.get(&location).unwrap().value.clone();
        Ok(value)
    }

    pub fn write(&mut self, reference: Reference, value: Value) -> Result<(), String> {
        let location = reference.location;
        self.cells.get_mut(&location).unwrap().value = value;
        Ok(())
    }
    
    pub fn drop_lifetime(&mut self, lifetime: usize) {
        // filter out all cells with the given lifetime
        let cells = self.cells.clone();
        for (location, slot) in cells {
            if slot.lifetime == lifetime {
                self.cells.remove(&location);
            }
        }
    }

    pub fn drop(&mut self, value: &Value) -> Result<(), String> {
        match value {
            Value::Reference(r) => {
                if r.owned {
                    let location = &r.location;
                    let value = self.cells.get(location).unwrap().value.clone();
                    self.drop(&value)?;
                    self.cells.remove(location);
                }
            },
            _ => ()
        };
        Ok(())
    }

    pub fn get(&self, reference: Reference) -> Option<&Slot<Value>> {
        self.cells.get(&reference.location)
    }

}

#[derive(Debug, Clone)]

pub struct State {
    pub stack: Vec<StackFrame>,
    pub heap: Store,
}


impl State {
    pub fn new(stack: Vec<StackFrame>, heap: Store) -> State {
        State {
            stack,
            heap
        }
    }

    #[allow(dead_code)]
    pub fn output(&self) -> HashMap<String, String> {
        let mut output = HashMap::new();
        for frame in &self.stack {
            for (name, reference) in &frame.locations {
                let mut _ref = false;
                let mut value = self.heap.read(reference.clone()).unwrap();
                while match value {
                    Value::Reference(r) => {
                        value = self.heap.read(r).unwrap();
                        _ref = true;
                        true
                    },
                    _ => false
                } {};
                output.insert(name.clone(), format!("{:} {:}", if _ref {"ref"} else {""}, value));
            }
        }  
        return output  
    }

    pub fn print(&self) {
        for frame in &self.stack {
            for (name, reference) in &frame.locations {
                let mut _ref = false;
                let mut value = self.heap.read(reference.clone()).unwrap();
                while match value {
                    Value::Reference(r) => {
                        value = self.heap.read(r).unwrap();
                        _ref = true;
                        true
                    },
                    _ => false
                } {};
                println!("{}: {:} {:}", name, if _ref {"ref"} else {""}, value);
            }
        }
    }

    pub fn locate(&self, name: String) -> Result<Reference, String> {
        match self.top().locations.get(name.as_str()) {
            Some(reference) => Ok(reference.clone()),
            None => Err(format!("Error locating variable: {:?} does not exist in {:#?}", name, self.top()))
        }
    }


    pub fn dom(&self) -> Vec<String> {
        // for each stack frame, get the keys of the locations and create a set of all the keys
        let mut keys = vec![];
        for frame in &self.stack {
            for key in frame.locations.keys() {
                if keys.contains(key) {
                    continue
                } else {
                keys.push(key.clone());
                }
            }
        }
        return keys
    }

    pub fn top(&self) -> &StackFrame {
        let length = self.stack.len();
        let env = &self.stack.get(length - 1).unwrap();
        return env
    }

    pub fn top_mut(&mut self) -> &mut StackFrame {
        let length = self.stack.len();
        return &mut self.stack[length - 1]
    }

    pub fn add_function(&mut self, name: String, args: Vec<Argument>, body: Vec<Term>) {
        self.top_mut().functions.insert(name, (args, body));
    }

}

// Helper functions
pub fn add_function(mut s: State, name: String, args: Vec<Argument>, body: Vec<Term>) -> State {
    s.add_function(name, args, body);
    return s
}

pub fn loc(s: &State, variable: &LVal) -> Result<Reference, String> {
    // loc(S, x) = ℓ
    match variable {
        LVal::Variable{name, ..} => {
            s.locate(name.clone())
        },
        LVal::Deref { var, .. } => {
            let name = var.get_name();
            // get the reference to the value
            let reference = s.locate(name.clone())?;
            // get the value from the heap
            let value = s.heap.get(reference).unwrap().value.clone();
            match value {
                Value::Reference(r) => Ok(r),
                _ => Err(format!("Error dereferencing variable: {:?} is not a reference", name))
            }
        }
    }
}

pub fn read(s: &State, variable: &LVal) -> Result<Value, String>
{
    s.heap.read(loc(s, variable)?)
}

pub fn write(mut s: State, variable: &LVal, value: &Value) -> Result<State, String> {    
    s.heap.write(loc(&s, variable)?, value.clone())?;
    Ok(s)  
}

pub fn insert(mut s: State, lifetime: usize, value: &Value) -> (State, Reference) {
    // S [ℓw ↦ → ⟨v⊥⟩m]
    let r = s.heap.allocate(value.clone(), lifetime as usize);
    (s, r)
}

pub fn bind(mut s: State, variable: &String, reference: Reference) -> State {
    // S [ℓw ↦ → ⟨v⊥⟩m]
    s.top_mut().locations.insert(variable.clone(), reference);
    s
}

pub fn drop(mut s: State, value: &Value) -> Result<State, String> {
    // S [ℓw ↦ → ⟨·⟩m]
    s.heap.drop(value)?;
    Ok(s)
}

pub fn drop_lifetime(mut s: State, lifetime: usize) -> State {
    // S [ℓw ↦ → ⟨·⟩m]
    s.stack.remove(lifetime);
    s.heap.drop_lifetime(lifetime);
    s
}

