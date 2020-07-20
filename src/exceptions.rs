use crate::{CallSnapshot, Symbol};
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
}

#[derive(Clone)]
pub struct Exception<'a> {
    value: ExceptionValue,
    snapshot: Option<Arc<RwLock<CallSnapshot<'a>>>>,
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
        }
    }
}
