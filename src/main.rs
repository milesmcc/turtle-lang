use std::sync::{Arc, RwLock};

extern crate ansi_term;
extern crate pest;
extern crate rustyline;
extern crate rustyline_derive;
#[macro_use]
extern crate pest_derive;
extern crate relative_path;

pub mod call_snapshot;
pub mod environment;
pub mod exceptions;
pub mod expression;
pub mod parser;
pub mod repl;
pub mod resolver;
pub mod source;
pub mod stdlib;

pub use call_snapshot::CallSnapshot;
pub use environment::Environment;
pub use exceptions::{Exception, ExceptionValue};
pub use expression::{Expression, Keyword, Operator, Symbol, Value};
pub use parser::parse;
pub use resolver::resolve_resource;
pub use source::{Source, SourcePosition};

fn main() {
    let env = Arc::new(RwLock::new(environment::Environment::root()));
    match parse("(include \"@prelude\")", "<builtin>", env.clone()) {
        Ok(expressions) => {
            for mut expression in expressions {
                let snapshot = CallSnapshot::root(&expression);
                match expression.eval(snapshot) {
                    Ok(_) => {}
                    Err(err) => eprintln!("{}", err),
                }
            }
        }
        Err(err) => eprintln!("{}", err),
    };
    repl::spawn(env);
}
