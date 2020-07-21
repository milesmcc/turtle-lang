use rustyline::error::ReadlineError;
use rustyline::validate::{ValidationContext, ValidationResult, Validator};
use rustyline::Editor;
use rustyline_derive::{Completer, Helper, Highlighter, Hinter};
use std::sync::{Arc, RwLock};

use crate::{parse, CallSnapshot, Environment};

#[derive(Completer, Helper, Highlighter, Hinter)]
struct ReplHelper {}

impl Validator for ReplHelper {
    fn validate(&self, ctx: &mut ValidationContext) -> Result<ValidationResult, ReadlineError> {
        use ValidationResult::{Invalid, Valid};
        let input = ctx.input();
        let mut depth = 0;
        for character in input.chars() {
            if character == '(' {
                depth += 1;
            }
            if character == ')' {
                depth -= 1;
            }
        }
        Ok(match depth {
            0 => Valid(None),
            n => Invalid(Some(format!("{} deep", n))),
        })
    }
}

pub fn spawn(env: Arc<RwLock<Environment>>) {
    let mut rl = Editor::<ReplHelper>::new();
    if rl.load_history(".turtle_history.txt").is_err() {
        println!(
            "Welcome to Turtle v{}, © 2020 R. Miles McCain (distributed under the MIT license)",
            env!("CARGO_PKG_VERSION")
        );
        println!("It looks like this is your first time running Turtle from this directory; no history was loaded.")
    }

    let helper = ReplHelper {};
    rl.set_helper(Some(helper));

    loop {
        match rl.readline("🐢 > ") {
            Ok(line) => {
                rl.add_history_entry(line.as_str());
                match parse(line.as_str(), "<stdin>", env.clone()) {
                    Ok(mut values) => {
                        for value in &mut values {
                            let snapshot = CallSnapshot::root(&value.clone());
                            match value.eval(snapshot) {
                                Ok(result) => println!("   = {:#}", result),
                                Err(error) => eprintln!("{}", error),
                            }
                        }
                    }
                    Err(err) => eprintln!("{:#}", err),
                }
            }
            Err(ReadlineError::Interrupted) => break,
            Err(ReadlineError::Eof) => break,
            Err(err) => {
                eprintln!("Error: {:?}", err);
                break;
            }
        }
    }
    rl.save_history(".turtle_history.txt").unwrap();
}
