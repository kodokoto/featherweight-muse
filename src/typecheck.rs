
use core::borrow;

use crate::{ast::{Argument, LVal, Program, Term, Value}, typing::{dom, move_var, read_prohibited, shape_compatible, write, write_prohibited, Slot, Type, TypeEnviroment, _mut}};
use crate::constants::TypeError;
pub trait TypeCheck {
    fn type_check(&mut self, gamma: TypeEnviroment, lifetime: usize) -> Result<(TypeEnviroment, Type), String>;
}

impl TypeCheck for Value {
    fn type_check(&mut self, gamma: TypeEnviroment, lifetime: usize) -> Result<(TypeEnviroment, Type), String> {
        match self {
            Value::NumericLiteral(_) => return Ok((gamma, Type::Numeric)),
            // Value::Reference() => return Ok((gamma, Type::Reference)),
            // Value::Epsilon => return Ok((gamma, Type::Epsilon)),
            _ => return Err("Invalid value".to_string())
        }
    }
}

impl TypeCheck for LVal {
    fn type_check(&mut self, gamma: TypeEnviroment, lifetime: usize) -> Result<(TypeEnviroment, Type), String> {
        match self {
            LVal::Variable { name, copyable } => {
                let Slot {value: t, ..} = gamma.get(&name)?;
                return Ok((gamma, t.clone()))
            },
            LVal::Deref { var } => {
                let Slot {value: t, ..} = gamma.get(&var.get_name())?;
                // println!("{:?}", gamma);
                // println!("Type of variable: {:?} is {:?}", var.get_name(), t);
                match t.clone() {
                    Type::Reference { var: v, .. } => {
                        let Slot{ value: t3, ..} = gamma.get(&v.get_name())?;
                        return Ok((gamma, t3))
                    },
                    Type::Box(tb) => return Ok((gamma, *tb.clone())),
                    _ => return Err(format!("Error type-checking dereference:  variable {:?} is not a reference", var))
                }
            }
        }
        // let (t, l) = match gamma.get(&self.get_name()) {
        //     Ok(t) => t,
        //     Err(e) => return Err(e)
        // };
        // return Ok((gamma, t))
    }
}

impl TypeCheck for Program {
    fn type_check(&mut self, gamma: TypeEnviroment, lifetime: usize) -> Result<(TypeEnviroment, Type), String> {
        let mut gamma = gamma;
        let mut t = Type::Epsilon;
        for term in &mut self.terms {
            let (g, ty) = term.type_check(gamma, lifetime)?;
            gamma = g;
            t = ty;
        }
        return Ok((gamma, t))
    }
}

