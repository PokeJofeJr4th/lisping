use std::{
    collections::HashMap,
    fmt::{Debug, Display},
    hash::Hash,
    rc::Rc,
};

use crate::{env::Env, eval::eval};

pub type DynFn = dyn Fn(Vec<Value>, Env) -> Result<Value, Value>;

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
    ///
    /// Evaluates each key and value
    Table(Rc<HashMap<Value, Value>>),
    /// A builtin function
    ///
    /// Evaluates to itself
    Function { fn_ref: Rc<DynFn>, is_macro: bool },
    /// A function that captures variables from its environment
    ///
    /// Evaluates to itself
    Lambda {
        args: Box<Value>,
        body: Box<Value>,
        captures: Env,
        is_macro: bool,
    },
}

impl Value {
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
        Self::List(args.into())
    }

    #[allow(clippy::missing_errors_doc)]
    pub fn quasiquote(&self, env: Env) -> Result<Self, Self> {
        match self {
            other @ (Self::Int(_)
            | Self::String(_)
            | Self::Symbol(_)
            | Self::Function { .. }
            | Self::Table(_)
            | Self::Lambda { .. }) => Ok(other.clone()),
            Self::List(vec) => {
                if vec.first().is_some_and(|val| val.is_symbol("unquote")) {
                    eval(vec[1].clone(), env)
                } else {
                    Ok(Self::List(
                        vec.iter()
                            .map(|v| Self::quasiquote(v, env.clone()))
                            .collect::<Result<_, _>>()?,
                    ))
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

    #[must_use]
    pub fn as_list(&self) -> Option<&[Self]> {
        match self {
            Self::List(l) => Some(l),
            _ => None,
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
                        write!(f, "{k:?} {v:?}")?;
                    } else {
                        write!(f, " {k:?} {v:?}")?;
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
            ) => *a_m == *b_m && core::ptr::eq(a.as_ref(), b.as_ref()),
            (
                Self::Lambda {
                    args: a_a,
                    body: a_b,
                    captures: _,
                    is_macro: a_c,
                },
                Self::Lambda {
                    args: b_a,
                    body: b_b,
                    captures: _,
                    is_macro: b_c,
                },
            ) => **a_a == **b_a && **a_b == **b_b && *a_c == *b_c,
            (Self::Table(a), Self::Table(b)) => a == b,
            _ => false,
        }
    }
}

impl Eq for Value {}

impl Hash for Value {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        core::mem::discriminant(self).hash(state);
        match &self {
            Self::Function { fn_ref, is_macro } => {
                core::ptr::from_ref(&**fn_ref).hash(state);
                is_macro.hash(state);
            }
            Self::Int(i) => {
                i.hash(state);
            }
            Self::Lambda {
                args,
                body,
                captures: _,
                is_macro,
            } => {
                args.hash(state);
                body.hash(state);
                is_macro.hash(state);
            }
            Self::List(l) => {
                l.hash(state);
            }
            Self::String(s) | Self::Symbol(s) => {
                s.hash(state);
            }
            Self::Table(t) => {
                for (k, v) in &**t {
                    k.hash(state);
                    v.hash(state);
                }
            }
        }
    }
}
