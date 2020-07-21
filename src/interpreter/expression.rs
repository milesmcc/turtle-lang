use std::fmt;
use std::sync::{Arc, RwLock, RwLockReadGuard, RwLockWriteGuard};

use crate::{
    exp, exp_assert, resolve_resource, CallSnapshot, Environment, Exception, ExceptionValue as EV,
    SourcePosition,
};

#[derive(Debug, Clone, PartialEq, PartialOrd, Eq, Hash)]
pub struct Symbol(String);

impl Symbol {
    pub fn new(val: String) -> Self {
        Self(val)
    }

    pub fn from_str(val: &str) -> Self {
        Self(String::from(val))
    }

    pub fn string_value(&self) -> &'_ String {
        &self.0
    }
}

#[derive(Debug, Clone, PartialEq, PartialOrd, Eq, Hash)]
pub struct Keyword(String);

impl Keyword {
    pub fn new(val: String) -> Self {
        Self(val)
    }

    pub fn from_str(val: &str) -> Self {
        Self(String::from(val))
    }

    pub fn string_value(&self) -> &'_ String {
        &self.0
    }
}

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
    Include,
    Eval,
    While,
    Lambda,
    Macro,
    List,
}

#[derive(Debug, Clone, PartialEq, PartialOrd)]
pub enum Value<'a> {
    List(Vec<Expression<'a>>),
    Number(f64),
    Text(String),
    Keyword(Keyword),
    Symbol(Symbol),
    True,

    // Primitive (axiomatic) operators
    Operator(Operator),

    Lambda {
        params: Vec<Symbol>,
        expressions: Vec<Expression<'a>>,
        collapse_input: bool,
    },
    Macro {
        params: Vec<Symbol>,
        expressions: Vec<Expression<'a>>,
        collapse_input: bool,
    },
}

impl<'a> Value<'a> {
    pub fn as_type(&self) -> Self {
        use Value::*;

        Value::Keyword(crate::Keyword::new(match self {
            List(_) => "list".to_string(),
            Number(_) => "number".to_string(),
            Text(_) => "text".to_string(),
            Keyword(_) => "keyword".to_string(),
            Symbol(_) => "symbol".to_string(),
            Operator(_) => "operator".to_string(),
            Lambda { .. } => "lambda".to_string(),
            Macro { .. } => "macro".to_string(),
            _ => "unknown".to_string(),
        }))
    }
}

