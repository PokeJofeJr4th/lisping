use std::{fmt::Debug, rc::Rc};

use crate::parser::Syntax;

mod builtins;

#[derive(Clone)]
pub enum Value {
    Int(i32),
    Error(String),
    String(String),
    Array(Vec<Value>),
    Quote(Box<Syntax>),
    Function(Rc<dyn Fn(Vec<Value>) -> Value>),
    Macro(Rc<dyn Fn(Vec<Syntax>) -> Syntax>),
}

impl Debug for Value {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Int(arg0) => write!(f, "{arg0}"),
            Self::Error(arg0) => f.debug_tuple("Error").field(arg0).finish(),
            Self::String(arg0) => write!(f, "{arg0:?}"),
            Self::Array(arg0) => f.debug_list().entries(arg0).finish(),
            Self::Function(_) => f.debug_struct("Function").finish_non_exhaustive(),
            Self::Macro(_) => f.debug_struct("Macro").finish_non_exhaustive(),
            Self::Quote(q) => write!(f, "'{q:?}"),
        }
    }
}

// syntax => value
pub fn eval(syn: &Syntax) -> Value {
    match syn {
        Syntax::Literal(v) => v.clone(),
        Syntax::Array(arr) => {
            if arr.is_empty() {
                return Value::Array(Vec::new());
            }
            if arr.len() == 1 {
                // parentheses can group things
                return eval(&arr[0]);
            }
            match eval(&arr[0]) {
                Value::Function(func) => func(arr.iter().skip(1).map(eval).collect()),
                Value::Macro(mac) => eval(&mac(arr.iter().skip(1).cloned().collect())),
                other => Value::Error(format!("Invalid function {other:?}")),
            }
        }
        Syntax::Identifier(id) => match &**id {
            "+" => Value::Function(builtins::ADD.with(|c| c.borrow().clone())),
            "-" => Value::Function(builtins::SUB.with(|c| c.borrow().clone())),
            "*" => Value::Function(builtins::MUL.with(|c| c.borrow().clone())),
            "\"" => Value::Function(builtins::QUOTE.with(|c| c.borrow().clone())),
            "list" => Value::Function(builtins::LIST.with(|c| c.borrow().clone())),
            "\\" => Value::Macro(builtins::LAMBDA.with(|c| c.borrow().clone())),
            o => Value::Error(format!("Unresolved identifier `{o}`")),
        },
        Syntax::Quote(q) => Value::Quote(q.clone()),
    }
}
