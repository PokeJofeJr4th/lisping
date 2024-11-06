#![allow(clippy::needless_pass_by_value)]
use std::{collections::HashMap, rc::Rc};

use crate::env::Env;

use crate::types::{DynFn, Value};

pub fn add(args: Vec<Value>, _env: Env) -> Value {
    let mut sum = 0;
    for arg in args {
        let Value::Int(i) = arg else {
            return Value::error("NotANumber", vec![arg]);
        };
        sum += i;
    }
    Value::Int(sum)
}

pub fn sub(args: Vec<Value>, _env: Env) -> Value {
    let mut difference = 0;
    for arg in args {
        let Value::Int(i) = arg else {
            return Value::error("NotANumber", vec![arg]);
        };
        // I promise this makes a little bit of sense
        difference = -i - difference;
    }
    Value::Int(difference)
}

pub fn mul(args: Vec<Value>, _env: Env) -> Value {
    let mut product = 1;
    for arg in args {
        let Value::Int(i) = arg else {
            return Value::error("NotANumber", vec![arg]);
        };
        product *= i;
    }
    Value::Int(product)
}

pub fn div(args: Vec<Value>, _env: Env) -> Value {
    let [Value::Int(a), Value::Int(b)] = &args[..] else {
        return Value::error("InvalidArgs", vec![Value::List(args.into())]);
    };
    if *b == 0 {
        Value::error("DivideByZero", vec![])
    } else {
        Value::Int(*a / *b)
    }
}

#[allow(clippy::needless_pass_by_value)]
pub fn eq(args: Vec<Value>, _env: Env) -> Value {
    if args.len() <= 1 || args.iter().skip(1).all(|x| x == &args[0]) {
        Value::symbol("true")
    } else {
        Value::symbol("false")
    }
}

pub fn not(args: Vec<Value>, _env: Env) -> Value {
    let [x] = &args[..] else {
        return Value::error("InvalidArgs", args);
    };
    if x.is_truthy() {
        Value::symbol("false")
    } else {
        Value::symbol("true")
    }
}

pub fn typ(args: Vec<Value>, _env: Env) -> Value {
    if args.len() != 1 {
        return Value::error("InvalidArgs", args);
    }
    match &args[0] {
        Value::Int(_) => Value::symbol("int"),
        Value::Symbol(s) => match &**s {
            "err" => Value::symbol("err"),
            _ => Value::symbol("symbol"),
        },
        Value::String(_) => Value::symbol("string"),
        Value::List(_) => Value::symbol("list"),
        Value::Table(_) => Value::symbol("table"),
        Value::Function { is_macro, .. } | Value::Lambda { is_macro, .. } => {
            Value::symbol(if *is_macro { "macro" } else { "function" })
        }
    }
}

#[must_use]
pub fn type_is(type_is: &'static str) -> Rc<DynFn> {
    Rc::new(|args, env| {
        if typ(args, env).is_symbol(type_is) {
            Value::symbol("true")
        } else {
            Value::symbol("false")
        }
    })
}

pub fn str(args: Vec<Value>, _env: Env) -> Value {
    let mut str = String::new();
    for arg in args {
        str.push_str(&format!("{arg}"));
    }
    Value::String(str)
}

pub fn chr(mut args: Vec<Value>, _env: Env) -> Value {
    if args.len() != 1 {
        return Value::error("InvalidArgs", args);
    }
    let Value::Int(i) = args.remove(0) else {
        return Value::error("InvalidArgs", args);
    };
    #[allow(clippy::cast_sign_loss)]
    char::try_from(i as u32).map_or_else(
        |_| Value::error("InvalidChar", vec![Value::Int(i)]),
        |c| Value::String(c.to_string()),
    )
}

/// Apply a function to each value of a list
pub fn map(mut args: Vec<Value>, env: Env) -> Value {
    if args.len() != 2 {
        return Value::error("InvalidArgs", args);
    }
    let func = args.remove(0);
    let Value::List(l) = args.remove(0) else {
        return Value::error("InvalidArgs", args);
    };
    Value::List(
        l.iter()
            .map(|v| {
                super::eval(
                    Value::List(vec![func.clone(), v.clone()].into()),
                    env.clone(),
                )
            })
            .collect(),
    )
}

