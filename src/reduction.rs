
use crate::ast::{Term, Value, Reference, Program, AST, Variable};
use std::{cell::Ref, collections::HashMap};


struct State {
    state: HashMap<Reference, Value>,
    locations: HashMap<String, Reference>,
}

impl State {

    fn new() -> State {
        return State {
            state: HashMap::new(),
            locations: HashMap::new()
        }
    }

    fn create_reference_from_variable(&mut self, variable: &Variable) -> Reference {
        // assert that reference does not already exist in program state
        let reference  = Reference { loc: variable.name.clone() };
        assert!(self.state.get(&reference).is_none());
        assert!(self.locations.get(&variable.name).is_none());
        self.locations.insert(variable.name.clone(), reference.clone());
        return reference
    }

    fn create_reference_from_value(&mut self, value: &Value) -> Reference {
        // assert that reference does not already exist in program state
        let reference = Reference { loc: value.to_string() };
        assert!(self.state.get(&reference).is_none());
        assert!(self.locations.get(&value.to_string()).is_none());
        self.locations.insert(value.to_string(), reference.clone());
        return Reference { loc: value.to_string() }
    }

    fn read(&self, variable: &Variable) -> Value {
        let r = self.loc(variable);
        return match self.state.get(&r) {
            Some(value) => value.clone(),
            None => panic!("Reference not found in state: {:?}", r)
        }
    }

    fn write(&mut self, variable: &Variable, value: &Value) {
        let reference = self.loc(variable);
        self.state.insert(reference, value.clone());
    }

    fn insert(&mut self, reference: Reference, value: &Value) {
        self.state.insert(reference, value.clone());
    }

    fn drop(&mut self, variable: &Variable) {
        let reference = self.loc(variable);
        self.state.remove(&reference);
    }

    fn loc(&self, variable: &Variable) -> Reference {
        // get reference from hashmap
        match  self.locations.get(&variable.name) {
            Some(reference) => reference.clone(),
            None => panic!("Reference not found in state: {:?}", variable)
        }
    }
}

pub struct Reducer {
    state: State,
}   

impl Reducer {
    pub fn new() -> Reducer {
        return Reducer {
            state: State::new(),
        }
    }

    pub fn r_program(&mut self, program: &Program) -> Value {
        let mut value = Value::Epsilon;
        for term in &program.terms {
            value = self.r_term(term);
            println!("State: {:?}", self.state.state);
            println!("Locations: {:?}", self.state.locations);
        }
        return value
    }

    fn r_term(&mut self, term: &Term) -> Value {
        println!("Reducing term: {:?}", term);
        return match term {
            Term::Let { mutable, var, t } => {
                let v = self.r_term(t);
                self.r_declare(mutable, var, &v)
            }
            Term::Assign { var, t } => {
                let v = self.r_term(t);
                self.r_assign(var, &v)
            }
            Term::Move { var } => {
                self.r_move(var)
            }
            Term::Copy { var } => {
                self.r_copy(var)
            }
            Term::Box { t } => {
                let v = self.r_term(t);
                self.r_box(&v)
            }
            Term::Ref { mutable, t } => {
                let v = self.r_term(t);
                self.r_reference(mutable, &v)
            }
            Term::Variable(var) => {
                self.state.read(var)
            }
            Term::Value(val) => {
                val.clone()
            }

        }
    }

    fn r_declare(&mut self, mutable: &bool, var: &Variable, value: &Value) -> Value {
        let reference: Reference = self.state.create_reference_from_variable(var);
        self.state.insert(reference, value);
        return Value::Epsilon
    }

    fn r_assign(&mut self, var: &Variable, value: &Value) -> Value {
        self.state.drop(var);
        self.state.write(var, value);
        return Value::Epsilon
    }

    fn r_move(&mut self, var: &Variable) -> Value {
        let value = self.state.read(&var);
        self.state.drop(var); // TODO: double check this
        return value
    }

    fn r_copy(&mut self, var: &Variable) -> Value {
        let value = self.state.read(&var);
        return value
    }

    fn r_reference(&mut self, mutable: &bool, val: &Value) -> Value {
        return Value::Reference(self.state.create_reference_from_value(val));
    }

    fn r_box(&mut self, val: &Value) -> Value {
        let reference = self.state.create_reference_from_value(val);
        self.state.insert(reference.clone(), val);
        return Value::Reference(reference)
    }
}



