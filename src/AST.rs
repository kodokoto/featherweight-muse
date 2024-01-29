struct Variable {
    name: String,
}

enum Term {
    Variable(Variable),
    Value(Value),
    Move {
        var: Variable,
    },
    Copy {
        var: Variable,
    },
    Box {
        t: Term,
    },
    Ref {
        mutable: bool, 
        t: Term,
    },
    Let {
        mutable: bool,
        var: Variable,
        t: Term,
    },
    Assign {
        var: Variable,
        t: Term,
    },
}


enum Value
{
    NumericLiteral(i64),
    Reference(String)
}

struct Program {
    terms: Vec<Term>,
}

