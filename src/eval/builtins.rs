#![allow(clippy::needless_pass_by_value, clippy::missing_errors_doc)]
use std::io::stdin;
use std::{collections::HashMap, rc::Rc};

use regex::Regex;

use crate::env::{new_env, Env};

use crate::types::{DynFn, Value};

pub fn add(args: Vec<Value>, _env: Env) -> Result<Value, Value> {
    let mut sum = 0;
    for arg in args {
        let Value::Int(i) = arg else {
            return Err(Value::error("NotANumber", vec![arg]));
        };
        sum += i;
    }
    Ok(Value::Int(sum))
}

pub fn sub(args: Vec<Value>, _env: Env) -> Result<Value, Value> {
    let mut difference = 0;
    for arg in args {
        let Value::Int(i) = arg else {
            return Err(Value::error("NotANumber", vec![arg]));
        };
        // I promise this makes a little bit of sense
        difference = -i - difference;
    }
    Ok(Value::Int(difference))
}

pub fn mul(args: Vec<Value>, _env: Env) -> Result<Value, Value> {
    let mut product = 1;
    for arg in args {
        let Value::Int(i) = arg else {
            return Err(Value::error("NotANumber", vec![arg]));
        };
        product *= i;
    }
    Ok(Value::Int(product))
}

pub fn div(args: Vec<Value>, _env: Env) -> Result<Value, Value> {
    let [Value::Int(a), Value::Int(b)] = &args[..] else {
        return Err(Value::error("InvalidArgs@div", vec![Value::List(args.into())]));
    };
    if *b == 0 {
        Err(Value::error("DivideByZero", vec![]))
    } else {
        Ok(Value::Int(*a / *b))
    }
}

#[allow(clippy::needless_pass_by_value)]
pub fn eq(args: Vec<Value>, _env: Env) -> Result<Value, Value> {
    if args.len() <= 1 || args.iter().skip(1).all(|x| x == &args[0]) {
        Ok(Value::symbol("true"))
    } else {
        Ok(Value::symbol("false"))
    }
}

pub fn lt(args: Vec<Value>, _env: Env) -> Result<Value, Value> {
    let [Value::Int(a), Value::Int(b)] = &args[..] else {
        return Err(Value::error("InvalidArgs@lt", args));
    };
    if *a < *b {
        Ok(Value::symbol("true"))
    } else {
        Ok(Value::symbol("false"))
    }
}

pub fn le(args: Vec<Value>, _env: Env) -> Result<Value, Value> {
    let [Value::Int(a), Value::Int(b)] = &args[..] else {
        return Err(Value::error("InvalidArgs@le", args));
    };
    if *a <= *b {
        Ok(Value::symbol("true"))
    } else {
        Ok(Value::symbol("false"))
    }
}

pub fn gt(args: Vec<Value>, _env: Env) -> Result<Value, Value> {
    let [Value::Int(a), Value::Int(b)] = &args[..] else {
        return Err(Value::error("InvalidArgs@gt", args));
    };
    if *a > *b {
        Ok(Value::symbol("true"))
    } else {
        Ok(Value::symbol("false"))
    }
}

pub fn ge(args: Vec<Value>, _env: Env) -> Result<Value, Value> {
    let [Value::Int(a), Value::Int(b)] = &args[..] else {
        return Err(Value::error("InvalidArgs@ge", args));
    };
    if *a >= *b {
        Ok(Value::symbol("true"))
    } else {
        Ok(Value::symbol("false"))
    }
}

pub fn print(args: Vec<Value>, _env: Env) -> Result<Value, Value> {
    for (i, val) in args.into_iter().enumerate() {
        if i == 0 {
            print!("{val}");
        } else {
            print!(" {val}");
        }
    }
    println!();
    Ok(Value::nil())
}

/// # Panics
/// If something goes wrong reading from stdin
#[allow(unused_variables)]
pub fn input(args: Vec<Value>, _env: Env) -> Result<Value, Value> {
    let mut buf = String::new();
    stdin().read_line(&mut buf).unwrap();
    Ok(Value::String(buf))
}

