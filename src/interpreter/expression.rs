use std::fmt;
use std::sync::{Arc, RwLock, RwLockReadGuard, RwLockWriteGuard};

use crate::{
    exp, exp_assert, CallSnapshot, Environment, Exception, ExceptionValue as EV, SourcePosition,
    Value,
};

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

        let snapshot = CallSnapshot::new(&self, &parent_snapshot)?;

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
                                    EV::ArgumentMismatch(
                                        arguments.len(),
                                        format!("{}", params.len())
                                    ),
                                    snap()
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

    pub fn get_env(&self) -> RwLockReadGuard<Environment<'a>> {
        self.env
            .read()
            .expect("unable to access environment (are threads locked?)")
    }

    pub fn get_env_mut(&mut self) -> RwLockWriteGuard<Environment<'a>> {
        self.env
            .write()
            .expect("unable to mutably access environment (are threads locked?)")
    }
}

impl<'a> fmt::Display for Expression<'a> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.value)
    }
}

impl<'a> PartialOrd for Expression<'a> {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        self.value.partial_cmp(&other.value)
    }
}
