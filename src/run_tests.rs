
#[cfg(test)]
mod tests {
    use crate::{ast::{LVal, Value}, constants::TypeError, interpreter::Interpreter, lexer::Lexer, parser::Parser, typing::Type};

    fn run(file: &str) -> Result<Value, String> {
        let input = std::fs::read_to_string(file).expect("Error reading file");
        let mut lexer = Lexer::new(input.as_str());
        let tokens = lexer.tokenize();
        let mut parser = Parser::new(tokens);
        let ast = parser.parse();
        let mut interpreter = Interpreter::new();
        let result = interpreter.run(ast);
        result
    }

    macro_rules! run_tests {
        ($($name:ident: $value:expr,)*) => {
        $(
            #[test]
            fn $name() {
                let (input, expected) = $value;
                assert_eq!(expected, run(input));
            }
        )*
        }
    }

    run_tests! {
        double_box_deref: ("tests/good/double_box_deref.mu", Ok(Value::Epsilon)),
        fn_borrow: ("tests/good/fn_borrow.mu", Ok(Value::Epsilon)),
        fn_inplace: ("tests/good/fn_inplace.mu", Ok(Value::Epsilon)),
        fn_lifetime_transfer: ("tests/good/fn_lifetime_transfer.mu", Ok(Value::Epsilon)),
        immut_after_mut: ("tests/good/immut_after_mut.mu", Ok(Value::Epsilon)),
        multiple_move: ("tests/good/multiple_move.mu", Ok(Value::Epsilon)),
        reassign_after_move: ("tests/good/reassign_after_move.mu", Ok(Value::Epsilon)),
        reassign_deref: ("tests/good/reassign_deref.mu", Ok(Value::Epsilon)),
        reassign_in_diff_scope: ("tests/good/reassign_in_diff_scope.mu", Ok(Value::Epsilon)),
        reassign_ref: ("tests/good/reassign_ref.mu", Ok(Value::Epsilon)),
        swap: ("tests/good/swap.mu", Ok(Value::Epsilon)),

        // bad

        double_mut_ref: ("tests/bad/double_mut_ref.mu", Err(TypeError::MUTREF_ALREADY_BORROWED_IMMUT("x".to_string()).to_string())),
        assign_borrowed: ("tests/bad/assign_borrowed.mu", Err(TypeError::ASSIGN_BORROWED("x".to_string()).to_string())),
        assign_mut_borrowed: ("tests/bad/assign_mut_borrowed.mu", Err(TypeError::ASSIGN_BORROWED("x".to_string()).to_string())),
        bad_typing: ("tests/bad/bad_typing.mu", Err(TypeError::INCOMPATABLE_TYPES(Type::Box(Box::new(Type::Numeric)), Type::Reference { var: LVal::Variable { name: "z".to_string(), copyable: None }, mutable: false }).to_string())),
        dec_after_partial_move: ("tests/bad/dec_after_partial_move.mu", Err(TypeError::TYPE_MOVED(Type::Box(Box::new(Type::Numeric))).to_string())),
        mut_after_immut: ("tests/bad/mut_after_immut.mu", Err(TypeError::MUTREF_ALREADY_BORROWED_IMMUT("x".to_string()).to_string())),
        mut_from_immut: ("tests/bad/mut_from_immut.mu", Err(TypeError::MUTREF_IMMUT("y".to_string()).to_string())),
        function_incorrect_arg_type: ("tests/bad/function_incorrect_arg_type.mu", Err(TypeError::FUNCTION_CALL_INCOMPATABLE_ARGUMENT_TYPE(Type::Numeric, Type::Box(Box::new(Type::Numeric))).to_string())),
        function_incorrect_arg_count: ("tests/bad/function_incorrect_arg_count.mu", Err(TypeError::FUNCTION_CALL_INCOMPATABLE_ARGUMENT_COUNT(2, 1).to_string())),
        function_incorrect_return_type: ("tests/bad/function_incorrect_return_type.mu", Err(TypeError::FUNCTION_UNEXPECTED_RETURN(Type::Numeric, Type::Box(Box::new(Type::Numeric))).to_string())),
        worked_example: ("tests/bad/worked_example.mu", Err(TypeError::TYPE_MOVED(Type::Box(Box::new(Type::Numeric))).to_string())),
    }
        
    
}