use crate::Expression;
use ansi_term::Color;
use std::fmt;
use std::sync::{Arc, RwLock};

#[derive(Debug, Clone)]
pub struct CallSnapshot<'a> {
    parent: Option<Arc<RwLock<Self>>>,
    expression: Expression<'a>,
}

impl<'a> CallSnapshot<'a> {
    pub fn root(exp: &Expression<'a>) -> Arc<RwLock<Self>> {
        Arc::new(RwLock::new(CallSnapshot {
            parent: None,
            expression: exp.clone(),
        }))
    }

    pub fn new(exp: &Expression<'a>, parent: &Arc<RwLock<Self>>) -> Arc<RwLock<Self>> {
        Arc::new(RwLock::new(CallSnapshot {
            parent: Some(parent.clone()),
            expression: exp.clone(),
        }))
    }

    pub fn expression(&self) -> &'_ Expression<'a> {
        &self.expression
    }
}

impl fmt::Display for CallSnapshot<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match &self.parent {
            Some(parent_ref) => match parent_ref.read() {
                // This crazy match makes sure we don't print redundant lines
                Ok(parent) => {
                    match format!("{}", self.expression) == format!("{}", parent.expression) {
                        true => match &parent.parent {
                            Some(superparent_ref) => {
                                if let Ok(superparent) = superparent_ref.read() {
                                    write!(f, "{}", superparent)?
                                }
                            }
                            None => {}
                        },
                        false => write!(f, "{}", parent)?,
                    }
                }
                Err(_) => writeln!(
                    f,
                    "      {} unable to access parent call (are threads locked?)",
                    Color::Yellow.bold().paint("!")
                )?,
            },
            None => {}
        };
        match self.expression().source() {
            Some(source) => write!(f, "{}", source)?,
            None => {}
        }
        write!(f, "")
    }
}
