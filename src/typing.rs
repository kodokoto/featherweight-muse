use std::{collections::HashMap, rc::Weak};

use crate::ast::{Path, LVal};

type Lifetime = usize;

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
        ret: Option<Box<Type>>
    }
}

impl Type {
    pub fn copyable(&self) -> bool {
        match self {
            Type::Reference { mutable, .. } => !mutable,
            Type::Box(t) => false,
            _ => true
        }
    }

    fn prohibits_reading(&self, variable: LVal) -> bool {
        match self {
            Type::Reference { mutable, var } => {
                self.prohibits_writing(variable) && *mutable
            },
            Type::Box(t) => t.prohibits_reading(variable),
            _ => false
        }
    }

    fn prohibits_writing(&self, variable: LVal) -> bool {
        match self {
            Type::Reference { mutable, var } => {
                var.get_name() == variable.get_name() && !mutable
            },
            Type::Box(t) => t.prohibits_writing(variable),
            _ => false
        }
    }

    pub fn within(&self, gamma: &TypeEnviroment, lifetime: Lifetime) -> bool {
        match self {
            Type::Reference { var, mutable } => {
                let (_, l) = gamma.get(&var.get_name()).unwrap();
                return l <= lifetime;
            },
            Type::Box(t) => t.within(gamma, lifetime),
            _ => true
        }
    }
}

// pub trait Type where Self: Sized {
//     fn copyable(&self) -> bool;
//     fn prohibits_reading(&self, variable: LVal) -> bool;
//     fn prohibits_writing(&self, variable: LVal) -> bool;
//     fn within(&self, gamma: &TypeEnviroment, lifetime: Lifetime) -> bool;
// }

// #[derive(Debug, Clone, PartialEq)]
// pub enum Type {
//     Numeric,
//     Epsilon,
//     Box(Box<Type>),
//     Reference { 
//         var: LVal,
//         mutable: bool,
//     },
//     Function { 
//         args: Vec<Type>,
//         ret: Option<Box<Type>>
//     }
// }

// impl Type for Type {
//     fn copyable(&self) -> bool {
//         match self {
//             Type::Reference { mutable, .. } => !mutable,
//             Type::Box(_) => false,
//             _ => true
//         }
//     }

//     fn prohibits_reading(&self, variable: LVal) -> bool {
//         match self {
//             Type::Reference { mutable, var} => if *mutable { self.prohibits_writing(variable) } else { false },
//             Type::Box(t) => t.prohibits_reading(variable),
//             _ => false
//         }
//     }

//     fn prohibits_writing(&self, variable: LVal) -> bool {
//         return match self {
//             Type::Reference{ mutable, var } => {
//                 var.get_name() == variable.get_name() && root(&self, var) == root(&self, variable) && !mutable
//             }
//             Type::Box(t) => t.prohibits_writing(variable),
//             _ => false
//         }
//     }

//     fn within(&self, gamma: &TypeEnviroment, lifetime: Lifetime) -> bool {
//         match self {
//             Type::Reference { var, mutable } => {
//                 let (_, l) = gamma.get(&var.name).unwrap();
//                 return l <= lifetime;
//             },
//             Type::Box(t) => t.within(gamma, lifetime),
//             _ => true
//         }
//     }
// }

// #[derive(Debug, Clone, PartialEq)]
// pub enum Type {
//     Undefined(Type),
//     Defined(Type),
//     Box(Box<Type>),
// }

// impl Type for Type {
//     fn copyable(&self) -> bool {
//         match self {
//             Type::Box(pt) => pt.copyable(),
//             Type::Defined(t) => t.copyable(),
//             _ => false
//         }
//     }

//     fn prohibits_reading(&self, variable: LVal) -> bool {
//         match self {
//             Type::Box(pt) => pt.prohibits_reading(variable),
//             Type::Defined(t) => t.prohibits_reading(variable),
//             _ => false
//         }
//     }

