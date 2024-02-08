use crate::{ast::{Term, Value, AST}, state::State};

pub struct Interpreter {
    state: State,
}

impl Interpreter {
    pub fn new() -> Interpreter {
        return Interpreter {
            state: State::new(),
        }
    }

    pub fn run(&mut self, ast: impl AST) -> Value {
        let (s, t) = ast.execute(self.state.clone());
        self.state = s;
        println!("State: {:?}", self.state);
        let v = match t {
            Term::Value(v) => v,
            _ => Value::Epsilon
        };
        return v.clone();
    }
}