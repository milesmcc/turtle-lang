use crate::{
    Environment, Exception, ExceptionValue as EV, Expression, Keyword, Operator, Source,
    SourcePosition, Symbol, Value,
};
use pest::iterators::Pair;
use pest::Parser;
use std::sync::{Arc, RwLock};

#[derive(Parser)]
#[grammar = "parser/syntax.pest"]
pub struct SyntaxParser;

pub fn parse<'a>(
    code: &str,
    location: &str,
    env: Arc<RwLock<Environment<'a>>>,
) -> Result<Vec<Expression<'a>>, Exception<'a>> {
    let source = Arc::new(RwLock::new(Source::new(
        String::from(code),
        String::from(location),
    )));

    match SyntaxParser::parse(Rule::expressions, code) {
        Ok(pairs) => {
            let mut exps = Vec::new();
            for pair in pairs {
                if pair.as_rule() == Rule::EOI {
                    // SOI isn't given
                    continue;
                }
                exps.push(
                    build_expression(pair.clone(), env.clone(), source.clone())?
                        .with_source(SourcePosition::from_pair(&pair, &source)),
                );
            }
            Ok(exps)
        }
        Err(err) => Err(Exception::from(err)),
    }
}

fn build_expression<'a>(
    pair: Pair<'_, Rule>,
    env: Arc<RwLock<Environment<'a>>>,
    source: Arc<RwLock<Source>>,
) -> Result<Expression<'a>, Exception<'a>> {
    let pos = SourcePosition::new(
        pair.as_span().start_pos().pos(),
        pair.as_span().end_pos().pos(),
        source.clone(),
    );
    match &pair.as_rule() {
        Rule::list => {
            let mut values: Vec<Expression<'a>> = Vec::new();
            for elem in pair.into_inner() {
                values.push(
                    build_expression(elem.clone(), env.clone(), source.clone())?
                        .with_source(SourcePosition::from_pair(&elem, &source)),
                )
            }
            Ok(Expression::new(Value::List(values), env.clone()))
        }
        Rule::symbol => Ok(Expression::new(
            Value::Symbol(Symbol::new(String::from(pair.as_str()))),
            env.clone(),
        )
        .with_source(pos)),
        Rule::keyword => Ok(Expression::new(
            Value::Keyword(Keyword::new(String::from(
                pair.into_inner().next().unwrap().as_str(),
            ))),
            env.clone(),
        )
        .with_source(pos)),
        Rule::number => match pair.as_str().parse::<f64>() {
            Ok(num) => Ok(Expression::new(Value::Number(num), env.clone()).with_source(pos)),
            Err(_) => Err(Exception::new(
                EV::Syntax,
                None,
                Some(format!("`{}` is not a valid number", pair.as_str())),
            )),
        },
        Rule::text => Ok(Expression::new(
            Value::Text(pair.into_inner().as_str().to_string()),
            env.clone(),
        )),
        Rule::lambda_def | Rule::macro_def => {
            let rule = pair.as_rule();

            let child_env = Environment::with_parent(env.clone());

            let mut inner = pair.into_inner();
            let mut symbol_expressions: Vec<Expression> = Vec::new();
            let arg_symbol = inner.next().unwrap();
            let collapse_input = match arg_symbol.as_rule() {
                Rule::arg_symbols => false,
                _ => true,
            };
            for pair in arg_symbol.into_inner() {
                symbol_expressions.push(
                    build_expression(pair.clone(), child_env.clone(), source.clone())?
                        .with_source(SourcePosition::from_pair(&pair, &source)),
                )
            }
            let mut symbols: Vec<Symbol> = Vec::new();
            for exp in symbol_expressions {
                match exp.into_value() {
                    Value::Symbol(sym) => symbols.push(sym),
                    _ => panic!("cannot have args that aren't symbols"),
                }
            }
            let mut expressions = Vec::new();
            for exp in inner {
                expressions.push(
                    build_expression(exp.clone(), child_env.clone(), source.clone())?
                        .with_source(SourcePosition::from_pair(&exp, &source)),
                );
            }
            Ok(Expression::new(
                match rule {
                    Rule::lambda_def => Value::Lambda {
                        params: symbols,
                        expressions,
                        collapse_input,
                    },
                    Rule::macro_def => Value::Macro {
                        params: symbols,
                        expressions,
                        collapse_input,
                    },
                    _ => unreachable!(),
                },
                env,
            )
            .with_source(pos))
        }

        // Sugar
        Rule::quote | Rule::eval => {
            let mut elements = vec![Expression::new(
                Value::Operator(match &pair.as_rule() {
                    Rule::quote => Operator::Quote,
                    Rule::eval => Operator::Eval,
                    _ => unreachable!(),
                }),
                env.clone(),
            )];
            for elem in pair.into_inner() {
                elements.push(
                    build_expression(elem.clone(), env.clone(), source.clone())?
                        .with_source(SourcePosition::from_pair(&elem, &source)),
                );
            }
            Ok(Expression::new(Value::List(elements), env))
        }
        _ => Err(Exception::new(
            EV::Syntax,
            None,
            Some(format!("unknown syntax element `{}`", pair.as_str())),
        )),
    }
}
