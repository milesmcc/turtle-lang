use crate::{Expression, Exception, parse, Environment, CallSnapshot};
use std::sync::{Arc, RwLock};

fn exec<'a>(code: &str) -> Result<Expression<'a>, Exception<'a>> {
    let root = Arc::new(RwLock::new(Environment::root()));
    let expressions = parse(code, "<test module>", root)?;
    let mut ret = Expression::nil();
    for mut expression in expressions {
        ret = expression.eval(CallSnapshot::root(&expression))?
    }
    Ok(ret)
}

pub fn check<'a>(code: &str) -> Result<Expression<'a>, Exception<'a>> {
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
}
