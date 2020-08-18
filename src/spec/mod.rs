use crate::{parse, CallSnapshot, Environment, Exception, Expression};
use std::sync::{Arc, RwLock};

fn exec(code: &str) -> Result<Expression, Exception> {
    let root = Arc::new(RwLock::new(Environment::root()));
    let expressions = parse(code, "<test module>")?;
    let mut ret = Expression::nil();
    for expression in expressions {
        let snapshot = CallSnapshot::root(&expression);
        ret = expression
            .eval_async(snapshot, root.clone())?
            .recv()
            .unwrap()?;
    }
    Ok(ret)
}

pub fn check(code: &str) -> Result<Expression, Exception> {
    match exec(code) {
        Ok(value) => {
            println!("{}", value);
            Ok(value)
        }
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

    #[test]
    fn map() {
        assert!(check(include_str!("map.lisp")).is_ok());
    }

    #[test]
    fn euler_1() {
        assert!(check(include_str!("euler_1.lisp")).is_ok());
    }
}
