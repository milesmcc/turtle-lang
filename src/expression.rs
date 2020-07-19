use std::fmt;
use std::sync::{Arc, RwLock, RwLockReadGuard, RwLockWriteGuard};

use crate::exp; // macro
use crate::{Environment, Exception, ExceptionValue as EV};

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
pub enum Value {
    List(Vec<Expression>),
    Number(f64),
    Text(String),
    Keyword(String),
    Symbol(Symbol),
    True,

    // Primitive (axiomatic) operators
    Operator(Operator),

    Lambda {
        params: Vec<Symbol>,
        expressions: Vec<Expression>,
    },
}

impl Value {
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
pub struct Expression {
    value: Value,
    env: Arc<RwLock<Environment>>,
}

impl PartialEq for Expression {
    fn eq(&self, other: &Self) -> bool {
        // TODO: do we need to check whether the environments are the same?
        self.value == other.value
    }
}

impl Expression {
    pub fn new(value: Value, env: Arc<RwLock<Environment>>) -> Self {
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

    pub fn get_value(&self) -> &Value {
        &self.value
    }

    pub fn into_value(self) -> Value {
        self.value
    }

    pub fn eval(&mut self) -> Result<Self, Exception> {
        use Value::*;

        match &self.value {
            List(vals) => {
                if vals.len() > 0 {
                    let mut operator = vals.get(0).unwrap().clone();
                    let arguments: Vec<Expression> = vals.iter().skip(1).cloned().collect();
                    match operator.value {
                        Operator(operand) => operand.apply(arguments, self),
                        List(_) | Symbol(_) => {
                            let evaled_operator = operator.eval()?;
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
                                    exp.get_env_mut()
                                        .assign(symbol.clone(), arg_evaled.clone()?);
                                }
                            }
                            let mut result = Expression::nil();
                            for mut exp in expressions {
                                result = exp.eval()?;
                            }
                            Ok(result)
                        }
                        val => unimplemented!(
                            "unimplemented operator `{}` in list `{}`",
                            val,
                            self.value
                        ),
                    }
                } else {
                    Ok(self.clone())
                }
            }
            Symbol(sym) => match self.get_env().lookup(&sym) {
                Some(exp) => Ok(exp),
                None => exp!(EV::UndefinedSymbol(sym.clone()), self.clone()),
            },
            _ => Ok(self.clone()),
        }
    }

    fn get_env(&self) -> RwLockReadGuard<Environment> {
        self.env.read().expect("unable to access environment")
    }

    fn get_env_mut(&mut self) -> RwLockWriteGuard<Environment> {
        self.env
            .write()
            .expect("unable to mutably access environment")
    }
}

