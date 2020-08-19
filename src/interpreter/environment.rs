use std::collections::HashMap;

use std::fmt;
use std::sync::{Arc, RwLock};

use crate::{
    exp, CallSnapshot, Exception, ExceptionValue as EV, Expression, Operator, Symbol, Value,
};

#[derive(Debug)]
struct ParentEnvironment {
    namespace: Option<String>,
    environment: Arc<RwLock<Environment>>,
}

#[derive(Debug)]
pub struct Environment {
    values: HashMap<Symbol, Arc<RwLock<Expression>>>,
    // This unreadable memory model might cause issues going forward
    parents: Vec<ParentEnvironment>,
    // Whether this environment is a "shadow environment" -- that is, whether
    // it defers local assignment to the first non-namespaced parent.
    shadow: bool,
}

impl Environment {
    // TODO: see if this can be done without mutexes, at least for values

    pub fn root() -> Self {
        Self {
            values: HashMap::new(),
            parents: vec![],
            shadow: false,
        }
    }

    pub fn shadow(mut self) -> Self {
        self.shadow = true;
        self
    }

    pub fn with_parent(mut self, parent: Arc<RwLock<Self>>, namespace: Option<String>) -> Self {
        self.add_parent(parent, namespace);
        self
    }

    fn get_literal(symbol: &Symbol) -> Option<Value> {
        use Operator::*;

        match symbol.string_value().as_str() {
            "nil" => Some(Value::List(vec![])),
            "t" | "true" => Some(Value::True),
            "quote" => Some(Value::Operator(Quote)),
            "atom" => Some(Value::Operator(Atom)),
            "eq" => Some(Value::Operator(Eq)),
            "car" => Some(Value::Operator(Car)),
            "cdr" => Some(Value::Operator(Cdr)),
            "cons" => Some(Value::Operator(Cons)),
            "cond" => Some(Value::Operator(Cond)),
            "export" => Some(Value::Operator(Export)),
            "let" => Some(Value::Operator(Let)),
            "sum" => Some(Value::Operator(Sum)),
            "prod" => Some(Value::Operator(Prod)),
            "exp" => Some(Value::Operator(Exp)),
            "modulo" => Some(Value::Operator(Modulo)),
            "gt" => Some(Value::Operator(Gt)),
            "ge" => Some(Value::Operator(Ge)),
            "type" => Some(Value::Operator(Type)),
            "disp" => Some(Value::Operator(Disp)),
            "import" => Some(Value::Operator(Import)),
            "eval" => Some(Value::Operator(Eval)),
            "while" => Some(Value::Operator(While)),
            "macro" => Some(Value::Operator(Macro)),
            "lambda" => Some(Value::Operator(Lambda)),
            "list" => Some(Value::Operator(List)),
            "catch" => Some(Value::Operator(Catch)),
            "throw" => Some(Value::Operator(Throw)),
            "format" => Some(Value::Operator(Format)),
            "parse" => Some(Value::Operator(Parse)),
            "length" => Some(Value::Operator(Length)),
            "append" => Some(Value::Operator(Append)),
            "do" => Some(Value::Operator(Do)),
            _ => None,
        }
    }

    fn resolve_symbol(
        &self,
        symbol: &Symbol,
        namespace: Option<String>,
    ) -> Option<(Arc<RwLock<Expression>>, usize)> {
        if namespace == None {
            if let Some(value) = self.values.get(&symbol) {
                return Some((value.clone(), 0));
            }
        } else {
            for parent in &self.parents {
                if namespace == parent.namespace {
                    return parent
                        .environment
                        .read()
                        .unwrap()
                        .resolve_symbol(symbol, None);
                }
            }
        }
        let mut best_match: (Option<Arc<RwLock<Expression>>>, usize) = (None, 0);
        for parent in &self.parents {
            if parent.namespace.is_some() {
                continue;
            }
            if let Some((exp, depth)) = parent
                .environment
                .read()
                .unwrap()
                .resolve_symbol(symbol, None)
            {
                if best_match.0.is_none() || depth < best_match.1 {
                    best_match = (Some(exp), depth);
                }
            }
        }
        if let Some(exp) = best_match.0 {
            return Some((exp, best_match.1 + 1));
        }
        match Self::get_literal(symbol) {
            Some(value) => Some((Arc::new(RwLock::new(Expression::new(value))), 9999)),
            None => None,
        }
    }

    fn extract_components(symbol: &Symbol) -> (Option<String>, Symbol) {
        let components: Vec<&str> = symbol.string_value().split("::").collect();

        match components.len() {
            1 => (None, Symbol::from_str(components.get(0).unwrap())),
            _ => (
                Some(components.get(0).unwrap().to_string()),
                Symbol::from_str(
                    &components
                        .iter()
                        .skip(1)
                        .map(|x| x.to_string())
                        .collect::<Vec<String>>()
                        .join("::"),
                ),
            ),
        }
    }

    pub fn lookup(&self, symbol: &Symbol) -> Option<Arc<RwLock<Expression>>> {
        let (namespace, identifier) = Self::extract_components(symbol);
        match self.resolve_symbol(&identifier, namespace) {
            Some((exp, _)) => Some(exp),
            None => None,
        }
    }

    pub fn add_parent(&mut self, parent: Arc<RwLock<Self>>, namespace: Option<String>) {
        self.parents.push(ParentEnvironment {
            namespace,
            environment: parent,
        });
    }

    pub fn assign(
        &mut self,
        symbol: Symbol,
        exp: Expression,
        only_local: bool,
        snapshot: Arc<RwLock<CallSnapshot>>,
    ) -> Result<Arc<RwLock<Expression>>, Exception> {
        let (namespace, identifier) = Self::extract_components(&symbol);

        if only_local && namespace.is_some() {
            exp!(
                EV::Assignment(symbol, exp),
                snapshot,
                "cannot perform local assignment with namespace".to_string()
            )
        }

        if !self.shadow
            && (only_local || self.values.contains_key(&identifier) || self.parents.is_empty())
        {
            let lock = Arc::new(RwLock::new(exp));
            self.values.insert(identifier, lock.clone());
            Ok(lock)
        } else {
            for parent in &self.parents {
                if parent.namespace == namespace {
                    return parent
                        .environment
                        .write()
                        .unwrap()
                        .assign(identifier, exp, only_local, snapshot);
                }
            }
            exp!(EV::Assignment(symbol, exp), snapshot, format!("could not find suitable environment for assignment (namespace `{}` not available for assignment)", match namespace {
                Some(value) => value,
                None => "no namespace".to_string(),
            }))
        }
    }
}

impl fmt::Display for Environment {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "[values: {}]\n{}\nimported namespaces: {}",
            self.values.len(),
            self.values
                .iter()
                .map(|(k, v)| format!("{} := {}", k, v.read().unwrap()))
                .collect::<Vec<String>>()
                .join("\n"),
            self.parents
                .iter()
                .map(|p| match &p.namespace {
                    Some(val) => val.clone(),
                    None => "(directly injected)".to_string(),
                })
                .collect::<Vec<String>>()
                .join(", ")
        )
    }
}
