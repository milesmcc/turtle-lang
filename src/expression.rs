#[derive(Debug)]
pub struct Expression {
    value: ExpressionValue
}

#[derive(Debug)]
pub enum ExpressionValue {
    List(Vec<Expression>),
    Number(f64),
    Symbol(String),

    // Primitive (axiomatic) operators
    Quote,
    Atom,
    Eq,
    Car,
    Cdr,
    Cons,
    Cond,
}

impl Expression {
    pub fn new(value: ExpressionValue) -> Self {
        Self {
            value: value
        }
    }
}