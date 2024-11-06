use std::{collections::HashMap, rc::Rc};

use crate::{
    env::{new_env, Env},
    types::Value,
};

pub mod builtins;

/// syntax => value
/// # Panics
/// Whenever :3
#[allow(clippy::too_many_lines)]
pub fn eval(mut syn: Value, mut env: Env) -> Value {
    loop {
        match syn {
            Value::List(ref arr) => {
                if arr.is_empty() {
                    return Value::List(Rc::new([]));
                }
                if arr[0].is_symbol("\\") {
                    let [param, body] = &arr[1..] else {
                        return Value::error("InvalidLambdaError", vec![arr[0].clone()]);
                    };
                    let Value::List(ids) = param else {
                        return Value::error("InvalidLambdaError", vec![arr[0].clone()]);
                    };
                    let mut param_ids = Vec::new();
                    for id in &**ids {
                        let Value::Symbol(id) = id else {
                            return Value::error("InvalidLambdaError", vec![arr[0].clone()]);
                        };
                        param_ids.push(id.clone());
                    }
                    let body = body.clone();
                    let sub_env = new_env(env);
                    return Value::Lambda {
                        args: param_ids,
                        body: Box::new(body),
                        captures: sub_env,
                        is_macro: false,
                    };
                } else if arr[0].is_symbol("if") {
                    match &arr[1..] {
                        [cond, t] => {
                            if eval(cond.clone(), env.clone()).is_truthy() {
                                syn = t.clone();
                            } else {
                                return Value::symbol("nil");
                            }
                        }
                        [cond, t, f] => {
                            if eval(cond.clone(), env.clone()).is_truthy() {
                                syn = t.clone();
                            } else {
                                syn = f.clone();
                            }
                        }
                        _ => return Value::error("InvalidArgs", vec![syn.clone()]),
                    }
                } else if arr[0].is_symbol("quote") {
                    return arr[1].clone();
                } else if arr[0].is_symbol("quasiquote") {
                    return arr[1].quasiquote(env);
                } else if arr[0].is_symbol("err") {
                    // if it is an error, evaluate everything but the first and return itself
                    return Value::List(
                        // start with the first value unchanged
                        arr.iter()
                            .take(2)
                            .cloned()
                            // evaluate everything but the first if they exist
                            .chain(arr.iter().skip(2).map(|arg| eval(arg.clone(), env.clone())))
                            .collect(),
                    );
                } else if arr[0].is_symbol("let*") {
                    if arr.len() != 3 {
                        return Value::error("InvalidArgs", arr.to_vec());
                    }
                    env = new_env(env);
                    let Value::List(assigns) = &arr[1] else {
                        return Value::error("InvalidArgs", arr.to_vec());
                    };
                    for i in 0..(assigns.len() / 2) {
                        let Value::Symbol(id) = &assigns[2 * i] else {
                            return Value::error("InvalidArgs", arr.to_vec());
                        };
                        let result = eval(assigns[2 * i + 1].clone(), env.clone());
                        env.borrow_mut().set(id, result);
                    }
                    syn = arr[2].clone();
                } else if arr[0].is_symbol("def!") {
                    if arr.len() != 3 {
                        return Value::error("InvalidArgs", arr.to_vec());
                    }
                    let Value::Symbol(i) = &arr[1] else {
                        return Value::error("InvalidArgs", arr.to_vec());
                    };
                    let result = eval(arr[2].clone(), env.clone());
                    env.borrow_mut().set(i, result);
                    return Value::nil();
                } else if arr[0].is_symbol("do") {
                    for i in arr.iter().take(arr.len() - 1).skip(1) {
                        eval(i.clone(), env.clone());
                    }
                    syn = arr.last().unwrap().clone();
                } else if arr[0].is_symbol("try*") {
                    let result = eval(arr[1].clone(), env.clone());
                    if !result.is_err() {}
                } else {
                    match eval(arr[0].clone(), env.clone()) {
                        Value::Function {
                            fn_ref: func,
                            is_macro: false,
                        } => {
                            return func(
                                arr.iter()
                                    .skip(1)
                                    .cloned()
                                    .map(|v| eval(v, env.clone()))
                                    .collect(),
                                env,
                            )
                        }
                        Value::Function {
                            fn_ref: func,
                            is_macro: true,
                        } => {
                            syn = func(arr[1..].into(), env.clone());
                        }
                        Value::Lambda {
                            args: params,
                            body,
                            captures,
                            is_macro: false,
                        } => {
                            let args: Vec<_> = arr
                                .iter()
                                .skip(1)
                                .cloned()
                                .map(|v| eval(v, env.clone()))
                                .collect();
                            env = new_env(captures);
                            syn = *body;
                            let mut env_borrow = env.borrow_mut();
                            for (param, arg) in params.into_iter().zip(args) {
                                env_borrow.set(&param, arg);
                            }
                            drop(env_borrow);
                        }
                        Value::Lambda {
                            args: params,
                            body,
                            captures,
                            is_macro: true,
                        } => {
                            let sub_env = new_env(captures);
                            let mut sub_env_borrow = sub_env.borrow_mut();
                            for (param, arg) in params.into_iter().zip(arr.iter().skip(1)) {
                                sub_env_borrow.set(&param, arg.clone());
                            }
                            drop(sub_env_borrow);
                            syn = eval(*body, sub_env);
                        }
                        other => return Value::error("NotAFunction", vec![other]),
                    }
                }
            }
            Value::Symbol(ref id) if id == "true" || id == "false" || id == "nil" => {
                return syn.clone()
            }
            Value::Symbol(ref id) => match env.borrow().get(id) {
                Some(x) => return x,
                None => return Value::error("UnresolvedIdentifier", vec![syn.clone()]),
            },
            Value::Table(table) => {
                let mut t = HashMap::new();
                for (k, v) in &*table {
                    let v = eval(v.clone(), env.clone());
                    t.insert(k.clone(), v);
                }
                return Value::Table(Rc::new(t));
            }
            other => return other,
        }
    }
}