/// # Panics
#[allow(clippy::cast_sign_loss)]
pub fn nth(mut args: Vec<Value>, _env: Env) -> Value {
    if args.len() != 2 {
        return Value::error("InvalidArgs", args);
    }
    let l = match args.remove(0) {
        Value::List(l) => l,
        o => return Value::error("NotAList", vec![o]),
    };
    let i = match args.remove(1) {
        Value::Int(i) => i,
        o => return Value::error("NotANumber", vec![o]),
    };
    if (i as usize) < l.len() {
        Value::error("InvalidIndex", vec![Value::List(l), Value::Int(i)])
    } else {
        l.get(i as usize).unwrap().clone()
    }
}

pub fn first(mut args: Vec<Value>, _env: Env) -> Value {
    if args.len() != 1 {
        return Value::error("InvalidArgs", args);
    }
    let arg = args.remove(0);
    if arg.is_symbol("nil") {
        arg
    } else if let Value::List(l) = arg {
        l.first().cloned().unwrap_or_else(Value::nil)
    } else {
        Value::error("NotAList", vec![arg])
    }
}

pub fn rest(mut args: Vec<Value>, _env: Env) -> Value {
    if args.len() != 1 {
        return Value::error("InvalidArgs", args);
    }
    let arg = args.remove(0);
    if arg.is_symbol("nil") {
        arg
    } else if let Value::List(l) = arg {
        Value::List(l[1..].to_vec().into())
    } else {
        Value::error("NotAList", vec![arg])
    }
}

#[allow(clippy::needless_pass_by_value)]
pub fn eval(mut args: Vec<Value>, env: Env) -> Value {
    if args.len() == 1 {
        super::eval(args.remove(0), env)
    } else {
        Value::List(
            args.into_iter()
                .map(|v| super::eval(v, env.clone()))
                .collect(),
        )
    }
}

pub fn as_macro(mut args: Vec<Value>, _env: Env) -> Value {
    if args.len() != 1 {
        return Value::error("InvalidArgs", args);
    }
    match args.remove(0) {
        Value::Function {
            fn_ref,
            is_macro: false,
        } => Value::Function {
            fn_ref,
            is_macro: true,
        },
        Value::Lambda {
            args,
            body,
            captures,
            is_macro: false,
        } => Value::Lambda {
            args,
            body,
            captures,
            is_macro: true,
        },
        other => Value::error("NotAFunction", vec![other]),
    }
}

pub fn assoc(args: Vec<Value>, _env: Env) -> Value {
    if args.len() % 2 == 0 {
        return Value::error("InvalidArgs", args);
    }
    let mut args = args.into_iter();
    let table = match args.next() {
        Some(Value::Table(t)) => t,
        Some(other) => return Value::error("NotATable", vec![other]),
        None => unreachable!(),
    };
    let mut table: HashMap<_, _> = (*table).clone();
    while let (Some(k), Some(v)) = (args.next(), args.next()) {
        table.insert(k, v);
    }
    Value::Table(Rc::new(table))
}

pub fn dissoc(args: Vec<Value>, _env: Env) -> Value {
    if args.len() <= 1 {
        return Value::error("InvalidArgs", args);
    }
    let mut args = args.into_iter();
    let table = match args.next() {
        Some(Value::Table(t)) => t,
        Some(other) => return Value::error("NotATable", vec![other]),
        None => unreachable!(),
    };
    let mut table: HashMap<_, _> = (*table).clone();
    for k in args {
        table.remove(&k);
    }
    Value::Table(Rc::new(table))
}

pub fn get(args: Vec<Value>, _env: Env) -> Value {
    let [Value::Table(t), k] = &args[..] else {
        return Value::error("InvalidArgs", args);
    };
    t.get(k).cloned().unwrap_or_else(Value::nil)
}

pub fn keys(args: Vec<Value>, _env: Env) -> Value {
    let [Value::Table(t)] = &args[..] else {
        return Value::error("InvalidArgs", args);
    };
    Value::List(t.keys().cloned().collect::<Vec<_>>().into())
}

pub fn values(args: Vec<Value>, _env: Env) -> Value {
    let [Value::Table(t)] = &args[..] else {
        return Value::error("InvalidArgs", args);
    };
    Value::List(t.values().cloned().collect::<Vec<_>>().into())
}

pub fn contains(args: Vec<Value>, _env: Env) -> Value {
    let [Value::Table(t), k] = &args[..] else {
        return Value::error("InvalidArgs", args);
    };
    Value::symbol(if t.contains_key(k) { "true" } else { "false" })
}
