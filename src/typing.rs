use std::collections::HashMap;

use crate::{ast::{LVal, Reference}, constants::TypeError};

#[derive(Debug, Clone)]
pub struct Slot<T> {
    pub value: T,
    pub lifetime: usize,
}

type Lifetime = usize;
type Variable = String;

#[derive(Debug, Clone, PartialEq)]
pub enum Type {
    Epsilon,
    Numeric,
    Reference {
        var: LVal,
        mutable: bool,
    },
    Box(Box<Type>),
    Undefined(Box<Type>),
    Function {
        args: Vec<Type>,
        ret: Option<Box<Type>>,
    },
}

impl Type {
    pub fn copyable(&self) -> bool {
        match self {
            Type::Reference { mutable, .. } => !mutable,
            Type::Box(_) => false,
            _ => true,
        }
    }

    pub fn within(&self, gamma: &TypeEnviroment, lifetime: Lifetime) -> bool {
        match self {
            Type::Reference { var, .. } => {
                let Slot { lifetime: l, .. } = gamma.get(&var.get_name()).unwrap();
                return l <= lifetime;
            }
            Type::Box(t) => t.within(gamma, lifetime),
            _ => true,
        }
    }

    pub fn to_string(&self) -> String {
        match self {
            Type::Epsilon => "Epsilon".to_string(),
            Type::Numeric => "Numeric".to_string(),
            Type::Reference { var, mutable } => {
                format!(
                    "{}Ref {}",
                    if *mutable { " Mut" } else { "" },
                    var.get_name()
                )
            }
            Type::Box(t) => {
                format!("Box {}", t.to_string())
            }
            Type::Undefined(t) => {
                format!("Undefined {}", t.to_string())
            }
            Type::Function { args, ret } => {
                let mut s = "Function (".to_string();
                for arg in args {
                    s.push_str(&arg.to_string());
                    s.push_str(", ");
                }
                s.push_str(") -> ");
                match ret {
                    Some(t) => {
                        s.push_str(&t.to_string());
                    }
                    None => {
                        s.push_str("None");
                    }
                }
                return s;
            }
        }
    }
}

#[derive(Debug, Clone)]
pub struct TypeEnviroment {
    gamma: HashMap<Variable, Slot<Type>>,
}

impl TypeEnviroment {
    pub fn new() -> TypeEnviroment {
        return TypeEnviroment {
            gamma: HashMap::new(),
        };
    }

    pub fn get_partial(&self, key: &Variable) -> Result<Slot<Type>, String> {
        return match self.gamma.get(key) {
            Some(t) => Ok(t.clone()),
            None => Err("Type not found".to_string()),
        };
    }

    pub fn get_atomic(&self, partial: Slot<Type>) -> Result<Slot<Type>, String> {
        return match partial.value.clone() {
            Type::Undefined(t) => {
                println!("Getting atomic type of {:?}", partial.value.clone());
                println!("Gamma:");
                println!("{:#?}", self.gamma);
                // panic!("Type of {:?} is undefined, chances are it was moved", t);
                Err(TypeError::TypeMoved(*t.clone()).to_string())
            }
            Type::Box(t) => {
                match *t {
                    Type::Undefined(nt) => {
                        println!("Getting atomic type of {:?}", partial.value.clone());
                        println!("Gamma:");
                        println!("{:#?}", self.gamma);
                        // panic!("Type of {:?} is undefined, chances are it was moved", nt);
                        Err(TypeError::TypeMoved(*nt.clone()).to_string())
                    }
                    _ => Ok(partial.clone()),
                }
            }
            _ => Ok(partial.clone()),
        };
    }

    pub fn get(&self, key: &Variable) -> Result<Slot<Type>, String> {
        let s = self.get_partial(key)?;
        return Ok(self.get_atomic(s)?);
    }

    pub fn insert(&mut self, key: Variable, value: Type, lifetime: Lifetime) {
        self.gamma.insert(key, Slot { value, lifetime });
    }

    pub fn dom(&self) -> Vec<String> {
        return self
            .gamma
            .keys()
            .filter(|s| match self.get_partial(s) {
                Ok(t) => match t.value {
                    Type::Function { .. } => false,
                    _ => true,
                },
                Err(_) => false,
            })
            .map(|s| s.clone())
            .collect();
    }
}

pub fn root(gamma: &TypeEnviroment, lval: LVal) -> Result<LVal, String> {
    match lval.clone() {
        LVal::Variable { name, .. } => {
            // check if the variable is in the type environment
            if gamma.gamma.contains_key(&name) {
                return Ok(lval);
            } else {
                return Err(format!("Variable {:?} not found in type environment", name));
            }
        }
        LVal::Deref { var } => {
            // check if the variable is in the type environment
            if gamma.gamma.contains_key(&var.get_name()) {
                return Ok(root(gamma,*var)?);
            } else {
                return Err(format!(
                    "Variable {:?} not found in type environment",
                    var.get_name()
                ));
            }
        }
    }
}

