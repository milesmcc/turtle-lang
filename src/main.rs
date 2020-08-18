use std::fs;

use std::sync::{Arc, RwLock};

extern crate ansi_term;
extern crate pest;
extern crate rustyline;
extern crate rustyline_derive;
#[macro_use]
extern crate pest_derive;
extern crate clap;
extern crate regex;
extern crate relative_path;

pub mod interpreter;
pub mod parser;
pub mod repl;
pub mod spec;
pub mod stdlib;

pub use interpreter::call_snapshot::CallSnapshot;
pub use interpreter::environment::Environment;
pub use interpreter::exceptions::{Exception, ExceptionValue};
pub use interpreter::expression::Expression;
pub use interpreter::resolver::resolve_resource;
pub use interpreter::source::{Source, SourcePosition};
pub use interpreter::values::{Function, Keyword, Operator, Symbol, Value};
pub use parser::parse;

use clap::{App, Arg};

fn main() {
    let matches = App::new("turtle")
        .version(env!("CARGO_PKG_VERSION"))
        .author("R. Miles McCain <turtle@sendmiles.email>")
        .about("A friendly interpreted Lisp")
        .arg(
            Arg::with_name("INTERACTIVE")
                .short("i")
                .long("interactive")
                .help("Open a REPL after running")
                .takes_value(false),
        )
        .arg(
            Arg::with_name("NO_PRELUDE")
                .short("n")
                .long("no-prelude")
                .help("Run without the prelude")
                .takes_value(false),
        )
        .arg(
            Arg::with_name("FILE")
                .help("The file to run")
                .required(false)
                .index(1),
        )
        .get_matches();
    let env = Arc::new(RwLock::new(Environment::root()));

    if !matches.is_present("NO_PRELUDE") {
        match parse("(import \"@prelude\")", "<builtin>") {
            Ok(expressions) => {
                for expression in expressions {
                    let snapshot = CallSnapshot::root(&expression);
                    match expression.eval(snapshot, env.clone()) {
                        Ok(_) => {}
                        Err(err) => eprintln!("{}", err),
                    }
                }
            }
            Err(err) => eprintln!("{}", err),
        };
    }
    let file = matches.value_of("FILE");
    match file {
        Some(location) => match fs::read_to_string(location) {
            Ok(code) => {
                let exp_parsed = match parse(&code, location) {
                    Ok(val) => val,
                    Err(err) => {
                        eprintln!("{}", err);
                        std::process::exit(2);
                    }
                };
                for val in exp_parsed {
                    match val.clone().eval_async(CallSnapshot::root(&val), env.clone()).unwrap().recv().unwrap() {
                        Ok(_) => {}
                        Err(err) => {
                            eprintln!("{}", err);
                            std::process::exit(3);
                        }
                    }
                }
                if matches.is_present("INTERACTIVE") {
                    repl::spawn(env);
                }
            }
            Err(err) => {
                eprintln!("Unable to read file: {}", err);
                std::process::exit(1);
            }
        },
        None => repl::spawn(env),
    }
}
