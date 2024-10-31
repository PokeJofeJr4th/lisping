use std::{
    fmt::{Debug, Display},
    iter,
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
            Self::Int(i) => *i > 0,
            Self::String(s) => !s.is_empty(),
            Self::Symbol(i) => i == "true",
            Self::List(vec) => !vec.is_empty(),
            Self::Function(_) | Self::Lambda { .. } => true,
        }
    }

    #[must_use]
    pub fn nil() -> Self {
        Self::symbol("nil")
    }

    pub fn symbol(sym: &str) -> Self {
        Self::Symbol(sym.to_string())
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
            Self::Function(_) => write!(f, "#<function>"),
            Self::Lambda {
                args,
                body,
                captures: _,
            } => {
                write!(f, "(\\ (")?;
                for (i, arg) in args.iter().enumerate() {
                    if i == 0 {
                        write!(f, "{arg}")?;
                    } else {
                        write!(f, " {arg}")?;
                    }
                }
                write!(f, ") {body})")
            }
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
                if arr[0].is_symbol("\\") {
                    let [param, body] = &arr[1..] else {
                        return Value::error("InvalidLambdaError", vec![arr[0].clone()]);
                    };
                    let param_ids = match param {
                        Value::Symbol(id) => {
                            vec![id.clone()]
                        }
                        Value::List(param_ids) => {
                            let mut ids = Vec::new();
                            for id in param_ids {
                                let Value::Symbol(id) = id else {
                                    return Value::error(
                                        "InvalidLambdaError",
                                        vec![arr[0].clone()],
                                    );
                                };
                                ids.push(id.clone());
                            }
                            ids
                        }
                        _ => return Value::error("InvalidLambdaError", vec![arr[0].clone()]),
                    };
                    if arr.len() - 1 < param_ids.len() {
                        return Value::error("InvalidLambdaError", vec![arr[0].clone()]);
                    }
                    let body = body.clone();
                    let sub_env = new_env(env);
                    return Value::Function(Rc::new(move |values, _| {
                        let env = new_env(sub_env.clone());
                        if values.len() > param_ids.len() {
                            return Value::error("InvalidArgs", vec![Value::List(values)]);
                        };
                        let mut env_borrow = env.borrow_mut();
                        for (value, param) in values
                            .into_iter()
                            .chain(iter::from_fn(|| Some(Value::nil())))
                            .zip(&param_ids)
                        {
                            env_borrow.set(param, value);
                        }
                        drop(env_borrow);
                        eval(body.clone(), env)
                    }));
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
