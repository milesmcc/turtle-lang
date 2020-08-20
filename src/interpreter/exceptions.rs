use crate::{
    parser, CallSnapshot, Environment, Expression, Keyword, SourcePosition, Symbol, Value,
};
use ansi_term::{Color, Style};
use std::error::Error;
use std::fmt;

use crate::Locker;

#[macro_export]
macro_rules! exp {
    ($value:expr) => {
        return Err(Exception::new($value, None, None));
    };
    ($value:expr, $snapshot:expr) => {
        return Err(Exception::new($value, Some($snapshot.clone()), None));
    };
    ($value:expr, $snapshot:expr, $note:expr) => {
        return Err(Exception::new($value, Some($snapshot.clone()), Some($note)));
    };
}

#[macro_export]
macro_rules! exp_opt {
    ($value:expr $(, $rest:expr)*) => {
        match $value {
            Some(value) => value,
            None => exp!($($rest)*)
        }
    };
}

#[macro_export]
macro_rules! exp_assert {
    ($test:expr $(, $rest:expr)*) => {
        if (!$test) {
            exp!($($rest),*);
        }
    };
}

#[derive(Debug, Clone)]
pub enum ExceptionValue {
    Other(Expression),
    UndefinedSymbol(Symbol),
    ArgumentMismatch(usize, String),
    InvalidArgument,
    Syntax,
    InvalidIncludePath(String),
    InvalidOperator(Value),
    StackOverflow,
    Assignment(Symbol, Expression),
    Concurrency,
}

impl ExceptionValue {
    pub fn explain(&self) -> String {
        use ExceptionValue::*;

        match self {
            Other(exp) => format!("{}", exp),
            UndefinedSymbol(symbol) => format!(
                "the symbol `{}` has no assigned value (did you mean to quote this symbol?)",
                symbol
            ),
            ArgumentMismatch(given, expected) => format!(
                "wrong number of arguments: {} required, but {} given",
                expected, given,
            ),
            InvalidArgument => String::from("the arguments to this function are invalid"),
            Syntax => String::from("the syntax of this code is incorrect"),
            InvalidIncludePath(path) => format!("no code is available for import from `{}`", path),
            InvalidOperator(value) => format!(
                "`{}` is not a valid list operator (did you mean to quote this list?)",
                value
            ),
            StackOverflow => "the call stack exceeded the limit (1000)".to_string(),
            Assignment(sym, exp) => format!("could not assign `{}` to `{}`", sym, exp),
            Concurrency => {
                "something went wrong when evaluating this expression concurrently".to_string()
            }
        }
    }

    pub fn into_expression(self) -> Expression {
        use ExceptionValue::*;

        let _root_env = Locker::new(Environment::root());

        match self {
            Other(expression) => expression,
            UndefinedSymbol(_) => {
                Expression::new(Value::Keyword(Keyword::from_str("undefined-symbol-exp")))
            }
            ArgumentMismatch(_, _) => {
                Expression::new(Value::Keyword(Keyword::from_str("argument-mismatch-exp")))
            }
            Syntax => Expression::new(Value::Keyword(Keyword::from_str("syntax-exp"))),
            InvalidArgument => {
                Expression::new(Value::Keyword(Keyword::from_str("invalid-argument-exp")))
            }
            InvalidIncludePath(_) => Expression::new(Value::Keyword(Keyword::from_str(
                "invalid-include-path-exp",
            ))),
            InvalidOperator(_) => {
                Expression::new(Value::Keyword(Keyword::from_str("invalid-operator-exp")))
            }
            StackOverflow => {
                Expression::new(Value::Keyword(Keyword::from_str("stack-overflow-exp")))
            }
            Assignment(_, _) => {
                Expression::new(Value::Keyword(Keyword::from_str("assignment-exp")))
            }
            Concurrency => Expression::new(Value::Keyword(Keyword::from_str("concurrency-exp"))),
        }
    }
}

impl fmt::Display for ExceptionValue {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{} ({})", self.explain(), self.clone().into_expression())
    }
}

#[derive(Debug, Clone)]
pub struct Exception {
    value: ExceptionValue,
    snapshot: Option<Locker<CallSnapshot>>,
    additional_sources: Vec<SourcePosition>,
    note: Option<String>,
}

impl Exception {
    pub fn new(
        value: ExceptionValue,
        snapshot: Option<Locker<CallSnapshot>>,
        note: Option<String>,
    ) -> Self {
        Exception {
            value,
            snapshot,
            note,
            additional_sources: vec![],
        }
    }

    pub fn into_value(self) -> ExceptionValue {
        self.value
    }
}

impl From<pest::error::Error<parser::Rule>> for Exception {
    fn from(err: pest::error::Error<parser::Rule>) -> Self {
        use pest::error::InputLocation::*;

        let (_start, _end) = match err.location {
            Pos(start) => (start, start),
            Span((start, end)) => (start, end),
        };

        Self {
            value: ExceptionValue::Syntax,
            snapshot: None,
            note: Some(format!("{}", err)),
            // TODO: find a nice way to extract the text-level information
            additional_sources: vec![],
        }
    }
}

impl fmt::Display for Exception {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        writeln!(
            f,
            "{}{}{} {}",
            Color::Red.bold().paint("error"),
            Color::Blue.bold().paint(" ┬ "),
            Style::new().paint("uncaught exception"),
            Color::Yellow.paint(format!("{}", self.value.clone().into_expression()))
        )?;

        match &self.snapshot {
            Some(snapshot_lock) => match snapshot_lock.read() {
                Ok(snapshot) => write!(f, "{}", snapshot)?,
                Err(_) => {
                    write!(
                        f,
                        "{}{}",
                        Color::Yellow.bold().paint("warning"),
                        Style::new()
                            .bold()
                            .paint(": unable to access execution snapshot (are threads locked?)")
                    )?;
                }
            },
            None => {}
        };

        for addl_source in &self.additional_sources {
            write!(f, "{}", addl_source)?;
        }

        write!(
            f,
            "      {}{}",
            Color::Blue.bold().paint("└ "),
            Style::new().bold().paint(self.value.explain()),
        )?;

        match &self.note {
            Some(note) => write!(
                f,
                "\n        {} {}",
                Style::new().dimmed().paint("note:"),
                note
            ),
            None => write!(f, ""),
        }
    }
}

impl Error for Exception {}
