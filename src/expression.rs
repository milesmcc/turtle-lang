use std::fmt;

#[derive(Debug, Clone, PartialEq)]
pub struct Expression {
    value: Value,
}

#[derive(Debug, Clone, PartialEq)]
pub enum Value {
    List(Vec<Expression>),
    Number(f64),
    Text(String),
    Symbol(String),
    True,

    // Primitive (axiomatic) operators
    Quote,
    Atom,
    Eq,
    Car,
    Cdr,
    Cons,
    Cond,

    // Not-quite-as-axiomatic but nice-to-have operators
    Lambda,
}

impl Expression {
    pub fn new(value: Value) -> Self {
        Self { value: value }
    }

    pub fn get_value(&self) -> &Value {
        &self.value
    }

    pub fn eval(&self) -> Self {
        use Value::*;

        match &self.value {
            List(vals) => {
                // TODO: do as match on slice
                if vals.len() > 0 {
                    let operator = vals.get(0).unwrap();
                    let arguments = &vals.as_slice()[1..];
                    match operator.get_value() {
                        Quote => arguments[0].clone(),
                        Atom => match arguments[0].eval().get_value() {
                            List(_) => Expression::new(Value::List(vec![])),
                            _ => Expression::new(Value::True),
                        },
                        Eq => {
                            let first = arguments[0].eval();
                            let second = arguments[1].eval();
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
                            let mut list = arguments[0].eval();
                            match list.value {
                                List(mut vals) => vals.remove(0),
                                _ => panic!("car expects a list, got `{}`", list),
                            }
                        }
                        Cdr => {
                            let list = arguments[0].eval();
                            match list.value {
                                List(mut vals) => {
                                    vals.remove(0);
                                    Expression::new(List(vals))
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
                                        if *cond.eval().get_value() == Value::True {
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
                        _ => unimplemented!("unimplemented operator `{:?}`", operator.get_value()),
                    }
                } else {
                    panic!("cannot evaluate empty list!");
                }
            }
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
            Number(val) => write!(f, "{}", val),
            Text(val) => write!(f, "\"{}\"", val),
            Symbol(val) => write!(f, "{}", val),
            True => write!(f, "t"),
            _ => write!(f, "{}", format!("{:?}", self).to_lowercase()),
        }
    }
}
