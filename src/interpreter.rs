use std::{collections::HashMap, env};

use crate::{
    ast::{Program, Value},
    properties::{assert_preservation, assert_progess},
    reduction::Evaluate,
    state::{StackFrame, State, Store},
    typecheck::TypeCheck,
    typing::TypeEnviroment,
};
pub struct Interpreter {
    pub program_state: State,
    typing_enviroment: TypeEnviroment,
}

impl Interpreter {
    pub fn new() -> Interpreter {
        return Interpreter {
            program_state: State::new(
                vec![StackFrame {
                    locations: HashMap::new(),
                    functions: HashMap::new(),
                }],
                Store::new(),
            ),
            typing_enviroment: TypeEnviroment::new(),
        };
    }

    pub fn run(&mut self, mut ast: Program) -> Result<Value, String> {
        // initial type check
        ast.type_check(self.typing_enviroment.clone(), 0)?;

        for mut term in ast.terms {
            assert_progess(
                self.program_state.clone(),
                term.clone(),
                self.typing_enviroment.clone(),
                0,
            )?;

            assert_preservation(
                self.program_state.clone(),
                term.clone(),
                self.typing_enviroment.clone(),
                0,
            )?;

            let (s, _) = term.evaluate(self.program_state.clone(), 0)?;

            let (gamma2, _) = term.type_check(self.typing_enviroment.clone(), 0)?;

            self.typing_enviroment = gamma2;
            self.program_state = s;
        }

        println!("{:#?}", self.typing_enviroment);

        if env::var("EVAL_OUT").is_ok() {
            self.program_state.print();
        }

        return Ok(Value::Epsilon);
    }
}
