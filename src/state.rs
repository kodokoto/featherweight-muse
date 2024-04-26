use std::{cell::Ref, collections::{btree_map::Values, HashMap}, hash::Hash, sync::atomic::{AtomicUsize, Ordering}};

use crate::{ast::{Argument, LVal, Reference, Term, Value}, typing::{Slot, Type}};
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

        // println!("Allocating value: {:?} with lifetime: {:?} to location {:?}", value, lifetime, self.cells.len());
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
        // println!("READING value from location: {:?}", reference.location);
        let location = reference.location;
        let mut value: Value = self.cells.get(&location).unwrap().value.clone();
        // if value is a reference, recursively read from the heap
        // value = match value.clone() {
        //     Value::Reference(r) => {
        //             self.read(r)?
        //     },
        //     _ => (value)
        // };
        Ok(value)
    }

    pub fn write(&mut self, reference: Reference, value: Value) -> Result<(), String> {
        // println!("Writing value: {:?} to location {:?}", value, reference.location);
        // println!("Heap before write: {:#?}", self.cells);
        let location = reference.location;
        // println!("Location: {:?}", location);
        self.cells.get_mut(&location).unwrap().value = value;
        // println!("Heap after write: {:#?}", self.cells);
        Ok(())
    }

    // pub fn drop(&mut self, reference: Reference) -> Result<(), String> {
    //     let location = reference.location.parse::<usize>().unwrap();
    //     let value = self.cells.get(location)?.value.clone();
    //     match value {
    //         Value::Reference(r) => {
    //             if r.owned {
    //                 self.drop(r)?;
    //             }
    //         },
    //         _ => Ok(())
    //     };
    //     self.cells.remove(location);
    //     Ok(())
    // }
    
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
    // pub fn allocate(&mut self, value: Value, lifetime: u64) -> Reference {
    //     let reference = Reference {
    //         location: format!("l-{}", self.heap.len()),
    //         owned: true
    //     };
    //     let slot = Slot {
    //         value: value.clone(),
    //         lifetime: lifetime
    //     };
    //     self.heap.push(slot);
    //     return reference;
    // }

    // pub fn read(&self, variable: &Variable) -> Result<Value, String>
    // {
    //     let reference = self.locate(variable.name.clone())?;
    //     match self.heap.get(reference) {
    //         Some(slot) => Ok(slot.value.clone()),
    //         None => Err(format!("Error reading from program state: {:?} does not exist in {:#?}", variable.name, self.top()))
    //     }
    // }

    // pub fn write(&mut self, variable: &Variable, value: &Value) -> Result<(), String> {
    //     let reference = self.locate(variable.name.clone())?;
    //     self.state.insert(reference, value.clone());
    //     Ok(())
    // }

    // pub fn create_heap_reference(&mut self) -> Reference {
    //     let location = format!("l-{}", self.heap_ref_counter);
    //     let reference = Reference {
    //         location: location.clone(),
    //         owned: false
    //     };
    //     self.heap_ref_counter += 1;
    //     return reference
    // }

    // pub fn create_variable_reference(&mut self, variable: &Variable) -> Reference {
    //     let location = format!("l-{:?}", variable.name);
    //     let reference = Reference {
    //         location: location.clone(),
    //         owned: true
    //     };
    //     assert!(self.top().state.get(&reference).is_none());
    //     assert!(self.top().locations.get(&variable.name.to_string()).is_none());
    //     self.top_mut().locations.insert(variable.name.to_string(), reference.clone());
    //     return reference
    // }

    pub fn top(&self) -> &StackFrame {
        let length = self.stack.len();
        let env = &self.stack.get(length - 1).unwrap();
        return env
    }

    pub fn top_mut(&mut self) -> &mut StackFrame {
        let length = self.stack.len();
        return &mut self.stack[length - 1]
    }

    pub fn push(&mut self, env: StackFrame) {
        self.stack.push(env);
    }

    pub fn pop(&mut self) {
        self.stack.pop();
    }

    pub fn add_function(&mut self, name: String, args: Vec<Argument>, body: Vec<Term>, ty: Option<Type>) {
        self.top_mut().functions.insert(name, (args, body));
    }

}



// Helper functions


pub fn add_function(mut s: State, name: String, args: Vec<Argument>, body: Vec<Term>, ty: Option<Type>) -> State {
    s.add_function(name, args, body, ty);
    return s
}

pub fn loc(s: &State, variable: &LVal) -> Result<Reference, String> {

    // println!("Locating variable: {:?}", variable.get_name());
    // loc(S, x) = ℓ
    match variable {
        LVal::Variable{name, ..} => {
            s.locate(name.clone())
        },
        LVal::Deref { var, .. } => {
            let name = var.get_name();
            // get the reference to the value
            let reference = s.locate(name.clone())?;
            // println!("Reference: {:?}", reference);
            // get the value from the heap
            let value = s.heap.get(reference).unwrap().value.clone();
            // println!("Value: {:?}", value);
            match value {
                Value::Reference(r) => Ok(r),
                _ => Err(format!("Error dereferencing variable: {:?} is not a reference", name))
            }
        }
    }
}

pub fn read(mut s: &State, variable: &LVal) -> Result<Value, String>
{
    s.heap.read(loc(s, variable)?)
}

pub fn write(mut s: State, variable: &LVal, value: &Value) -> Result<State, String> {
    
    s.heap.write(loc(&s, variable)?, value.clone())?;
    // println!("State after write: {:?}", s);
    Ok(s)  
}

pub fn insert(mut s: State, lifetime: usize, value: &Value) -> (State, Reference) {
    // S [ℓw ↦ → ⟨v⊥⟩m]
    let r = s.heap.allocate(value.clone(), lifetime as usize);
    // s.top_mut().locations.insert(format!("{}", lifetime), r.clone());
    (s, r)
}

pub fn bind(mut s: State, variable: &String, reference: Reference) -> State {
    // S [ℓw ↦ → ⟨v⊥⟩m]
    s.top_mut().locations.insert(variable.clone(), reference);
    s
}

pub fn drop(mut s: State, value: &Value) -> State {
    // S [ℓw ↦ → ⟨·⟩m]
    s.heap.drop(value);
    s
}

pub fn drop_lifetime(mut s: State, lifetime: usize) -> State {
    // S [ℓw ↦ → ⟨·⟩m]
    // println!("Dropping lifetime: {:?}", lifetime);
    s.stack.remove(lifetime);
    s.heap.drop_lifetime(lifetime);
    s
}

