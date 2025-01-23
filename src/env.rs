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

macro_rules! builtins {
    ($($name:literal $($val:expr)? $(=>$doc:literal)?);*$(;)?) => {{
        let mut data = HashMap::new();
        let mut docs: HashMap<String,String> = HashMap::new();
        $(
            $(
                data.insert($name.to_string(), $crate::types::Value::function(Rc::from($val)));
            )?
            $(
                docs.insert($name.to_string(), $doc.to_string());
            )?
        )*
        (data, docs)
    }};
}

#[must_use]
#[allow(clippy::too_many_lines)]
/// # Panics
pub fn default_env(args: Rc<[Value]>) -> Env {
    let (mut data, docs) = builtins!(
        "+" builtins::add => "Calculate the sum of the arguments";
        "-" builtins::sub => "Calculate the difference between arguments. One argument calculates the negative.";
        "*" builtins::mul => "Calculate the product of the arguments.";
        "/" builtins::div => "Divide the two arguments, rounding down.";
        "%" builtins::div => "Calculate the remainder when dividing the two arguments.";
        "=" builtins::eq => "Check if two values are equal";
        "<" builtins::lt => "Check if one numerical value is less than antother";
        "<=" builtins::le => "Check if one numerical value is less than or equal to another";
        ">" builtins::gt => "Check if a numerical value is greater than another";
        ">=" builtins::ge => "Check if a numerical value is greater than or equal to another";
        "print" builtins::print => "Print the arguments to stdout";
        "input" builtins::input => "Read a line of input from stdin";
        "findall" builtins::findall => "Search a string for all non-overlapping matches to a regular expression";
        "eval" builtins::eval => "Evaluate a given expression as code";
        "apply" builtins::apply => "Apply the given function using the given arguments.";
        "str" builtins::str => "Concatenate the arguments into a string";
        "symbol" builtins::symbol => "Convert a string to a symbol";
        "int" builtins::int => "Convert a string to an int";
        "chr" builtins::chr => "Convert an integer to its corresponding character in UTF-8";
        "nth" builtins::nth => "Get the nth value of a sequence";
        "count" builtins::count => "Find the size of a sequence";
        "assoc" builtins::assoc => "Return a table with the additional keys and values combined with the original";
        "dissoc" builtins::dissoc => "Return a table without the specified keys";
        "first" builtins::first => "Get the first value of a sequence.";
        "last" builtins::last => "Get the last value of a sequence.";
        "rest" builtins::rest => "Get a copy of a list without its first value";
        "get" builtins::get => "Get the value associated with a given key in a table";
        "keys" builtins::keys => "Get a table's keys as a sequence";
        "values" builtins::values => "Get a table's values as a sequence";
        "contains?" builtins::contains => "Check if a table contains a key";
        "type" builtins::typ => "Get the type of a value, as a symbol";
        "err?" builtins::type_is("err") => "Check if the value is an error";
        "function?" builtins::type_is("function") => "Check if the value is a function";
        "atom?" builtins::type_is("atom") => "Check if the value is an atom";
        "macro?" builtins::type_is("macro") => "Check if the value is a macro";
        "list?" builtins::type_is("list") => "Check if the value is a list";
        "table?" builtins::type_is("table") => "Check if the value is a table";
        "nil?" builtins::type_is("nil") => "Check if the value is nil";
        "bool?" builtins::type_is("bool") => "Check if the value is a boolean";
        "symbol?" builtins::type_is("symbol") => "Check if the value is a symbol";
        "int?" builtins::type_is("int") => "Check if the value is an integer";
        "macro" builtins::as_macro => "Convert a function to a macro. The function should take syntax as an input and produce it as output.";
        "atom" builtins::atom => "Create a new atom with the given value inside it";
        "set!" builtins::set_atom => "Set the value inside an atom, returning the original value";
        "inspect!" builtins::inspect_atom => "Modify the value in the atom with a function";
        "\\" => "Create a lambda function that accepts the given parameters and returns the result of evaluating the body expression";
        "if" => "If the first value is truthy, evaluate and return the second value. Otherwise, evaluate and return the third value.";
        "quote" => "Return the arguments without evaluating them";
        "quasiquote" => "Return the arguments, only evaluating the parts within a (unquote ...) expression";
        "err" => "Throw an error with the provided information. The first argument is not evaluated and should be an identifier.";
        "let*" => "The first argument is an alternating list of patterns and values. Each value is evaluated in order and bound to its corresponding pattern. Returns the result of evaluating the second argument with the context of the bindings created from the first argument.";
        "def!" => "Define a variable, providing its name and a value";
        "try*" => "Attempt to evaluate the first argument. If an exception is thrown, goes to each catch block to recover";
        "doc" => "Attach documentation to the next value that is defined";
        "help" => "Retrieve the documentation for a function";
        "*ARGS*" => "Arguments provided in the command line"
    );
    data.insert("*ARGS*".to_string(), Value::List(args));
    let env = Rc::new(RefCell::new(EnvData { parent: None, data }));
    eval::eval(parse(include_str!("../stdlib.lisp")).unwrap(), env.clone()).unwrap();
    *eval::DOCS.write().unwrap() = docs;
    env
}
