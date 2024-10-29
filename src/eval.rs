use std::{
    fmt::{Debug, Display},
    rc::Rc,
};

mod builtins;

#[derive(Clone)]
pub enum Value {
    Int(i32),
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

    pub fn is_identifier(&self, id: &str) -> bool {
        let Self::Identifier(my_id) = self else {
            return false;
        };
        my_id == id
    }

    pub fn error(mut msg: Vec<Self>) -> Self {
        msg.insert(0, Self::Identifier("err".to_string()));
        Self::Array(msg)
    }

    pub fn quasiquote(&self) -> Self {
        match self {
            other @ (Self::Int(_) | Self::String(_) | Self::Identifier(_) | Self::Function(_)) => {
                other.clone()
            }
            Self::Array(vec) => {
                if vec.first().is_some_and(|val| val.is_identifier("unquote")) {
                    eval(&vec[1])
                } else {
                    Self::Array(vec.iter().map(Self::quasiquote).collect())
                }
            }
        }
    }
}

impl Display for Value {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Int(arg0) => write!(f, "{arg0}"),
            Self::String(arg0) => write!(f, "{arg0:?}"),
            Self::Identifier(arg0) => write!(f, "{arg0}"),
            Self::Array(arg0) => {
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

// syntax => value
pub fn eval(syn: &Value) -> Value {
    match syn {
        Value::Array(arr) => {
            if arr.is_empty() {
                return Value::Array(Vec::new());
            }
            if let Some(args) = arr[0].as_function("\\") {
                let [param, body] = args else {
                    return Value::error(vec![
                        Value::Identifier("InvalidLambdaError".to_string()),
                        arr[0].clone(),
                    ]);
                };
                let param_ids = match param {
                    Value::Identifier(id) => {
                        vec![id.clone()]
                    }
                    Value::Array(param_ids) => {
                        let mut ids = Vec::new();
                        for id in param_ids {
                            let Value::Identifier(id) = id else {
                                return Value::error(vec![
                                    Value::Identifier("InvalidLambdaError".to_string()),
                                    arr[0].clone(),
                                ]);
                            };
                            ids.push(id.clone());
                        }
                        ids
                    }
                    _ => {
                        return Value::error(vec![
                            Value::Identifier("InvalidLambdaError".to_string()),
                            arr[0].clone(),
                        ])
                    }
                };
                if arr.len() - 1 < param_ids.len() {
                    return Value::error(vec![
                        Value::Identifier("InvalidLambdaError".to_string()),
                        arr[0].clone(),
                    ]);
                }
                let mut body = body.clone();
                body.replace(&param_ids, &arr[1..]);
                eval(&body)
            } else if arr[0].is_identifier("quote") {
                arr[1].clone()
            } else if arr[0].is_identifier("quasiquote") {
                arr[1].quasiquote()
            } else if arr[0].is_identifier("\\") || arr[0].is_identifier("err") {
                // if it is a function, return the function
                Value::Array(arr.clone())
            } else {
                match eval(&arr[0]) {
                    Value::Function(func) => func(arr.iter().skip(1).map(eval).collect()),
                    other => {
                        Value::error(vec![Value::Identifier("NotAFunction".to_string()), other])
                    }
                }
            }
        }
        Value::Identifier(id) => match &**id {
            "+" => Value::Function(builtins::ADD.with(|c| c.borrow().clone())),
            "-" => Value::Function(builtins::SUB.with(|c| c.borrow().clone())),
            "*" => Value::Function(builtins::MUL.with(|c| c.borrow().clone())),
            "\"" => Value::Function(builtins::QUOTE.with(|c| c.borrow().clone())),
            "list" => Value::Function(builtins::LIST.with(|c| c.borrow().clone())),
            "eval" => Value::Function(builtins::EVAL.with(|c| c.borrow().clone())),
            _ => Value::error(vec![
                Value::Identifier("UnresolvedIdentifier".to_string()),
                syn.clone(),
            ]),
        },
        other => other.clone(),
    }
}
