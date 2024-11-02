#![warn(clippy::pedantic, clippy::nursery)]
use std::{fs, path::PathBuf};

use clap::Parser;
use eval::Value;

pub mod env;
pub mod eval;
pub mod line_count;
pub mod parser;

#[derive(Parser)]
struct Args {
    /// The source file to run
    src: PathBuf,
    /// The arguments to pass to the program
    args: Vec<String>,
}

fn main() {
    let args = Args::parse();
    let src = fs::read_to_string(&args.src).unwrap();
    let code = parser::parse(&src).unwrap();
    // println!("{code:?}");
    let args = args.args.into_iter().map(Value::String).collect();
    let result = eval::eval(Value::List(code), env::default_env(args));
    println!("{result:?}");
}
