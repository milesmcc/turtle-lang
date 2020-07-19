use std::collections::HashMap;
use std::fmt;
use std::sync::{Arc, RwLock, RwLockReadGuard, RwLockWriteGuard};

pub type Symbol = String;

#[derive(Debug)]
pub struct Environment<'a> {
    values: HashMap<Symbol, Expression<'a>>,
    // This unreadable memory model might cause issues going forward
    parent: Option<&'a Environment<'a>>,
}

impl<'a> Environment<'a> {
    // TODO: see if this can be done without mutexes, at least for values

    pub fn root() -> Self {
        Self {
            values: HashMap::new(),
            parent: None,
        }
    }

    pub fn with_parent(parent: &'a Self) -> Self {
        Self {
            values: HashMap::new(),
            parent: Some(parent),
        }
    }

    pub fn new_child(&'a self) -> Arc<RwLock<Environment<'a>>> {
        Arc::new(RwLock::new(Self::with_parent(self)))
    }

    pub fn lookup(&self, symbol: &Symbol) -> Option<Expression<'a>> {
        match self.values.get(symbol) {
            Some(val) => Some(val.clone()),
            None => match self.parent {
                Some(parent) => parent.lookup(symbol),
                None => None,
            },
        }
    }

    pub fn assign(&mut self, symbol: Symbol, exp: Expression<'a>) {
        self.values.insert(symbol, exp);
    }
}

#[derive(Debug, Clone)]
pub struct Expression<'a> {
    value: Value<'a>,
    env: Arc<RwLock<Environment<'a>>>,
}

impl<'a> PartialEq for Expression<'a> {
    fn eq(&self, other: &Self) -> bool {
        // TODO: do we need to check whether the environments are the same?
        self.value == other.value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum Value<'a> {
    List(Vec<Expression<'a>>),
    Number(f64),
    Text(String),
    Symbol(Symbol),
    True,

    // Primitive (axiomatic) operators
    Quote,
    Atom,
    Eq,
    Car,
    Cdr,
    Cons,
    Cond,
    Label,

    Function {
        params: Vec<Symbol>,
        expression: Box<Expression<'a>>,
    },
}

impl<'a> Expression<'a> {
    pub fn new(value: Value<'a>, env: Arc<RwLock<Environment<'a>>>) -> Self {
        Self {
            value: value,
            env: env,
        }
    }

    pub fn nil() -> Self {
        Self {
            value: Value::List(vec![]),
            env: Arc::new(RwLock::new(Environment::root())),
        }
    }

    pub fn get_value(&self) -> &'a Value {
        &self.value
    }

    pub fn into_value(self) -> Value<'a> {
        self.value
    }

