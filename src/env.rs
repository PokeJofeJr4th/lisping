#![allow(clippy::module_name_repetitions)]
use std::{cell::RefCell, collections::HashMap, rc::Rc};

use crate::{
    eval::{self, builtins},
    parser::parse,
    types::Value,
};

pub struct EnvData {
    parent: Option<Env>,
    data: HashMap<String, Value>,
}

impl EnvData {
    #[must_use]
    pub fn get(&self, k: &str) -> Option<Value> {
        if let Some(v) = self.data.get(k) {
            Some(v.clone())
        } else {
            self.parent.as_ref()?.borrow_mut().get(k)
        }
    }

    pub fn set(&mut self, k: &str, v: Value) {
        self.data.insert(k.to_string(), v);
    }
}

pub type Env = Rc<RefCell<EnvData>>;

pub fn new_env(parent: Env) -> Env {
    Rc::new(RefCell::new(EnvData {
        parent: Some(parent),
        data: HashMap::new(),
    }))
}

#[must_use]
/// # Panics
pub fn default_env(args: Rc<[Value]>) -> Env {
    let mut data = HashMap::new();

    data.insert("+".to_string(), Value::function(Rc::new(builtins::add)));
    data.insert("-".to_string(), Value::function(Rc::new(builtins::sub)));
    data.insert("*".to_string(), Value::function(Rc::new(builtins::mul)));
    data.insert("/".to_string(), Value::function(Rc::new(builtins::div)));
    data.insert("=".to_string(), Value::function(Rc::new(builtins::eq)));
    data.insert(
        "list".to_string(),
        Value::function(Rc::new(|v, _| Value::List(v.into()))),
    );
    data.insert("not".to_string(), Value::function(Rc::new(builtins::not)));
    data.insert(
        "print".to_string(),
        Value::function(Rc::new(builtins::print)),
    );
    data.insert(
        "input".to_string(),
        Value::function(Rc::new(builtins::input)),
    );
    data.insert(
        "findall".to_string(),
        Value::function(Rc::new(builtins::findall)),
    );
    data.insert("eval".to_string(), Value::function(Rc::new(builtins::eval)));
    data.insert("str".to_string(), Value::function(Rc::new(builtins::str)));
    data.insert("chr".to_string(), Value::function(Rc::new(builtins::chr)));
    data.insert("map".to_string(), Value::function(Rc::new(builtins::map)));
    data.insert("nth".to_string(), Value::function(Rc::new(builtins::nth)));
    data.insert(
        "assoc".to_string(),
        Value::function(Rc::new(builtins::assoc)),
    );
    data.insert("nth".to_string(), Value::function(Rc::new(builtins::nth)));
    data.insert(
        "first".to_string(),
        Value::function(Rc::new(builtins::first)),
    );
    data.insert("rest".to_string(), Value::function(Rc::new(builtins::rest)));
    data.insert("cons".to_string(), Value::function(Rc::new(builtins::cons)));
    data.insert("get".to_string(), Value::function(Rc::new(builtins::get)));
    data.insert("keys".to_string(), Value::function(Rc::new(builtins::keys)));
    data.insert(
        "values".to_string(),
        Value::function(Rc::new(builtins::values)),
    );
    data.insert(
        "contains?".to_string(),
        Value::function(Rc::new(builtins::contains)),
    );
    data.insert("type".to_string(), Value::function(Rc::new(builtins::typ)));
    data.insert(
        "err?".to_string(),
        Value::function(builtins::type_is("err")),
    );
    data.insert(
        "function?".to_string(),
        Value::function(builtins::type_is("function")),
    );
    data.insert(
        "macro?".to_string(),
        Value::function(builtins::type_is("macro")),
    );
    data.insert(
        "list?".to_string(),
        Value::function(builtins::type_is("list")),
    );
    data.insert(
        "table?".to_string(),
        Value::function(builtins::type_is("table")),
    );
    data.insert(
        "symbol?".to_string(),
        Value::function(builtins::type_is("symbol")),
    );
    data.insert(
        "int?".to_string(),
        Value::function(builtins::type_is("int")),
    );
    data.insert(
        "macro".to_string(),
        Value::function(Rc::new(builtins::as_macro)),
    );
    data.insert("*ARGS*".to_string(), Value::List(args));
    let env = Rc::new(RefCell::new(EnvData { parent: None, data }));
    eval::eval(parse(include_str!("../stdlib.lisp")).unwrap(), env.clone());
    env
}
