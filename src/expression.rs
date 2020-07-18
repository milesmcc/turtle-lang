use std::collections::HashMap;
use std::fmt;
use std::sync::{Arc, Mutex};

pub type Symbol = String;

#[derive(Debug)]
pub struct Environment<'a> {
    values: HashMap<Symbol, Arc<Mutex<Expression>>>,
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

    pub fn new_child(&'a self) -> Self {
        Self::with_parent(self)
    }

    pub fn lookup(&self, symbol: &Symbol) -> Option<Expression> {
        match self.values.get(symbol) {
            Some(val) => Some(
                val.clone()
                    .lock()
                    .expect("could not get expression")
                    .clone(),
            ),
            None => match self.parent {
                Some(parent) => parent.lookup(symbol),
                None => None,
            },
        }
    }

    pub fn assign(&mut self, symbol: Symbol, exp: Arc<Mutex<Expression>>) {
        self.values.insert(symbol, exp);
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct Expression {
    value: Value,
}

#[derive(Debug, Clone, PartialEq)]
pub enum Value {
    List(Vec<Expression>),
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
        expression: Box<Expression>,
    },
}

impl Expression {
    pub fn new(value: Value) -> Self {
        Self { value: value }
    }

    pub fn get_value(&self) -> &Value {
        &self.value
    }

    pub fn into_value(self) -> Value {
        self.value
    }

    pub fn eval(&self, env: &mut Environment) -> Self {
        use Value::*;

        println!("evaluating: {}", self);

        match &self.value {
            List(vals) => {
                // TODO: do as match on slice
                if vals.len() > 0 {
                    let operator = vals.get(0).unwrap();
                    let arguments = &vals.as_slice()[1..];
                    match operator.get_value() {
                        Quote => arguments[0].clone(),
                        Atom => match arguments[0].eval(&mut env.new_child()).get_value() {
                            List(_) => Expression::new(Value::List(vec![])),
                            _ => Expression::new(Value::True),
                        },
                        Eq => {
                            let first = arguments[0].eval(&mut env.new_child());
                            let second = arguments[1].eval(&mut env.new_child());
                            Expression::new(match (first.get_value(), second.get_value()) {
                                (List(l1), List(l2)) => {
                                    if l1.len() == 0 && l2.len() == 0 {
                                        True
                                    } else {
                                        List(vec![])
                                    }
                                }
                                _ => {
                                    if first == second {
                                        True
                                    } else {
                                        List(vec![])
                                    }
                                }
                            })
                        }
                        Car => {
                            let mut list = arguments[0].eval(&mut env.new_child());
                            match list.value {
                                List(mut vals) => vals.remove(0),
                                _ => panic!("car expects a list, got `{}`", list),
                            }
                        }
                        Cdr => {
                            let list = arguments[0].eval(&mut env.new_child());
                            match list.value {
                                List(mut vals) => {
                                    vals.remove(0);
                                    Expression::new(List(vals))
                                }
                                _ => panic!("cdr expects a list, got `{}`", list),
                            }
                        }
                        Cons => {
                            let first = arguments[0].eval(&mut env.new_child());
                            let list = arguments[1].eval(&mut env.new_child());
                            match list.value {
                                List(mut vals) => {
                                    vals.insert(0, first);
                                    Expression::new(List(vals))
                                }
                                _ => panic!("cons expects a list, got `{}`", list),
                            }
                        }
                        Cond => {
                            for argument in arguments {
                                match &argument.value {
                                    List(elems) => {
                                        let cond =
                                            elems.get(0).expect("cond must have a conditional");
                                        let val =
                                            elems.get(1).expect("cond must have a value to eval");
                                        if *cond.eval(&mut env.new_child()).get_value() == Value::True {
                                            return val.eval(&mut env.new_child());
                                        }
                                    }
                                    _ => {
                                        panic!("cond must be called on a list, got `{}`", argument)
                                    }
                                }
                            }
                            panic!("none of cond was true");
                        }
                        Function { params, expression } => {
                            let mut exp_env = env.new_child();
                            for (symbol, exp) in params.iter().zip(arguments.iter()) {
                                // TODO: is there a way to get `exp` without cloning?
                                exp_env.assign(symbol.clone(), Arc::new(Mutex::new(exp.clone())));
                            }
                            expression.eval(&mut exp_env)
                        }
                        Symbol(_) | List(_) => { // TODO: check if list is what we want here
                            println!("evaluating list operator {}", operator);
                            let evaled_operator = operator.eval(&mut env.new_child());
                            // TODO: is there a cleaner way to do this? Yes, there is...
                            let mut new_list = vec![evaled_operator];
                            new_list.append(&mut Vec::from(arguments));
                            Expression::new(Value::List(new_list)).eval(&mut env.new_child())
                        }
                        Label => {
                            // TODO: cleanup
                            let symbol = match &arguments[0].get_value() {
                                Symbol(sym) => sym,
                                _ => panic!(
                                    "first arg of label must be symbol (received `{}`)",
                                    arguments[0]
                                ),
                            };
                            let expr = &arguments[1];
                            env.assign(symbol.clone(), Arc::new(Mutex::new(expr.clone())));
                            expr.clone()
                        }
                        _ => unimplemented!("unimplemented operator `{:?}`", operator.get_value()),
                    }
                } else {
                    panic!("cannot evaluate an empty list!");
                }
            }
            True => Expression::new(Value::True),
            Symbol(sym) => match env.lookup(sym) {
                Some(exp) => exp,
                None => panic!("symbol `{}` is undefined", sym),
            },
            _ => panic!("cannot evaluate literal value `{}`", self),
        }
    }
}

impl fmt::Display for Expression {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.value)
    }
}

impl fmt::Display for Value {
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
            True => write!(f, "<t>"),
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
