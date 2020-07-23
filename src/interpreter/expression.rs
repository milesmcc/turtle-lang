use std::fmt;
use std::sync::{Arc, RwLock};

use crate::{
    exp, exp_assert, CallSnapshot, Environment, Exception, ExceptionValue as EV, SourcePosition,
    Value,
};

#[derive(Debug, Clone)]
pub struct Expression {
    value: Value,
    source: Option<SourcePosition>,
}

impl PartialEq for Expression {
    fn eq(&self, other: &Self) -> bool {
        // TODO: do we need to check whether the environments are the same?
        self.value == other.value
    }
}

impl Expression {
    pub fn new(value: Value) -> Self {
        Self {
            value,
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
            source: None,
        }
    }

    pub fn t() -> Self {
        Self {
            value: Value::True,
            source: None,
        }
    }

    pub fn get_value(&self) -> &Value {
        &self.value
    }

    pub fn into_value(self) -> Value {
        self.value
    }

    pub fn source(&self) -> &'_ Option<SourcePosition> {
        &self.source
    }

    pub fn eval(
        &mut self,
        parent_snapshot: Arc<RwLock<CallSnapshot>>,
        env: Arc<RwLock<Environment>>
    ) -> Result<Self, Exception> {
        use Value::*;

        let snapshot = CallSnapshot::new(&self, &parent_snapshot)?;

        let snap = || snapshot.clone();

        match self.value.clone() {
            List(vals) => {
                if !vals.is_empty() {
                    let mut operator = vals.get(0).unwrap().clone();
                    let arguments: Vec<Expression> = vals.iter().skip(1).cloned().collect();
                    match &operator.value {
                        Operator(operand) => operand.apply(snapshot, arguments, self, env),
                        List(_) | Symbol(_) => {
                            let evaled_operator = operator.eval(snap(), env.clone())?;
                            let mut new_list = vec![evaled_operator];
                            for arg in arguments {
                                new_list.push(arg);
                            }
                            Expression::new(Value::List(new_list)).eval(snap(), env)
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
                            let scoped_env = Environment::with_parent(env.clone());
                            let mut scoped_env_lock = scoped_env.write().unwrap();

                            if *collapse_input {
                                let sym = params.get(0).unwrap(); // this unwrap will always be ok; it is enforced by the parser
                                let args_evaled = {
                                    let mut list = Vec::new();
                                    for arg_expr in arguments {
                                        list.push(match &operator.value {
                                            Lambda { .. } => arg_expr.clone().eval(snap(), env.clone())?,
                                            Macro { .. } => arg_expr.clone(),
                                            _ => unreachable!(),
                                        });
                                    }
                                    list
                                };
                                let arg =
                                    Expression::new(Value::List(args_evaled));
                                for _exp in expressions.clone() {
                                    scoped_env_lock.assign(sym.clone(), arg.clone(), true);
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
                                        Lambda { .. } => arg_expr.clone().eval(snap(), env.clone())?,
                                        Macro { .. } => arg_expr.clone(),
                                        _ => unreachable!(),
                                    };
                                    for _exp in expressions.clone() {
                                        scoped_env_lock.assign(
                                            symbol.clone(),
                                            arg_evaled.clone(),
                                            true,
                                        );
                                    }
                                }
                            }
                            let mut result = Expression::nil();
                            for mut exp in expressions.clone() {
                                result = exp.eval(snap(), scoped_env.clone())?;
                            }
                            Ok(result)
                        }
                        val => exp!(EV::InvalidOperator(val.clone()), snapshot),
                    }
                } else {
                    Ok(self.clone())
                }
            }
            Symbol(sym) => match env.read().expect("unable to access environment (are threads locked?)").lookup(&sym) {
                Some(exp) => Ok(exp),
                None => exp!(EV::UndefinedSymbol(sym.clone()), snapshot),
            },
            _ => Ok(self.clone()),
        }
    }
}

impl fmt::Display for Expression {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.value)
    }
}

impl PartialOrd for Expression {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        self.value.partial_cmp(&other.value)
    }
}
