use crate::{Environment, Expression, Operator, Symbol, Value};
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
                Value::List(
                    pair.into_inner()
                        .map(|elem| build_expression(elem, env.clone()))
                        .collect(),
                )
            }
            Rule::symbol => match pair.as_str() {
                "t" | "true" => Value::True,
                "nil" => Value::List(vec![]),
                _ => Value::Symbol(String::from(pair.as_str())),
            },
            Rule::keyword => {
                Value::Keyword(String::from(pair.into_inner().next().unwrap().as_str()))
            }
            Rule::number => Value::Number(
                pair.as_str()
                    .parse()
                    .expect(format!("cannot parse number `{}`", pair.as_str()).as_str()),
            ),
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
                Value::Lambda {
                    params: symbols,
                    expressions,
                }
            }

            // Sugar
            Rule::quote => {
                let mut elements = vec![Expression::new(
                    Value::Operator(Operator::Quote),
                    env.clone(),
                )];
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
