use std::{cell::RefCell, collections::VecDeque, rc::Rc};

use super::Value;

thread_local! {
    pub static ADD: RefCell<Rc<dyn Fn(Vec<Value>) -> Value>> = RefCell::new(Rc::new(add));
    pub static SUB: RefCell<Rc<dyn Fn(Vec<Value>) -> Value>> = RefCell::new(Rc::new(sub));
    pub static MUL: RefCell<Rc<dyn Fn(Vec<Value>) -> Value>> = RefCell::new(Rc::new(mul));
    pub static QUOTE: RefCell<Rc<dyn Fn(Vec<Value>) -> Value>> = RefCell::new(Rc::new(quote));
    pub static LIST: RefCell<Rc<dyn Fn(Vec<Value>) -> Value>> = RefCell::new(Rc::new(Value::Array));
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
            Value::Identifier(_) => todo!(),
        }
    }
    Value::String(str)
}