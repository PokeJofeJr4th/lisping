#![allow(clippy::needless_pass_by_value)]
use std::io::stdin;
use std::{collections::HashMap, rc::Rc};

use regex::Regex;

use crate::env::{new_env, Env};

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

pub fn lt(args: Vec<Value>, _env: Env) -> Value {
    let [Value::Int(a), Value::Int(b)] = &args[..] else {
        return Value::error("InvalidArgs", args);
    };
    if *a < *b {
        Value::symbol("true")
    } else {
        Value::symbol("false")
    }
}

pub fn le(args: Vec<Value>, _env: Env) -> Value {
    let [Value::Int(a), Value::Int(b)] = &args[..] else {
        return Value::error("InvalidArgs", args);
    };
    if *a <= *b {
        Value::symbol("true")
    } else {
        Value::symbol("false")
    }
}

pub fn gt(args: Vec<Value>, _env: Env) -> Value {
    let [Value::Int(a), Value::Int(b)] = &args[..] else {
        return Value::error("InvalidArgs", args);
    };
    if *a > *b {
        Value::symbol("true")
    } else {
        Value::symbol("false")
    }
}

pub fn ge(args: Vec<Value>, _env: Env) -> Value {
    let [Value::Int(a), Value::Int(b)] = &args[..] else {
        return Value::error("InvalidArgs", args);
    };
    if *a >= *b {
        Value::symbol("true")
    } else {
        Value::symbol("false")
    }
}

pub fn print(args: Vec<Value>, _env: Env) -> Value {
    for (i, val) in args.into_iter().enumerate() {
        if i == 0 {
            print!("{val}");
        } else {
            print!(" {val}");
        }
    }
    println!();
    Value::nil()
}

pub fn input(args: Vec<Value>, _env: Env) -> Value {
    let mut buf = String::new();
    stdin().read_line(&mut buf).unwrap();
    Value::String(buf)
}

