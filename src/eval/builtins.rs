use std::{cell::RefCell, collections::VecDeque, rc::Rc};

use crate::parser::Syntax;

use super::{eval, Value};

thread_local! {
    pub static ADD: RefCell<Rc<dyn Fn(Vec<Value>) -> Value>> = RefCell::new(Rc::new(add));
    pub static SUB: RefCell<Rc<dyn Fn(Vec<Value>) -> Value>> = RefCell::new(Rc::new(sub));
    pub static MUL: RefCell<Rc<dyn Fn(Vec<Value>) -> Value>> = RefCell::new(Rc::new(mul));
    pub static QUOTE: RefCell<Rc<dyn Fn(Vec<Value>) -> Value>> = RefCell::new(Rc::new(quote));
    pub static LIST: RefCell<Rc<dyn Fn(Vec<Value>) -> Value>> = RefCell::new(Rc::new(Value::Array));
    pub static LAMBDA: RefCell<Rc<dyn Fn(Vec<Syntax>) -> Syntax>> = RefCell::new(Rc::new(lambda));
}

pub fn add(args: Vec<Value>) -> Value {
    let mut sum = 0;
    for arg in args {
        let Value::Int(i) = arg else {
            return Value::Error(format!("{arg:?} is not a number"));
        };
        sum += i;
    }
    Value::Int(sum)
}

pub fn sub(args: Vec<Value>) -> Value {
    let mut sum = 0;
    for arg in args {
        let Value::Int(i) = arg else {
            return Value::Error(format!("{arg:?} is not a number"));
        };
        // I promise this makes a little bit of sense
        sum = -i - sum;
    }
    Value::Int(sum)
}

pub fn mul(args: Vec<Value>) -> Value {
    let mut sum = 1;
    for arg in args {
        let Value::Int(i) = arg else {
            return Value::Error(format!("{arg:?} is not a number"));
        };
        // I promise this makes a little bit of sense
        sum *= i;
    }
    Value::Int(sum)
}

pub fn quote(args: Vec<Value>) -> Value {
    let mut str = String::new();
    let mut args = VecDeque::from(args);
    while let Some(arg) = args.pop_front() {
        match arg {
            Value::Int(i) => match char::try_from(i as u32) {
                Ok(c) => str.push(c),
                Err(err) => return Value::Error(err.to_string()),
            },
            Value::Error(e) => return Value::Error(e),
            Value::String(s) => str.push_str(&s),
            Value::Array(values) => args.extend(values),
            Value::Function(_) => todo!(),
            Value::Macro(_) => todo!(),
            Value::Quote(_) => todo!(),
        }
    }
    Value::String(str)
}

pub fn lambda(args: Vec<Syntax>) -> Syntax {
    let [param, body] = &args[..] else {
        return Syntax::Literal(Value::Error(format!(
            "Function `\\` expected 2 quotes arguments; got {}",
            args.len()
        )));
    };
    let param_ids = match param {
        Syntax::Identifier(id) => {
            vec![id.clone()]
        }
        Syntax::Array(arr) => {
            let mut ids = Vec::new();
            for id in arr {
                let Syntax::Identifier(id) = id else {
                    return Syntax::Literal(Value::Error(
                        "First argument of '\\' must be a quoted literal or list of literals"
                            .to_string(),
                    ));
                };
                ids.push(id.clone());
            }
            ids
        }
        _ => {
            return Syntax::Literal(Value::Error(
                "First argument of '\\' must be a quoted literal or list of literals".to_string(),
            ))
        }
    };
    let body = body.clone();
    Syntax::Literal(Value::Function(Rc::new(move |args| {
        if args.len() < param_ids.len() {
            return Value::Error(format!(
                "Lambda expression requires {} arguments; got {}",
                param_ids.len(),
                args.len()
            ));
        }
        let mut body = body.clone();
        for (param, value) in param_ids.iter().zip(args) {
            body.replace(param, &Syntax::Literal(value));
        }
        eval(&body)
    })))
}
