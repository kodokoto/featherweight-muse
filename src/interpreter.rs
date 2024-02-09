use std::env;

use crate::{ast::{Program, Value}, reduction::Evaluate, state::State};

pub struct Interpreter {
    state: State,
}

impl Interpreter {
    pub fn new() -> Interpreter {
        return Interpreter {
            state: State::new(),
        }
    }

    pub fn run(&mut self, mut ast: Program) -> Result<Value, String> {
        while ast.terms.len() > 0 {
            let (s, _) = match ast.evaluate(self.state.clone()) {
                Ok((s, t)) => (s, t),
                Err(e) => return Err(e)
            };
            self.state = s;
            if env::var("EVAL_OUT").is_ok() {
                println!("{:#?}", self.state);
            }
        }
        return Ok(Value::Epsilon)
    }
}