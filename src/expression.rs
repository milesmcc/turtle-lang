use std::fmt;
use std::sync::{Arc, RwLock, RwLockReadGuard, RwLockWriteGuard};

use crate::Environment;

pub type Symbol = String;

#[derive(Debug, Clone, PartialEq, PartialOrd)]
pub enum Operator {
    Quote,
    Atom,
    Eq,
    Car,
    Cdr,
    Cons,
    Cond,
    Label,
    Sum,
    Prod,
    Exp,
    Modulo,
    Gt,
    Ge,
    Type,
    Disp,
}

#[derive(Debug, Clone, PartialEq, PartialOrd)]
pub enum Value<'a> {
    List(Vec<Expression<'a>>),
    Number(f64),
    Text(String),
    Keyword(String),
    Symbol(Symbol),
    True,

    // Primitive (axiomatic) operators
    Operator(Operator),

    Lambda {
        params: Vec<Symbol>,
        expressions: Vec<Expression<'a>>,
    },
}

impl<'a> Value<'a> {
    pub fn as_type(&self) -> Self {
        use Value::*;

        Value::Keyword(match self {
            List(_) => "list".to_string(),
            Number(_) => "number".to_string(),
            Text(_) => "text".to_string(),
            Keyword(_) => "keyword".to_string(),
            Symbol(_) => "symbol".to_string(),
            Operator(_) => "operator".to_string(),
            Lambda {
                params: _,
                expressions: _,
            } => "lambda".to_string(),
            val => format!("{:?}", val).to_lowercase(),
        })
    }
}

#[derive(Debug, Clone)]
pub struct Expression<'a> {
    value: Value<'a>,
    env: Arc<RwLock<Environment<'a>>>,
}

#[derive(Debug, Clone)]
pub enum ExceptionValue {
    Other(String),
}

#[derive(Debug, Clone)]
pub struct Exception<'a> {
    expression: Option<Expression<'a>>,
    value: ExceptionValue,
}

impl<'a> PartialEq for Expression<'a> {
    fn eq(&self, other: &Self) -> bool {
        // TODO: do we need to check whether the environments are the same?
        self.value == other.value
    }
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

