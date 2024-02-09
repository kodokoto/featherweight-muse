use crate::{ast::{Program, Term, Value, Variable}, typing::{dom, move_type, read_prohibited, shape_compatible, write_prohibited, Type, TypeEnviroment}};

pub trait TypeCheck {
    fn type_check(&self, gamma: TypeEnviroment) -> Result<(TypeEnviroment, Type), String>;
}

impl TypeCheck for Value {
    fn type_check(&self, gamma: TypeEnviroment) -> Result<(TypeEnviroment, Type), String> {
        match self {
            Value::NumericLiteral(_) => return Ok((gamma, Type::Numeric)),
            // Value::Reference() => return Ok((gamma, Type::Reference)),
            Value::Epsilon => return Ok((gamma, Type::Epsilon)),
            _ => return Err("Invalid value".to_string())
        }
    }
}

impl TypeCheck for Variable {
    fn type_check(&self, gamma: TypeEnviroment) -> Result<(TypeEnviroment, Type), String> {
        let t = match gamma.get(&self) {
            Ok(t) => t,
            Err(e) => return Err(e)
        };
        return Ok((gamma, t))
    }
}

impl TypeCheck for Program {
    fn type_check(&self, gamma: TypeEnviroment) -> Result<(TypeEnviroment, Type), String> {
        let mut gamma = gamma;
        let mut t = Type::Epsilon;
        for term in &self.terms {
            let (g, ty) = term.type_check(gamma)?;
            gamma = g;
            t = ty;
        }
        return Ok((gamma, t))
    }
}

impl TypeCheck for Term {
    fn type_check(&self, gamma: TypeEnviroment) -> Result<(TypeEnviroment, Type), String> {
        match self {
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
                if write_prohibited(&g, &variable)? {
                    return Err(format!("Error type-checking Move, variable {} is not writable", variable))
                };
                let gamma2 = move_type(g, variable)?;
                return Ok((gamma2, t))
            },
            Term::Copy { variable } => {
                let t = gamma.get(&variable).unwrap();
                if !t.is_copy_type() {
                    return Err(format!("Variable: {} is not copyable", variable))
                };
                if read_prohibited(&gamma, &variable)? {
                    return Err(format!("Variable: {} is not readable", variable))
                };
                return Ok((gamma, t.clone()))
            },
            Term::Box { term } => {
                let (g, t) = term.type_check(gamma)?;
                return Ok((g, Type::Box(Box::new(t))))
            },
            Term::Ref { mutable, term } => {
                let (g, t) = term.type_check(gamma)?;

                return Ok((g, Type::Reference { 
                    mutable: *mutable, 
                    kind: Box::new(t)
                }))
            },
            Term::Let { variable, term, .. } => {
                if dom(&gamma).contains(&variable) {
                    return Err(format!("Error type-checking let declaration:  variable{} already exists", variable))
                };
                let (mut g, t) = term.type_check(gamma)?;
                g.insert(variable.clone(), t.clone());
                return Ok((g, Type::Epsilon))
            },
            Term::Assign { variable, term } => {
                let t1 = gamma.get_partial(&variable)?;
                let (g2, t2) = term.type_check(gamma)?;
                if !shape_compatible(&t1, &t2) {
                    return Err(format!("Error type-checking assignment:  types {:?} {:?} are not compatible", t1, t2))
                };
                if write_prohibited(&g2, &variable)? {
                    return Err(format!("Error type-checking assignment:  variable {} is not writable", variable))
                };
                return Ok((g2, Type::Epsilon))
            },
        }
    }
}
