use super::*;
use crate::expr::Expr;
use std::collections::BTreeMap;
use std::sync::RwLock;
use std::{collections::HashMap, sync::Arc};

/// Associativity of an operator.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Assoc {
    Left,
    Right,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct OpInfo {
    pub precedence: u8,
    pub associativity: Assoc,
    pub unary: bool,
}

#[derive(Clone, Debug, PartialEq)]
pub struct ParsingContext {
    /// The parsing context holds operator information for parsing expressions.
    operators: HashMap<String, OpInfo>,
}

impl ParsingContext {
    pub fn define_op(&mut self, symbol: impl ToString, info: OpInfo) {
        self.operators.insert(symbol.to_string(), info);
    }
}

#[derive(Debug, Default)]
pub struct Scope {
    pub vars: RwLock<HashMap<Expr, Expr>>,
    pub parent: Option<Arc<Scope>>,
}

// Drop implementation can be derived or simplified if not doing manual iterative drop?
// Manual drop is to avoid stack overflow on deep recursion.
impl Drop for Scope {
    fn drop(&mut self) {
        let mut current = self.parent.take();
        while let Some(arc_scope) = current {
            match Arc::try_unwrap(arc_scope) {
                Ok(mut scope) => {
                    current = scope.parent.take();
                }
                Err(_) => {
                    break;
                }
            }
        }
    }
}

#[derive(Clone, Debug)]
pub struct Context {
    /// The parsing context used for parsing expressions.
    pub parsing: Arc<RwLock<ParsingContext>>,
    /// Current scope.
    pub scope: Arc<Scope>,
}

impl Context {
    pub fn new() -> Self {
        Self {
            parsing: Arc::new(RwLock::new(ParsingContext {
                operators: HashMap::new(),
            })),
            scope: Arc::new(Scope::default()),
        }
    }

    pub fn define_op(&mut self, symbol: impl ToString, info: OpInfo, e: Expr) {
        let symbol = Symbol::new(&symbol.to_string());
        {
            let mut parsing = self.parsing.write().unwrap();
            parsing.define_op(&symbol, info);
        }
        self.define(symbol.into(), e);
    }

    pub fn get_op(&self, symbol: &str) -> Option<OpInfo> {
        let parsing = self.parsing.read().unwrap();
        parsing.operators.get(symbol).cloned()
    }

    pub fn get_operator_keys(&self) -> Vec<String> {
        let parsing = self.parsing.read().unwrap();
        parsing.operators.keys().cloned().collect()
    }

    pub fn define(&self, key: Expr, value: Expr) {
        self.scope.vars.write().unwrap().insert(key, value);
    }

    pub fn resolve(&self, key: &Expr) -> Option<Expr> {
        let mut current = Some(&self.scope);
        while let Some(scope) = current {
            if let Some(val) = scope.vars.read().unwrap().get(key) {
                return Some(val.clone());
            }
            current = scope.parent.as_ref();
        }
        None
    }
}

pub fn eval(mut expr: Expr, ctx: &mut Context) -> Expr {
    let saved_ctx = ctx.clone();
    let mut is_in_new_env = false;
    loop {
        if let Some(replacement) = ctx.resolve(&expr) {
            expr = replacement.clone();
            return expr;
        }

        match expr {
            Expr::List(list) => {
                if list.is_empty() {
                    return Expr::List(vec![]);
                }

                // Treat the list as a function application
                let func_expr = list[0].clone();
                let args = list[1..].to_vec();

                match eval(func_expr, ctx) {
                    Expr::Extern(f) => expr = f.call(&args, ctx),
                    Expr::Function {
                        params,
                        body,
                        env,
                        name,
                    } => {
                        is_in_new_env = true;
                        if params.len() != args.len() {
                            panic!(
                                "Function {:?} expected {} arguments, got {}. Args: {:?}",
                                name,
                                params.len(),
                                args.len(),
                                args
                            );
                        }

                        // Create new layered context
                        // We do NOT clone the `vars` map. We pointer-link to `env.scope`.
                        let new_scope = Scope {
                            vars: RwLock::new(HashMap::new()),
                            parent: Some(env.scope.clone()),
                        };

                        let mut new_ctx = Context {
                            parsing: env.parsing.clone(),
                            scope: Arc::new(new_scope),
                        };

                        // If named, bind self to support recursion
                        if let Some(fn_name) = &name {
                            // We construct the same function again.
                            // Wait! This recreates the entire closure.
                            // But `env` inside the closure MUST point to the *definition* environment.
                            // Here we are in the *execution* environment.
                            // `new_ctx` is the execution environment.
                            // We bind the function name in `new_ctx` (shadowing if exists).

                            let func_clone = Expr::Function {
                                params: params.clone(),
                                body: body.clone(),
                                env: env.clone(),
                                name: name.clone(),
                            };
                            new_ctx.define(fn_name.clone().into(), func_clone);
                        }

                        for (param, arg) in params.into_iter().zip(args.into_iter()) {
                            new_ctx.define(param.into(), eval(arg, ctx));
                        }

                        expr = *body;
                        *ctx = new_ctx;
                        continue;
                    }
                    other => {
                        expr = other;
                    }
                }
            }

            Expr::Map(m) => {
                let mut evaluated_map = BTreeMap::new();
                for (k, v) in m {
                    let eval_k = eval(k, ctx);
                    let eval_v = eval(v, ctx);
                    evaluated_map.insert(eval_k, eval_v);
                }
                expr = Expr::Map(evaluated_map);
            }
            Expr::Vector(v) => {
                let mut evaluated_vec = Vec::new();
                for item in v {
                    evaluated_vec.push(eval(item, ctx));
                }
                expr = Expr::Vector(evaluated_vec);
            }

            Expr::HashMap(m) => {
                let mut evaluated_map = HashMap::new();
                for (k, v) in m {
                    let eval_k = eval(k, ctx);
                    let eval_v = eval(v, ctx);
                    evaluated_map.insert(eval_k, eval_v);
                }
                expr = Expr::HashMap(evaluated_map);
            }

            Expr::Quoted(q) => {
                expr = *q;
            }

            _ => {
                break;
            }
        }

        break;
    }
    if is_in_new_env {
        *ctx = saved_ctx;
    }

    expr
}