pub fn write_prohibited(gamma: &TypeEnviroment, variable: LVal) -> bool {
    println!("Checking if {} is borrowed", variable.get_name());
    // for each type in the type environment
    let v2 = root(gamma, variable.clone()).unwrap();

    for (_, Slot { value: t, .. }) in gamma.gamma.iter() {
        if contains(t.clone(), Type::Reference { var: v2.clone(), mutable: false }) || contains(t.clone(), Type::Reference { var: v2.clone(), mutable: true }) {
            return true;
        }
    }
    return false;
}


pub fn contains(t: Type, t2: Type) -> bool {
    match t {
        Type::Box(t1) => {
            contains(*t1, t2)
        },
        _ => t == t2
    }
}

pub fn read_prohibited(gamma: &TypeEnviroment, variable: LVal) -> bool {
    // for each type in the type environment
    let v2 = root(gamma, variable.clone()).unwrap();
    for (_, Slot { value: t, .. }) in gamma.gamma.iter() {
        if contains(t.clone(), Type::Reference { var: v2.clone(), mutable: true }) {
            return true;
        }
    }
    return false;
}

pub fn move_var(
    mut gamma: TypeEnviroment,
    variable: LVal,
    lifetime: Lifetime,
) -> Result<TypeEnviroment, String> {
    let t = gamma.get_partial(&variable.get_name())?.value;
    gamma.insert(variable.get_name(), undefine(variable, t), lifetime);
    Ok(gamma)
}

pub fn undefine(lval: LVal, t: Type) -> Type {
    match (lval.clone(), t.clone()) {
        (LVal::Variable { .. }, _) => {
            return Type::Undefined(Box::new(t));
        }
        (LVal::Deref { var }, Type::Box(t)) => {
            return Type::Box(Box::new(undefine(*var, *t)));
        }
        _ => {
            panic!(
                "Tried to undefine something that you should not have lval: {:?}, type: {:?}",
                lval.clone(),
                t.clone()
            );
        }
    }
}

pub fn dom(gamma: &TypeEnviroment) -> Vec<String> {
    return gamma.gamma.keys().map(|s| s.clone()).collect();
}

pub fn shape_compatible(gamma: &TypeEnviroment, t1: &Type, t2: &Type) -> bool {
    match (t1, t2) {
        (Type::Numeric, Type::Numeric) => true,
        (Type::Box(bt1), Type::Box(bt2)) => shape_compatible(gamma, bt1, bt2),
        (Type::Reference { mutable: m1, .. }, Type::Reference { mutable: m2, .. }) => m1 == m2,
        (Type::Undefined(nt1), t2) => shape_compatible(gamma, nt1, t2),
        (t1, Type::Undefined(nt2)) => shape_compatible(gamma, t1, nt2),
        _ => false,
    }
}

pub fn _mut(gamma: &TypeEnviroment, variable: LVal) -> bool {
    let t = gamma.get(&variable.get_name()).unwrap().value;
    match (variable, t) {
        (LVal::Deref { var }, Type::Box(_)) => _mut(gamma, *var),
        (LVal::Deref { .. }, Type::Reference { mutable, var: rvar }) => {
            if mutable {
                // TODO: check if this should be var or rvar, my brain is not working
                return _mut(gamma, rvar);
            } else {
                return false;
            }
        }
        (LVal::Variable { name, copyable }, Type::Reference { mutable: false, .. }) => {
            false
        }
        _ => true,
    }
}

pub fn update(
    gamma: TypeEnviroment,
    lv: &LVal,
    t1: Type,
    t2: Type,
) -> Result<(TypeEnviroment, Type), String> {
    match lv {
        LVal::Variable { .. } => {
            return Ok((gamma, t2));
        }
        LVal::Deref { var } => match t1.clone() {
            Type::Box(t) => {
                let (gamma2, t3) = update(gamma, var, *t, t2)?;
                return Ok((gamma2, Type::Box(Box::new(t3))));
            }
            Type::Reference { var: rvar, mutable } => {
                if !mutable {
                    return Err(format!(
                        "Error updating reference: variable {:?} is not mutable",
                        rvar
                    ));
                }
                let g3 = write(gamma, rvar, t2);
                return Ok((g3?, t1));
            }
            _ => {
                panic!("This should not happen");
            }
        },
    }
}

pub fn write(gamma: TypeEnviroment, variable: LVal, t1: Type) -> Result<TypeEnviroment, String> {
    let Slot {
        value: t2,
        lifetime: l,
    } = gamma.get(&variable.get_name())?;
    let (mut gamma2, t3) = update(gamma, &variable, t2, t1)?;

    gamma2.insert(variable.get_name(), t3, l);
    return Ok(gamma2);
}