pub fn typ(args: Vec<Value>, _env: Env) -> Result<Value, Value> {
    if args.len() != 1 {
        return Err(Value::error("InvalidArgs@type", args));
    }
    Ok(match &args[0] {
        Value::Int(_) => Value::symbol("int"),
        Value::Symbol(s) => match &**s {
            "true" | "false" => Value::symbol("bool"),
            "nil" => Value::symbol("nil"),
            _ => Value::symbol("symbol"),
        },
        Value::String(_) => Value::symbol("string"),
        Value::List(_) => Value::symbol("list"),
        Value::Table(_) => Value::symbol("table"),
        Value::Function { is_macro, .. } | Value::Lambda { is_macro, .. } => {
            Value::symbol(if *is_macro { "macro" } else { "function" })
        }
    })
}

#[must_use]
pub fn type_is(type_is: &'static str) -> Rc<DynFn> {
    Rc::new(|args, env| {
        if typ(args, env)?.is_symbol(type_is) {
            Ok(Value::symbol("true"))
        } else {
            Ok(Value::symbol("false"))
        }
    })
}

pub fn str(args: Vec<Value>, _env: Env) -> Result<Value, Value> {
    let mut str = String::new();
    for arg in args {
        str.push_str(&format!("{arg}"));
    }
    Ok(Value::String(str))
}

pub fn symbol(args: Vec<Value>, _env: Env) -> Result<Value, Value> {
    if args.len() != 1 {
        return Err(Value::error("InvalidArgs@symbol", args));
    }
    let Value::String(s) = args[0].clone() else {
        return Err(Value::error("InvalidArgs@symbol", args));
    };
    Ok(Value::Symbol(s))
}

pub fn chr(mut args: Vec<Value>, _env: Env) -> Result<Value, Value> {
    if args.len() != 1 {
        return Err(Value::error("InvalidArgs@chr", args));
    }
    let Value::Int(i) = args.remove(0) else {
        return Err(Value::error("InvalidArgs@chr", args));
    };
    #[allow(clippy::cast_sign_loss)]
    char::try_from(i as u32).map_or_else(
        |_| Err(Value::error("InvalidChar", vec![Value::Int(i)])),
        |c| Ok(Value::String(c.to_string())),
    )
}

pub fn int(mut args: Vec<Value>, _env: Env) -> Result<Value, Value> {
    if args.len() != 1 {
        return Err(Value::error("InvalidArgs@int", args));
    }
    match args.remove(0) {
        Value::Int(i) => Ok(Value::Int(i)),
        Value::String(s) => match s.parse() {
            Ok(i) => Ok(Value::Int(i)),
            Err(err) => Err(Value::error(
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
            )),
        },
        other => Err(Value::error("InvalidArgs@int", vec![other])),
    }
}

/// # Panics
#[allow(clippy::cast_sign_loss)]
pub fn nth(mut args: Vec<Value>, _env: Env) -> Result<Value, Value> {
    if args.len() != 2 {
        return Err(Value::error("InvalidArgs@nth", args));
    }
    let i = match args.remove(1) {
        Value::Int(i) => i,
        o => return Err(Value::error("NotANumber", vec![o])),
    };
    match args.remove(0) {
        Value::List(l) => l
            .get(i as usize)
            .cloned()
            .ok_or_else(|| Value::error("InvalidIndex", vec![Value::List(l), Value::Int(i)])),
        Value::String(s) => s
            .chars()
            .nth(i as usize)
            .map(|c| Value::String(c.to_string()))
            .ok_or_else(|| Value::error("InvalidIndex", Vec::new())),
        o => Err(Value::error("NotAList", vec![o])),
    }
}

pub fn first(mut args: Vec<Value>, _env: Env) -> Result<Value, Value> {
    if args.len() != 1 {
        return Err(Value::error("InvalidArgs@first", args));
    }
    let arg = args.remove(0);
    match arg {
        Value::Symbol(ref s) if s == "nil" => Ok(arg),
        Value::List(l) => Ok(l.first().cloned().unwrap_or_else(Value::nil)),
        Value::String(s) => s
            .chars()
            .next()
            .map(|c| Value::String(c.to_string()))
            .ok_or_else(Value::nil),
        other => Err(Value::error("InvalidArgs@first", vec![other])),
    }
}

