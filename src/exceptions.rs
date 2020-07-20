use crate::{parser, CallSnapshot, Keyword, SourcePosition, Symbol};
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
    Syntax,
}

impl ExceptionValue {
    pub fn explain(&self) -> String {
        use ExceptionValue::*;

        match self {
            Other(val) => val.clone(),
            UndefinedSymbol(symbol) => format!("the symbol `{}` has no assigned value", symbol),
            ArgumentMismatch => format!("the arguments to this function are invalid"),
            Syntax => format!("the syntax of this code is incorrect"),
        }
    }

    pub fn keyword(&self) -> Keyword {
        use ExceptionValue::*;

        Keyword::from_str(match self {
            Other(_) => "other-exp",
            UndefinedSymbol(_) => "undefined-symbol-exp",
            ArgumentMismatch => "argument-mismatch-exp",
            Syntax => "syntax-exp",
        })
    }
}

impl fmt::Display for ExceptionValue {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{} ({})", self.explain(), self.keyword())
    }
}

#[derive(Clone)]
pub struct Exception<'a> {
    value: ExceptionValue,
    snapshot: Option<Arc<RwLock<CallSnapshot<'a>>>>,
    additional_sources: Vec<SourcePosition>,
    note: Option<String>,
}

impl<'a> Exception<'a> {
    pub fn new(
        value: ExceptionValue,
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

        let (start, end) = match err.location {
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
            "{}{} {}",
            Color::Red.bold().paint("error"),
            Style::new()
                .bold()
                .paint(format!(": {}", self.value.explain())),
            Color::Yellow.paint(format!("{}", self.value.keyword()))
        );

        match &self.snapshot {
            Some(snapshot_lock) => match snapshot_lock.read() {
                Ok(snapshot) => match snapshot.expression().source() {
                    Some(source) => {
                        // TODO: print the snapshot instead (include call stack)
                        write!(f, "{}", source);
                    }
                    None => {}
                },
                Err(_) => {
                    writeln!(
                        f,
                        "{}{}",
                        Color::Yellow.bold().paint("warning"),
                        Style::new()
                            .bold()
                            .paint(": unable to access execution snapshot (are threads locked?)")
                    );
                }
            },
            None => {}
        };

        for addl_source in &self.additional_sources {
            writeln!(f, "{}", addl_source);
        }

        match &self.note {
            Some(note) => writeln!(f, "{}: {}", Style::new().bold().paint("note"), note),
            None => {write!(f, "")},
        }
    }
}
