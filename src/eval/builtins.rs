#![allow(clippy::needless_pass_by_value)]
use std::rc::Rc;

use crate::env::Env;

use super::{DynFn, Value};

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
        return Value::error("InvalidArgs", vec![Value::List(args)]);
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
        Value::Function(_) | Value::Lambda { .. } => Value::symbol("function"),
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
        l.into_iter()
            .map(|v| super::eval(Value::List(vec![func.clone(), v]), env.clone()))
            .collect(),
    )
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
