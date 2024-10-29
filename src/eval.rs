use std::{fmt::Debug, rc::Rc};

mod builtins;

#[derive(Clone)]
pub enum Value {
    Int(i32),
    Error(String),
    String(String),
    Identifier(String),
    Array(Vec<Value>),
    Function(Rc<dyn Fn(Vec<Value>) -> Value>),
}

impl Value {
    pub fn replace(&mut self, id: &[impl AsRef<str>], value: &[Self]) {
        match self {
            Self::Identifier(mid) => {
                for (id, value) in id.iter().zip(value) {
                    if mid == id.as_ref() {
                        *self = value.clone();
                        return;
                    }
                }
            }
            Self::Array(arr) => {
                for s in arr {
                    s.replace(id, value);
                }
            }
            _ => {}
        }
    }

    pub fn as_function(&self, func: &str) -> Option<&[Self]> {
        let Self::Array(arr) = self else {
            return None;
        };
        let [Self::Identifier(id), args @ ..] = &arr[..] else {
            return None;
        };
        if id != func {
            return None;
        }
        Some(args)
    }
}

impl Debug for Value {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Int(arg0) => write!(f, "{arg0}"),
            Self::Error(arg0) => f.debug_tuple("Error").field(arg0).finish(),
            Self::String(arg0) => write!(f, "{arg0:?}"),
            Self::Identifier(arg0) => write!(f, "`{arg0}`"),
            Self::Array(arg0) => f.debug_list().entries(arg0).finish(),
            Self::Function(_) => f.debug_struct("Function").finish_non_exhaustive(),
        }
    }
}

// syntax => value
pub fn eval(syn: &Value) -> Value {
    match syn {
        Value::Array(arr) => {
            if arr.is_empty() {
                return Value::Array(Vec::new());
            }
            if let Some(args) = arr[0].as_function("\\") {
                let [param, body] = args else {
                    return Value::Error(format!(
                        "Function `\\` expected 2 quotes arguments; got {}",
                        arr.len() - 1
                    ));
                };
                let param_ids = match param {
                    Value::Identifier(id) => {
                        vec![id.clone()]
                    }
                    Value::Array(arr) => {
                        let mut ids = Vec::new();
                        for id in arr {
                            let Value::Identifier(id) = id else {
                                return Value::Error(
                                    "First argument of '\\' must be an identifier or list of identifiers"
                                        .to_string(),
                                );
                            };
                            ids.push(id.clone());
                        }
                        ids
                    }
                    _ => {
                        return Value::Error(
                            "First argument of '\\' must be an identifier or list of identifiers"
                                .to_string(),
                        )
                    }
                };
                if arr.len() - 1 < param_ids.len() {
                    return Value::Error(format!(
                        "Lambda expression requires {} arguments; got {}",
                        param_ids.len(),
                        arr.len() - 1
                    ));
                }
                let mut body = body.clone();
                body.replace(&param_ids, &arr[1..]);
                eval(&body)
            } else {
                match eval(&arr[0]) {
                    Value::Function(func) => func(arr.iter().skip(1).map(eval).collect()),
                    other => Value::Error(format!("Invalid function {other:?}")),
                }
            }
        }
        Value::Identifier(id) => match &**id {
            "+" => Value::Function(builtins::ADD.with(|c| c.borrow().clone())),
            "-" => Value::Function(builtins::SUB.with(|c| c.borrow().clone())),
            "*" => Value::Function(builtins::MUL.with(|c| c.borrow().clone())),
            "\"" => Value::Function(builtins::QUOTE.with(|c| c.borrow().clone())),
            "list" => Value::Function(builtins::LIST.with(|c| c.borrow().clone())),
            o => Value::Error(format!("Unresolved identifier `{o}`")),
        },
        other => other.clone(),
    }
}