    pub fn t() -> Self {
        Self {
            value: Value::True,
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

        match &self.value {
            List(vals) => {
                if vals.len() > 0 {
                    let mut operator = vals
                        .get(0)
                        .expect("list must have operator (this should never happen)")
                        .clone();
                    let mut arguments: Vec<Expression<'a>> = vals.iter().skip(1).cloned().collect();
                    match operator.value {
                        Operator(operand) => operand.apply(arguments, self),
                        List(_) | Symbol(_) => {
                            let evaled_operator = operator.eval();
                            let mut new_list = vec![evaled_operator];
                            for arg in arguments {
                                new_list.push(arg);
                            }
                            Expression::new(Value::List(new_list), self.env.clone()).eval()
                        }
                        Lambda {
                            params,
                            mut expressions,
                        } => {
                            for (symbol, arg_expr) in params.iter().zip(arguments.iter()) {
                                // Note: because evaluating the argument expression requires
                                // accessing the environment, it cannot be done while `get_env_mut`
                                // is active (as the thread would deadlock).
                                let arg_evaled = arg_expr.clone().eval();
                                for exp in &mut expressions {
                                    exp.get_env_mut().assign(symbol.clone(), arg_evaled.clone());
                                }
                            }
                            let mut result = Expression::nil();
                            for mut exp in expressions {
                                result = exp.eval();
                            }
                            result
                        }
                        val => unimplemented!(
                            "unimplemented operator `{}` in list `{}`",
                            val,
                            self.value
                        ),
                    }
                } else {
                    self.clone()
                }
            }
            True => Expression::new(Value::True, self.env.clone()),
            Symbol(sym) => match self.get_env().lookup(&sym) {
                Some(exp) => exp,
                None => panic!("symbol `{}` is undefined", sym),
            },
            _ => self.clone(),
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

impl<'a> Operator {
    pub fn apply(
        &self,
        mut arguments: Vec<Expression<'a>>,
        expr: &mut Expression<'a>,
    ) -> Expression<'a> {
        use crate::Operator::*;
        use Value::*;

        match self {
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
                List(_) => Expression::new(Value::List(vec![]), expr.env.clone()),
                _ => Expression::new(Value::True, expr.env.clone()),
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
                match (first.into_value(), second.into_value()) {
                    (List(l1), List(l2)) => {
                        if l1.len() == 0 && l2.len() == 0 {
                            Expression::t()
                        } else {
                            Expression::nil()
                        }
                    }
                    (v1, v2) => {
                        if v1 == v2 {
                            Expression::t()
                        } else {
                            Expression::nil()
                        }
                    }
                }
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
                        if vals.len() > 0 {
                            vals.remove(0);
                        }
                        Expression::new(List(vals), expr.env.clone())
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
                        Expression::new(List(vals), expr.env.clone())
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
            Label => {
                let sym_exp = arguments
                    .get(0)
                    .expect("label requires an argument for the symbol")
                    .clone()
                    .eval();
                let symbol = match sym_exp.into_value() {
                    Symbol(sym) => sym,
                    _ => panic!(
                        "first arg of label must evaluate to a symbol (received `{}`)",
                        arguments.get(0).unwrap()
                    ),
                };
                let assigned_expr = arguments
                    .get(1)
                    .expect(
                        "label requires a second argument for the assigned expression",
                    )
                    .clone()
                    .eval();
                    expr.get_env_mut().assign(symbol.clone(), assigned_expr.clone());
                    assigned_expr
            }
            Sum => Expression::new(
                Value::Number(arguments.iter().fold(0.0, |acc, el| {
                    match el.clone().eval().into_value() {
                        Number(val) => acc + val,
                        val => panic!(
                            "add expects numbers as its arguments (got `{}`)",
                            val
                        ),
                    }
                })),
                expr.env.clone(),
            ),
            Prod => Expression::new(
                Value::Number(arguments.iter().fold(1.0, |acc, el| {
                    match el.clone().eval().into_value() {
                        Number(val) => acc * val,
                        val => panic!(
                            "mult expects numbers as its arguments (got `{}`)",
                            val
                        ),
                    }
                })),
                expr.env.clone(),
            ),
            Exp => Expression::new(
                Value::Number(
                    match (
                        arguments
                            .get_mut(0)
                            .expect("exp requires a first argument")
                            .eval().into_value(),
                        arguments
                            .get_mut(1)
                            .expect("exp requires a second argument")
                            .eval().into_value(),
                    ) {
                        (Number(base), Number(exp)) => base.powf(exp),
                        (base, exp) => panic!("exp requires its arguments to be both numerical (got `{}` and `{}`)", base, exp),
                    },
                ),
                expr.env.clone(),
            ),
            Modulo => Expression::new(
                Value::Number(
                    match (
                        arguments
                            .get_mut(0)
                            .expect("modulo requires a first argument")
                            .eval().into_value(),
                        arguments
                            .get_mut(1)
                            .expect("modulo requires a second argument")
                            .eval().into_value(),
                    ) {
                        (Number(base), Number(modu)) => base % modu,
                        (base, modu) => panic!("modulo requires its arguments to be both numerical (got `{}` and `{}`)", base, modu),
                    },
                ),
                expr.env.clone(),
            ),
            Gt => {
                let args_evaled = arguments.iter_mut().map(|arg| arg.eval().into_value()).collect::<Vec<Value<'a>>>();
                match args_evaled.iter().skip(1).zip(args_evaled.iter()).all(|(g, l)| g > l) {
                    true => Expression::t(),
                    false => Expression::nil(),
                }
            }
            Ge => {
                let args_evaled = arguments.iter_mut().map(|arg| arg.eval().into_value()).collect::<Vec<Value<'a>>>();
                match args_evaled.iter().skip(1).zip(args_evaled.iter()).all(|(g, l)| g >= l) {
                    true => Expression::t(),
                    false => Expression::nil(),
                }
            }
            Type => {
                let arg_type = arguments.get_mut(0).expect("type requires an argument").eval().into_value().as_type();
                Expression::new(arg_type, expr.env.clone())
            }
            Disp => {
                for mut arg in arguments {
                    print!("{}\n", arg.eval());
                }
                Expression::nil()
            }
        }
    }
}

impl fmt::Display for Operator {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", format!("{:?}", self).to_lowercase().as_str())
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
            List(vals) => match vals.len() {
                0 => write!(f, "nil"),
                _ => write!(
                    f,
                    "({})",
                    vals.iter()
                        .map(|v| format!("{}", v))
                        .collect::<Vec<String>>()
                        .join(" ")
                ),
            },
            Number(val) => write!(f, "{}", val),
            Text(val) => write!(f, "<\"{}\">", val),
            Symbol(val) => write!(f, "{}", val),
            Keyword(val) => write!(f, ":{}", val),
            True => write!(f, "true"),
            Lambda {
                params,
                expressions,
            } => write!(
                f,
                "<lambda {} -> {}>",
                params
                    .iter()
                    .map(|x| format!("{}", x))
                    .collect::<Vec<String>>()
                    .join(" "),
                expressions
                    .iter()
                    .map(|x| format!("{}", x))
                    .collect::<Vec<String>>()
                    .join(" ")
            ),
            _ => write!(f, "<{}>", format!("{:?}", self).to_lowercase()),
        }
    }
}

impl<'a> PartialOrd for Expression<'a> {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        self.value.partial_cmp(&other.value)
    }
}
