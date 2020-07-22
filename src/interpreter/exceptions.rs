use crate::{parser, CallSnapshot, Keyword, SourcePosition, Symbol, Value};
use ansi_term::{Color, Style};
use std::error::Error;
use std::fmt;
use std::sync::{Arc, RwLock};

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
pub enum ExceptionValue<'a> {
    Other(String, Keyword),
    UndefinedSymbol(Symbol),
    ArgumentMismatch(usize, String),
    InvalidArgument,
    Syntax,
    InvalidIncludePath(String),
    InvalidOperator(Value<'a>),
}

impl ExceptionValue<'_> {
    pub fn explain(&self) -> String {
        use ExceptionValue::*;

        match self {
            Other(val, _) => val.clone(),
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
        }
    }

    pub fn keyword(&self) -> Keyword {
        use ExceptionValue::*;

        match self {
            Other(_, keyword) => keyword.clone(),
            UndefinedSymbol(_) => Keyword::from_str("undefined-symbol-exp"),
            ArgumentMismatch(_, _) => Keyword::from_str("argument-mismatch-exp"),
            Syntax => Keyword::from_str("syntax-exp"),
            InvalidArgument => Keyword::from_str("invalid-argument-exp"),
            InvalidIncludePath(_) => Keyword::from_str("invalid-include-path-exp"),
            InvalidOperator(_) => Keyword::from_str("invalid-operator-exp"),
        }
    }
}

impl fmt::Display for ExceptionValue<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{} ({})", self.explain(), self.keyword())
    }
}

#[derive(Debug, Clone)]
pub struct Exception<'a> {
    value: ExceptionValue<'a>,
    snapshot: Option<Arc<RwLock<CallSnapshot<'a>>>>,
    additional_sources: Vec<SourcePosition>,
    note: Option<String>,
}

impl<'a> Exception<'a> {
    pub fn new(
        value: ExceptionValue<'a>,
        snapshot: Option<Arc<RwLock<CallSnapshot<'a>>>>,
        note: Option<String>,
    ) -> Self {
        Exception {
            value,
            snapshot,
            note,
            additional_sources: vec![],
        }
    }
}

impl From<pest::error::Error<parser::Rule>> for Exception<'_> {
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

impl fmt::Display for Exception<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        writeln!(
            f,
            "{}{}{} {}",
            Color::Red.bold().paint("error"),
            Color::Blue.bold().paint(" ┬ "),
            Style::new().paint("uncaught exception"),
            Color::Yellow.paint(format!("{}", self.value.keyword()))
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

impl Error for Exception<'_> {}