#[derive(Debug, Clone)]
pub struct Expression<'a> {
    value: Value<'a>,
    env: Arc<RwLock<Environment<'a>>>,
    source: Option<SourcePosition>,
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
            value,
            env,
            source: None,
        }
    }

    pub fn with_source(mut self, source_position: SourcePosition) -> Self {
        self.source = Some(source_position);
        self
    }

    pub fn nil() -> Self {
        Self {
            value: Value::List(vec![]),
            env: Arc::new(RwLock::new(Environment::root())),
            source: None,
        }
    }

    pub fn clone_env(&self) -> Arc<RwLock<Environment<'a>>> {
        self.env.clone()
    }

    pub fn t() -> Self {
        Self {
            value: Value::True,
            env: Arc::new(RwLock::new(Environment::root())),
            source: None,
        }
    }

    pub fn get_value(&'a self) -> &Value {
        &self.value
    }

    pub fn into_value(self) -> Value<'a> {
        self.value
    }

    pub fn source(&self) -> &'_ Option<SourcePosition> {
        &self.source
    }

    pub fn eval(
        &mut self,
        parent_snapshot: Arc<RwLock<CallSnapshot<'a>>>,
    ) -> Result<Self, Exception<'a>> {
        use Value::*;

        let snapshot = CallSnapshot::new(&self, &parent_snapshot);

        let snap = || snapshot.clone();

        match self.value.clone() {
            List(vals) => {
                if !vals.is_empty() {
                    let mut operator = vals.get(0).unwrap().clone();
                    let arguments: Vec<Expression<'a>> = vals.iter().skip(1).cloned().collect();
                    match &operator.value {
                        Operator(operand) => operand.apply(snapshot, arguments, self),
                        List(_) | Symbol(_) => {
                            let evaled_operator = operator.eval(snap())?;
                            let mut new_list = vec![evaled_operator];
                            for arg in arguments {
                                new_list.push(arg);
                            }
                            Expression::new(Value::List(new_list), self.env.clone()).eval(snap())
                        }
                        Lambda {
                            params,
                            expressions,
                            collapse_input,
                        }
                        | Macro {
                            params,
                            expressions,
                            collapse_input,
                        } => {
                            if *collapse_input {
                                let sym = params.get(0).unwrap(); // this unwrap will always be ok; it is enforced by the parser
                                let args_evaled = {
                                    let mut list = Vec::new();
                                    for arg_expr in arguments {
                                        list.push(match &operator.value {
                                            Lambda { .. } => arg_expr.clone().eval(snap())?,
                                            Macro { .. } => arg_expr.clone(),
                                            _ => unreachable!(),
                                        });
                                    }
                                    list
                                };
                                let arg =
                                    Expression::new(Value::List(args_evaled), self.env.clone());
                                for mut exp in expressions.clone() {
                                    exp.get_env_mut().assign(sym.clone(), arg.clone(), true);
                                }
                            } else {
                                exp_assert!(
                                    params.len() == arguments.len(),
                                    EV::ArgumentMismatch,
                                    snap(),
                                    format!(
                                        "{} argument{} required, but {} given",
                                        params.len(),
                                        match params.len() {
                                            1 => "",
                                            _ => "s",
                                        },
                                        arguments.len()
                                    )
                                );
                                for (symbol, arg_expr) in params.iter().zip(arguments.iter()) {
                                    let arg_evaled = match &operator.value {
                                        Lambda { .. } => arg_expr.clone().eval(snap())?,
                                        Macro { .. } => arg_expr.clone(),
                                        _ => unreachable!(),
                                    };
                                    for mut exp in expressions.clone() {
                                        exp.get_env_mut().assign(
                                            symbol.clone(),
                                            arg_evaled.clone(),
                                            true,
                                        );
                                    }
                                }
                            }
                            let mut result = Expression::nil();
                            for mut exp in expressions.clone() {
                                result = exp.eval(snap())?;
                            }
                            Ok(result)
                        }
                        val => exp!(EV::InvalidOperator(val.clone()), snapshot),
                    }
                } else {
                    Ok(self.clone())
                }
            }
            Symbol(sym) => match self.get_env().lookup(&sym) {
                Some(exp) => Ok(exp),
                None => exp!(EV::UndefinedSymbol(sym.clone()), snapshot),
            },
            _ => Ok(self.clone()),
        }
    }

    fn get_env(&self) -> RwLockReadGuard<Environment<'a>> {
        self.env
            .read()
            .expect("unable to access environment (are threads locked?)")
    }

    fn get_env_mut(&mut self) -> RwLockWriteGuard<Environment<'a>> {
        self.env
            .write()
            .expect("unable to mutably access environment (are threads locked?)")
    }
}