impl TypeCheck for Term {
    fn type_check(&mut self, gamma: TypeEnviroment, lifetime: usize) -> Result<(TypeEnviroment, Type), String> {
        match self {
            Term::FunctionCall { name, params } => {

                println!("Type checking function call: {:?}({:?})", name, params);
                // get the function type from the type environment
                match gamma.get(name)?.value {
                    Type::Function { args, ret } => {
                        if args.len() != params.len() {
                            return Err(TypeError::FUNCTION_CALL_INCOMPATABLE_ARGUMENT_COUNT(args.len(), params.len()).to_string())
                        };
                        let mut g1 = gamma;
                        for (arg, param) in args.iter().zip(params.iter_mut()) {
                            let (g2, t) = param.type_check(g1, lifetime)?;
                            if !shape_compatible(&g2, &arg.clone(), &t.clone()) {
                                // println!("Shape compatible: {:?} and {:?}", arg, t);
                                return Err(TypeError::FUNCTION_CALL_INCOMPATABLE_ARGUMENT_TYPE(arg.clone(), t.clone()).to_string())
                            };
                            g1 = g2;
                        }

                        match ret {
                            Some(t) => return Ok((g1, *t.clone())),
                            None => return Ok((g1, Type::Epsilon))
                        }
                    },
                    _ => return Err(TypeError::FUNCTION_NOT_DEFINED(name.clone()).to_string())
                }
            },
            Term::FunctionDeclaration { name: fn_name, args, body, ty } => {

                // println!("Type checking function declaration: {:?}({:?}) -> {:?}", fn_name, args.into_iter().map(|a| {a.name.clone()}).collect::<Vec<String>>(), ty);
                // add function to type environment
                let mut g2 = gamma;

                let arg_names: Vec<String> = args.iter().map(|arg| arg.name.clone()).collect();
                // check for duplicate argument names
                for (i, arg) in arg_names.iter().enumerate() {
                    if arg_names[i + 1..].contains(arg) {
                        return Err(TypeError::FUNCTION_DECL_DUP_ARG(arg.clone()).to_string())
                    }
                }

                let arg_types = args.iter().map(|arg| arg.ty.clone()).collect();
                g2.insert(fn_name.clone(), Type::Function { args: arg_types, ret: ty.clone().map(Box::new) }, lifetime);
                
                let mut g_block = TypeEnviroment::new();

                // add arguments to the type environment
                for arg in args {
                    match arg {
                        Argument { name, ty, mutable: false, reference: false } => {
                            g_block.insert(name.clone(), ty.clone(), lifetime);
                        },
                        Argument { name, ty, mutable: true, reference: false } => {
                            g_block.insert(name.clone(), ty.clone(), lifetime);
                        },
                        Argument { name, ty, mutable, reference: true } => {
                            g_block.insert(format!("{}-{}", fn_name, name), ty.clone(), lifetime);
                            g_block.insert(
                                name.clone(), 
                            Type::Reference { 
                                        mutable: mutable.clone(), 
                                        var: LVal::Variable { 
                                            name: format!("{}-{}", fn_name, name), 
                                            copyable: Some(ty.copyable())
                                        } 
                                    },
                                lifetime
                            );
                        },
                    }
                }
                
                // println!("Type enviroment after adding arguments: {:#?}", g_block);

                // type check the body of the function
                let mut t = Type::Epsilon;
                for term in body {
                    let (g_block_2, t3) = term.type_check(g_block, lifetime + 1)?;
                    g_block = g_block_2;
                    t = t3;
                }

                // check that the return type of the function matches the type of the body
                match ty {
                    Some(ty) => {
                        if t != *ty {
                            return Err(TypeError::FUNCTION_UNEXPECTED_RETURN(t.clone(), ty.clone()).to_string())
                        }
                    },
                    None => {}
                }

                return Ok((g2, Type::Epsilon))
            },
            Term::Variable(ref mut var) => {
                let (g, t) = var.type_check(gamma, lifetime)?;

                // println!("LOL Type of variable: {:?}", t);

                var.set_copyable(t.copyable()); // mark the variable as copyable or not for reduction step

                // // // if the Lvel is a dereference, typecheck the variable it points to
                // t = match var.clone() {
                //     LVal::Variable { name, .. } => {
                //         t
                //     },
                //     LVal::Deref { var, .. } => {
                //         match t.clone() {
                //             Type::Reference { var, .. } => {
                //                 // typecheck the variable wh
                //                 let (t3, _l) = g.get(&var.get_name())?;
                //                 t3
                //             },
                //             Type::Box(tb) => *tb.clone(),
                //             _ => t
                //         }
                //     }
                // };

                // match var {
                //     LVal::Variable { name, .. } => {
                //         t = g.get(key)
                //     },
                //     LVal::Deref { var, .. } => {
                //         match t {
                //             Type::Reference { var: v, .. } => {
                //                 let (t3, _l) = g.get(&v.get_name())?;
                //                 return Ok((g, t3))
                //             },
                //             Type::Box(tb) => {
                //                 return Ok((g, *tb.clone()))
                //             },
                //             _ => return Ok((g, t))
                //         }
                //     }
                // }

                // if the variable is copyable, copy semmantics apply, else we use move semmantics
                if t.copyable() {
                    println!("Type checking copy of variable: {:?} with type {:?}", var, t);
                    if read_prohibited(&g, var.clone()) {
                        return Err(TypeError::COPY_NOT_READABLE(var.get_name()).to_string())
                    };
                    return Ok((g, (t.clone())))
                } else {
                    // let (g2, t) = var.type_check(g, lifetime)?;
                    println!("Type checking move of variable: {:?} with type {:?}", var, t);
                    if write_prohibited(&g, var.clone()) {
                        return Err(TypeError::MOVE_NOT_WRITABLE(var.get_name()).to_string())
                    };
                    let g3 = move_var(g, var.clone(), lifetime)?;
                    println!("Move successfully completed, new type environment: {:?}", g3);
                    return Ok((g3, t))
                }
            },
            Term::Value(val) => {
                let (g, t) = val.type_check(gamma, lifetime)?;
                return Ok((g, t))
            },
            Term::Box { term } => {
                let (g, t) = term.type_check(gamma, lifetime)?;
                return Ok((g, Type::Box(Box::new(t))))
            },
            Term::Ref { mutable, var } => {
                if *mutable {
                    if write_prohibited(&gamma, var.clone()) {
                        return Err(TypeError::MUTREF_ALREADY_BORROWED_IMMUT(var.get_name()).to_string())
                    };
                    if !_mut(&gamma, var.clone()) {
                        return Err(TypeError::MUTREF_IMMUT(var.get_name()).to_string())
                    };
                    return Ok((gamma, Type::Reference { 
                        mutable: *mutable, 
                        var: var.clone()
                    }))
                } else {
                    if read_prohibited(&gamma, var.clone()) {
                        return Err(TypeError::REF_ALREADY_BORROWED_MUT(var.get_name()).to_string())
                    };
                    return Ok((gamma, Type::Reference { 
                        mutable: *mutable, 
                        var: var.clone()
                    }))
                }
            },
            Term::Let { variable, term, .. } => {
                // println!("Type checking let declaration of variable {:?} ", variable.get_name());
                if dom(&gamma).contains(&variable.get_name()) {
                    return Err(TypeError::LET_ALREADY_DEFINED(variable.get_name()).to_string())
                };
                let (mut g, t) = term.type_check(gamma, lifetime)?;
                // println!("Type of term: {:?}", t);
                match t {
                    Type::Epsilon => return Err(TypeError::LET_EXPR_NO_RETURN(*term.clone()).to_string()),
                    _ => {}
                }
                g.insert(variable.get_name().clone(), t.clone(), lifetime);
                // println!("{:?}", g);
                return Ok((g, Type::Epsilon))
            },
            Term::Assign { variable, term } => {
                println!("Type checking assignment: {:?} = {:?}", variable, term);

                let (mut g1, t1) = variable.type_check(gamma, lifetime)?;

                // println!("Partial type of lval {:? }: {:?}", variable.get_name(), t1);

                let (g2, t2) = term.type_check(g1, lifetime)?;

                // println!("Type of rval: {:?}", t2);

                if !shape_compatible(&g2, &t1, &t2.clone()) {
                    return Err(TypeError::INCOMPATABLE_TYPES(t1.clone(), t2.clone()).to_string())
                };

                if !t2.within(&g2, lifetime) {
                    return Err(TypeError::NOT_WITHIN_SCOPE(t2.to_string()).to_string())
                }

                // println!("Type enviroment before write: {:#?}", g2);

                let g3 = write(g2, variable.clone(), t2)?;

                // println!("Type enviroment after write: {:#?}", g3);

                if write_prohibited(&g3, variable.clone()) {
                    return Err(TypeError::ASSIGN_BORROWED(variable.get_name()).to_string())
                };
                return Ok((g3, Type::Epsilon))        
            },
        }
    }
}
