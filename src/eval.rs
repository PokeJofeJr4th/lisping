use std::{
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
    List(Vec<Value>),
    /// A builtin function
    ///
    /// Evaluates to itself?
    Function(Rc<DynFn>),
    Lambda {
        args: Vec<String>,
        body: Box<Value>,
        captures: Env,
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
        Self::List(args)
    }

    #[must_use]
    pub fn quasiquote(&self, env: Env) -> Self {
        match self {
            other @ (Self::Int(_)
            | Self::String(_)
            | Self::Symbol(_)
            | Self::Function(_)
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
            Self::Function(_) | Self::Lambda { .. } => write!(f, "#<function>"),
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
            Self::Function(_) | Self::Lambda { .. } => write!(f, "#<function>"),
        }
    }
}

impl PartialEq for Value {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (Self::Int(a), Self::Int(b)) => *a == *b,
            (Self::String(a), Self::String(b)) | (Self::Symbol(a), Self::Symbol(b)) => a == b,
            (Self::List(a), Self::List(b)) => a == b,
            (Self::Function(a), Self::Function(b)) => core::ptr::eq(a.as_ref(), b.as_ref()),
            _ => false,
        }
    }
}

/// syntax => value
#[allow(clippy::too_many_lines)]
pub fn eval(mut syn: Value, mut env: Env) -> Value {
    loop {
        match syn {
            Value::List(ref arr) => {
                if arr.is_empty() {
                    return Value::List(Vec::new());
                }
                if arr[0].is_symbol("\\") {
                    let [param, body] = &arr[1..] else {
                        return Value::error("InvalidLambdaError", vec![arr[0].clone()]);
                    };
                    let Value::List(ids) = param else {
                        return Value::error("InvalidLambdaError", vec![arr[0].clone()]);
                    };
                    let mut param_ids = Vec::new();
                    for id in ids {
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
                } else {
                    match eval(arr[0].clone(), env.clone()) {
                        Value::Function(func) => {
                            return func(
                                arr.iter()
                                    .skip(1)
                                    .cloned()
                                    .map(|v| eval(v, env.clone()))
                                    .collect(),
                                env,
                            )
                        }
                        Value::Lambda {
                            args: params,
                            body,
                            captures,
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
                        other => return Value::error("NotAFunction", vec![other]),
                    }
                }
            }
            ref value @ Value::Symbol(ref id) if id == "true" || id == "false" || id == "nil" => {
                return value.clone()
            }
            Value::Symbol(ref id) => match env.borrow().get(id) {
                Some(x) => return x,
                None => return Value::error("UnresolvedIdentifier", vec![syn.clone()]),
            },
            other => return other,
        }
    }
}
