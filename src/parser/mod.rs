use crate::{
    Exception, ExceptionValue as EV, Expression, Keyword, Operator, Source, SourcePosition, Symbol,
    Value,
};
use pest::iterators::Pair;
use pest::Parser;

use crate::Locker;

#[derive(Parser)]
#[grammar = "parser/syntax.pest"]
pub struct SyntaxParser;

pub fn parse(code: &str, location: &str) -> Result<Vec<Expression>, Exception> {
    let source = Locker::new(Source::new(String::from(code), String::from(location)));

    match SyntaxParser::parse(Rule::expressions, code) {
        Ok(pairs) => {
            let mut exps = Vec::new();
            for pair in pairs {
                if pair.as_rule() == Rule::EOI {
                    // SOI isn't given
                    continue;
                }
                exps.push(
                    build_expression(pair.clone(), source.clone())?
                        .with_source(SourcePosition::from_pair(&pair, &source)),
                );
            }
            Ok(exps)
        }
        Err(err) => Err(Exception::from(err)),
    }
}

fn build_expression(pair: Pair<'_, Rule>, source: Locker<Source>) -> Result<Expression, Exception> {
    let pos = SourcePosition::new(
        pair.as_span().start_pos().pos(),
        pair.as_span().end_pos().pos(),
        source.clone(),
    );
    match &pair.as_rule() {
        Rule::list => {
            let mut values: Vec<Expression> = Vec::new();
            for elem in pair.into_inner() {
                values.push(
                    build_expression(elem.clone(), source.clone())?
                        .with_source(SourcePosition::from_pair(&elem, &source)),
                )
            }
            Ok(Expression::new(Value::List(values)))
        }
        Rule::symbol => Ok(Expression::new(Value::Symbol(Symbol::new(String::from(
            pair.as_str(),
        ))))
        .with_source(pos)),
        Rule::keyword => Ok(Expression::new(Value::Keyword(Keyword::new(String::from(
            pair.into_inner().next().unwrap().as_str(),
        ))))
        .with_source(pos)),
        Rule::number => match pair.as_str().parse::<f64>() {
            Ok(num) => Ok(Expression::new(Value::Number(num)).with_source(pos)),
            Err(_) => Err(Exception::new(
                EV::Syntax,
                None,
                Some(format!("`{}` is not a valid number", pair.as_str())),
            )),
        },
        Rule::byte => match pair.as_str().replace('b', "").parse::<u8>() {
            Ok(num) => Ok(Expression::new(Value::Byte(num)).with_source(pos)),
            Err(_) => Err(Exception::new(
                EV::Syntax,
                None,
                Some(format!("`{}` is not a valid byte (0-255)", pair.as_str())),
            )),
        },
        Rule::text => Ok(Expression::new(Value::Text(
            pair.into_inner().as_str().to_string(),
        ))),

        // Sugar
        Rule::quote | Rule::eval => {
            let mut elements = vec![Expression::new(Value::Operator(match &pair.as_rule() {
                Rule::quote => Operator::Quote,
                Rule::eval => Operator::Eval,
                _ => unreachable!(),
            }))];
            for elem in pair.into_inner() {
                elements.push(
                    build_expression(elem.clone(), source.clone())?
                        .with_source(SourcePosition::from_pair(&elem, &source)),
                );
            }
            Ok(Expression::new(Value::List(elements)))
        }
        _ => Err(Exception::new(
            EV::Syntax,
            None,
            Some(format!("unknown syntax element `{}`", pair.as_str())),
        )),
    }
}
