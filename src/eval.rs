use std::{
    fmt::{Debug, Display},
    rc::Rc,
};

use crate::env::Env;

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
    pub fn is_identifier(&self, id: &str) -> bool {
        let Self::Symbol(my_id) = self else {
            return false;
        };
        my_id == id
    }

    #[must_use]
    pub fn error(mut msg: Vec<Self>) -> Self {
        msg.insert(0, Self::Symbol("err".to_string()));
        Self::List(msg)
    }

    #[must_use]
    pub fn quasiquote(&self, env: Env) -> Self {
        match self {
            other @ (Self::Int(_) | Self::String(_) | Self::Symbol(_) | Self::Function(_)) => {
                other.clone()
            }
            Self::List(vec) => {
                if vec.first().is_some_and(|val| val.is_identifier("unquote")) {
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
            Self::Int(i) => *i > 0,
            Self::String(s) => !s.is_empty(),
            Self::Symbol(i) => i == "true",
            Self::List(vec) => !vec.is_empty(),
            Self::Function(_) => true,
        }
    }
}

impl Display for Value {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Int(arg0) => write!(f, "{arg0}"),
            Self::String(arg0) => write!(f, "{arg0:?}"),
            Self::Symbol(arg0) => write!(f, "{arg0}"),
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
            Self::Function(_) => f.debug_struct("Function").finish_non_exhaustive(),
        }
    }
}

impl Debug for Value {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{self}")
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
pub fn eval(mut syn: Value, env: Env) -> Value {
    loop {
        match syn {
            Value::List(ref arr) => {
                if arr.is_empty() {
                    return Value::List(Vec::new());
                }
                if let Some(args) = arr[0].as_function("\\") {
                    let [param, body] = args else {
                        return Value::error(vec![
                            Value::Symbol("InvalidLambdaError".to_string()),
                            arr[0].clone(),
                        ]);
                    };
                    let param_ids = match param {
                        Value::Symbol(id) => {
                            vec![id.clone()]
                        }
                        Value::List(param_ids) => {
                            let mut ids = Vec::new();
                            for id in param_ids {
                                let Value::Symbol(id) = id else {
                                    return Value::error(vec![
                                        Value::Symbol("InvalidLambdaError".to_string()),
                                        arr[0].clone(),
                                    ]);
                                };
                                ids.push(id.clone());
                            }
                            ids
                        }
                        _ => {
                            return Value::error(vec![
                                Value::Symbol("InvalidLambdaError".to_string()),
                                arr[0].clone(),
                            ])
                        }
                    };
                    if arr.len() - 1 < param_ids.len() {
                        return Value::error(vec![
                            Value::Symbol("InvalidLambdaError".to_string()),
                            arr[0].clone(),
                        ]);
                    }
                    let mut body = body.clone();
                    todo!("Create a new env and insert all the parameters");
                    // body.replace_quoted(&param_ids, &arr[1..]);
                    syn = body;
                } else if arr[0].is_identifier("if") {
                    match &arr[1..] {
                        [cond, t] => {
                            if eval(cond.clone(), env.clone()).is_truthy() {
                                syn = t.clone();
                            } else {
                                return Value::Symbol("nil".to_string());
                            }
                        }
                        [cond, t, f] => {
                            if eval(cond.clone(), env.clone()).is_truthy() {
                                syn = t.clone();
                            } else {
                                syn = f.clone();
                            }
                        }
                        _ => {
                            return Value::error(vec![
                                Value::Symbol("InvalidArgs".to_string()),
                                syn.clone(),
                            ])
                        }
                    }
                } else if arr[0].is_identifier("quote") {
                    return arr[1].clone();
                } else if arr[0].is_identifier("quasiquote") {
                    return arr[1].quasiquote(env);
                } else if arr[0].is_identifier("\\") {
                    // if it is a function, return itself
                    return Value::List(arr.clone());
                } else if arr[0].is_identifier("err") {
                    // if it is an error, evaluate everything but the first and return itself
                    return Value::List(
                        // start with the first value unchanged
                        arr.get(1)
                            .into_iter()
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
                        other => {
                            return Value::error(vec![
                                Value::Symbol("NotAFunction".to_string()),
                                other,
                            ])
                        }
                    }
                }
            }
            ref value @ Value::Symbol(ref id) if id == "true" || id == "false" || id == "nil" => {
                return value.clone()
            }
            Value::Symbol(ref id) => match env.borrow().get(id) {
                Some(x) => return x,
                None => {
                    return Value::error(vec![
                        Value::Symbol("UnresolvedIdentifier".to_string()),
                        syn.clone(),
                    ])
                }
            },
            other => return other,
        }
    }
}