impl Operator {
    pub fn apply<'a>(
        &self,
        snapshot: Arc<RwLock<CallSnapshot<'a>>>,
        mut arguments: Vec<Expression<'a>>,
        expr: &mut Expression<'a>,
    ) -> Result<Expression<'a>, Exception<'a>> {
        use crate::Operator::*;
        use Value::*;

        let snap = || snapshot.clone();

        match self {
            Quote => {
                if arguments.len() != 1 {
                    exp!(
                        EV::ArgumentMismatch,
                        snapshot,
                        format!("`quote` requires 1 argument (received {})", arguments.len())
                    );
                }
                Ok(arguments.get(0).unwrap().clone())
            }
            Atom => {
                exp_assert!(
                    arguments.len() == 1,
                    EV::ArgumentMismatch,
                    snapshot,
                    format!("`atom` requires 1 argument (received {})", arguments.len())
                );
                match arguments.get_mut(0).unwrap().eval(snapshot)?.into_value() {
                    Value::List(_) => Ok(Expression::new(Value::List(vec![]), expr.env.clone())),
                    _ => Ok(Expression::new(Value::True, expr.env.clone())),
                }
            }
            Eq => {
                exp_assert!(
                    arguments.len() > 1,
                    EV::ArgumentMismatch,
                    snap(),
                    format!("2+ arguments required, but {} given", arguments.len())
                );

                let mut prev: Option<Expression> = None;
                for mut argument in arguments {
                    let evaled = argument.eval(snap())?;
                    match &prev {
                        None => prev = Some(evaled.clone()),
                        Some(val) => match (evaled.into_value(), val.clone().into_value()) {
                            (Value::List(l1), Value::List(l2)) => {
                                if !(l1.is_empty() && l2.is_empty()) {
                                    return Ok(Expression::nil());
                                }
                            }
                            (v1, v2) => {
                                if v1 != v2 {
                                    return Ok(Expression::nil());
                                }
                            }
                        },
                    }
                }
                return Ok(Expression::t());
            }
            Car => {
                exp_assert!(
                    arguments.len() == 1,
                    EV::ArgumentMismatch,
                    snap(),
                    format!("`car` requires one argument (got {})", arguments.len())
                );

                let list = arguments.get_mut(0).unwrap().eval(snap())?;

                match list.value {
                    Value::List(vals) => {
                        exp_assert!(
                            vals.len() > 0,
                            EV::InvalidArgument,
                            snap(),
                            format!("cannot `car` an empty list (nil)")
                        );
                        Ok(vals.get(0).unwrap().clone())
                }
                    _ => exp!(
                        EV::InvalidArgument,
                        snap(),
                        format!("`car` expects a list, got `{}`", list)
                    ),
            }
            }
            Cdr => {
                exp_assert!(
                    arguments.len() == 1,
                    EV::ArgumentMismatch,
                    snap(),
                    format!("`cdr` requires 1 argument, but {} given", arguments.len())
                );
                let list = arguments.get_mut(0).unwrap().eval(snap())?;
                match list.value {
                    Value::List(mut vals) => {
                        if !vals.is_empty() {
                            vals.remove(0);
                        }
                        Ok(Expression::new(Value::List(vals), expr.env.clone()))
                    }
                    val => exp!(
                        EV::InvalidArgument,
                        snap(),
                        format!("`cdr` expects a list, not `{}`", val)
                    ),
                }
            }
            Cons => {
                let first = arguments[0].eval(snap())?;
                let list = arguments[1].eval(snap())?;
                match list.value {
                    Value::List(mut vals) => {
                        vals.insert(0, first);
                        Ok(Expression::new(Value::List(vals), expr.env.clone()))
                    }
                    _ => panic!(
                        "`cons` expects a list as its second argument, got `{}`",
                        list
                    ),
                }
            }
            Cond => {
                for argument in arguments {
                    match argument.value {
                        Value::List(mut elems) => {
                            let cond =
                                { elems.get_mut(0).expect("`cond` must have a conditional") };
                            if cond.eval(snap())?.into_value() == Value::True {
                                let val =
                                    { elems.get_mut(1).expect("`cond` must have a value to eval") };
                                return val.eval(snapshot);
                            }
                        }
                        _ => panic!("`cond` must be called on a list, got `{}`", argument),
                    }
                }
                Ok(Expression::nil())
            }
            Label => {
                let sym_exp = arguments
                    .get(0)
                    .expect("`label` requires an argument for the symbol")
                    .clone()
                    .eval(snap())?;
                let symbol = match sym_exp.into_value() {
                    Symbol(sym) => sym,
                    _ => panic!(
                        "first arg of label must evaluate to a symbol (received `{}`)",
                        arguments.get(0).unwrap()
                    ),
                };
                let assigned_expr = arguments
                    .get(1)
                    .expect("`label` requires a second argument for the assigned expression")
                    .clone()
                    .eval(snap())?;
                expr.get_env_mut()
                    .assign(symbol, assigned_expr.clone(), false);
                Ok(assigned_expr)
            }
            Sum => {
                let mut sum = 0.0;
                for mut arg in arguments {
                    match arg.eval(snap())?.into_value() {
                        Number(val) => sum += val,
                        val => panic!("`add` expects numbers as its arguments (got `{}`)", val),
                    }
                }
                Ok(Expression::new(Value::Number(sum), expr.env.clone()))
            }
            Prod => {
                let mut prod = 1.0;
                for mut arg in arguments {
                    match arg.eval(snap())?.into_value() {
                        Number(val) => prod *= val,
                        val => panic!("`prod` expects numbers as its arguments (got `{}`)", val),
                    }
                }
                Ok(Expression::new(Value::Number(prod), expr.env.clone()))
            }
            Exp => {
                let base = arguments
                    .get_mut(0)
                    .expect("`exp` requires a first argument")
                    .eval(snap())?
                    .into_value();
                let exp = arguments
                    .get_mut(1)
                    .expect("`exp` requires a second argument")
                    .eval(snap())?
                    .into_value();
                match (base, exp) {
                    (Number(base), Number(exp)) => Ok(Expression::new(
                        Value::Number(base.powf(exp)),
                        expr.env.clone(),
                    )),
                    (base, exp) => panic!(
                        "`exp` requires its arguments to be both numeric (got `{}` and `{}`)",
                        base, exp
                    ),
                }
            }
            Modulo => {
                let val = arguments
                    .get_mut(0)
                    .expect("`modulo` requires a first argument")
                    .eval(snap())?
                    .into_value();
                let modu = arguments
                    .get_mut(1)
                    .expect("`modulo` requires a second argument")
                    .eval(snap())?
                    .into_value();
                match (val, modu) {
                    (Number(first), Number(second)) => Ok(Expression::new(
                        Value::Number(first % second),
                        expr.env.clone(),
                    )),
                    (base, exp) => panic!(
                        "`modulo` requires its arguments to be both numeric (got `{}` and `{}`)",
                        base, exp
                    ),
                }
            }
            Gt => {
                let mut args_evaled = Vec::with_capacity(arguments.len());
                for mut arg in arguments {
                    args_evaled.push(arg.eval(snap())?);
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
                    args_evaled.push(arg.eval(snap())?);
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
                    .expect("`type` requires an argument")
                    .eval(snap())?
                    .into_value()
                    .as_type();
                Ok(Expression::new(arg_type, expr.env.clone()))
            }
            Disp => {
                for mut arg in arguments {
                    println!("{}", arg.eval(snap())?);
                }
                Ok(Expression::nil())
            }
            Include => {
                if arguments.len() != 1 {
                    exp!(
                        EV::ArgumentMismatch,
                        snapshot,
                        format!(
                            "`include` requires 1 argument (received {})",
                            arguments.len()
                        )
                    );
                }
                let path = match arguments.get_mut(0).unwrap().eval(snap())?.value {
                    Text(val) => val,
                    val => exp!(
                        EV::InvalidArgument,
                        snapshot,
                        format!(
                            "`include` requires the path (:text) as its argument (got `{}` instead)",
                            val
                        )
                    ),
                };

                resolve_resource(&path, snapshot, expr)
            }
            Eval => {
                if arguments.len() != 1 {
                    exp!(
                        EV::ArgumentMismatch,
                        snapshot,
                        format!("`eval` requires 1 argument (received {})", arguments.len())
                    );
                }
                arguments.get_mut(0).unwrap().eval(snap())?.eval(snap())
            }
            While => {
                exp_assert!(
                    arguments.len() == 2,
                    EV::ArgumentMismatch,
                    snapshot,
                    format!(
                        "`while` requires 2 arguments (received {})",
                        arguments.len()
                    )
                );
                let condition = arguments.get(0).unwrap();
                let action = arguments.get(1).unwrap();
                let mut result = Expression::nil();
                while condition.clone().eval(snap())? != Expression::nil() {
                    result = action.clone().eval(snap())?
                }
                Ok(result)
            }
            crate::Operator::Lambda | crate::Operator::Macro => {
                exp_assert!(
                    arguments.len() >= 2,
                    EV::ArgumentMismatch,
                    snapshot,
                    format!(
                        "requires at least 2 arguments (received {})",
                        arguments.len()
                    )
                );

                let mut collapse_input = true;
                let func_args = match arguments.get_mut(0).unwrap().eval(snap())?.into_value() {
                    Value::List(vals) => {
                        collapse_input = false;
                        let mut symbols = Vec::new();
                        for val in vals {
                            match val.into_value() {
                            Value::Symbol(sym) => symbols.push(sym),
                            other => exp!(EV::InvalidArgument, snapshot, format!("each item in the first argument (a list) must be a symbol (got `{}`)", other)),
                        }
                        }
                        symbols
                    }
                    Value::Symbol(sym) => vec![sym],
                    val => exp!(
                        EV::InvalidArgument,
                        snapshot,
                        format!(
                            "the first argument must only evaluate to symbol(s) (got `{}`)",
                            val
                        )
                    ),
                };
                let mut func_expressions = Vec::new();
                for arg_expr in arguments.iter().skip(1) {
                    func_expressions.push(arg_expr.clone().eval(snap())?);
                }

                (match self {
                    crate::Operator::Lambda => Expression::new(
                        Value::Lambda {
                            params: func_args,
                            expressions: func_expressions,
                            collapse_input,
                        },
                        expr.clone_env(),
                    ),
                    crate::Operator::Macro => Expression::new(
                        Value::Macro {
                            params: func_args,
                            expressions: func_expressions,
                            collapse_input,
                        },
                        expr.clone_env(),
                    ),
                    _ => unreachable!(),
                })
                .eval(snap())
            }
            crate::Operator::List => {
                let mut args_evaled = Vec::new();
                for mut argument in arguments {
                    args_evaled.push(argument.eval(snap())?);
                }
                Ok(Expression::new(Value::List(args_evaled), expr.clone_env()))
            }
        }
    }
}

