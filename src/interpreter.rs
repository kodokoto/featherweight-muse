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
        println!("Performing program type check");

        ast.type_check(self.typing_enviroment.clone(), 0)?;

        println!("Program type check successful");

        for mut term in ast.terms {
            println!("Evaluating term\n");

            println!("{:#?}", term);

            println!("Current state:\n");
            println!("{:#?}", self.program_state);

            println!("Current type enviroment:\n");
            println!("{:#?}", self.typing_enviroment);

            match assert_progess(
                self.program_state.clone(),
                term.clone(),
                self.typing_enviroment.clone(),
                0,
            ) {
                Ok(_) => {
                    println!("");
                    println!("Progress");
                    println!("");
                }
                Err(e) => {
                    let progress_error = format!("Progress error: {}", e);
                    return Err(progress_error);
                }
            }

            match assert_preservation(
                self.program_state.clone(),
                term.clone(),
                self.typing_enviroment.clone(),
                0,
            ) {
                Ok(_) => {
                    println!("");
                    println!("Preservation");
                    println!("");
                }
                Err(e) => {
                    let progress_error = format!("Progress error: {}", e);
                    return Err(progress_error);
                }
            }

            let (s, _) = match term.evaluate(self.program_state.clone(), 0) {
                Ok((s, t)) => (s, t),
                Err(e) => return Err(e),
            };

            let (gamma2, _) = term.type_check(self.typing_enviroment.clone(), 0)?;

            self.typing_enviroment = gamma2;
            self.program_state = s;
        }
        println!();

        if env::var("EVAL_OUT").is_ok() {
            println!("{:#?}", self.program_state);
        }

        println!("Program evaluation successful");

        println!("Final type enviroment:\n");

        println!("{:#?}", self.typing_enviroment);

        println!("Final state:\n");

        self.program_state.print();

        return Ok(Value::Epsilon);
    }
}
