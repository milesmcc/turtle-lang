use crate::Expression;

#[macro_export]
macro_rules! exception {
    ($value:expr) => {
        return Err(Exception {
            value: $value,
            expression: None
        })
    };
    ($value:expr, $expr:expr) => {
        return Err(Exception {
            value: $value,
            expression: Some($expr),
        })
    };
}

#[derive(Debug, Clone)]
pub enum ExceptionValue {
    Other(String),
    UndefinedSymbol(String),
}

#[derive(Debug, Clone)]
pub struct Exception<'a> {
    expression: Option<Expression<'a>>,
    value: ExceptionValue,
}