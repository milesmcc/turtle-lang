use crate::{exp, Exception, ExceptionValue as EV, Expression};
use ansi_term::Color;
use std::fmt;
use std::sync::{Arc, RwLock};

#[derive(Debug, Clone)]
pub struct CallSnapshot<'a> {
    parent: Option<Arc<RwLock<Self>>>,
    expression: Expression<'a>,
    depth: usize,
}

impl<'a> CallSnapshot<'a> {
    pub fn root(exp: &Expression<'a>) -> Arc<RwLock<Self>> {
        Arc::new(RwLock::new(CallSnapshot {
            parent: None,
            expression: exp.clone(),
            depth: 0,
        }))
    }

    pub fn new(
        exp: &Expression<'a>,
        parent: &Arc<RwLock<Self>>,
    ) -> Result<Arc<RwLock<Self>>, Exception<'a>> {
        // TODO: make read lock check return an exception instead of panicking
        let depth = parent
            .read()
            .expect("could not access call snapshot parent (are threads locked?)")
            .depth
            + 1;
        if depth > 500 {
            exp!(
                EV::StackOverflow,
                parent,
                "this can happen when recursion goes too deep; verify there aren't any endless loops, or consider using `while` instead".to_string()
            )
        }

        Ok(Arc::new(RwLock::new(CallSnapshot {
            parent: Some(parent.clone()),
            expression: exp.clone(),
            depth,
        })))
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
