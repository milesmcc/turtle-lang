use crate::{Expression};

use std::{fmt};

pub mod operator;
pub use operator::Operator;

pub mod symbol;
pub use symbol::Symbol;

pub mod keyword;
pub use keyword::Keyword;

pub mod function;
pub use function::Function;

#[derive(Debug, Clone, PartialEq, PartialOrd)]
pub enum Value {
    List(Vec<Expression>),
    Number(f64),
    Text(String),
    Keyword(Keyword),
    Symbol(Symbol),
    Byte(u8),
    True,

    // Primitive (axiomatic) operators
    Operator(Operator),

    Lambda(Function),
    Macro(Function),
}

impl Value {
    pub fn as_type(&self) -> Self {
        use Value::*;

        Value::Keyword(crate::Keyword::new(match self {
            List(_) => "list".to_string(),
            Number(_) => "number".to_string(),
            Text(_) => "text".to_string(),
            Keyword(_) => "keyword".to_string(),
            Symbol(_) => "symbol".to_string(),
            Operator(_) => "operator".to_string(),
            Byte(_) => "byte".to_string(),
            Lambda { .. } => "lambda".to_string(),
            Macro { .. } => "macro".to_string(),
            _ => "unknown".to_string(),
        }))
    }
}

impl<'a> fmt::Display for Value {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        use Value::*;

        match &self {
            List(vals) => match vals.len() {
                0 => write!(f, "nil"),
                _ => write!(
                    f,
                    "({})",
                    vals.iter()
                        .map(|v| format!("{}", v))
                        .collect::<Vec<String>>()
                        .join(" ")
                ),
            },
            Number(val) => write!(f, "{}", val),
            Text(val) => write!(f, "{}", val),
            Symbol(val) => write!(f, "{}", val),
            Keyword(val) => write!(f, "{}", val),
            Byte(val) => write!(f, "b{}", val),
            True => write!(f, "true"),
            Lambda(function) | Macro(function) => write!(
                f,
                "<{} {}{}{} -> {}>",
                match self {
                    Lambda(_) => "lambda",
                    Macro(_) => "macro",
                    _ => unreachable!(),
                },
                match function.collapse_input {
                    true => "",
                    false => "(",
                },
                function
                    .params
                    .iter()
                    .map(|x| format!("{}", x))
                    .collect::<Vec<String>>()
                    .join(" "),
                match function.collapse_input {
                    true => "",
                    false => ")",
                },
                function
                    .expressions
                    .iter()
                    .map(|x| format!("{}", x))
                    .collect::<Vec<String>>()
                    .join(" ")
            ),
            _ => write!(f, "<{}>", format!("{:?}", self).to_lowercase()),
        }
    }
}
