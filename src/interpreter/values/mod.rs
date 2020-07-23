use crate::Expression;
use std::fmt;

pub mod operator;
pub use operator::Operator;

pub mod symbol;
pub use symbol::Symbol;

pub mod keyword;
pub use keyword::Keyword;

#[derive(Debug, Clone, PartialEq, PartialOrd)]
pub enum Value {
    List(Vec<Expression>),
    Number(f64),
    Text(String),
    Keyword(Keyword),
    Symbol(Symbol),
    True,

    // Primitive (axiomatic) operators
    Operator(Operator),

    Lambda {
        params: Vec<Symbol>,
        expressions: Vec<Expression>,
        collapse_input: bool,
    },
    Macro {
        params: Vec<Symbol>,
        expressions: Vec<Expression>,
        collapse_input: bool,
    },
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
            Lambda { .. } => "lambda".to_string(),
            Macro { .. } => "macro".to_string(),
            _ => "unknown".to_string(),
        }))
    }
}

impl<'a> fmt::Display for Value {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        use Value::*;

        match self {
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
            True => write!(f, "true"),
            Lambda {
                params,
                expressions,
                collapse_input,
            } => write!(
                f,
                "<lambda {}{}{} -> {}>",
                match collapse_input {
                    true => "",
                    false => "(",
                },
                params
                    .iter()
                    .map(|x| format!("{}", x))
                    .collect::<Vec<String>>()
                    .join(" "),
                match collapse_input {
                    true => "",
                    false => ")",
                },
                expressions
                    .iter()
                    .map(|x| format!("{}", x))
                    .collect::<Vec<String>>()
                    .join(" ")
            ),
            Macro {
                params,
                expressions,
                collapse_input,
            } => write!(
                f,
                "<macro {}{}{} -> {}>",
                match collapse_input {
                    true => "",
                    false => "(",
                },
                params
                    .iter()
                    .map(|x| format!("{}", x))
                    .collect::<Vec<String>>()
                    .join(" "),
                match collapse_input {
                    true => "",
                    false => ")",
                },
                expressions
                    .iter()
                    .map(|x| format!("{}", x))
                    .collect::<Vec<String>>()
                    .join(" ")
            ),
            _ => write!(f, "<{}>", format!("{:?}", self).to_lowercase()),
        }
    }
}
