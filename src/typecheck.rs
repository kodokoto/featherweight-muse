use crate::constants::TypeError;
use crate::{
    ast::{Argument, LVal, Program, Term, Value},
    typing::{
        dom, move_var, read_prohibited, shape_compatible, write, write_prohibited, Slot, Type,
        TypeEnviroment, _mut,
    },
};
pub trait TypeCheck {
    fn type_check(
        &mut self,
        gamma: TypeEnviroment,
        lifetime: usize,
    ) -> Result<(TypeEnviroment, Type), String>;
}

impl TypeCheck for Value {
    fn type_check(
        &mut self,
        gamma: TypeEnviroment,
        _lifetime: usize,
    ) -> Result<(TypeEnviroment, Type), String> {
        match self {
            Value::NumericLiteral(_) => return Ok((gamma, Type::Numeric)),
            _ => return Err("Invalid value".to_string()),
        }
    }
}

impl TypeCheck for LVal {
    fn type_check(
        &mut self,
        gamma: TypeEnviroment,
        _lifetime: usize,
    ) -> Result<(TypeEnviroment, Type), String> {
        match self {
            LVal::Variable { name, .. } => {
                let Slot { value: t, .. } = gamma.get(&name)?;
                return Ok((gamma, t.clone()));
            }
            LVal::Deref { var } => {
                let Slot { value: t, .. } = gamma.get(&var.get_name())?;
                match t.clone() {
                    Type::Reference { var: v, .. } => {
                        let Slot { value: t3, .. } = gamma.get(&v.get_name())?;
                        return Ok((gamma, t3));
                    }
                    Type::Box(tb) => return Ok((gamma, *tb.clone())),
                    _ => {
                        return Err(format!(
                            "Error type-checking dereference:  variable {:?} is not a reference",
                            var
                        ))
                    }
                }
            }
        }
    }
}

impl TypeCheck for Program {
    fn type_check(
        &mut self,
        gamma: TypeEnviroment,
        lifetime: usize,
    ) -> Result<(TypeEnviroment, Type), String> {
        let mut gamma = gamma;
        let mut t = Type::Epsilon;
        for term in &mut self.terms {
            let (g, ty) = term.type_check(gamma, lifetime)?;
            gamma = g;
            t = ty;
        }
        return Ok((gamma, t));
    }
}

impl TypeCheck for Term {
    fn type_check(
        &mut self,
        gamma: TypeEnviroment,
        lifetime: usize,
    ) -> Result<(TypeEnviroment, Type), String> {
        match self {
            Term::FunctionCall { name, params } => {
                // get the function type from the type environment
                let Ok(Slot { value, .. }) = gamma.get(name) else {
                    return Err(TypeError::FunctionNotDefined(name.clone()).to_string());
                };
                match value {
                    Type::Function { args, ret } => {
                        if args.len() != params.len() {
                            return Err(TypeError::FunctionCallIncompatableArgumentCount(
                                args.len(),
                                params.len(),
                            )
                            .to_string());
                        };
                        let mut g1 = gamma;
                        for (arg, param) in args.iter().zip(params.iter_mut()) {
                            let (g2, t) = param.type_check(g1, lifetime)?;
                            if !shape_compatible(&g2, &arg.clone(), &t.clone()) {
                                return Err(TypeError::FunctionCallIncompatableArgumentType(
                                    arg.clone(),
                                    t.clone(),
                                )
                                .to_string());
                            };
                            g1 = g2;
                        }

                        match ret {
                            Some(t) => return Ok((g1, *t.clone())),
                            None => return Ok((g1, Type::Epsilon)),
                        }
                    }
                    _ => return Err(TypeError::FunctionNotDefined(name.clone()).to_string()),
                }
            }
            Term::FunctionDeclaration {
                name: fn_name,
                args,
                body,
                ty,
            } => {
                // add function to type environment
                let mut g2 = gamma;

                let arg_names: Vec<String> = args.iter().map(|arg| arg.name.clone()).collect();
                // check for duplicate argument names
                for (i, arg) in arg_names.iter().enumerate() {
                    if arg_names[i + 1..].contains(arg) {
                        return Err(TypeError::FunctionDeclDupArg(arg.clone()).to_string());
                    }
                }

                let arg_types = args.iter().map(|arg| arg.ty.clone()).collect();
                g2.insert(
                    fn_name.clone(),
                    Type::Function {
                        args: arg_types,
                        ret: ty.clone().map(Box::new),
                    },
                    lifetime,
                );

                let mut g_block = g2.clone();

                // add arguments to the type environment
                for arg in args {
                    match arg {
                        Argument {
                            name,
                            ty,
                            mutable: false,
                            reference: false,
                        } => {
                            g_block.insert(name.clone(), ty.clone(), lifetime);
                        }
                        Argument {
                            name,
                            ty,
                            mutable: true,
                            reference: false,
                        } => {
                            g_block.insert(name.clone(), ty.clone(), lifetime);
                        }
                        Argument {
                            name,
                            ty,
                            mutable,
                            reference: true,
                        } => {
                            g_block.insert(format!("{}-{}", fn_name, name), ty.clone(), lifetime);
                            g_block.insert(
                                name.clone(),
                                Type::Reference {
                                    mutable: mutable.clone(),
                                    var: LVal::Variable {
                                        name: format!("{}-{}", fn_name, name),
                                        copyable: Some(ty.copyable()),
                                    },
                                },
                                lifetime,
                            );
                        }
                    }
                }

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
                            return Err(TypeError::FunctionUnexpectedReturn(t.clone(), ty.clone())
                                .to_string());
                        }
                    }
                    None => {}
                }

