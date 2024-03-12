use std::{collections::HashMap, rc::Weak};

use crate::ast::{Path, Variable};

pub trait Type where Self: Sized {
    fn copyable(&self) -> bool;
    fn prohibits_reading(&self, variable: Variable) -> bool;
    fn prohibits_writing(&self, variable: Variable) -> bool;
}

#[derive(Debug, Clone, PartialEq)]
pub enum AtomicType {
    Numeric,
    Epsilon,
    Box(Box<AtomicType>),
    Reference { 
        var: Variable,
        mutable: bool,
    },
    Function { 
        args: Vec<AtomicType>,
        ret: Option<Box<AtomicType>>
    }
}

impl Type for AtomicType {
    fn copyable(&self) -> bool {
        match self {
            AtomicType::Reference { mutable, .. } => !mutable,
            AtomicType::Box(_) => false,
            _ => true
        }
    }

    fn prohibits_reading(&self, variable: Variable) -> bool {
        match self {
            AtomicType::Reference { mutable, var} => if *mutable { self.prohibits_writing(variable) } else { false },
            AtomicType::Box(t) => t.prohibits_reading(variable),
            _ => false
        }
    }

    fn prohibits_writing(&self, variable: Variable) -> bool {
        return match self {
            AtomicType::Reference{ mutable, var } => {
                var.name == variable.name && var.path == variable.path && !mutable
            }
            AtomicType::Box(t) => t.prohibits_writing(variable),
            _ => false
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum PartialType {
    Undefined(AtomicType),
    Defined(AtomicType),
    Box(Box<PartialType>),
}

impl Type for PartialType {
    fn copyable(&self) -> bool {
        match self {
            PartialType::Box(pt) => pt.copyable(),
            PartialType::Defined(t) => t.copyable(),
            _ => false
        }
    }

    fn prohibits_reading(&self, variable: Variable) -> bool {
        match self {
            PartialType::Box(pt) => pt.prohibits_reading(variable),
            PartialType::Defined(t) => t.prohibits_reading(variable),
            _ => false
        }
    }

    fn prohibits_writing(&self, variable: Variable) -> bool {
        match self {
            PartialType::Box(pt) => pt.prohibits_writing(variable),
            PartialType::Defined(t) => t.prohibits_writing(variable),
            _ => false
        }
    }
}

#[derive(Debug, Clone)]
pub struct TypeEnviroment {
    gamma: HashMap<String, PartialType>,
}

impl TypeEnviroment {
    pub fn new() -> TypeEnviroment {
        return TypeEnviroment {
            gamma: HashMap::new()
        }
    }

    pub fn get_partial(&self, key: &String) -> Result<PartialType, String> {
        return match self.gamma.get(key) {
            Some(t) => Ok(t.clone()),
            None => panic!("Type not found")
        }
    }

    pub fn get_atomic(&self, partial: &PartialType) -> Result<AtomicType, String> {
        return match partial {
            PartialType::Undefined(t) => Err(format!("Type of {:?} is undefined, chances are it was moved", t)),
            PartialType::Defined(t) => Ok(t.clone()),
            PartialType::Box(pt) => {
                // recursively get the type of the box
                self.get_atomic(&pt)
            }
        }
    }

    pub fn get(&self, key: &String) -> Result<AtomicType, String> {
        let pt = self.get_partial(key)?;
        return self.get_atomic(&pt);
    }

    pub fn insert(&mut self, key: String, value: PartialType) {
        self.gamma.insert(key, value);
    }
}

pub fn contains(gamma: &TypeEnviroment, pt: PartialType, t: AtomicType) -> bool {
    match pt {
        PartialType::Box(ptp) => contains(gamma, *ptp, t),
        PartialType::Defined(t1) => t1 == t,
        _ => false
    }
}

pub fn read_prohibited(gamma: &TypeEnviroment, variable: Variable) -> bool {
    // for each type in the type environment
    for (var, t) in gamma.gamma.iter() {
        if t.prohibits_reading(variable.clone()) {
            return true;
        }
    }
    return false;
}

pub fn write_prohibited(gamma: &TypeEnviroment, variable: Variable) -> bool {

    println!("Checking if writing is prohibited for {:?}", variable.name);
    // for each type in the type environment
    for (var, t) in gamma.gamma.iter() {

        println!("Chcking if {:?} of type {:?} prohibits writing", var, t);
        if t.prohibits_writing(variable.clone()) {
            return true;
        }
    }
    return false;
}

pub fn t() {
    let mut x = 0;
    let mut y = &mut x;
    x = 1;
}

pub fn dom(gamma: &TypeEnviroment) -> Vec<String> {
    return gamma.gamma.keys().map(|s| s.clone()).collect();
}

pub fn shape_compatible(gamma: &TypeEnviroment, t1: &PartialType, t2: &PartialType) -> bool {
    match (t1, t2) {
        (PartialType::Defined(AtomicType::Numeric), PartialType::Defined(AtomicType::Numeric)) => true,
        (PartialType::Defined(AtomicType::Box(bt1)), PartialType::Defined(AtomicType::Box(bt2))) => shape_compatible(gamma, &PartialType::Defined(*bt1.clone()), &PartialType::Defined(*bt2.clone())),
        (PartialType::Box(t1), PartialType::Box(t2)) => shape_compatible(gamma, t1, t2),
        (PartialType::Defined(AtomicType::Reference { mutable: m1, .. }), PartialType::Defined(AtomicType::Reference { mutable: m2, .. })) => {
            m1 == m2
        },
        (PartialType::Undefined(nt1), t2) => shape_compatible(gamma, &PartialType::Defined(nt1.clone()), t2),
        (t1, PartialType::Undefined(nt2)) => shape_compatible(gamma, t1, &PartialType::Defined(nt2.clone())),
        _ => false
    }
}

pub fn move_var(gamma: TypeEnviroment, variable: &Variable) -> Result<TypeEnviroment, String> {
    let t1 = gamma.get(&variable.name)?;
    let t2 = strike(t1, variable.path.clone(), 0)?;
    let mut gamma2 = gamma.clone();
    gamma2.insert(variable.name.clone(), t2);
    return Ok(gamma2);
}

pub fn strike(t: AtomicType, p: Path, i: usize) -> Result<PartialType, String> {
    if p.selectors.len() == i {
        return Ok(PartialType::Undefined(t));
    } else {
        match t {
            AtomicType::Box(pt) => {
                let pt2 = strike(*pt, p, i + 1)?;
                return Ok(PartialType::Box(Box::new(pt2)));
            },
            _ => {
                panic!("This should not happen, cannot move through borrow");
            }
        }
    }
}


pub fn update(gamma: TypeEnviroment, t1: PartialType, p: Path, i: usize, t2: AtomicType, strong: bool) -> Result<(TypeEnviroment, PartialType), String> {

    println!("Updating type {:?} with {:?} at {:?}", t1, t2, p);

    if i == p.selectors.len() {
        return Ok((gamma, PartialType::Defined(t2)));
    } else {
        match t1.clone() {
            PartialType::Box(t) => {
                let (gamma2, t3) = update(gamma, *t, p, i + 1, t2, strong)?;
                return Ok((gamma2, PartialType::Box(Box::new(t3))));
            },
            PartialType::Defined(AtomicType::Reference { var, mutable }) => {
                if !mutable {
                    return Err(format!("Error updating reference: variable {:?} is not mutable", var));
                }

                let v = var.traverse(p, i+1);
                let g3 = write(gamma, v, t2);
                return Ok((g3?, t1));
            },
            _ => {
                panic!("This should not happen");
            }
        }
    }
}

pub fn write(gamma: TypeEnviroment, variable: Variable, t1: AtomicType) -> Result<TypeEnviroment, String> {

    println!("Writing type {:?} to {:?}", t1, variable.name);

    let p: Path = variable.path;

    let t2 = gamma.get(&variable.name)?;

    println!("Type of {:?} is currently {:?}", variable.name, t2);

    let (mut gamma2, t3) = update(gamma, PartialType::Defined(t2), p, 0, t1, true)?;

    println!("Type of {:?} is now {:?}", variable.name, t3);

    gamma2.insert(variable.name, t3);
    return Ok(gamma2);
}
