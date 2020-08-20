use crate::{Environment, Expression, Symbol};
use std::cmp::Ordering;

use crate::Locker;

#[derive(Debug, Clone)]
pub struct Function {
    pub params: Vec<Symbol>,
    pub expressions: Vec<Expression>,
    pub collapse_input: bool,
    pub lexical_scope: Locker<Environment>,
}

impl PartialEq for Function {
    fn eq(&self, other: &Self) -> bool {
        self.params == other.params
            && self.expressions == other.expressions
            && self.collapse_input == other.collapse_input
    }
}

impl PartialOrd for Function {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        if self == other {
            Some(Ordering::Equal)
        } else {
            None
        }
    }
}

impl Function {
    pub fn new(
        params: Vec<Symbol>,
        expressions: Vec<Expression>,
        collapse_input: bool,
        lexical_scope: Locker<Environment>,
    ) -> Self {
        Self {
            params,
            expressions,
            collapse_input,
            lexical_scope,
        }
    }
}
