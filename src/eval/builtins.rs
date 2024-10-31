#![allow(clippy::needless_pass_by_value)]
use std::{collections::VecDeque, rc::Rc};

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
        Value::Symbol("true".to_string())
    } else {
        Value::Symbol("false".to_string())
    }
}

pub fn typ(args: Vec<Value>, _env: Env) -> Value {
    if args.len() != 1 {
        return Value::error("InvalidArgs", args);
    }
    match &args[0] {
        Value::Int(_) => Value::Symbol("int".to_string()),
        Value::Symbol(s) => match &**s {
            "\\" => Value::Symbol("function".to_string()),
            "err" => Value::Symbol("err".to_string()),
            _ => Value::Symbol("symbol".to_string()),
        },
        Value::String(_) => Value::Symbol("string".to_string()),
        Value::List(_) => Value::Symbol("list".to_string()),
        Value::Function(_) => Value::Symbol("function".to_string()),
    }
}

#[must_use]
pub fn type_is(type_is: &'static str) -> Rc<DynFn> {
    Rc::new(|args, env| {
        if typ(args, env).is_symbol(type_is) {
            Value::Symbol("true".to_string())
        } else {
            Value::Symbol("false".to_string())
        }
    })
}

pub fn quote(args: Vec<Value>, _env: Env) -> Value {
    let mut str = String::new();
    let mut args = VecDeque::from(args);
    while let Some(arg) = args.pop_front() {
        match arg {
            #[allow(clippy::cast_sign_loss)]
            Value::Int(i) => match char::try_from(i as u32) {
                Ok(c) => str.push(c),
                Err(_) => return Value::error("NotACharacter", vec![Value::Int(i)]),
            },
            Value::String(s) => str.push_str(&s),
            Value::List(values) => args.extend(values),
            Value::Function(_) => todo!(),
            Value::Symbol(_) => todo!(),
        }
    }
    Value::String(str)
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
