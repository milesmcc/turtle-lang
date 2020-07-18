use crate::expression::{Expression, ExpressionValue};
use pest::iterators::{Pair, Pairs};
use pest::Parser;

#[derive(Parser)]
#[grammar = "syntax.pest"]
pub struct SyntaxParser;

pub fn parse(source: &str) -> Result<Vec<Expression>, pest::error::Error<Rule>> {
    match SyntaxParser::parse(Rule::expressions, source) {
        Ok(pairs) => Ok(pairs.map(|pair| build_expression(pair)).collect()),
        Err(err) => Err(err),
    }
}

fn build_expression(pair: Pair<Rule>) -> Expression {
    Expression::new(match pair.as_rule() {
        Rule::list => ExpressionValue::List(
            pair.into_inner()
                .map(|elem| build_expression(elem))
                .collect(),
        ),
        Rule::symbol => ExpressionValue::Symbol(String::from(pair.as_str())),
        Rule::t => ExpressionValue::True,

        // Builtins
        Rule::quote => ExpressionValue::Quote,
        Rule::atom => ExpressionValue::Atom,
        Rule::eq => ExpressionValue::Eq,
        Rule::car => ExpressionValue::Car,
        Rule::cdr => ExpressionValue::Cdr,
        Rule::cons => ExpressionValue::Cons,
        Rule::cond => ExpressionValue::Cond,

        // Sugar
        Rule::quote_sugar => {
            let mut elements = vec![Expression::new(ExpressionValue::Quote)];
            elements.append(
                &mut pair.into_inner()
                    .map(|elem| build_expression(elem))
                    .collect(),
            );
            ExpressionValue::List(elements)
        }

        _ => todo!(
            "unknown syntax element `{}` ({:?})",
            pair.as_str(),
            pair.as_rule()
        ),
    })
}