impl Operator {
    pub fn apply(
        &self,
        mut arguments: Vec<Expression>,
        expr: &mut Expression,
    ) -> Result<Expression, Exception> {
        use crate::Operator::*;
        use Value::*;

        match self {
            Quote => Ok(arguments
                .get(0)
                .expect("quote requires one argument")
                .clone()),
            Atom => match arguments
                .get_mut(0)
                .expect("atom requires one argument")
                .eval()?
                .into_value()
            {
                List(_) => Ok(Expression::new(Value::List(vec![]), expr.env.clone())),
                _ => Ok(Expression::new(Value::True, expr.env.clone())),
            },
            Eq => {
                let first = arguments
                    .get_mut(0)
                    .expect("eq requires a first argument")
                    .eval()?;
                let second = arguments
                    .get_mut(1)
                    .expect("eq requires a second argument")
                    .eval()?;
                match (first.into_value(), second.into_value()) {
                    (List(l1), List(l2)) => {
                        if l1.len() == 0 && l2.len() == 0 {
                            Ok(Expression::t())
                        } else {
                            Ok(Expression::nil())
                        }
                    }
                    (v1, v2) => {
                        if v1 == v2 {
                            Ok(Expression::t())
                        } else {
                            Ok(Expression::nil())
                        }
                    }
                }
            }
            Car => {
                let list = arguments
                    .get_mut(0)
                    .expect("car requires an argument")
                    .eval()?;
                match list.value {
                    List(mut vals) => Ok(vals.get(0).expect("cannot car empty list").clone()),
                    _ => panic!("car expects a list, got `{}`", list),
                }
            }
            Cdr => {
                let list = arguments
                    .get_mut(0)
                    .expect("cdr requires an argument")
                    .eval()?;
                match list.value {
                    List(mut vals) => {
                        if vals.len() > 0 {
                            vals.remove(0);
                        }
                        Ok(Expression::new(List(vals), expr.env.clone()))
                    }
                    _ => panic!("cdr expects a list, got `{}`", list),
                }
            }
            Cons => {
                let first = arguments[0].eval()?;
                let list = arguments[1].eval()?;
                match list.value {
                    List(mut vals) => {
                        vals.insert(0, first);
                        Ok(Expression::new(List(vals), expr.env.clone()))
                    }
                    _ => panic!("cons expects a list as its second argument, got `{}`", list),
                }
            }
            Cond => {
                for argument in arguments {
                    match argument.value {
                        List(mut elems) => {
                            let cond = { elems.get_mut(0).expect("cond must have a conditional") };
                            if cond.eval()?.into_value() == Value::True {
                                let val =
                                    { elems.get_mut(1).expect("cond must have a value to eval") };
                                return val.eval();
                            }
                        }
                        _ => panic!("cond must be called on a list, got `{}`", argument),
                    }
                }
                panic!("none of cond was true");
            }
            Label => {
                let sym_exp = arguments
                    .get(0)
                    .expect("label requires an argument for the symbol")
                    .clone()
                    .eval()?;
                let symbol = match sym_exp.into_value() {
                    Symbol(sym) => sym,
                    _ => panic!(
                        "first arg of label must evaluate to a symbol (received `{}`)",
                        arguments.get(0).unwrap()
                    ),
                };
                let assigned_expr = arguments
                    .get(1)
                    .expect("label requires a second argument for the assigned expression")
                    .clone()
                    .eval()?;
                expr.get_env_mut()
                    .assign(symbol.clone(), assigned_expr.clone());
                Ok(assigned_expr)
            }
            Sum => {
                let mut sum = 0.0;
                for mut arg in arguments {
                    match arg.eval()?.into_value() {
                        Number(val) => sum += val,
                        val => panic!("add expects numbers as its arguments (got `{}`)", val),
                    }
                }
                Ok(Expression::new(Value::Number(sum), expr.env.clone()))
            }
            Prod => {
                let mut prod = 1.0;
                for mut arg in arguments {
                    match arg.eval()?.into_value() {
                        Number(val) => prod *= val,
                        val => panic!("prod expects numbers as its arguments (got `{}`)", val),
                    }
                }
                Ok(Expression::new(Value::Number(prod), expr.env.clone()))
            }
            Exp => {
                let base = arguments
                    .get_mut(0)
                    .expect("exp requires a first argument")
                    .eval()?
                    .into_value();
                let exp = arguments
                    .get_mut(1)
                    .expect("exp requires a second argument")
                    .eval()?
                    .into_value();
                match (base, exp) {
                    (Number(base), Number(exp)) => Ok(Expression::new(
                        Value::Number(base.powf(exp)),
                        expr.env.clone(),
                    )),
                    (base, exp) => panic!(
                        "exp requires its arguments to be both numeric (got `{}` and `{}`)",
                        base, exp
                    ),
                }
            }
            Modulo => {
                let val = arguments
                    .get_mut(0)
                    .expect("modulo requires a first argument")
                    .eval()?
                    .into_value();
                let modu = arguments
                    .get_mut(1)
                    .expect("modulo requires a second argument")
                    .eval()?
                    .into_value();
                match (val, modu) {
                    (Number(first), Number(second)) => Ok(Expression::new(
                        Value::Number(first % second),
                        expr.env.clone(),
                    )),
                    (base, exp) => panic!(
                        "modulo requires its arguments to be both numeric (got `{}` and `{}`)",
                        base, exp
                    ),
                }
            }
            Gt => {
                let mut args_evaled = Vec::with_capacity(arguments.len());
                for mut arg in arguments {
                    args_evaled.push(arg.eval()?);
                }
                match args_evaled
                    .iter()
                    .skip(1)
                    .zip(args_evaled.iter())
                    .all(|(g, l)| g > l)
                {
                    true => Ok(Expression::t()),
                    false => Ok(Expression::nil()),
                }
            }
            Ge => {
                let mut args_evaled = Vec::with_capacity(arguments.len());
                for mut arg in arguments {
                    args_evaled.push(arg.eval()?);
                }
                match args_evaled
                    .iter()
                    .skip(1)
                    .zip(args_evaled.iter())
                    .all(|(g, l)| g >= l)
                {
                    true => Ok(Expression::t()),
                    false => Ok(Expression::nil()),
                }
            }
            Type => {
                let arg_type = arguments
                    .get_mut(0)
                    .expect("type requires an argument")
                    .eval()?
                    .into_value()
                    .as_type();
                Ok(Expression::new(arg_type, expr.env.clone()))
            }
            Disp => {
                for mut arg in arguments {
                    print!("{}\n", arg.eval()?);
                }
                Ok(Expression::nil())
            }
        }
    }
}

impl fmt::Display for Operator {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", format!("{:?}", self).to_lowercase().as_str())
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

impl PartialOrd for Expression {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        self.value.partial_cmp(&other.value)
    }
}