    pub fn eval(&mut self) -> Self {
        use Value::*;

        // let mut exp = self.clone();

        println!("evaluating: {}", self);
        println!(
            "environment ======================\n{}\n===========================================",
            self.get_env()
        );

        match &self.value {
            List(vals) => {
                // TODO: do as match on slice
                if vals.len() > 0 {
                    let mut operator = vals
                        .get(0)
                        .expect("list must have operator (this should never happen)")
                        .clone();
                    let mut arguments: Vec<Expression<'a>> = vals.iter().skip(1).cloned().collect();
                    match operator.value {
                        Quote => arguments
                            .get(0)
                            .expect("quote requires one argument")
                            .clone(),
                        Atom => match arguments
                            .get_mut(0)
                            .expect("atom requires one argument")
                            .eval()
                            .into_value()
                        {
                            List(_) => Expression::new(Value::List(vec![]), self.env.clone()),
                            _ => Expression::new(Value::True, self.env.clone()),
                        },
                        Eq => {
                            let first = arguments
                                .get_mut(0)
                                .expect("eq requires a first argument")
                                .eval();
                            let second = arguments
                                .get_mut(1)
                                .expect("eq requires a second argument")
                                .eval();
                            Expression::new(
                                match (first.into_value(), second.into_value()) {
                                    (List(l1), List(l2)) => {
                                        if l1.len() == 0 && l2.len() == 0 {
                                            True
                                        } else {
                                            List(vec![])
                                        }
                                    }
                                    (v1, v2) => {
                                        if v1 == v2 {
                                            True
                                        } else {
                                            List(vec![])
                                        }
                                    }
                                },
                                self.env.clone(),
                            )
                        }
                        Car => {
                            let list = arguments
                                .get_mut(0)
                                .expect("car requires an argument")
                                .eval();
                            match list.value {
                                List(mut vals) => vals.remove(0),
                                _ => panic!("car expects a list, got `{}`", list),
                            }
                        }
                        Cdr => {
                            let list = arguments
                                .get_mut(0)
                                .expect("cdr requires an argument")
                                .eval();
                            match list.value {
                                List(mut vals) => {
                                    vals.remove(0);
                                    Expression::new(List(vals), self.env.clone())
                                }
                                _ => panic!("cdr expects a list, got `{}`", list),
                            }
                        }
                        Cons => {
                            let first = arguments[0].eval();
                            let list = arguments[1].eval();
                            match list.value {
                                List(mut vals) => {
                                    vals.insert(0, first);
                                    Expression::new(List(vals), self.env.clone())
                                }
                                _ => panic!(
                                    "cons expects a list as its second argument, got `{}`",
                                    list
                                ),
                            }
                        }
                        Cond => {
                            for argument in arguments {
                                match argument.value {
                                    List(mut elems) => {
                                        let cond = {
                                            elems.get_mut(0).expect("cond must have a conditional")
                                        };
                                        if cond.eval().into_value() == Value::True {
                                            let val = {
                                                elems
                                                    .get_mut(1)
                                                    .expect("cond must have a value to eval")
                                            };
                                            return val.eval();
                                        }
                                    }
                                    _ => {
                                        panic!("cond must be called on a list, got `{}`", argument)
                                    }
                                }
                            }
                            panic!("none of cond was true");
                        }
                        Function {
                            params,
                            mut expression,
                        } => {
                            for (symbol, arg_expr) in params.iter().zip(arguments.iter()) {
                                // Note: because evaluating the argument expression requires
                                // accessing the environment, it cannot be done while `get_env_mut`
                                // is active (as the thread would deadlock).
                                let arg_evaled = arg_expr.clone().eval();
                                expression.get_env_mut().assign(symbol.clone(), arg_evaled);
                            }
                            expression.eval()
                        }
                        Label => {
                            // TODO: cleanup
                            let symbol = match &arguments
                                .get(0)
                                .expect("label requires an argument for the symbol")
                                .value
                            {
                                Symbol(sym) => sym,
                                _ => panic!(
                                    "first arg of label must be symbol (received `{}`)",
                                    arguments.get(0).unwrap()
                                ),
                            };
                            let expr = arguments
                                .get(1)
                                .expect(
                                    "label requires a second argument for the assigned expression",
                                )
                                .clone();
                            self.get_env_mut().assign(symbol.clone(), expr);
                            Expression::nil()
                        }
                        List(_) | Symbol(_) => {
                            // TODO: check if list is what we want here
                            let evaled_operator = operator.eval();
                            // TODO: is there a cleaner way to do this? Yes, there is...
                            let mut new_list = vec![evaled_operator];
                            for arg in arguments {
                                new_list.push(arg); // TODO: clone?
                            }
                            Expression::new(Value::List(new_list), self.env.clone()).eval()
                        }
                        val => unimplemented!("unimplemented operator `{:?}`", val),
                    }
                } else {
                    panic!("cannot evaluate an empty list!");
                }
            }
            True => Expression::new(Value::True, self.env.clone()),
            Symbol(sym) => match self.get_env().lookup(&sym) {
                Some(exp) => exp,
                None => panic!("symbol `{}` is undefined", sym),
            },
            _ => panic!("cannot evaluate literal value `{}`", self),
        }
    }

    fn get_env(&self) -> RwLockReadGuard<Environment<'a>> {
        self.env.read().expect("unable to access environment")
    }

    fn get_env_mut(&mut self) -> RwLockWriteGuard<Environment<'a>> {
        self.env
            .write()
            .expect("unable to mutably access environment")
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
            match self.parent {
                Some(parent) => format!("{}", parent),
                None => format!("env has no parent"),
            }
        )
    }
}

impl<'a> fmt::Display for Expression<'a> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.value)
    }
}

impl<'a> fmt::Display for Value<'a> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        use Value::*;

        match self {
            List(vals) => write!(
                f,
                "({})",
                vals.iter()
                    .map(|v| format!("{}", v))
                    .collect::<Vec<String>>()
                    .join(" ")
            ),
            Number(val) => write!(f, "<{}>", val),
            Text(val) => write!(f, "<\"{}\">", val),
            Symbol(val) => write!(f, "{}", val),
            True => write!(f, "<true>"),
            Function { params, expression } => write!(
                f,
                "<lambda {} -> {}>",
                params
                    .iter()
                    .map(|x| format!("{}", x))
                    .collect::<Vec<String>>()
                    .join(" "),
                expression
            ),
            _ => write!(f, "<{}>", format!("{:?}", self).to_lowercase()),
        }
    }
}
