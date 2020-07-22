use crate::{
    exp, exp_assert, resolve_resource, CallSnapshot, Exception, ExceptionValue as EV, Expression,
    Value,
};
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

impl fmt::Display for Operator {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", format!("{:?}", self).to_lowercase().as_str())
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
                        EV::ArgumentMismatch(arguments.len(), "1".to_string()),
                        snapshot
                    );
                }
                Ok(arguments.get(0).unwrap().clone())
            }
            Atom => {
                exp_assert!(
                    arguments.len() == 1,
                    EV::ArgumentMismatch(arguments.len(), "1".to_string()),
                    snapshot
                );
                match arguments.get_mut(0).unwrap().eval(snapshot)?.into_value() {
                    Value::List(_) => Ok(Expression::new(Value::List(vec![]), expr.clone_env())),
                    _ => Ok(Expression::new(Value::True, expr.clone_env())),
                }
            }
            Eq => {
                exp_assert!(
                    arguments.len() > 1,
                    EV::ArgumentMismatch(arguments.len(), "2+".to_string()),
                    snap()
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
                    EV::ArgumentMismatch(arguments.len(), "1".to_string()),
                    snap()
                );

                let list = arguments.get_mut(0).unwrap().eval(snap())?;

                match list.into_value() {
                    Value::List(vals) => {
                        exp_assert!(
                            vals.len() > 0,
                            EV::InvalidArgument,
                            snap(),
                            format!("cannot `car` an empty list (nil)")
                        );
                        Ok(vals.get(0).unwrap().clone())
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
                let list = arguments.get_mut(0).unwrap().eval(snap())?;
                match list.into_value() {
                    Value::List(mut vals) => {
                        if !vals.is_empty() {
                            vals.remove(0);
                        }
                        Ok(Expression::new(Value::List(vals), expr.clone_env()))
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
                let first = arguments.get_mut(0).unwrap().eval(snap())?;
                let list = arguments.get_mut(1).unwrap().eval(snap())?;
                match list.into_value() {
                    Value::List(mut vals) => {
                        vals.insert(0, first);
                        Ok(Expression::new(Value::List(vals), expr.clone_env()))
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
                for argument in arguments {
                    match argument.into_value() {
                        Value::List(mut elems) => {
                            exp_assert!(
                                elems.len() == 2,
                                EV::InvalidArgument,
                                snap(),
                                format!(
                                    "each `cond` condition must be a list of length two (the given list has {} elements)",
                                    elems.len()
                                )
                            );
                            let cond = { elems.get_mut(0).unwrap() };
                            if cond.eval(snap())? != Expression::nil() {
                                let val = { elems.get_mut(1).unwrap() };
                                return val.eval(snapshot);
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
            Label => {
                exp_assert!(
                    arguments.len() == 2,
                    EV::ArgumentMismatch(arguments.len(), "2".to_string()),
                    snap()
                );
                let sym_exp = arguments.get(0).unwrap().clone().eval(snap())?;
                let symbol = match sym_exp.into_value() {
                    Symbol(sym) => sym,
                    _ => exp!(
                        EV::InvalidArgument,
                        snap(),
                        format!(
                            "first arg of label must evaluate to a symbol (received `{}`)",
                            arguments.get(0).unwrap()
                        )
                    ),
                };

                let assigned_expr = arguments.get_mut(1).unwrap().eval(snap())?;
                expr.get_env_mut()
                    .assign(symbol, assigned_expr.clone(), false);
                Ok(assigned_expr)
            }
            Sum => {
                let mut sum = 0.0;
                for mut arg in arguments {
                    match arg.eval(snap())?.into_value() {
                        Number(val) => sum += val,
                        val => exp!(
                            EV::InvalidArgument,
                            snap(),
                            format!("`sum` expects numbers as its arguments (got `{}`)", val)
                        ),
                    }
                }
                Ok(Expression::new(Value::Number(sum), expr.clone_env()))
            }
            Prod => {
                let mut prod = 1.0;
                for mut arg in arguments {
                    match arg.eval(snap())?.into_value() {
                        Number(val) => prod *= val,
                        val => exp!(
                            EV::InvalidArgument,
                            snap(),
                            format!("`prod` expects numbers as its arguments (got `{}`)", val)
                        ),
                    }
                }
                Ok(Expression::new(Value::Number(prod), expr.clone_env()))
            }
            Exp => {
                exp_assert!(
                    arguments.len() == 2,
                    EV::ArgumentMismatch(arguments.len(), "2".to_string()),
                    snap()
                );
                let base = arguments.get_mut(0).unwrap().eval(snap())?.into_value();
                let exp = arguments.get_mut(1).unwrap().eval(snap())?.into_value();
                match (base, exp) {
                    (Number(base), Number(exp)) => Ok(Expression::new(
                        Value::Number(base.powf(exp)),
                        expr.clone_env(),
                    )),
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
                let val = arguments.get_mut(0).unwrap().eval(snap())?.into_value();
                let modu = arguments.get_mut(1).unwrap().eval(snap())?.into_value();
                match (val, modu) {
                    (Number(first), Number(second)) => Ok(Expression::new(
                        Value::Number(first % second),
                        expr.clone_env(),
                    )),
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
                exp_assert!(
                    arguments.len() == 1,
                    EV::ArgumentMismatch(arguments.len(), "1".to_string()),
                    snap()
                );
                let arg_type = arguments
                    .get_mut(0)
                    .unwrap()
                    .eval(snap())?
                    .into_value()
                    .as_type();
                Ok(Expression::new(arg_type, expr.clone_env()))
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
                        EV::ArgumentMismatch(arguments.len(), "1".to_string()),
                        snapshot
                    );
                }
                let path = match arguments.get_mut(0).unwrap().eval(snap())?.into_value() {
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
                        EV::ArgumentMismatch(arguments.len(), "1".to_string()),
                        snapshot
                    );
                }
                arguments.get_mut(0).unwrap().eval(snap())?.eval(snap())
            }
            While => {
                exp_assert!(
                    arguments.len() == 2,
                    EV::ArgumentMismatch(arguments.len(), "2".to_string()),
                    snapshot
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
                    EV::ArgumentMismatch(arguments.len(), "2".to_string()),
                    snapshot
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
