use crate::expression::{Environment, Expression, Symbol, Value};
use pest::iterators::Pair;
use pest::Parser;
use std::sync::{Arc, RwLock};

#[derive(Parser)]
#[grammar = "syntax.pest"]
pub struct SyntaxParser;

pub fn parse<'a>(
    source: &str,
    env: Arc<RwLock<Environment<'a>>>,
) -> Result<Vec<Expression<'a>>, pest::error::Error<Rule>> {
    match SyntaxParser::parse(Rule::expressions, source) {
        Ok(pairs) => Ok(pairs
            .map(|pair| build_expression(pair, env.clone()))
            .collect()),
        Err(err) => Err(err),
    }
}

fn build_expression<'a>(pair: Pair<Rule>, env: Arc<RwLock<Environment<'a>>>) -> Expression<'a> {
    Expression::new(
        match pair.as_rule() {
            Rule::list => {
                let child_env = Environment::with_parent(env.clone());
                Value::List(
                    pair.into_inner()
                        .map(|elem| build_expression(elem, child_env.clone()))
                        .collect(),
                )
            }
            Rule::symbol => match pair.as_str() {
                "t" | "true" => Value::True,
                "nil" => Value::List(vec![]),
                _ => Value::Symbol(String::from(pair.as_str())),
            },

            // Builtins
            Rule::quote => Value::Quote,
            Rule::atom => Value::Atom,
            Rule::eq => Value::Eq,
            Rule::car => Value::Car,
            Rule::cdr => Value::Cdr,
            Rule::cons => Value::Cons,
            Rule::cond => Value::Cond,
            Rule::label => Value::Label,
            Rule::lambda => {
                let child_env = Environment::with_parent(env.clone());
                let mut inner = pair.into_inner();
                let symbol_expressions: Vec<Expression> = inner
                    .next()
                    .expect("lambda must have symbols")
                    .into_inner()
                    .map(|pair| build_expression(pair, child_env.clone()))
                    .collect();
                let mut symbols: Vec<Symbol> = Vec::new();
                for exp in symbol_expressions {
                    match exp.into_value() {
                        Value::Symbol(sym) => symbols.push(sym),
                        _ => panic!("cannot have lambda args that aren't symbols"),
                    }
                }
                let expressions = inner
                    .map(|exp| build_expression(exp, child_env.clone()))
                    .collect();
                Value::Function {
                    params: symbols,
                    expressions,
                }
            }

            // Sugar
            Rule::quote_sugar => {
                let mut elements = vec![Expression::new(Value::Quote, env.clone())];
                elements.append(
                    &mut pair
                        .into_inner()
                        .map(|elem| build_expression(elem, env.clone()))
                        .collect(),
                );
                Value::List(elements)
            }

            _ => todo!(
                "unknown syntax element `{}` ({:?})",
                pair.as_str(),
                pair.as_rule()
            ),
        },
        env,
    )
}
