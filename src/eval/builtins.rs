use std::{cell::RefCell, collections::VecDeque, rc::Rc};

use super::Value;

thread_local! {
    pub static ADD: RefCell<Rc<dyn Fn(Vec<Value>) -> Value>> = RefCell::new(Rc::new(add));
    pub static SUB: RefCell<Rc<dyn Fn(Vec<Value>) -> Value>> = RefCell::new(Rc::new(sub));
    pub static MUL: RefCell<Rc<dyn Fn(Vec<Value>) -> Value>> = RefCell::new(Rc::new(mul));
    pub static QUOTE: RefCell<Rc<dyn Fn(Vec<Value>) -> Value>> = RefCell::new(Rc::new(quote));
    pub static LIST: RefCell<Rc<dyn Fn(Vec<Value>) -> Value>> = RefCell::new(Rc::new(Value::Array));
    pub static EVAL: RefCell<Rc<dyn Fn(Vec<Value>) -> Value>> = RefCell::new(Rc::new(eval));
    pub static EQ: RefCell<Rc<dyn Fn(Vec<Value>) -> Value>> = RefCell::new(Rc::new(eq));
    pub static TYPE: RefCell<Rc<dyn Fn(Vec<Value>) -> Value>> = RefCell::new(Rc::new(typ));
}

pub fn add(args: Vec<Value>) -> Value {
    let mut sum = 0;
    for arg in args {
        let Value::Int(i) = arg else {
            return Value::error(vec![Value::Identifier("NotANumber".to_string()), arg]);
        };
        sum += i;
    }
    Value::Int(sum)
}

pub fn sub(args: Vec<Value>) -> Value {
    let mut difference = 0;
    for arg in args {
        let Value::Int(i) = arg else {
            return Value::error(vec![Value::Identifier("NotANumber".to_string()), arg]);
        };
        // I promise this makes a little bit of sense
        difference = -i - difference;
    }
    Value::Int(difference)
}

pub fn mul(args: Vec<Value>) -> Value {
    let mut product = 1;
    for arg in args {
        let Value::Int(i) = arg else {
            return Value::error(vec![Value::Identifier("NotANumber".to_string()), arg]);
        };
        product *= i;
    }
    Value::Int(product)
}

#[allow(clippy::needless_pass_by_value)]
pub fn eq(args: Vec<Value>) -> Value {
    if args.len() <= 1 || args.iter().skip(1).all(|x| x == &args[0]) {
        Value::Identifier("true".to_string())
    } else {
        Value::Identifier("false".to_string())
    }
}

pub fn typ(mut args: Vec<Value>) -> Value {
    if args.len() != 1 {
        args.insert(0, Value::Identifier("TooManyArgs".to_string()));
        return Value::error(args);
    }
    match &args[0] {
        Value::Int(_) => Value::Identifier("int".to_string()),
        Value::String(_) => Value::Identifier("string".to_string()),
        Value::Identifier(_) => Value::Identifier("identifier".to_string()),
        Value::Array(_) => Value::Identifier("array".to_string()),
        Value::Function(_) => Value::Identifier("function".to_string()),
    }
}

pub fn quote(args: Vec<Value>) -> Value {
    let mut str = String::new();
    let mut args = VecDeque::from(args);
    while let Some(arg) = args.pop_front() {
        match arg {
            #[allow(clippy::cast_sign_loss)]
            Value::Int(i) => match char::try_from(i as u32) {
                Ok(c) => str.push(c),
                Err(_) => {
                    return Value::error(vec![Value::Identifier("NotACharacter".to_string())])
                }
            },
            Value::String(s) => str.push_str(&s),
            Value::Array(values) => args.extend(values),
            Value::Function(_) => todo!(),
            Value::Identifier(_) => todo!(),
        }
    }
    Value::String(str)
}

#[allow(clippy::needless_pass_by_value)]
pub fn eval(args: Vec<Value>) -> Value {
    if args.len() == 1 {
        super::eval(&args[0])
    } else {
        Value::Array(args.iter().map(super::eval).collect())
    }
}
