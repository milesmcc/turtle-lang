use crate::{Expression, Symbol};

#[macro_export]
macro_rules! exp {
    ($value:expr) => {
        return Err(Exception::new($value, None, None));
    };
    ($value:expr, $expr:expr) => {
        return Err(Exception::new($value, Some($expr), None));
    };
    ($value:expr, $expr:expr, $note:expr) => {
        return Err(Exception::new($value, Some($expr), Some($note)));
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
    UndefinedSymbol(Symbol),
    ArgumentMismatch,
}

#[derive(Debug, Clone)]
pub struct Exception {
    expression: Option<Expression>,
    value: ExceptionValue,
    note: Option<String>,
}

impl Exception {
    pub fn new(value: ExceptionValue, expression: Option<Expression>, note: Option<String>) -> Self {
        Exception {
            expression,
            value,
            note
        }
    }
}