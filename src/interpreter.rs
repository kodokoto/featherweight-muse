use std::{cell::Ref, collections::{HashMap, HashSet}, env};

use crate::{ast::{LVal, Program, Term, Value, AST}, properties::{assert_preservation, assert_progess}, reduction::Evaluate, state::{StackFrame, State, Store}, typecheck::TypeCheck, typing::{contains, Slot, Type, TypeEnviroment}};
pub struct Interpreter {
    program_state: State,
    typing_enviroment: TypeEnviroment
}

impl Interpreter {
    pub fn new() -> Interpreter {
        return Interpreter {
            program_state: State::new(
                vec![
                    StackFrame {
                        locations: HashMap::new(),
                        functions: HashMap::new()
                    }
                ],
                Store::new()
            ),
            typing_enviroment: TypeEnviroment::new()
        }
    }
 
    pub fn run(&mut self, mut ast: Program) -> Result<Value, String> {

        // Type check the program

        println!("Performing program type check");

        let (gamma, t) = ast.type_check(self.typing_enviroment.clone(), 0)?;

        println!("Program type check successful");

        for mut term in ast.terms {
            
            println!("Evaluating term\n");

            println!("{:#?}", term);

            println!("Current state:\n");
            println!("{:#?}", self.program_state);

            println!("Current type enviroment:\n");
            println!("{:#?}", self.typing_enviroment);

            match assert_progess(self.program_state.clone(), term.clone(), self.typing_enviroment.clone(), 0) {
                Ok(_) => {
                    println!("");
                    println!("Progress");
                    println!("");
                },
                Err(e) => {
                    let progress_error = format!("Progress error: {}", e);
                    return Err(progress_error)
                }
            }

            match assert_preservation(self.program_state.clone(), term.clone(), self.typing_enviroment.clone(), 0) {
                Ok(_) => {
                    println!("");
                    println!("Preservation");
                    println!("");
                },
                Err(e) => {
                    let progress_error = format!("Progress error: {}", e);
                    return Err(progress_error)
                }
            }

            let (s, _) = match term.evaluate(self.program_state.clone(), 0) {
                Ok((s, t)) => (s, t),
                Err(e) => return Err(e)
            };

            let (gamma2, _) = term.type_check(self.typing_enviroment.clone(), 0)?;

            self.typing_enviroment = gamma2;
            self.program_state = s;
            // if env::var("EVAL_OUT").is_ok() {
            //     println!("{:#?}", self.program_state);
            // }
            
        }
        // while ast.terms.len() > 0 {

        //     assert_progess(self.program_state, ast, self.typing_enviroment, 0);

        //     let (s, _) = match ast.evaluate(self.program_state.clone(), 0) {
        //         Ok((s, t)) => (s, t),
        //         Err(e) => return Err(e)
        //     };
        //     self.program_state = s;
        //     let (gamma2, t2) = ast.type_check(self.typing_enviroment.clone(), 0)?;
        //     self.typing_enviroment = gamma2;
        //     if !safe_abstraction(self.program_state.clone(), self.typing_enviroment.clone())? {
        //         return Err("Type error".to_string())
        //     }
        //     if env::var("EVAL_OUT").is_ok() {
        //         println!("{:#?}", self.program_state);
        //     }
        // }

        println!();

        println!("Program evaluation successful");

        println!("Final type enviroment:\n");

        println!("{:#?}", self.typing_enviroment);

        println!("Final state:\n");
        
        self.program_state.print();
        

        return Ok(Value::Epsilon)
    }
}

