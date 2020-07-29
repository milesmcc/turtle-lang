use crate::{Expression, Exception, parse, Environment, CallSnapshot};
use std::sync::{Arc, RwLock};

fn exec(code: &str) -> Result<Expression, Exception> {
    let root = Arc::new(RwLock::new(Environment::root()));
    let expressions = parse(code, "<test module>")?;
    let mut ret = Expression::nil();
    for expression in expressions {
        ret = expression.eval(CallSnapshot::root(&expression), root.clone())?
    }
    Ok(ret)
}

pub fn check(code: &str) -> Result<Expression, Exception> {
    match exec(code) {
        Ok(value) => {
            println!("{}", value);
            Ok(value)
        },
        Err(value) => {
            eprintln!("{}", value);
            Err(value)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::check;

    #[test]
    fn smoke_test() {
        assert!(check(include_str!("smoke_test.lisp")).is_ok());
    }

    #[test]
    fn math() {
        assert!(check(include_str!("math.lisp")).is_ok());
    }
}
