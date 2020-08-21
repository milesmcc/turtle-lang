use std::fmt;
use std::sync::mpsc;
use std::sync::RwLockReadGuard;
use crate::Locker;
use std::thread;

use crate::{
    exp, exp_assert, CallSnapshot, Environment, Exception, ExceptionValue as EV, SourcePosition,
    Value,
};

#[derive(Debug, Clone)]
pub struct Expression {
    value: Locker<Value>,
    source: Option<SourcePosition>,
}

impl PartialEq for Expression {
    fn eq(&self, other: &Self) -> bool {
        // TODO: do we need to check whether the environments are the same?
        self.value.read().unwrap().eq(&other.value.read().unwrap())
    }
}

impl Expression {
    pub fn new(value: Value) -> Self {
        Self {
            value: Locker::new(value),
            source: None,
        }
    }

    pub fn with_source(mut self, source_position: SourcePosition) -> Self {
        self.source = Some(source_position);
        self
    }

    pub fn nil() -> Self {
        Self::new(Value::List(vec![]))
    }

    pub fn t() -> Self {
        Self::new(Value::True)
    }

    pub fn value(&self) -> Locker<Value> {
        self.value.clone()
    }

    pub fn into_value(self) -> Locker<Value> {
        self.value
    }

    pub fn source(&self) -> &'_ Option<SourcePosition> {
        &self.source
    }

    pub fn eval_async(
        self,
        parent_snapshot: Locker<CallSnapshot>,
        env: Locker<Environment>,
    ) -> Result<mpsc::Receiver<Result<Self, Exception>>, Exception> {
        let exp = self;
        let snap = parent_snapshot.clone();
        // TODO: do this without clone
        let (sender, receiver) = mpsc::channel::<Result<Self, Exception>>();
        if let Ok(th) = thread::Builder::new()
            .stack_size(64 * 1024 * 1024)
            .spawn(move || {
                sender.send(exp.eval(snap, env)).unwrap();
            })
        {
            match th.join() {
                Ok(_) => {}
                Err(_) => {
                    return Err(Exception::new(
                        EV::Concurrency,
                        Some(parent_snapshot),
                        Some("could not wait for thread to finish executing".to_string()),
                    ))
                }
            }
        };
        Ok(receiver)
    }

    pub fn eval(
        &self,
        parent_snapshot: Locker<CallSnapshot>,
        env: Locker<Environment>,
    ) -> Result<Self, Exception> {
        use Value::*;

        let snapshot = CallSnapshot::new(&self, &parent_snapshot)?;

        let snap = || snapshot.clone();

        match &*self.value.read().unwrap() {
            List(vals) => {
                if !vals.is_empty() {
                    let operator = vals.get(0).unwrap();
                    let arguments: Vec<&Expression> = vals.iter().skip(1).collect();
                    match &*operator.value.read().unwrap() {
                        Operator(operand) => operand.apply(snapshot, arguments, self, env),
                        List(_) | Symbol(_) => {
                            let evaled_operator = operator.eval(snap(), env.clone())?;
                            let mut new_list = vec![evaled_operator];
                            for arg in arguments {
                                new_list.push(arg.clone());
                            }
                            Expression::new(Value::List(new_list)).eval(snap(), env)
                        }
                        Lambda(function) | Macro(function) => {
                            let mut scoped_env = match *operator.value.read().unwrap() {
                                Lambda(_) => Environment::root()
                                    .with_parent(function.lexical_scope.clone(), None),
                                Macro(_) => {
                                    let mut env =
                                        Environment::root().with_parent(env.clone(), None);
                                    env.add_parent(function.lexical_scope.clone(), None);
                                    env
                                }
                                _ => unreachable!(),
                            };

                            if function.collapse_input {
                                let sym = function.params.get(0).unwrap(); // this unwrap will always be ok; it is enforced by the parser
                                let args_evaled = {
                                    let mut list = Vec::new();
                                    for arg_expr in arguments {
                                        list.push(match *operator.value.read().unwrap() {
                                            Lambda { .. } => arg_expr.eval(snap(), env.clone())?,
                                            Macro { .. } => arg_expr.clone(),
                                            _ => unreachable!(),
                                        });
                                    }
                                    list
                                };
                                let arg = Expression::new(Value::List(args_evaled));
                                scoped_env.assign(sym.clone(), arg, true, snap())?;
                            } else {
                                exp_assert!(
                                    function.params.len() == arguments.len(),
                                    EV::ArgumentMismatch(
                                        arguments.len(),
                                        format!("{}", function.params.len())
                                    ),
                                    snap()
                                );
                                for (symbol, arg_expr) in
                                    function.params.iter().zip(arguments.into_iter())
                                {
                                    let arg_evaled = match *operator.value.read().unwrap() {
                                        Lambda { .. } => arg_expr.eval(snap(), env.clone())?,
                                        Macro { .. } => arg_expr.clone(),
                                        _ => unreachable!(),
                                    };
                                    scoped_env.assign(symbol.clone(), arg_evaled, true, snap())?;
                                }
                            }
                            if let Macro { .. } = *operator.value.read().unwrap() {
                                scoped_env = scoped_env.shadow();
                            };
                            let mut result = Expression::nil();
                            let scoped_env_lock = Locker::new(scoped_env);
                            for exp in &function.expressions {
                                result = exp.eval(snap(), scoped_env_lock.clone())?;
                            }
                            Ok(result)
                        }
                        val => exp!(EV::InvalidOperator(val.clone()), snapshot),
                    }
                } else {
                    Ok(self.clone())
                }
            }
            Symbol(sym) => match env
                .read()
                .expect("unable to access environment (are threads locked?)")
                .lookup(&sym)
            {
                Some(exp) => Ok(exp.read().unwrap().clone()), // TODO: make this not need a clone (allow returning pointers)
                None => exp!(EV::UndefinedSymbol(sym.clone()), snapshot),
            },
            _ => Ok(self.clone()),
        }
    }
}

impl fmt::Display for Expression {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.value.read().unwrap())
    }
}

impl PartialOrd for Expression {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        self.value.read().unwrap().partial_cmp(&other.value.read().unwrap())
    }
}
