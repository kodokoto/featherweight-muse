#[cfg(test)]
mod tests {
    use std::collections::HashMap;

    use crate::{
        ast::LVal, constants::TypeError, interpreter::Interpreter, lexer::Lexer, parser::Parser,
        typing::Type,
    };

    fn run(file: &str) -> Result<HashMap<String, String>, String> {
        let input = std::fs::read_to_string(file).expect("Error reading file");
        let mut lexer = Lexer::new(input.as_str());
        let tokens = lexer.tokenize();
        let mut parser = Parser::new(tokens);
        let ast = parser.parse();
        let mut interpreter = Interpreter::new();
        let result = interpreter.run(ast);
        match result {
            Ok(_) => Ok(interpreter.program_state.output()),
            Err(e) => return Err(e),
        }
    }

    macro_rules! run_tests {
        ($($name:ident: $value:expr,)*) => {
        $(
            #[test]
            fn $name() {
                let (input, expected) : (&str, Result<Vec<(&str, &str)>, String>)= $value;

                match expected {
                    Ok(out) => {
                        let mut o: HashMap<String, String> = HashMap::new();
                        for (k, v) in out {
                            o.insert(k.to_string(), v.to_string());
                        }
                        assert_eq!(Ok(o), run(input));
                    },
                    Err(e) => {
                        assert_eq!(e, run(input).unwrap_err());
                    }
                }
            }
        )*
        }
    }

    run_tests! {
        double_box_deref: ("tests/good/double_box_deref.mu", Ok(vec![
            ("x", "ref Undefined"),
            ("y", "ref 1")
        ])),

        fn_borrow: ("tests/good/fn_borrow.mu", Ok(vec![
            ("x", "ref 0"),
            ("y", "ref 0"),
        ])),
        fn_inplace: ("tests/good/fn_inplace.mu", Ok(vec![
            ("x", " 5"),
        ])),
        fn_lifetime_transfer: ("tests/good/fn_lifetime_transfer.mu", Ok(vec![
            ("x", " 5"),
        ])),
        immut_after_mut: ("tests/good/immut_after_mut.mu", Ok(vec![
            ("x", " 0"),
            ("y", "ref 0"),
            ("z", "ref 0"),
        ])),
        multiple_move: ("tests/good/multiple_move.mu", Ok(vec![
            ("res", "ref 5")
        ])),
        reassign_after_move: ("tests/good/reassign_after_move.mu", Ok(vec![
            ("x", " 1"),
            ("y", " 0"),
        ])),
        reassign_deref: ("tests/good/reassign_deref.mu", Ok(vec![
            ("x", " 4"),
            ("y", "ref 4"),
        ])),
        reassign_in_diff_scope: ("tests/good/reassign_in_diff_scope.mu", Ok(vec![
            ("x", " 0"),
            ("y", " 1"),
            ("w", " 1")
        ])),
        reassign_ref: ("tests/good/reassign_ref.mu", Ok(vec![
            ("x", " 0"),
            ("y", " 1"),
            ("z", "ref 1")
        ])),
        swap: ("tests/good/swap.mu", Ok(vec![
            ("x", " 1"),
            ("y", " 0"),
        ])),

        // bad

        double_mut_ref: ("tests/bad/double_mut_ref.mu", Err(TypeError::MutrefAlreadyBorrowedImmut("x".to_string()).to_string())),
        assign_borrowed: ("tests/bad/assign_borrowed.mu", Err(TypeError::AssignBorrowed("x".to_string()).to_string())),
        assign_mut_borrowed: ("tests/bad/assign_mut_borrowed.mu", Err(TypeError::AssignBorrowed("x".to_string()).to_string())),
        bad_typing: ("tests/bad/bad_typing.mu", Err(TypeError::IncompatibleTypes(Type::Box(Box::new(Type::Numeric)), Type::Reference { var: LVal::Variable { name: "z".to_string(), copyable: None }, mutable: false }).to_string())),
        dec_after_partial_move: ("tests/bad/dec_after_partial_move.mu", Err(TypeError::TypeMoved(Type::Box(Box::new(Type::Numeric))).to_string())),
        mut_after_immut: ("tests/bad/mut_after_immut.mu", Err(TypeError::MutrefAlreadyBorrowedImmut("x".to_string()).to_string())),
        mut_from_immut: ("tests/bad/mut_from_immut.mu", Err(TypeError::MutrefImmut("y".to_string()).to_string())),
        function_incorrect_arg_type: ("tests/bad/function_incorrect_arg_type.mu", Err(TypeError::FunctionCallIncompatableArgumentType(Type::Numeric, Type::Box(Box::new(Type::Numeric))).to_string())),
        function_incorrect_arg_count: ("tests/bad/function_incorrect_arg_count.mu", Err(TypeError::FunctionCallIncompatableArgumentCount(2, 1).to_string())),
        function_incorrect_return_type: ("tests/bad/function_incorrect_return_type.mu", Err(TypeError::FunctionUnexpectedReturn(Type::Numeric, Type::Box(Box::new(Type::Numeric))).to_string())),
        worked_example: ("tests/bad/worked_example.mu", Err(TypeError::TypeMoved(Type::Box(Box::new(Type::Numeric))).to_string())),
    }
}
