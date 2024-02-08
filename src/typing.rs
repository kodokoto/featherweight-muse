
type Variable = String;

struct Type {
    kind: TypeKind,
    copyable: bool,
}

enum TypeKind {
    Int,
    UType(String),
}

struct Typing {
    // ...
    environment: HashMap<Variable, Type>
}

impl Typing {
    fn new() -> Typing {
        Typing {
            environment: HashMap::new()
        }
    }

    fn type_check(&mut self, term: &Term) -> Result<Type> {
        
    }
}

