#![warn(clippy::pedantic, clippy::nursery)]
use std::{
    cell::RefCell,
    fs,
    io::{stdin, stdout, Write},
    path::PathBuf,
    rc::Rc,
};

use clap::Parser;

pub mod env;
pub mod eval;
pub mod line_count;
pub mod parser;
pub mod types;

use types::Value;

#[derive(Parser)]
struct Args {
    /// The source file to run. Omit to enter REPL mode
    src: Option<PathBuf>,
    /// The arguments to pass to the program
    args: Vec<String>,
}

fn main() {
    let args = Args::parse();
    if let Some(src) = args.src {
        let src = fs::read_to_string(&src).unwrap();
        let code = parser::parse(&src).unwrap();
        // println!("{code:?}");
        let args = args.args.into_iter().map(Value::String).collect();
        let result = eval::eval(code, env::default_env(args));
        println!("{result:?}");
    } else {
        let env = env::default_env(Rc::new([]));
        let mut code = Value::nil();
        let quit_flag = Rc::new(RefCell::new(false));
        let qf = quit_flag.clone();
        env.borrow_mut().set(
            "quit",
            Value::Function {
                fn_ref: Rc::new(move |_, _| {
                    qf.replace(true);
                    Value::nil()
                }),
                is_macro: false,
            },
        );
        while !*quit_flag.borrow() {
            print!("> ");
            stdout().flush().unwrap();
            let mut s = String::new();
            stdin().read_line(&mut s).unwrap();
            if !s.is_empty() {
                match parser::parse(&s) {
                    Ok(c) => {
                        code = c;
                    }
                    Err(e) => {
                        println!("Parse error: {e}");
                        continue;
                    }
                }
            };
            let result = eval::eval(code.clone(), env.clone());
            println!("{result:?}");
            env.borrow_mut().set("_", result);
        }
    }
}
