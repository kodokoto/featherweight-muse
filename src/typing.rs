use std::collections::HashMap;

use crate::ast::Variable;

#[derive(Debug, Clone, PartialEq)]
pub enum Type {
    Numeric,
    Epsilon,
    Box(Box<Type>),
    Reference { 
        kind: Box<Type>,
        mutable: bool,
    },
    Undefined(Box<Type>), // internal use only
}

impl Type {
    pub fn is_copy_type(&self) -> bool {
        match self {
            Type::Reference { mutable, .. } => {
                return !mutable;
            },
            Type::Box(_) => {
                return false;
            },
            _ => return true
        }
    }

}


pub struct TypeEnviroment {
    gamma: HashMap<String, Type>,
}

impl TypeEnviroment {
    pub fn new() -> TypeEnviroment {
        return TypeEnviroment {
            gamma: HashMap::new()
        }
    }

    pub fn get_partial(&self, key: &String) -> Result<Type, String> {
        return match self.gamma.get(key) {
            Some(t) => Ok(t.clone()),
            None => Err(format!("Type not found for variable: {}", key))
        }
    }

    pub fn get(&self, key: &String) -> Result<Type, String> {
        return match self.get_partial(key)? {
            Type::Undefined(_) => Err(format!("Error: Undefined type for variable: {}", key)),
            t => Ok(t)
        }
    }

    pub fn insert(&mut self, key: String, value: Type) {
        self.gamma.insert(key, value);
    }
}

// true if the type of the variable is a reference that is mutable
pub fn read_prohibited(gamma: &TypeEnviroment, variable: &String) -> Result<bool, String> {
    match gamma.get(variable)? {
        Type::Reference { mutable, .. } => {
            return Ok(mutable);
        },
        _ => return Ok(false)        
    }
}

pub fn write_prohibited(gamma: &TypeEnviroment, variable: &String) -> Result<bool, String> {
    match gamma.get(variable)? {
        Type::Reference { mutable, .. } => {
            return Ok(!mutable);
        },
        _ => return Ok(false)
    }
}

pub fn move_type(mut gamma: TypeEnviroment, variable: &Variable) -> Result<TypeEnviroment, String> {
    let t = gamma.get(&variable)?;
    gamma.insert(variable.clone(), Type::Undefined(Box::new(t.clone())));
    return Ok(gamma);
}

pub fn dom(gamma: &TypeEnviroment) -> Vec<String> {
    return gamma.gamma.keys().map(|s| s.clone()).collect();
}

pub fn shape_compatible(t1: &Type, t2: &Type) -> bool {
    match (t1, t2) {
        (Type::Numeric, Type::Numeric) => true,
        (Type::Epsilon, Type::Epsilon) => true,
        (Type::Box(t1), Type::Box(t2)) => shape_compatible(t1, t2),
        (Type::Reference { kind: t1, mutable: m1 }, Type::Reference { kind: t2, mutable: m2 }) => {
            shape_compatible(t1, t2) && m1 == m2
        },
        (Type::Undefined(t1), t2) => shape_compatible(t1, t2),
        (t1, Type::Undefined(t2)) => shape_compatible(t1, t2),
        _ => false
    }
}