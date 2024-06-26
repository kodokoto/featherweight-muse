use std::collections::HashSet;

use crate::{
    ast::{LVal, Reference, Term, Value},
    reduction::Evaluate,
    state::State,
    typecheck::TypeCheck,
    typing::{contains, Slot, Type, TypeEnviroment},
};

pub fn assert_preservation(
    s1: State,
    t1: Term,
    g1: TypeEnviroment,
    lifetime: usize,
) -> Result<(), String> {
    println!("Asserting progress");

    if !valid_state(s1.clone(), t1.clone())? {
        return Err("Invalid state".to_string());
    }

    if !well_formed(g1.clone())? {
        return Err("Type enviroment is not well formed".to_string());
    }

    if !safe_abstraction(s1.clone(), g1.clone())? {
        return Err("Type enviroment is not a safe abstraction of current state".to_string());
    }

    let (g2, t) = t1.clone().type_check(g1, lifetime)?;

    let (s2, t2) = t1.clone().evaluate(s1, lifetime)?;

    if !safe_abstraction(s2.clone(), g2)? {
        return Err(
            "Type enviroment is not a safe abstraction of current state after preservation"
                .to_string(),
        );
    }

    match t2 {
        Term::Value(v) => {
            if !valid_type(&s2, &v, t.clone())? {
                println!("Invalid type");
                println!("{:?} {:?}", v, t.clone());
                return Err("Invalid type".to_string());
            } else {
                return Ok(());
            }
        }
        _ => {
            return Err("Term did not evaluate to a value".to_string());
        }
    }
}

pub fn assert_progess(
    s1: State,
    t1: Term,
    g1: TypeEnviroment,
    lifetime: usize,
) -> Result<(), String> {
    println!("Asserting progress");

    if !valid_state(s1.clone(), t1.clone())? {
        return Err("Invalid state".to_string());
    }

    if !well_formed(g1.clone())? {
        return Err("Type enviroment is not well formed".to_string());
    }

    if !safe_abstraction(s1.clone(), g1.clone())? {
        return Err("Type enviroment is not a safe abstraction of current state".to_string());
    }

    t1.clone().type_check(g1, lifetime)?;

    match t1 {
        Term::Value(_) => {
            return Ok(());
        }
        _ => {
            t1.clone().evaluate(s1, lifetime)?;
            return Ok(());
        }
    }
}

pub fn safe_abstraction(s: State, g: TypeEnviroment) -> Result<bool, String> {
    println!("P: {:#?}", s);
    println!("G: {:#?}", g);

    let xs: HashSet<String> = s.dom().into_iter().collect();
    let ys: HashSet<String> = g.dom().into_iter().collect();

    if xs != ys {
        println!("{:?} {:?}", s.dom(), g.dom());
        println!("Not the same domain");
        return Ok(false);
    }
    for x in g.dom() {
        let l = &s.locate(x.clone())?;
        if !valid_type(
            &s,
            &s.store.get(l.clone()).unwrap().value,
            g.get_partial(&x)?.value,
        )? {
            println!("Invalid type");
            println!(
                "{:?} {:?}",
                s.store.get(l.clone()).unwrap().value,
                g.get_partial(&x)?.value
            );
            // panic!();
            return Ok(false);
        }
    }
    return Ok(true);
}

pub fn valid_store(s: State) -> Result<bool, String> {
    let mut set = HashSet::new();
    for value in s.store.cells.values() {
        if set.contains(&value.value) {
            return Ok(false); // Duplicate value found
        }
        match value.value {
            Value::NumericLiteral(_) => {}
            _ => {
                set.insert(value.value.clone());
            }
        }
    }
    Ok(true)
}

pub fn get_values(t: Term, mut set: HashSet<Value>) -> HashSet<Value> {
    match t {
        Term::Box { term, .. } => {
            set.extend(get_values(*term, set.clone()));
            return set;
        }
        Term::Assign { term, .. } => {
            set.extend(get_values(*term, set.clone()));
            return set;
        }
        Term::FunctionDeclaration { body, .. } => {
            for term in body {
                set.extend(get_values(term, set.clone()));
            }
            return set;
        }
        _ => return set,
    }
}

pub fn valid_state(s: State, t: Term) -> Result<bool, String> {
    if !valid_store(s.clone())? {
        return Err("Invalid store".to_string());
    }
    let mut set = HashSet::new();
    set = get_values(t, set);
    for value in s.store.cells.values() {
        if set.contains(&value.value) {
            return Ok(false);
        }
    }

    return Ok(true);
}

pub fn well_formed(g: TypeEnviroment) -> Result<bool, String> {
    for x in g.dom() {
        for y in g.dom() {
            if x == y {
                continue;
            }
            let Slot {
                value: t1,
                lifetime: l1,
            } = g.get_partial(&x)?;
            let lv = LVal::Variable {
                name: y.clone(),
                copyable: None,
            };
            if contains(
                t1.clone(),
                Type::Reference {
                    var: lv.clone(),
                    mutable: true,
                },
            ) || contains(
                t1.clone(),
                Type::Reference {
                    var: lv.clone(),
                    mutable: false,
                },
            ) {
                lv.clone().type_check(g.clone(), l1)?;
            }
        }
    }
    return Ok(true);
}

pub fn valid_type(s: &State, v: &Value, t: Type) -> Result<bool, String> {
    match (v, t) {
        (Value::Undefined, Type::Undefined(_)) => return Ok(true),
        (_, Type::Undefined(_t)) => return valid_type(s, v, *_t),
        (Value::Epsilon, Type::Epsilon) => return Ok(true),
        (Value::NumericLiteral(_), Type::Numeric) => return Ok(true),
        (Value::Reference(r @ Reference { owned: true, .. }), Type::Box(bt)) => {
            let vt = &s.store.get(r.clone()).unwrap().value;
            return valid_type(s, &vt, *bt);
        }
        (Value::Reference(r @ Reference { owned: false, .. }), Type::Reference { var, .. }) => {
            let x: String = var.get_name();
            if s.locate(x.clone())? == r.clone() {
                return Ok(true);
            }
            return Ok(true);
        }
        _ => return Ok(false),
    }
}
