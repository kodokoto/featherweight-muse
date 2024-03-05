use crate::{ast::{Program, Term, Value, Variable}, typing::{dom, move_var, read_prohibited, shape_compatible, write_prohibited, write, AtomicType, PartialType, Type, TypeEnviroment}};

pub trait TypeCheck {
    fn type_check(&self, gamma: TypeEnviroment) -> Result<(TypeEnviroment, AtomicType), String>;
}

impl TypeCheck for Value {
    fn type_check(&self, gamma: TypeEnviroment) -> Result<(TypeEnviroment, AtomicType), String> {
        match self {
            Value::NumericLiteral(_) => return Ok((gamma, AtomicType::Numeric)),
            // Value::Reference() => return Ok((gamma, Type::Reference)),
            // Value::Epsilon => return Ok((gamma, Type::Epsilon)),
            _ => return Err("Invalid value".to_string())
        }
    }
}

impl TypeCheck for Variable {
    fn type_check(&self, gamma: TypeEnviroment) -> Result<(TypeEnviroment, AtomicType), String> {
        let t = match gamma.get(&self.name) {
            Ok(t) => t,
            Err(e) => return Err(e)
        };
        return Ok((gamma, t))
    }
}

impl TypeCheck for Program {
    fn type_check(&self, gamma: TypeEnviroment) -> Result<(TypeEnviroment, AtomicType), String> {
        let mut gamma = gamma;
        let mut t = AtomicType::Epsilon;
        for term in &self.terms {
            let (g, ty) = term.type_check(gamma)?;
            gamma = g;
            t = ty;
        }
        return Ok((gamma, t))
    }
}

impl TypeCheck for Term {
    fn type_check(&self, gamma: TypeEnviroment) -> Result<(TypeEnviroment, AtomicType), String> {
        match self {
            Term::FunctionCall { .. } => {
                return Err("Function calls are not supported".to_string())
            },
            Term::FunctionDeclaration { name, args, body, ty } => {
                return Err("Function declarations are not supported".to_string())
            },
            Term::Variable(var) => {
                let (g, t) = var.type_check(gamma)?;
                return Ok((g, t))
            },
            Term::Value(val) => {
                let (g, t) = val.type_check(gamma)?;
                return Ok((g, t))
            },
            Term::Move { variable } => {
                let (g, t) = variable.type_check(gamma)?;
                println!("Type checking move of variable: {:?} with type {:?}", variable, t);
                if write_prohibited(&g, variable.clone()) {
                    return Err(format!("Error type-checking Move, variable {:?} is not writable", variable))
                };
                let gamma2 = move_var(g, &variable)?;
                println!("Move successfully completed, new type environment: {:?}", gamma2);
                return Ok((gamma2, t))
            },
            Term::Copy { variable } => {
                let t = gamma.get(&variable.name).unwrap();
                if !t.copyable() {
                    return Err(format!("Variable: {:?} is not copyable", variable))
                };
                if read_prohibited(&gamma, variable.clone()) {
                    return Err(format!("Variable: {:?} is not readable", variable))
                };
                return Ok((gamma, t.clone()))
            },
            Term::Box { term } => {
                let (g, t) = term.type_check(gamma)?;
                return Ok((g, AtomicType::Box(Box::new(t))))
            },
            Term::Ref { mutable, var } => {
                if *mutable {
                    if write_prohibited(&gamma, var.clone()) {
                        return Err(format!("Error type-checking Ref, variable {:?} is not writable", var))
                    };
                    return Ok((gamma, AtomicType::Reference { 
                        mutable: *mutable, 
                        var: var.clone()
                    }))
                } else {
                    if read_prohibited(&gamma, var.clone()) {
                        return Err(format!("Error type-checking Ref, variable {:?} is not readable", var))
                    };
                    return Ok((gamma, AtomicType::Reference { 
                        mutable: *mutable, 
                        var: var.clone()
                    }))
                }
            },
            Term::Let { variable, term, .. } => {
                println!("Type checking let declaration: ");
                if dom(&gamma).contains(&variable.name) {
                    return Err(format!("Error type-checking let declaration:  variable {:?} already exists", variable))
                };
                let (mut g, t) = term.type_check(gamma)?;
                println!("Type of term: {:?}", t);
                g.insert(variable.name.clone(), PartialType::Defined(t.clone()));
                println!("{:?}", g);
                return Ok((g, AtomicType::Epsilon))
            },
            Term::Assign { variable, term } => {
                println!("Type checking assignment: ");

                println!("Assigning {:?} = {:?}", variable, term);

                let t1 = gamma.get_partial(&variable.name)?;

                println!("Partial type of lval {:? }: {:?}", variable, t1);

                let (g2, t2) = term.type_check(gamma)?;

                println!("Type of rval: {:?}", t2);

                if !shape_compatible(&g2, &t1, &PartialType::Defined(t2.clone())) {
                    return Err(format!("Error type-checking assignment:  types {:?} {:?} are not compatible", t1, t2))
                };
                let g3 = write(g2, variable.clone(), t2);
                match g3 {
                    Ok(g3) =>  {
                        if write_prohibited(&g3, variable.clone()) {
                            return Err(format!("Error type-checking assignment:  variable {:?} is not writable", variable))
                        };
                        return Ok((g3, AtomicType::Epsilon))        
                    }
                    Err(e) => return Err(format!("Error assigning variable {}, {}", variable.name, e))
                };
            },
        }
    }
}
