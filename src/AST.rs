#[derive(Debug)]
pub struct Variable {
    pub name: String,
}

#[derive(Debug)]
pub enum Term {
    Variable(Variable),
    Value(Value),
    Move {
        var: Variable,
    },
    Copy {
        var: Variable,
    },
    Box {
        t: Box<Term>,
    },
    Ref {
        mutable: bool, 
        t: Box<Term>,
    },
    Let {
        mutable: bool,
        var: Variable,
        t: Box<Term>,
    },
    Assign {
        var: Variable,
        t: Box<Term>,
    },
}

#[derive(Debug)]
pub enum Value
{
    NumericLiteral(i64),
    Reference(String)
}

#[derive(Debug)]
pub struct Program {
    pub terms: Vec<Term>,
}

#[derive(Debug)]
pub enum AST {
    Program(Program),
}

