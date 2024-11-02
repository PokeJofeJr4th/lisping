use std::{
    collections::HashMap,
    fmt::{Debug, Display},
    rc::Rc,
};

use crate::env::{new_env, Env};

pub mod builtins;

pub type DynFn = dyn Fn(Vec<Value>, Env) -> Value;

#[derive(Clone)]
/// A value in the lisp. This includes source code and runtime data.
pub enum Value {
    /// An int literal or value
    ///
    /// Evaluates to itself
    Int(i32),
    /// A string literal or value
    ///
    /// Evaluates to itself
    String(String),
    /// A symbol or identifier
    ///
    /// Keywords evaluate to constants or themself.
    Symbol(String),
    /// A list of values
    ///
    /// Attempts to evaluate as a function invocation. Special forms may apply
    List(Rc<[Value]>),
    /// A key-value store
    Table(Rc<HashMap<String, Value>>),
    /// A builtin function
    ///
    /// Evaluates to itself
    Function { fn_ref: Rc<DynFn>, is_macro: bool },
    /// A function that captures variables from its environment
    ///
    /// Evaluates to itself
    Lambda {
        args: Vec<String>,
        body: Box<Value>,
        captures: Env,
        is_macro: bool,
    },
}

impl Value {
    #[must_use]
    pub fn as_function(&self, func: &str) -> Option<&[Self]> {
        let Self::List(arr) = self else {
            return None;
        };
        let [Self::Symbol(id), args @ ..] = &arr[..] else {
            return None;
        };
        if id != func {
            return None;
        }
        Some(args)
    }

    #[must_use]
    pub fn is_symbol(&self, id: &str) -> bool {
        let Self::Symbol(my_id) = self else {
            return false;
        };
        my_id == id
    }

    #[must_use]
    #[allow(clippy::needless_pass_by_value)]
    pub fn error(msg: &str, mut args: Vec<Self>) -> Self {
        args.insert(0, Self::symbol(msg));
        args.insert(0, Self::symbol("err"));
        Self::List(args.into())
    }

    #[must_use]
    pub fn quasiquote(&self, env: Env) -> Self {
        match self {
            other @ (Self::Int(_)
            | Self::String(_)
            | Self::Symbol(_)
            | Self::Function { .. }
            | Self::Table(_)
            | Self::Lambda { .. }) => other.clone(),
            Self::List(vec) => {
                if vec.first().is_some_and(|val| val.is_symbol("unquote")) {
                    eval(vec[1].clone(), env)
                } else {
                    Self::List(
                        vec.iter()
                            .map(|v| Self::quasiquote(v, env.clone()))
                            .collect(),
                    )
                }
            }
        }
    }

    #[must_use]
    pub fn is_truthy(&self) -> bool {
        match self {
            Self::Symbol(i) => i != "false" && i != "nil",
            _ => true,
        }
    }

    #[must_use]
    pub fn nil() -> Self {
        Self::symbol("nil")
    }

    #[must_use]
    pub fn symbol(sym: &str) -> Self {
        Self::Symbol(sym.to_string())
    }

    pub fn function(func: Rc<DynFn>) -> Self {
        Self::Function {
            fn_ref: func,
            is_macro: false,
        }
    }
}

impl Display for Value {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Int(arg0) => write!(f, "{arg0}"),
            Self::String(arg0) | Self::Symbol(arg0) => write!(f, "{arg0}"),
            Self::List(arg0) => {
                write!(f, "(")?;
                for (i, v) in arg0.iter().enumerate() {
                    if i == 0 {
                        write!(f, "{v}")?;
                    } else {
                        write!(f, " {v}")?;
                    }
                }
                write!(f, ")")
            }
            Self::Table(t) => {
                write!(f, "{{")?;
                for (i, (k, v)) in t.iter().enumerate() {
                    if i == 0 {
                        write!(f, "{k} {v}")?;
                    } else {
                        write!(f, " {k} {v}")?;
                    }
                }
                write!(f, "}}")
            }
            Self::Function { .. } | Self::Lambda { .. } => write!(f, "#<function>"),
        }
    }
}

impl Debug for Value {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Int(arg0) => write!(f, "{arg0}"),
            Self::String(arg0) => write!(f, "{arg0:?}"),
            Self::Symbol(arg0) => write!(f, "{arg0}"),
            Self::List(arg0) => {
                write!(f, "(")?;
                for (i, v) in arg0.iter().enumerate() {
                    if i == 0 {
                        write!(f, "{v:?}")?;
                    } else {
                        write!(f, " {v:?}")?;
                    }
                }
                write!(f, ")")
            }
            Self::Table(t) => {
                write!(f, "{{")?;
                for (i, (k, v)) in t.iter().enumerate() {
                    if i == 0 {
                        write!(f, "{k:?}: {v:?}")?;
                    } else {
                        write!(f, ", {k:?}: {v:?}")?;
                    }
                }
                write!(f, "}}")
            }
            Self::Function { .. } | Self::Lambda { .. } => write!(f, "#<function>"),
        }
    }
}

impl PartialEq for Value {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (Self::Int(a), Self::Int(b)) => *a == *b,
            (Self::String(a), Self::String(b)) | (Self::Symbol(a), Self::Symbol(b)) => a == b,
            (Self::List(a), Self::List(b)) => a == b,
            (
                Self::Function {
                    fn_ref: a,
                    is_macro: a_m,
                },
                Self::Function {
                    fn_ref: b,
                    is_macro: b_m,
                },
            ) => a_m == b_m && core::ptr::eq(a.as_ref(), b.as_ref()),
            _ => false,
        }
    }
}

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
            other => return other,
        }
    }
}