pub fn last(mut args: Vec<Value>, _env: Env) -> Result<Value, Value> {
    if args.len() != 1 {
        return Err(Value::error("InvalidArgs@last", args));
    }
    let arg = args.remove(0);
    match arg {
        Value::Symbol(ref s) if s == "nil" => Ok(arg),
        Value::List(l) => Ok(l.last().cloned().unwrap_or_else(Value::nil)),
        Value::String(s) => Ok(s
            .chars()
            .last()
            .map_or_else(Value::nil, |c| Value::String(c.to_string()))),
        other => Err(Value::error("InvalidArgs@last", vec![other])),
    }
}

pub fn rest(mut args: Vec<Value>, _env: Env) -> Result<Value, Value> {
    if args.len() != 1 {
        return Err(Value::error("InvalidArgs@rest", args));
    }
    let arg = args.remove(0);
    if arg.is_symbol("nil") {
        Ok(arg)
    } else if let Value::List(l) = arg {
        if l.is_empty() {
            Ok(Value::List(Rc::new([])))
        } else {
            Ok(Value::List(l[1..].into()))
        }
    } else {
        Err(Value::error("NotAList", vec![arg]))
    }
}

#[allow(clippy::needless_pass_by_value)]
pub fn eval(mut args: Vec<Value>, env: Env) -> Result<Value, Value> {
    if args.len() == 1 {
        super::eval(args.remove(0), env)
    } else {
        Ok(Value::List(
            args.into_iter()
                .map(|v| super::eval(v, env.clone()))
                .collect::<Result<_, _>>()?,
        ))
    }
}

pub fn apply(mut args: Vec<Value>, env: Env) -> Result<Value, Value> {
    if args.len() != 2 {
        return Err(Value::error("InvalidArgs@apply", args));
    }
    let func = args.remove(0);
    let l = args.remove(0);
    match func {
        Value::Function {
            fn_ref,
            is_macro: false,
        } => fn_ref(
            match l {
                Value::List(l) => l.to_vec(),
                o => return Err(Value::error("NotAList", vec![o])),
            },
            env,
        ),
        Value::Lambda {
            args: params,
            body,
            captures,
            is_macro: false,
        } => {
            let env = new_env(captures);
            if super::destructure(&params, l, &env).is_none() {
                return Err(Value::error("PatternMismatch", vec![*params]));
            }
            super::eval(*body, env)
        }
        other => Err(Value::error("NotAFunction", vec![other])),
    }
}

pub fn as_macro(mut args: Vec<Value>, _env: Env) -> Result<Value, Value> {
    if args.len() != 1 {
        return Err(Value::error("InvalidArgs@macro", args));
    }
    match args.remove(0) {
        Value::Function {
            fn_ref,
            is_macro: false,
        } => Ok(Value::Function {
            fn_ref,
            is_macro: true,
        }),
        Value::Lambda {
            args,
            body,
            captures,
            is_macro: false,
        } => Ok(Value::Lambda {
            args,
            body,
            captures,
            is_macro: true,
        }),
        other => Err(Value::error("NotAFunction", vec![other])),
    }
}

pub fn assoc(args: Vec<Value>, _env: Env) -> Result<Value, Value> {
    if args.len() % 2 == 0 {
        return Err(Value::error("InvalidArgs@assoc", args));
    }
    let mut args = args.into_iter();
    let table = match args.next() {
        Some(Value::Table(t)) => t,
        Some(other) => return Err(Value::error("NotATable", vec![other])),
        None => unreachable!(),
    };
    let mut table: HashMap<_, _> = (*table).clone();
    while let (Some(k), Some(v)) = (args.next(), args.next()) {
        table.insert(k, v);
    }
    Ok(Value::Table(Rc::new(table)))
}

