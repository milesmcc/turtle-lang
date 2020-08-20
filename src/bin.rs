use clap::{App, Arg};
use std::fs;
use turtle::*;

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
    let env = Locker::new(Environment::root());

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
                    match val
                        .clone()
                        .eval_async(CallSnapshot::root(&val), env.clone())
                        .unwrap()
                        .recv()
                        .unwrap()
                    {
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
