#![warn(clippy::pedantic, clippy::nursery)]
use std::{fs, path::PathBuf};

use clap::Parser;

pub mod line_count;
mod parser;

#[derive(Parser)]
struct Args {
    /// The source file to run
    src: PathBuf,
}

fn main() {
    let args = Args::parse();
    let src = fs::read_to_string(&args.src).unwrap();
    let code = parser::parse(&src).unwrap();
    println!("{code:?}");
}
