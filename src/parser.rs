use crate::expression::{Expression, Value, Symbol};
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
        Rule::list => Value::List(
            pair.into_inner()
                .map(|elem| build_expression(elem))
                .collect(),
        ),
        Rule::symbol => Value::Symbol(String::from(pair.as_str())),
        Rule::t => Value::True,

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
            let mut inner = pair.into_inner();
            let symbol_expressions: Vec<Expression> = inner.next().expect("lambda must have symbols").into_inner().map(|pair| build_expression(pair)).collect();
            let mut symbols: Vec<Symbol> = Vec::new();
            for exp in symbol_expressions {
                match exp.into_value() {
                    Value::Symbol(sym) => symbols.push(sym),
                    _ => panic!("cannot have lambda args that aren't symbols")
                }
            }
            let expression = build_expression(inner.next().expect("lambda must have expression"));
            Value::Function {
                params: symbols,
                expression: Box::new(expression),
            }
        },

        // Sugar
        Rule::quote_sugar => {
            let mut elements = vec![Expression::new(Value::Quote)];
            elements.append(
                &mut pair.into_inner()
                    .map(|elem| build_expression(elem))
                    .collect(),
            );
            Value::List(elements)
        }

        _ => todo!(
            "unknown syntax element `{}` ({:?})",
            pair.as_str(),
            pair.as_rule()
        ),
    })
}
