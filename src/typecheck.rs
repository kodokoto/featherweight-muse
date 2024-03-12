
use crate::{ast::{Argument, Path, Program, Term, Value, Variable}, typing::{dom, move_var, read_prohibited, shape_compatible, write, write_prohibited, AtomicType, PartialType, Type, TypeEnviroment}};

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
            Term::FunctionCall { name, params } => {
                // get the function type from the type environment
                match gamma.get(name) {
                    Ok(AtomicType::Function { args, ret }) => {
                        if args.len() != params.len() {
                            return Err(format!("Error type-checking function call:  function {:?} expected {:?} arguments, got {:?} arguments", name, args.len(), params.len()))
                        };
                        let mut g1 = gamma;
                        for (arg, param) in args.iter().zip(params.iter()) {
                            let (g2, t) = param.type_check(g1)?;
                            if *arg != t {
                                return Err(format!("Error type-checking function call:  function {:?} expected argument of type {:?}, got argument of type {:?}", name, arg, t))
                            };
                            g1 = g2;
                        }
                        match ret {
                            Some(t) => return Ok((g1, *t.clone())),
                            None => return Ok((g1, AtomicType::Epsilon))
                        }
                    },
                    _ => return Err(format!("Error type-checking function call:  function {:?} not found", name))
                }
            },
            Term::FunctionDeclaration { name, args, body, ty } => {
                // add function to type environment
                let mut g2 = gamma;

                let arg_names: Vec<String> = args.iter().map(|arg| arg.name.clone()).collect();
                // check for duplicate argument names
                for (i, arg) in arg_names.iter().enumerate() {
                    if arg_names[i + 1..].contains(arg) {
                        return Err(format!("Error type-checking function declaration:  duplicate argument name: {:?}", arg))
                    }
                }

                let arg_types = args.iter().map(|arg| arg.ty.clone()).collect();
                g2.insert(name.clone(), PartialType::Defined(AtomicType::Function { args: arg_types, ret: ty.clone().map(Box::new) }));
                
                let mut g_block = TypeEnviroment::new();

                // add arguments to the type environment
                for arg in args {
                    match arg {
                        Argument { name, ty, mutable: false, reference: false } => {
                            g_block.insert(name.clone(), PartialType::Defined(ty.clone()));
                        },
                        Argument { name, ty, mutable: true, reference: false } => {
                            g_block.insert(name.clone(), PartialType::Defined(ty.clone()));
                        },
                        Argument { name, ty, mutable: false, reference: true } => {
                            g_block.insert(name.clone(), PartialType::Defined(AtomicType::Reference { mutable: false, var: Variable { name: name.clone(), path: Path { selectors : vec![] } } }));
                        },
                        _ => return Err(format!("Error type-checking function declaration:  invalid argument declaration: {:?}", arg))
                    }
                }

                // type check the body of the function
                let mut t = AtomicType::Epsilon;
                for term in body {
                    let (g_block_2, t3) = term.type_check(g_block)?;
                    g_block = g_block_2;
                    t = t3;
                }

                // check that the return type of the function matches the type of the body
                match ty {
                    Some(ty) => {
                        if t != *ty {
                            return Err(format!("Error type-checking function declaration:  function {:?} expected return type {:?}, got return type {:?}", name, ty, t))
                        }
                    },
                    None => {}
                }

                return Ok((g2, t))
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
                let t = gamma.get(&variable.name)?;
                if !t.copyable() {
                    return Err(format!("Variable: {:?} of type {:?} is not copyable", variable.name, t))
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
                    return Err(format!("Error type-checking let declaration:  variable {:?} already exists", variable.name))
                };
                let (mut g, t) = term.type_check(gamma)?;
                println!("Type of term: {:?}", t);
                match t {
                    AtomicType::Epsilon => return Err(format!("Error type-checking let declaration: expression {:?} does not return a type", term)),
                    _ => {}
                }
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
                    return Err(format!("Error type-checking assignment:  types {:?} and {:?} are not compatible", t1, PartialType::Defined(t2.clone())))
                };
                let g3 = write(g2, variable.clone(), t2);
                match g3 {
                    Ok(g3) =>  {
                        if write_prohibited(&g3, variable.clone()) {
                            return Err(format!("Error type-checking assignment:  variable {:?} has been borrowed", variable.name))
                        };
                        return Ok((g3, AtomicType::Epsilon))        
                    }
                    Err(e) => return Err(format!("Error assigning variable {}, {}", variable.name, e))
                };
            },
        }
    }
}
