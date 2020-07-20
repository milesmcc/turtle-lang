use std::sync::{Arc, RwLock};
use crate::Expression;

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