impl fmt::Display for Operator {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", format!("{:?}", self).to_lowercase().as_str())
    }
}

impl fmt::Display for Keyword {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, ":{}", self.string_value())
    }
}

impl fmt::Display for Symbol {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.string_value())
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
            Text(val) => write!(f, "\"{}\"", val),
            Symbol(val) => write!(f, "{}", val),
            Keyword(val) => write!(f, "{}", val),
            True => write!(f, "true"),
            Lambda {
                params,
                expressions,
                collapse_input,
            } => write!(
                f,
                "<lambda {}{}{} -> {}>",
                match collapse_input {
                    true => "",
                    false => "(",
                },
                params
                    .iter()
                    .map(|x| format!("{}", x))
                    .collect::<Vec<String>>()
                    .join(" "),
                match collapse_input {
                    true => "",
                    false => ")",
                },
                expressions
                    .iter()
                    .map(|x| format!("{}", x))
                    .collect::<Vec<String>>()
                    .join(" ")
            ),
            Macro {
                params,
                expressions,
                collapse_input,
            } => write!(
                f,
                "<macro {}{}{} -> {}>",
                match collapse_input {
                    true => "",
                    false => "(",
                },
                params
                    .iter()
                    .map(|x| format!("{}", x))
                    .collect::<Vec<String>>()
                    .join(" "),
                match collapse_input {
                    true => "",
                    false => ")",
                },
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