pub fn typ(args: Vec<Value>, _env: Env) -> Value {
    if args.len() != 1 {
        return Value::error("InvalidArgs", args);
    }
    match &args[0] {
        Value::Int(_) => Value::symbol("int"),
        Value::Symbol(s) => match &**s {
            "true" | "false" => Value::symbol("bool"),
            "nil" => Value::symbol("nil"),
            _ => Value::symbol("symbol"),
        },
        Value::String(_) => Value::symbol("string"),
        Value::List(l) => {
            if l.first().is_some_and(|v| v.is_symbol("err")) {
                Value::symbol("err")
            } else {
                Value::symbol("list")
            }
        }
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

pub fn symbol(mut args: Vec<Value>, _env: Env) -> Value {
    if args.len() != 1 {
        return Value::error("InvalidArgs", args);
    }
    let Value::String(s) = args.remove(0) else {
        return Value::error("InvalidArgs", args);
    };
    Value::Symbol(s)
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

pub fn int(mut args: Vec<Value>, _env: Env) -> Value {
    if args.len() != 1 {
        return Value::error("InvalidArgs", args);
    }
    match args.remove(0) {
        Value::Int(i) => Value::Int(i),
        Value::String(s) => match s.parse() {
            Ok(i) => Value::Int(i),
            Err(err) => Value::error(
                "ParseError",
                vec![
                    Value::String(s),
                    Value::symbol(match err.kind() {
                        std::num::IntErrorKind::Empty => "EmptyString",
                        std::num::IntErrorKind::InvalidDigit => "InvalidDigit",
                        std::num::IntErrorKind::PosOverflow => "PositiveOverflow",
                        std::num::IntErrorKind::NegOverflow => "NegativeOverflow",
                        _ => "UnknownReason",
                    }),
                ],
            ),
        },
        other => Value::error("InvalidArgs", vec![other]),
    }
}

/// Apply a function to each value of a list
pub fn map(mut args: Vec<Value>, env: Env) -> Value {
    if args.len() != 2 {
        return Value::error("InvalidArgs", args);
    }
    let func = args.remove(1);
    let func = match func {
        Value::Function {
            fn_ref,
            is_macro: false,
        } => fn_ref,
        Value::Lambda {
            args: params,
            body,
            captures,
            is_macro: false,
        } => Rc::new(move |args, _env| {
            let env = new_env(captures.clone());
            let mut env_borrow = env.borrow_mut();
            for (param, arg) in params.iter().zip(args) {
                env_borrow.set(param, arg);
            }
            drop(env_borrow);
            super::eval(*body.clone(), env)
        }),
        other => return Value::error("NotAFunction", vec![other]),
    };
    let Value::List(l) = args.remove(0) else {
        return Value::error("InvalidArgs", args);
    };
    Value::List(
        l.iter()
            .map(|v| func(vec![v.clone()], env.clone()))
            .collect(),
    )
}

/// # Panics
#[allow(clippy::cast_sign_loss)]
pub fn nth(mut args: Vec<Value>, _env: Env) -> Value {
    if args.len() != 2 {
        return Value::error("InvalidArgs", args);
    }
    let i = match args.remove(1) {
        Value::Int(i) => i,
        o => return Value::error("NotANumber", vec![o]),
    };
    match args.remove(0) {
        Value::List(l) => l
            .get(i as usize)
            .cloned()
            .unwrap_or_else(|| Value::error("InvalidIndex", vec![Value::List(l), Value::Int(i)])),
        Value::String(s) => s.chars().nth(i as usize).map_or_else(
            || Value::error("InvalidIndex", Vec::new()),
            |c| Value::String(c.to_string()),
        ),
        o => Value::error("NotAList", vec![o]),
    }
}

pub fn first(mut args: Vec<Value>, _env: Env) -> Value {
    if args.len() != 1 {
        return Value::error("InvalidArgs", args);
    }
    let arg = args.remove(0);
    match arg {
        Value::Symbol(ref s) if s == "nil" => arg,
        Value::List(l) => l.first().cloned().unwrap_or_else(Value::nil),
        Value::String(s) => s
            .chars()
            .next()
            .map(|c| c.to_string())
            .map_or_else(Value::nil, Value::String),
        other => Value::error("InvalidArgs", vec![other]),
    }
}

pub fn last(mut args: Vec<Value>, _env: Env) -> Value {
    if args.len() != 1 {
        return Value::error("InvalidArgs", args);
    }
    let arg = args.remove(0);
    match arg {
        Value::Symbol(ref s) if s == "nil" => arg,
        Value::List(l) => l.last().cloned().unwrap_or_else(Value::nil),
        Value::String(s) => s
            .chars()
            .last()
            .map(|c| c.to_string())
            .map_or_else(Value::nil, Value::String),
        other => Value::error("InvalidArgs", vec![other]),
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
        if l.is_empty() {
            Value::List(Rc::new([]))
        } else {
            Value::List(l[1..].into())
        }
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

pub fn apply(mut args: Vec<Value>, env: Env) -> Value {
    if args.len() != 2 {
        return Value::error("InvalidArgs", args);
    }
    let func = args.remove(0);
    let l = match args.remove(0) {
        Value::List(l) => l,
        other => return Value::error("NotAList", vec![other]),
    };
    match func {
        Value::Function {
            fn_ref,
            is_macro: false,
        } => fn_ref(l.to_vec(), env),
        Value::Lambda {
            args: params,
            body,
            captures,
            is_macro: false,
        } => {
            let env = new_env(captures);
            let mut env_borrow = env.borrow_mut();
            for (param, arg) in params.iter().zip(args) {
                env_borrow.set(param, arg);
            }
            drop(env_borrow);
            super::eval(*body, env)
        }
        other => Value::error("NotAFunction", vec![other]),
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

pub fn findall(args: Vec<Value>, _env: Env) -> Value {
    let [Value::String(re), Value::String(haystack)] = &args[..] else {
        return Value::error("InvalidArgs", args);
    };
    let re = match Regex::new(re) {
        Ok(re) => re,
        Err(regex::Error::CompiledTooBig(i)) => {
            return Value::error("RegexTooLong", vec![Value::Int(i as i32)])
        }
        Err(regex::Error::Syntax(syn)) => {
            return Value::error("InvalidRegex", vec![Value::String(syn)])
        }
        Err(_) => return Value::error("RegexError", Vec::new()),
    };
    Value::List(Rc::from(
        re.captures_iter(haystack)
            .map(|m| m.get(1).unwrap())
            .filter(|s| !s.is_empty())
            .map(|m| Value::String(m.as_str().to_string()))
            .collect::<Vec<Value>>(),
    ))
}

pub fn cons(args: Vec<Value>, _env: Env) -> Value {
    let [elem, Value::List(l)] = &args[..] else {
        return Value::error("InvalidArgs", args);
    };
    let mut elements = vec![elem.clone()];
    elements.extend(l.iter().cloned());
    Value::List(Rc::from(elements))
}

pub fn count(mut args: Vec<Value>, _env: Env) -> Value {
    if args.len() != 1 {
        return Value::error("InvalidArgs", args);
    }
    match args.remove(0) {
        Value::List(l) => Value::Int(l.len() as i32),
        Value::String(s) => Value::Int(s.len() as i32),
        Value::Table(t) => Value::Int(t.len() as i32),
        other => Value::error("NotASequence", vec![other]),
    }
}