//     fn prohibits_writing(&self, variable: LVal) -> bool {
//         match self {
//             Type::Box(pt) => pt.prohibits_writing(variable),
//             Type::Defined(t) => t.prohibits_writing(variable),
//             _ => false
//         }
//     }

//     fn within(&self, gamma: &TypeEnviroment, lifetime: Lifetime) -> bool {
//         match self {
//             Type::Box(pt) => pt.within(gamma, lifetime),
//             Type::Defined(t) => t.within(gamma, lifetime),
//             _ => panic!(" Should not be checking for lifetime within undefined type")
//         }
//     }
// }

#[derive(Debug, Clone)]
pub struct TypeEnviroment {
    gamma: HashMap<String, (Type, Lifetime)>,
}

impl TypeEnviroment {
    pub fn new() -> TypeEnviroment {
        return TypeEnviroment {
            gamma: HashMap::new()
        }
    }

    pub fn get_partial(&self, key: &String) -> Result<(Type, Lifetime), String> {
        return match self.gamma.get(key) {
            Some(t) => Ok(t.clone()),
            None => panic!("Type not found")
        }
    }

    pub fn get_atomic(&self, partial: &Type) -> Result<Type, String> {
        return match partial {
            Type::Undefined(t) => Err(format!("Type of {:?} is undefined, chances are it was moved", t)),
            _ => Ok(partial.clone())
        }
    }

    pub fn get(&self, key: &String) -> Result<(Type, Lifetime), String> {
        let (pt, l) = self.get_partial(key)?;
        return Ok((self.get_atomic(&pt)?, l));
    }

    pub fn insert(&mut self, key: String, value: Type, lifetime: Lifetime) {
        self.gamma.insert(key, (value, lifetime));
    }
}

pub fn root(gamma: &TypeEnviroment, lval: LVal) -> Result<LVal, String> {
    match lval.clone() {
        LVal::Variable { name, ..} => {
            // check if the variable is in the type environment
            if gamma.gamma.contains_key(&name) {
                return Ok(lval);
            } else {
                return Err(format!("Variable {:?} not found in type environment", name));
            }
        },
        LVal::Deref { var } => {
            // check if the variable is in the type environment
            if gamma.gamma.contains_key(&var.get_name()) {
                return Ok(*var);
            } else {
                return Err(format!("Variable {:?} not found in type environment", var.get_name()));
            }
        }
        
    }
}

pub fn write_prohibited(gamma: &TypeEnviroment, variable: LVal) -> bool {
    println!("Checking if {} is borrowed", variable.get_name());
    // for each type in the type environment
    let v2 = root(gamma, variable.clone()).unwrap();
    for (var, (t, l)) in gamma.gamma.iter() {
        if t.prohibits_writing(v2.clone()) {
            return true;
        }
    }
    return false;
}

pub fn read_prohibited(gamma: &TypeEnviroment, variable: LVal) -> bool {
    // for each type in the type environment
    let v2 = root(gamma, variable.clone()).unwrap();
    for (var, (t, l)) in gamma.gamma.iter() {
        if t.prohibits_reading(v2.clone()) {
            return true;
        }
    }
    return false;
}

pub fn move_var(mut gamma: TypeEnviroment, variable: LVal, lifetime: Lifetime) -> Result<TypeEnviroment, String> {
    // for each type in the type environment
    let t = gamma.get(&variable.get_name())?.0;
    gamma.insert(variable.get_name(), undefine(variable, t), lifetime);
    Ok(gamma)
}

