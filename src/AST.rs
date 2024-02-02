use std::fmt::Display;
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

#[derive(Debug,)]
pub struct Variable {
    pub name: String,
}

#[derive(Debug, Clone)]
pub enum Value
{
    NumericLiteral(i64),
    Reference(Reference),
    Epsilon,
}

 impl Display for Value {
     fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
         match self {
             Value::NumericLiteral(n) => write!(f, "{}", n),
             Value::Reference(r) => write!(f, "{}", r.loc),
             Value::Epsilon => write!(f, "Epsilon"),
         }
     }
 }

#[derive(Debug)]
pub struct Program {
    pub terms: Vec<Term>,
}

#[derive(Debug, PartialEq, Eq, Hash, Clone)]
pub struct Reference {
    pub loc: String,
}

#[derive(Debug)]
pub enum AST {
    Program(Program),
}

