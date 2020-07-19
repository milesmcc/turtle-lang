use crate::Expression;

#[macro_export]
macro_rules! exp {
    ($value:expr) => {
        return Err(Exception::new(None, $value));
    };
    ($value:expr, $expr:expr) => {
        return Err(Exception::new(Some($expr), $value));
    };
}

#[macro_export]
macro_rules! exp_opt {
    ($value:expr, $($rest:expr)*) => {
        match $value {
            Some(value) => value,
            None => exp!($($rest)*)
        }
    };
}

#[derive(Debug, Clone)]
pub enum ExceptionValue {
    Other(String),
    UndefinedSymbol(String),
}

#[derive(Debug, Clone)]
pub struct Exception {
    expression: Option<Expression>,
    value: ExceptionValue,
}

impl Exception {
    pub fn new(expression: Option<Expression>, value: ExceptionValue) -> Self {
        Exception {
            expression,
            value
        }
    }
}