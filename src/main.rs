use std::io::Read;

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
    match parser::parse(input_string.as_ref()) {
        Ok(value) => {
            println!("parsed successfully: {:?}", value);
        },
        Err(err) => eprintln!("{}", err),
    }
}
