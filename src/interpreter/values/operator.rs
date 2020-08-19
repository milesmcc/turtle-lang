use crate::{
    exp, exp_assert, parse, resolve_resource, CallSnapshot, Environment, Exception,
    ExceptionValue as EV, Expression, Value,
};
use regex::Regex;
use std::fmt;
use std::sync::{Arc, RwLock};

#[derive(Debug, Clone, PartialEq, PartialOrd)]
pub enum Operator {
    Quote,
    Atom,
    Eq,
    Car,
    Cdr,
    Cons,
    Cond,
    Export,
    Let,
    Sum,
    Prod,
    Exp,
    Modulo,
    Gt,
    Ge,
    Type,
    Disp,
    Import,
    Eval,
    While,
    Lambda,
    Macro,
    List,
    Catch,
    Throw,
    Format,
    Parse,
    Length,
    Append,
    Do,
    Floor,
    Rand
}

impl fmt::Display for Operator {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", format!("{:?}", self).to_lowercase().as_str())
    }
}

impl Operator {
    pub fn apply(
        &self,
        snapshot: Arc<RwLock<CallSnapshot>>,
        arguments: Vec<&Expression>,
        expr: &Expression,
        env: Arc<RwLock<Environment>>,
    ) -> Result<Expression, Exception> {
        use crate::Operator::*;
        use Value::*;

        let snap = || snapshot.clone();

        match self {
            Quote => {
                if arguments.len() != 1 {
                    exp!(
                        EV::ArgumentMismatch(arguments.len(), "1".to_string()),
                        snapshot
                    );
                }
                let arg = *arguments.get(0).unwrap();
                Ok(arg.clone())
            }
            Atom => {
                exp_assert!(
                    arguments.len() == 1,
                    EV::ArgumentMismatch(arguments.len(), "1".to_string()),
                    snapshot
                );
                match arguments.get(0).unwrap().eval(snapshot, env)?.into_value() {
                    Value::List(_) => Ok(Expression::new(Value::List(vec![]))),
                    _ => Ok(Expression::new(Value::True)),
                }
            }
            Eq => {
                exp_assert!(
                    arguments.len() > 1,
                    EV::ArgumentMismatch(arguments.len(), "2+".to_string()),
                    snap()
                );

                let mut prev: Option<Expression> = None;
                for argument in arguments {
                    let evaled = argument.eval(snap(), env.clone())?;
                    match &prev {
                        None => prev = Some(evaled),
                        Some(val) => match (evaled.get_value(), val.get_value()) {
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
                Ok(Expression::t())
            }
            Car => {
                exp_assert!(
                    arguments.len() == 1,
                    EV::ArgumentMismatch(arguments.len(), "1".to_string()),
                    snap()
                );

                let list = arguments.get(0).unwrap().eval(snap(), env)?;

                match list.into_value() {
                    Value::List(mut vals) => {
                        exp_assert!(
                            !vals.is_empty(),
                            EV::InvalidArgument,
                            snap(),
                            "cannot `car` an empty list (nil)".to_string()
                        );
                        Ok(vals.remove(0))
                    }
                    val => exp!(
                        EV::InvalidArgument,
                        snap(),
                        format!("`car` expects a list, got `{}`", val)
                    ),
                }
            }
            Cdr => {
                exp_assert!(
                    arguments.len() == 1,
                    EV::ArgumentMismatch(arguments.len(), "1".to_string()),
                    snap()
                );
                let list = arguments.get(0).unwrap().eval(snap(), env)?;
                match list.into_value() {
                    Value::List(mut vals) => {
                        if !vals.is_empty() {
                            vals.remove(0);
                        }
                        Ok(Expression::new(Value::List(vals)))
                    }
                    val => exp!(
                        EV::InvalidArgument,
                        snap(),
                        format!("`cdr` expects a list, not `{}`", val)
                    ),
                }
            }
            Cons => {
                exp_assert!(
                    arguments.len() == 2,
                    EV::ArgumentMismatch(arguments.len(), "2".to_string()),
                    snap()
                );
                let first = arguments.get(0).unwrap().eval(snap(), env.clone())?;
                let list = arguments.get(1).unwrap().eval(snap(), env)?;
                match list.into_value() {
                    Value::List(mut vals) => {
                        vals.insert(0, first);
                        Ok(Expression::new(Value::List(vals)))
                    }
                    val => exp!(
                        EV::InvalidArgument,
                        snap(),
                        format!(
                            "`cons` expects a list as its second argument, got `{}`",
                            val
                        )
                    ),
                }
            }
            Cond => {
                for argument in &arguments {
                    match argument.get_value() {
                        Value::List(elems) => {
                            exp_assert!(
                                elems.len() == 2,
                                EV::InvalidArgument,
                                snap(),
                                format!(
                                    "each `cond` condition must be a list of length two (the given list has {} elements)",
                                    elems.len()
                                )
                            );
                            let cond = { elems.get(0).unwrap() };
                            if cond.eval(snap(), env.clone())? != Expression::nil() {
                                let val = { elems.get(1).unwrap() };
                                return val.eval(snapshot, env);
                            }
                        }
                        val => exp!(
                            EV::InvalidArgument,
                            snap(),
                            format!("`cond` must be called on a list, got `{}`", val)
                        ),
                    }
                }
                Ok(Expression::nil())
            }
            Export | Let => {
                exp_assert!(
                    arguments.len() == 2,
                    EV::ArgumentMismatch(arguments.len(), "2".to_string()),
                    snap()
                );
                let sym_exp = arguments.get(0).unwrap().eval(snap(), env.clone())?;
                let symbol = match sym_exp.into_value() {
                    Symbol(sym) => sym,
                    other => exp!(
                        EV::InvalidArgument,
                        snap(),
                        format!(
                            "first arg of label must evaluate to a symbol (received `{}`)",
                            other
                        )
                    ),
                };

                let assigned_expr = arguments.get(1).unwrap().eval(snap(), env.clone())?;
                env.write().unwrap().assign(
                    symbol,
                    assigned_expr.clone(),
                    !matches!(self, Export),
                    snap(),
                )?;
                Ok(assigned_expr)
            }
            Sum => {
                let mut sum = 0.0;
                for arg in arguments {
                    match arg.eval(snap(), env.clone())?.into_value() {
                        Number(val) => sum += val,
                        val => exp!(
                            EV::InvalidArgument,
                            snap(),
                            format!("`sum` expects numbers as its arguments (got `{}`)", val)
                        ),
                    }
                }
                Ok(Expression::new(Value::Number(sum)))
            }
            Prod => {
                let mut prod = 1.0;
                for arg in arguments {
                    match arg.eval(snap(), env.clone())?.into_value() {
                        Number(val) => prod *= val,
                        val => exp!(
                            EV::InvalidArgument,
                            snap(),
                            format!("`prod` expects numbers as its arguments (got `{}`)", val)
                        ),
                    }
                }
                Ok(Expression::new(Value::Number(prod)))
            }
            Exp => {
                exp_assert!(
                    arguments.len() == 2,
                    EV::ArgumentMismatch(arguments.len(), "2".to_string()),
                    snap()
                );
                let base = arguments
                    .get(0)
                    .unwrap()
                    .eval(snap(), env.clone())?
                    .into_value();
                let exp = arguments.get(1).unwrap().eval(snap(), env)?.into_value();
                match (base, exp) {
                    (Number(base), Number(exp)) => {
                        Ok(Expression::new(Value::Number(base.powf(exp))))
                    }
                    (base, exp) => exp!(
                        EV::InvalidArgument,
                        snap(),
                        format!(
                            "`exp` requires its arguments to be both numeric (got `{}` and `{}`)",
                            base, exp
                        )
                    ),
                }
            }
            Modulo => {
                exp_assert!(
                    arguments.len() == 2,
                    EV::ArgumentMismatch(arguments.len(), "2".to_string()),
                    snap()
                );
                let val = arguments
                    .get(0)
                    .unwrap()
                    .eval(snap(), env.clone())?
                    .into_value();
                let modu = arguments.get(1).unwrap().eval(snap(), env)?.into_value();
                match (val, modu) {
                    (Number(first), Number(second)) => {
                        Ok(Expression::new(Value::Number(first % second)))
                    }
                    (base, exp) => exp!(
                        EV::InvalidArgument,
                        snap(),
                        format!(
                        "`modulo` requires its arguments to be both numeric (got `{}` and `{}`)",
                        base, exp)
                    ),
                }
            }
            Gt => {
                let mut args_evaled = Vec::with_capacity(arguments.len());
                for arg in arguments {
                    args_evaled.push(arg.eval(snap(), env.clone())?);
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
                for arg in arguments {
                    args_evaled.push(arg.eval(snap(), env.clone())?);
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
                exp_assert!(
                    arguments.len() == 1,
                    EV::ArgumentMismatch(arguments.len(), "1".to_string()),
                    snap()
                );
                let arg_type = arguments
                    .get(0)
                    .unwrap()
                    .eval(snap(), env)?
                    .into_value()
                    .as_type();
                Ok(Expression::new(arg_type))
            }
            Disp => {
                for arg in arguments {
                    println!("{}", arg.eval(snap(), env.clone())?);
                }
                Ok(Expression::nil())
            }
            Import => {
                if !(arguments.len() == 1 || arguments.len() == 2) {
                    exp!(
                        EV::ArgumentMismatch(arguments.len(), "1 or 2".to_string()),
                        snapshot
                    );
                }
                let path = match arguments.get(0).unwrap().eval(snap(), env.clone())?.into_value() {
                    Text(val) => val,
                    val => exp!(
                        EV::InvalidArgument,
                        snapshot,
                        format!(
                            "`import` requires the path (:text) as its first argument (got `{}` instead)",
                            val
                        )
                    ),
                };

                let namespace = match arguments.get(1) {
                    Some(val) => match val.eval(snap(), env.clone())?.into_value() {
                        Keyword(val) => Some(val.string_value().clone()),
                        val => exp!(
                            EV::InvalidArgument,
                            snapshot,
                            format!(
                                "`import` requires the namespace (:keyword) as its second argument (got `{}` instead)",
                                val
                            )
                        ),
                    },
                    None => None
                };

                let imported_env = Arc::new(RwLock::new(Environment::root()));
                let exp = resolve_resource(&path, snapshot, expr, imported_env.clone())?;
                env.write().unwrap().add_parent(imported_env, namespace);
                Ok(exp)
            }
            Eval => {
                if arguments.len() != 1 {
                    exp!(
                        EV::ArgumentMismatch(arguments.len(), "1".to_string()),
                        snapshot
                    );
                }
                arguments
                    .get(0)
                    .unwrap()
                    .eval(snap(), env.clone())?
                    .eval(snap(), env)
            }
            While => {
                exp_assert!(
                    arguments.len() >= 2,
                    EV::ArgumentMismatch(arguments.len(), "2+".to_string()),
                    snapshot
                );
                let condition = arguments.get(0).unwrap();
                let mut result = Expression::nil();
                while condition.eval(snap(), env.clone())? != Expression::nil() {
                    for action in arguments.iter().skip(1) {
                        result = action.eval(snap(), env.clone())?
                    }
                }
                Ok(result)
            }
            crate::Operator::Lambda | crate::Operator::Macro => {
                exp_assert!(
                    arguments.len() >= 2,
                    EV::ArgumentMismatch(arguments.len(), "2+".to_string()),
                    snapshot
                );

                let mut collapse_input = true;
                let func_args = match arguments
                    .get(0)
                    .unwrap()
                    .eval(snap(), env.clone())?
                    .into_value()
                {
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
                    func_expressions.push(arg_expr.eval(snap(), env.clone())?);
                }

                (match self {
                    crate::Operator::Lambda => {
                        Expression::new(Value::Lambda(crate::Function::new(
                            func_args,
                            func_expressions,
                            collapse_input,
                            env.clone(),
                        )))
                    }
                    crate::Operator::Macro => Expression::new(Value::Macro(crate::Function::new(
                        func_args,
                        func_expressions,
                        collapse_input,
                        env.clone(),
                    ))),
                    _ => unreachable!(),
                })
                .eval(snap(), env)
            }
            crate::Operator::List => {
                let mut args_evaled = Vec::new();
                for argument in arguments {
                    args_evaled.push(argument.eval(snap(), env.clone())?);
                }
                Ok(Expression::new(Value::List(args_evaled)))
            }
            Catch => {
                exp_assert!(
                    arguments.len() == 2,
                    EV::ArgumentMismatch(arguments.len(), "2".to_string()),
                    snapshot
                );
                let action = arguments.get(0).unwrap().eval(snap(), env.clone());
                let catch_func = arguments.get(1).unwrap().eval(snap(), env.clone())?;
                match action {
                    Ok(exp) => Ok(exp),
                    Err(err) => {
                        // TODO: remove extra clone
                        match catch_func.get_value() {
                            Value::Lambda{..} => Expression::new(Value::List(vec![catch_func.clone(), err.into_value().into_expression()])).eval(snapshot, Arc::new(RwLock::new(Environment::root().with_parent(env, None)))),
                            _ => exp!(
                                EV::InvalidArgument,
                                snapshot,
                                format!("the second argument of `catch` must be a lambda expression (got `{}`)", catch_func)
                            )
                        }
                    }
                }
            }
            Throw => {
                exp_assert!(
                    arguments.len() == 1,
                    EV::ArgumentMismatch(arguments.len(), "1".to_string()),
                    snapshot
                );
                Err(Exception::new(
                    EV::Other(arguments.get(0).unwrap().eval(snap(), env)?),
                    Some(snap()),
                    None,
                ))
            }
            Format => {
                exp_assert!(
                    !arguments.is_empty(),
                    EV::ArgumentMismatch(arguments.len(), "1+".to_string()),
                    snapshot
                );
                let mut literal = match arguments
                    .get(0)
                    .unwrap()
                    .eval(snap(), env.clone())?
                    .into_value()
                {
                    Text(value) => value,
                    other => return Ok(Expression::new(Value::Text(format!("{}", other)))),
                };
                let placeholder = Regex::new(r"\{\}").unwrap(); // todo: make lazy_static?
                let interpolations: Vec<regex::Match> = placeholder.find_iter(&literal).collect();
                exp_assert!(
                    arguments.len() == interpolations.len() + 1,
                    EV::ArgumentMismatch(arguments.len(), format!("{}", interpolations.len() + 1)),
                    snapshot,
                    format!("`{}` has {} placeholders, so {} total arguments are necessary (including the first string literal)", literal, interpolations.len(), interpolations.len() + 1)
                );
                for i in 1..arguments.len() {
                    let replace_with =
                        format!("{}", arguments.get(i).unwrap().eval(snap(), env.clone())?);
                    literal = String::from(placeholder.replace(&literal, replace_with.as_str()));
                }
                Ok(Expression::new(Value::Text(literal)))
            }
            Parse => {
                exp_assert!(
                    arguments.len() == 1,
                    EV::ArgumentMismatch(arguments.len(), "1".to_string()),
                    snapshot
                );
                let value_str = match arguments.get(0).unwrap().eval(snap(), env)?.into_value() {
                    Text(value) => value,
                    other => exp!(
                        EV::InvalidArgument,
                        snapshot,
                        format!("the argument of `parse` must be text (got `{}`)", other)
                    ),
                };
                let mut values = parse(&value_str, "<parse>")?;
                exp_assert!(
                    values.len() == 1,
                    EV::InvalidArgument,
                    snapshot,
                    format!(
                        "`parse` can only handle a single value, not {}",
                        values.len()
                    )
                );
                Ok(values.remove(0))
            }
            Length => {
                exp_assert!(
                    arguments.len() == 1,
                    EV::ArgumentMismatch(arguments.len(), "1".to_string()),
                    snapshot
                );
                match arguments.get(0).unwrap().eval(snap(), env)?.into_value() {
                    Value::List(vals) => Ok(Expression::new(Value::Number(vals.len() as f64))),
                    other => exp!(
                        EV::InvalidArgument,
                        snapshot,
                        format!(
                            "length expects a list as its first argument (got `{}`)",
                            other
                        )
                    ),
                }
            }
            Append => {
                exp_assert!(
                    !arguments.is_empty(),
                    EV::ArgumentMismatch(arguments.len(), "1+".to_string()),
                    snapshot
                );
                let mut new_list: Vec<Expression> = Vec::with_capacity(arguments.len());
                for argument in arguments {
                    match argument.eval(snap(), env.clone())?.into_value() {
                        Value::List(values) => new_list.extend(values),
                        other => exp!(
                            EV::InvalidArgument,
                            snapshot,
                            format!(
                                "append requires all its arguments to be a list (got `{}`)",
                                other
                            )
                        ),
                    }
                }
                Ok(Expression::new(Value::List(new_list)))
            }
            Do => {
                exp_assert!(
                    !arguments.is_empty(),
                    EV::ArgumentMismatch(arguments.len(), "1+".to_string()),
                    snapshot
                );

                let mut result = Expression::nil();
                for argument in arguments {
                    result = argument.eval(snap(), env.clone())?;
                }
                Ok(result)
            }
            Floor => {
                exp_assert!(
                    arguments.len() == 1,
                    EV::ArgumentMismatch(arguments.len(), "1".to_string()),
                    snapshot
                );
                match arguments
                    .get(0)
                    .unwrap()
                    .eval(snap(), env.clone())?
                    .into_value()
                {
                    Number(val) => Ok(Expression::new(Value::Number(val.floor()))),
                    val => exp!(
                        EV::InvalidArgument,
                        snap(),
                        format!("`floor` expects a number as its argument (got `{}`)", val)
                    ),
                }
            },
            Rand => {
                exp_assert!(
                    arguments.len() == 0,
                    EV::ArgumentMismatch(arguments.len(), "0".to_string()),
                    snapshot
                );
                Ok(Expression::new(Value::Number(rand::random())))
            },
        }
    }
}
