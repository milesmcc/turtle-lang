use std::io::Read;
use std::sync::{Arc, RwLock};

extern crate pest;
#[macro_use]
extern crate pest_derive;

pub mod expression;
pub mod parser;

fn main() {
    let mut stdin: Vec<u8> = Vec::new();
    match std::io::stdin().read_to_end(&mut stdin) {
        Ok(_) => {}
        Err(issue) => {
            eprintln!("An error occured while trying to read the input:");
            eprintln!("{}", issue);
        }
    };
    let input_string = String::from_utf8_lossy(stdin.as_slice());
    println!("Running: {}", input_string);
    let env = expression::Environment::root();
    match parser::parse(input_string.as_ref(), Arc::new(RwLock::new(env))) {
        Ok(mut values) => {
            println!("parsed successfully");
            for value in &values {
                println!("turtle> {}", value);
            }
            for value in &mut values {
                println!("{} -> {}", value.clone(), value.eval());
            }
        },
        Err(err) => eprintln!("{}", err),
    }
}
