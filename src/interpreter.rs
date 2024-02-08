use std::env;

use crate::{ast::{Program, Term, Value, AST}, state::State};

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
            let (s, t) = match ast.execute(self.state.clone()) {
                Ok((s, t)) => (s, t),
                Err(e) => return Err(e)
            };
            self.state = s;
            if env::var("MUSE_DEBUG").is_ok() {
                println!("{:#?}", self.state);
            }
        }
        return Ok(Value::Epsilon)
    }
}