pub fn undefine(lval: LVal, t: Type) -> Type {
    println!("Undefining {:?} with type {:?}", lval.clone(), t.clone());
    match (lval.clone(), t.clone()) {
        (LVal::Variable { .. }, _ ) => {
            return Type::Undefined(Box::new(t));
        },
        (LVal::Deref { var }, Type::Box(t)) => {
            return Type::Box(Box::new(undefine(*var, *t)));
        },
        _ => {
            panic!("Tried to undefine something that you should not have lval: {:?}, type: {:?}", lval.clone(), t.clone());
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
        (Type::Reference { mutable: m1, .. }, Type::Reference { mutable: m2, .. }) => {
            m1 == m2
        },
        (Type::Undefined(nt1), t2) => shape_compatible(gamma, nt1, t2),
        (t1, Type::Undefined(nt2)) => shape_compatible(gamma, t1, nt2),
        _ => false
    }
}

pub fn _mut(gamma: &TypeEnviroment, variable: LVal) -> bool {
    let t = gamma.get(&variable.get_name()).unwrap().0;
    // if its
    match (variable, t) {
        (LVal::Deref { var } , Type::Box(t)) => _mut(gamma, *var),
        (LVal::Deref { var }, Type::Reference { mutable, var: rvar }) => {
            if mutable {
                // TODO: check if this should be var or rvar, my brain is not working
                return _mut(gamma, rvar);
            } else {
                return false;
            }
        },
        _ => true
    }
}

// let x = 0
// let b = ref x

pub fn update(gamma: TypeEnviroment, lv: &LVal, t1: Type, t2: Type) -> Result<(TypeEnviroment, Type), String> {

    println!("Updating type {:?} with {:?} ", t1, t2);

    match lv {
        LVal::Variable { name, .. } => {
            return Ok((gamma, t2));
        },
        LVal::Deref { var } => {
            match t1.clone() {
                Type::Box(t) => {
                    let (gamma2, t3) = update(gamma, var, *t, t2)?;
                    return Ok((gamma2, Type::Box(Box::new(t3))));
                },
                Type::Reference { var: rvar, mutable } => {
                    if !mutable {
                        return Err(format!("Error updating reference: variable {:?} is not mutable", rvar));
                    }
                    let g3 = write(gamma, rvar, t2);
                    return Ok((g3?, t1));
                },
                _ => {
                    panic!("This should not happen");
                }
            }
        }
    }

    // match t1.clone() {
    //     Type::Box(t) => {
    //         let (gamma2, t3) = update(gamma, *t, t2)?;
    //         return Ok((gamma2, Type::Box(Box::new(t3))));
    //     },
    //     Type::Reference { var, mutable } => {
    //         if !mutable {
    //             return Err(format!("Error updating reference: variable {:?} is not mutable", var));
    //         }            
    //         let g3 = write(gamma, var, t2);
    //         return Ok((g3?, t1));
    //     },
    //     _ => {
    //         panic!("This should not happen");
    //     }
    // }
    // if i == p.selectors.len() {
    //     return Ok((gamma, Type::Defined(t2)));
    // } else {
    //     match t1.clone() {
    //         Type::Box(t) => {
    //             let (gamma2, t3) = update(gamma, *t, p, i + 1, t2, strong)?;
    //             return Ok((gamma2, Type::Box(Box::new(t3))));
    //         },
    //         Type::Defined(Type::Reference { var, mutable }) => {
    //             if !mutable {
    //                 return Err(format!("Error updating reference: variable {:?} is not mutable", var));
    //             }

    //             let v = var.traverse(p, i+1);
    //             let g3 = write(gamma, v, t2);
    //             return Ok((g3?, t1));
    //         },
    //         _ => {
    //             panic!("This should not happen");
    //         }
    //     }
    // }
}

pub fn write(gamma: TypeEnviroment, variable: LVal, t1: Type) -> Result<TypeEnviroment, String> {

    println!("Writing type {:?} to {:?}", t1, variable.get_name());

    let (t2, l) = gamma.get(&variable.get_name())?;

    println!("Type of {:?} is currently {:?}", variable.get_name(), t2);

    let (mut gamma2, t3) = update(gamma, &variable, t2, t1)?;

    println!("Type of {:?} is now {:?}", variable.get_name(), t3);

    gamma2.insert(variable.get_name(), t3, l);
    return Ok(gamma2);
}

// fn t() {
//     let mut x = 0;
//     let mut y = &mut x;
//     let mut z = &mut x;

//     *y = 1;
// }

// fn t2() {
//     let mut x = 0;
//     let mut y = &mut x;
//     let mut z = &x;

//     *y = 1;
// }