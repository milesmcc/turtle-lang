use std::sync::{Arc, RwLock};

extern crate ansi_term;
extern crate pest;
extern crate rustyline;
extern crate rustyline_derive;
#[macro_use]
extern crate pest_derive;
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
pub use interpreter::values::{Keyword, Operator, Symbol, Value};
pub use parser::parse;

fn main() {
    let env = Arc::new(RwLock::new(Environment::root()));
    match parse("(include \"@prelude\")", "<builtin>") {
        Ok(expressions) => {
            for mut expression in expressions {
                let snapshot = CallSnapshot::root(&expression);
                match expression.eval(snapshot, env.clone()) {
                    Ok(_) => {}
                    Err(err) => eprintln!("{}", err),
                }
            }
        }
        Err(err) => eprintln!("{}", err),
    };
    repl::spawn(env);
}
