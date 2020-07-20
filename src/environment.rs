use std::collections::HashMap;
use std::fmt;
use std::sync::{Arc, RwLock};

use crate::{Expression, Operator, Symbol, Value};

#[derive(Debug)]
pub struct Environment<'a> {
    values: HashMap<Symbol, Expression<'a>>,
    // This unreadable memory model might cause issues going forward
    parent: Option<Arc<RwLock<Environment<'a>>>>,
}

impl<'a> Environment<'a> {
    // TODO: see if this can be done without mutexes, at least for values

    pub fn root() -> Self {
        Self {
            values: HashMap::new(),
            parent: None,
        }
    }

    pub fn with_parent(parent: Arc<RwLock<Self>>) -> Arc<RwLock<Self>> {
        Arc::new(RwLock::new(Self {
            values: HashMap::new(),
            parent: Some(parent),
        }))
    }

    fn get_literal(symbol: &Symbol) -> Option<Value<'a>> {
        use Operator::*;

        match symbol.as_str() {
            "nil" => Some(Value::List(vec![])),
            "t" => Some(Value::True),
            "quote" => Some(Value::Operator(Quote)),
            "atom" => Some(Value::Operator(Atom)),
            "eq" => Some(Value::Operator(Eq)),
            "car" => Some(Value::Operator(Car)),
            "cdr" => Some(Value::Operator(Cdr)),
            "cons" => Some(Value::Operator(Cons)),
            "cond" => Some(Value::Operator(Cond)),
            "label" => Some(Value::Operator(Label)),
            "sum" => Some(Value::Operator(Sum)),
            "prod" => Some(Value::Operator(Prod)),
            "exp" => Some(Value::Operator(Exp)),
            "modulo" => Some(Value::Operator(Modulo)),
            "gt" => Some(Value::Operator(Gt)),
            "ge" => Some(Value::Operator(Ge)),
            "type" => Some(Value::Operator(Type)),
            "disp" => Some(Value::Operator(Disp)),
            _ => None,
        }
    }

    pub fn lookup(&self, symbol: &Symbol) -> Option<Expression<'a>> {
        match self.values.get(symbol) {
            Some(val) => Some(val.clone()),
            None => match &self.parent {
                Some(parent) => parent
                    .read()
                    .expect("cannot access environment parent")
                    .lookup(symbol),
                None => match Self::get_literal(symbol) {
                    Some(val) => Some(Expression::new(val, Arc::new(RwLock::new(Self::root())))),
                    _ => None,
                },
            },
        }
    }

    pub fn assign(&mut self, symbol: Symbol, exp: Expression<'a>) {
        self.values.insert(symbol, exp);
    }
}

impl<'a> fmt::Display for Environment<'a> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "[values: {}]\n{}\n--- showing parent ---\n{}",
            self.values.len(),
            self.values
                .iter()
                .map(|(k, v)| format!("{} := {}", k, v))
                .collect::<Vec<String>>()
                .join("\n"),
            match &self.parent {
                Some(parent) => format!("{}", parent.read().expect("cannot get parent")),
                None => format!("env has no parent"),
            }
        )
    }
}