use crate::{exp, Exception, ExceptionValue as EV, Expression, Locker};
use ansi_term::Color;
use std::fmt;


#[derive(Debug, Clone)]
pub struct CallSnapshot {
    parent: Option<Locker<Self>>,
    expression: Expression,
    depth: usize,
}

impl CallSnapshot {
    pub fn root(exp: &Expression) -> Locker<Self> {
        Locker::new(CallSnapshot {
            parent: None,
            expression: exp.clone(),
            depth: 0,
        })
    }

    pub fn new(
        exp: &Expression,
        parent: &Locker<Self>,
    ) -> Result<Locker<Self>, Exception> {
        // TODO: make read lock check return an exception instead of panicking
        let depth = parent
            .read()
            .expect("could not access call snapshot parent (are threads locked?)")
            .depth
            + 1;
        if depth > 1000 {
            exp!(
                EV::StackOverflow,
                parent,
                "this can happen when recursion goes too deep; verify there aren't any endless loops, or consider using `while` instead".to_string()
            )
        }

        Ok(Locker::new(CallSnapshot {
            parent: Some(parent.clone()),
            expression: exp.clone(),
            depth,
        }))
    }

    pub fn expression(&self) -> &'_ Expression {
        &self.expression
    }
}

impl fmt::Display for CallSnapshot {
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
