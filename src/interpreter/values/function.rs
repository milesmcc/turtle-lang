use crate::{Environment, Expression, Symbol};
use std::sync::{Arc, RwLock};

#[derive(Debug, Clone)]
pub struct Function {
    pub params: Vec<Symbol>,
    pub expressions: Vec<Expression>,
    pub collapse_input: bool,
    pub lexical_scope: Arc<RwLock<Environment>>,
}

impl Function {
    pub fn new(
        params: Vec<Symbol>,
        expressions: Vec<Expression>,
        collapse_input: bool,
        lexical_scope: Arc<RwLock<Environment>>,
    ) -> Self {
        Self {
            params,
            expressions,
            collapse_input,
            lexical_scope,
        }
    }
}