                return Ok((g2, Type::Epsilon));
            }
            Term::Variable(ref mut var) => {
                let (g, t) = var.type_check(gamma, lifetime)?;

                var.set_copyable(t.copyable()); // mark the variable as copyable or not for reduction step

                // if the variable is copyable, copy semmantics apply, else we use move semmantics
                if t.copyable() {
                    println!(
                        "Type checking copy of variable: {:?} with type {:?}",
                        var, t
                    );
                    if read_prohibited(&g, var.clone()) {
                        return Err(TypeError::CopyNotReadable(var.get_name()).to_string());
                    };
                    return Ok((g, (t.clone())));
                } else {
                    // let (g2, t) = var.type_check(g, lifetime)?;
                    println!(
                        "Type checking move of variable: {:?} with type {:?}",
                        var, t
                    );
                    if write_prohibited(&g, var.clone()) {
                        return Err(TypeError::MoveNotWritable(var.get_name()).to_string());
                    };
                    let g3 = move_var(g, var.clone(), lifetime)?;
                    println!(
                        "Move successfully completed, new type environment: {:?}",
                        g3
                    );
                    return Ok((g3, t));
                }
            }
            Term::Value(val) => {
                let (g, t) = val.type_check(gamma, lifetime)?;
                return Ok((g, t));
            }
            Term::Box { term } => {
                let (g, t) = term.type_check(gamma, lifetime)?;
                return Ok((g, Type::Box(Box::new(t))));
            }
            Term::Ref { mutable, var } => {
                if *mutable {
                    if write_prohibited(&gamma, var.clone()) {
                        return Err(
                            TypeError::MutrefAlreadyBorrowedImmut(var.get_name()).to_string()
                        );
                    };
                    if !_mut(&gamma, var.clone()) {
                        return Err(TypeError::MutrefImmut(var.get_name()).to_string());
                    };
                    return Ok((
                        gamma,
                        Type::Reference {
                            mutable: *mutable,
                            var: var.clone(),
                        },
                    ));
                } else {
                    if read_prohibited(&gamma, var.clone()) {
                        return Err(TypeError::RefAlreadyBorrowedMut(var.get_name()).to_string());
                    };
                    return Ok((
                        gamma,
                        Type::Reference {
                            mutable: *mutable,
                            var: var.clone(),
                        },
                    ));
                }
            }
            Term::Let { variable, term, .. } => {
                // check if the variable is already defined
                // x̸ ∈ dom(Γ1)
                if dom(&gamma).contains(&variable.get_name()) {
                    return Err(TypeError::LetAlreadyDefined(variable.get_name()).to_string());
                };

                // type check the term
                // Γ1 ⊢ t : T ⊣ Γ2
                let (mut g, t) = term.type_check(gamma, lifetime)?;
                match t {
                    Type::Epsilon => {
                        return Err(TypeError::LetExprNoReturn(*term.clone()).to_string())
                    }
                    _ => {}
                }

                // insert the variable into the type environment
                // Γ3 = Γ2[x 7 → T ]
                g.insert(variable.get_name().clone(), t.clone(), lifetime);
                return Ok((g, Type::Epsilon));
            }
            Term::Assign { variable, term } => {
                println!("Type checking assignment: {:?} = {:?}", variable, term);

                let (g1, t1) = variable.type_check(gamma, lifetime)?;

                let (g2, t2) = term.type_check(g1, lifetime)?;

                if !shape_compatible(&g2, &t1, &t2.clone()) {
                    return Err(TypeError::IncompatibleTypes(t1.clone(), t2.clone()).to_string());
                };

                if !t2.within(&g2, lifetime) {
                    return Err(TypeError::NotWithinScope(t2.to_string()).to_string());
                }

                let g3 = write(g2, variable.clone(), t2)?;

                if write_prohibited(&g3, variable.clone()) {
                    return Err(TypeError::AssignBorrowed(variable.get_name()).to_string());
                };
                return Ok((g3, Type::Epsilon));
            }
        }
    }
}
