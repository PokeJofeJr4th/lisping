use std::{collections::HashMap, rc::Rc};

use crate::{
    env::{new_env, Env},
    types::Value,
};

pub mod builtins;

/// syntax => value
/// # Panics
/// Whenever :3
#[allow(clippy::too_many_lines, clippy::missing_errors_doc)]
pub fn eval(mut syn: Value, mut env: Env) -> Result<Value, Value> {
    let mut cons: Vec<Value> = Vec::new();
    let out = 'main: loop {
        match syn {
            Value::List(ref arr) => {
                if arr.is_empty() {
                    break 'main Value::List(Rc::new([]));
                }
                if arr[0].is_symbol("\\") {
                    let [param, body] = &arr[1..] else {
                        return Err(Value::error("InvalidLambdaError", arr.to_vec()));
                    };
                    let body = body.clone();
                    let sub_env = new_env(env);
                    break 'main Value::Lambda {
                        args: Box::new(param.clone()),
                        body: Box::new(body),
                        captures: sub_env,
                        is_macro: false,
                    };
                } else if arr[0].is_symbol("if") {
                    match &arr[1..] {
                        [condition, t] => {
                            if eval(condition.clone(), env.clone())?.is_truthy() {
                                syn = t.clone();
                            } else {
                                break 'main Value::symbol("nil");
                            }
                        }
                        [condition, t, f] => {
                            if eval(condition.clone(), env.clone())?.is_truthy() {
                                syn = t.clone();
                            } else {
                                syn = f.clone();
                            }
                        }
                        _ => return Err(Value::error("InvalidArgs", vec![syn.clone()])),
                    }
                } else if arr[0].is_symbol("quote") {
                    break 'main arr[1].clone();
                } else if arr[0].is_symbol("quasiquote") {
                    break 'main arr[1].quasiquote(env)?;
                } else if arr[0].is_symbol("err") {
                    // if it is an error, evaluate everything but the first and return itself
                    return Err(Value::List(
                        // start with the first value unchanged
                        arr.iter()
                            .skip(1)
                            .take(1)
                            .cloned()
                            // evaluate everything but the first if they exist
                            .chain(
                                arr.iter()
                                    .skip(2)
                                    .map(|arg| eval(arg.clone(), env.clone()))
                                    .collect::<Result<Vec<_>, _>>()?,
                            )
                            .collect(),
                    ));
                } else if arr[0].is_symbol("cons") {
                    cons.push(eval(arr[1].clone(), env.clone())?);
                    syn = arr[2].clone();
                } else if arr[0].is_symbol("let*") {
                    if arr.len() != 3 {
                        return Err(Value::error("InvalidArgs", arr.to_vec()));
                    }
                    env = new_env(env);
                    let Value::List(assigns) = &arr[1] else {
                        return Err(Value::error("InvalidArgs", arr.to_vec()));
                    };
                    for i in 0..(assigns.len() / 2) {
                        let result = eval(assigns[2 * i + 1].clone(), env.clone())?;
                        if destructure(&assigns[2 * i], result, &env).is_none() {
                            return Err(Value::error("PatternMismatch", arr.to_vec()));
                        }
                    }
                    syn = arr[2].clone();
                } else if arr[0].is_symbol("def!") {
                    if arr.len() != 3 {
                        return Err(Value::error("InvalidArgs", arr.to_vec()));
                    }
                    let Value::Symbol(i) = &arr[1] else {
                        return Err(Value::error("InvalidArgs", arr.to_vec()));
                    };
                    let result = eval(arr[2].clone(), env.clone())?;
                    env.borrow_mut().set(i, result);
                    break 'main Value::nil();
                } else if arr[0].is_symbol("do") {
                    for i in arr.iter().take(arr.len() - 1).skip(1) {
                        eval(i.clone(), env.clone())?;
                    }
                    syn = arr.last().unwrap().clone();
                } else if arr[0].is_symbol("try*") {
                    let result = eval(arr[1].clone(), env.clone());
                    match result {
                        Ok(r) => break 'main r,
                        Err(e) => {
                            for catch_block in &arr[2..] {
                                let Value::List(l) = catch_block else {
                                    return Err(Value::error(
                                        "InvalidCatchBlock",
                                        vec![catch_block.clone()],
                                    ));
                                };
                                let [Value::Symbol(catch), Value::Symbol(capture_symbol), inner @ .., body] =
                                    &l[..]
                                else {
                                    return Err(Value::error(
                                        "InvalidCatchBlock",
                                        vec![catch_block.clone()],
                                    ));
                                };
                                if catch != "catch*" {
                                    return Err(Value::error(
                                        "InvalidCatchBlock",
                                        vec![catch_block.clone()],
                                    ));
                                }
                                match inner {
                                    [Value::Symbol(cap_sym)] => {
                                        if e.as_list().is_some_and(|e| {
                                            e.first().is_some_and(|x| x.is_symbol(capture_symbol))
                                        }) {
                                            env = new_env(env);
                                            env.borrow_mut().set(cap_sym, e);
                                            syn = body.clone();
                                            continue 'main;
                                        }
                                    }
                                    [] => {
                                        env = new_env(env);
                                        env.borrow_mut().set(capture_symbol, e);
                                        syn = body.clone();
                                        continue 'main;
                                    }
                                    _ => {
                                        return Err(Value::error(
                                            "InvalidCatchBlock",
                                            vec![catch_block.clone()],
                                        ))
                                    }
                                }
                            }
                            return Err(e);
                        }
                    }
                } else {
                    match eval(arr[0].clone(), env.clone())? {
                        Value::Function {
                            fn_ref: func,
                            is_macro: false,
                        } => {
                            break 'main func(
                                arr.iter()
                                    .skip(1)
                                    .cloned()
                                    .map(|v| eval(v, env.clone()))
                                    .collect::<Result<_, _>>()?,
                                env,
                            )?;
                        }
                        Value::Function {
                            fn_ref: func,
                            is_macro: true,
                        } => {
                            syn = func(arr[1..].into(), env.clone())?;
                        }
                        Value::Lambda {
                            args,
                            body,
                            captures,
                            is_macro: false,
                        } => {
                            let vals: Vec<_> = arr
                                .iter()
                                .skip(1)
                                .cloned()
                                .map(|v| eval(v, env.clone()))
                                .collect::<Result<_, _>>()?;
                            env = new_env(captures);
                            syn = *body;
                            let vals = Value::List(Rc::from(vals));
                            if destructure(&args, vals, &env).is_none() {
                                return Err(Value::error("PatternMismatch", vec![*args]));
                            }
                        }
                        Value::Lambda {
                            args,
                            body,
                            captures,
                            is_macro: true,
                        } => {
                            let sub_env = new_env(captures);
                            let vals = Value::List(Rc::from(
                                arr.iter().skip(1).cloned().collect::<Vec<_>>(),
                            ));
                            if destructure(&args, vals, &sub_env).is_none() {
                                return Err(Value::error("PatternMismatch", vec![*args]));
                            }
                            syn = eval(*body, sub_env)?;
                        }
                        other => return Err(Value::error("NotAFunction", vec![other])),
                    }
                }
            }
            Value::Symbol(ref id) if id == "true" || id == "false" || id == "nil" => {
                break 'main syn.clone()
            }
            Value::Symbol(ref id) => match env.borrow().get(id) {
                Some(x) => break 'main x,
                None => return Err(Value::error("UnresolvedIdentifier", vec![syn.clone()])),
            },
            Value::Table(table) => {
                let mut t = HashMap::new();
                for (k, v) in &*table {
                    let v = eval(v.clone(), env.clone())?;
                    t.insert(k.clone(), v);
                }
                break 'main Value::Table(Rc::new(t));
            }
            other => break 'main other,
        }
    };
    if cons.is_empty() {
        Ok(out)
    } else if let Value::List(l) = out {
        cons.extend(l.iter().cloned());
        Ok(Value::List(Rc::from(cons)))
    } else {
        Err(Value::error("NotAList", vec![out]))
    }
}

fn destructure(pat: &Value, value: Value, env: &Env) -> Option<()> {
    // println!("{pat:?} {value:?}");
    if let Value::Symbol(s) = &pat {
        env.borrow_mut().set(s, value);
        Some(())
    } else if let (Some(p), Some(v)) = (pat.as_list(), value.as_list()) {
        if p.len() > v.len() {
            return None;
        }
        for i in 0..p.len() {
            destructure(&p[i], v[i].clone(), env)?;
        }
        Some(())
    } else {
        None
    }
}