pub fn dissoc(args: Vec<Value>, _env: Env) -> Result<Value, Value> {
    if args.len() <= 1 {
        return Err(Value::error("InvalidArgs@dissoc", args));
    }
    let mut args = args.into_iter();
    let table = match args.next() {
        Some(Value::Table(t)) => t,
        Some(other) => return Err(Value::error("NotATable", vec![other])),
        None => unreachable!(),
    };
    let mut table: HashMap<_, _> = (*table).clone();
    for k in args {
        table.remove(&k);
    }
    Ok(Value::Table(Rc::new(table)))
}

pub fn get(args: Vec<Value>, _env: Env) -> Result<Value, Value> {
    let [Value::Table(t), k] = &args[..] else {
        return Err(Value::error("InvalidArgs@get", args));
    };
    Ok(t.get(k).cloned().unwrap_or_else(Value::nil))
}

pub fn keys(args: Vec<Value>, _env: Env) -> Result<Value, Value> {
    let [Value::Table(t)] = &args[..] else {
        return Err(Value::error("InvalidArgs@keys", args));
    };
    Ok(Value::List(t.keys().cloned().collect::<Vec<_>>().into()))
}

pub fn values(args: Vec<Value>, _env: Env) -> Result<Value, Value> {
    let [Value::Table(t)] = &args[..] else {
        return Err(Value::error("InvalidArgs@values", args));
    };
    Ok(Value::List(t.values().cloned().collect::<Vec<_>>().into()))
}

pub fn contains(args: Vec<Value>, _env: Env) -> Result<Value, Value> {
    let [Value::Table(t), k] = &args[..] else {
        return Err(Value::error("InvalidArgs@contains", args));
    };
    Ok(Value::symbol(if t.contains_key(k) {
        "true"
    } else {
        "false"
    }))
}

pub fn findall(args: Vec<Value>, _env: Env) -> Result<Value, Value> {
    let [Value::String(re), Value::String(haystack)] = &args[..] else {
        return Err(Value::error("InvalidArgs@findall", args));
    };
    let re = match Regex::new(re) {
        Ok(re) => re,
        Err(regex::Error::CompiledTooBig(i)) => {
            return Err(Value::error(
                "RegexTooLong",
                vec![i32::try_from(i).map_or_else(|_| Value::String(i.to_string()), Value::Int)],
            ))
        }
        Err(regex::Error::Syntax(syn)) => {
            return Err(Value::error("InvalidRegex", vec![Value::String(syn)]))
        }
        Err(_) => return Err(Value::error("RegexError", Vec::new())),
    };
    Ok(Value::List(Rc::from(
        re.captures_iter(haystack)
            .map(|m| m.iter().flatten().collect::<Vec<_>>())
            .filter(|s| !s.is_empty())
            .map(|m| {
                Value::List(
                    m.into_iter()
                        .map(|m| Value::String(m.as_str().to_string()))
                        .collect(),
                )
            })
            .collect::<Vec<Value>>(),
    )))
}

pub fn cons(args: Vec<Value>, _env: Env) -> Result<Value, Value> {
    let [elem, Value::List(l)] = &args[..] else {
        return Err(Value::error("InvalidArgs@cons", args));
    };
    let mut elements = vec![elem.clone()];
    elements.extend(l.iter().cloned());
    Ok(Value::List(Rc::from(elements)))
}

pub fn count(mut args: Vec<Value>, _env: Env) -> Result<Value, Value> {
    if args.len() != 1 {
        return Err(Value::error("InvalidArgs@count", args));
    }
    let i = match args.remove(0) {
        Value::List(l) => l.len(),
        Value::String(s) => s.len(),
        Value::Table(t) => t.len(),
        other => return Err(Value::error("NotASequence", vec![other])),
    };
    match i32::try_from(i) {
        Ok(i) => Ok(Value::Int(i)),
        Err(e) => Err(Value::error(
            "Overflow",
            vec![Value::String(i.to_string()), Value::String(e.to_string())],
        )),
    }
}
