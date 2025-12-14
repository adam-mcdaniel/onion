use crate::context::Context;
use crate::expr::Expr;
use std::collections::BTreeMap;
use std::sync::{Arc, RwLock};

pub fn register(ctx: &mut Context) {
    let mut reflect_exports = BTreeMap::new();

    // Type inspection
    reflect_exports.insert(Expr::sym("of"), Expr::extern_fun(|args, ctx| {
        match eval_first(args, ctx) {
            Expr::Int(_) => Expr::Str("int".to_string()),
            Expr::Float(_) => Expr::Str("float".to_string()),
            Expr::Str(_) => Expr::Str("string".to_string()),
            Expr::Sym(_) => Expr::Str("symbol".to_string()),
            Expr::List(_) => Expr::Str("list".to_string()),
            Expr::Vector(_) => Expr::Str("vector".to_string()),
            Expr::Map(_) | Expr::HashMap(_) => Expr::Str("map".to_string()),
            Expr::Function{..} | Expr::Extern{..} => Expr::Str("fn".to_string()),
            Expr::Nil => Expr::Str("nil".to_string()),
            Expr::Ref(_) => Expr::Str("ref".to_string()),
            Expr::Tagged { .. } => Expr::Str("tagged".to_string()),
            Expr::Quoted(_) => Expr::Str("quoted".to_string()),
        }
    }, "of", "Get type name"));

    // Type Checks
    reflect_exports.insert(Expr::sym("is_int"), Expr::extern_fun(|args, ctx| {
        match eval_first(args, ctx) {
            Expr::Int(_) => Expr::Int(1),
            _ => Expr::Nil
        }
    }, "is_int", "Is integer?"));

    reflect_exports.insert(Expr::sym("is_float"), Expr::extern_fun(|args, ctx| {
        match eval_first(args, ctx) {
            Expr::Float(_) => Expr::Int(1),
            _ => Expr::Nil
        }
    }, "is_float", "Is float?"));
    
    reflect_exports.insert(Expr::sym("is_string"), Expr::extern_fun(|args, ctx| {
        match eval_first(args, ctx) {
            Expr::Str(_) => Expr::Int(1),
            _ => Expr::Nil
        }
    }, "is_string", "Is string?"));

    reflect_exports.insert(Expr::sym("is_list"), Expr::extern_fun(|args, ctx| {
        match eval_first(args, ctx) {
            Expr::List(_) => Expr::Int(1),
            _ => Expr::Nil
        }
    }, "is_list", "Is list?"));

    reflect_exports.insert(Expr::sym("is_vector"), Expr::extern_fun(|args, ctx| {
        match eval_first(args, ctx) {
            Expr::Vector(_) => Expr::Int(1),
            _ => Expr::Nil
        }
    }, "is_vector", "Is vector?"));
    
    reflect_exports.insert(Expr::sym("is_map"), Expr::extern_fun(|args, ctx| {
        match eval_first(args, ctx) {
            Expr::Map(_) | Expr::HashMap(_) => Expr::Int(1),
            _ => Expr::Nil
        }
    }, "is_map", "Is map?"));
    
    reflect_exports.insert(Expr::sym("is_nil"), Expr::extern_fun(|args, ctx| {
        match eval_first(args, ctx) {
            Expr::Nil => Expr::Int(1),
            _ => Expr::Nil
        }
    }, "is_nil", "Is nil?"));


    // Conversions
    reflect_exports.insert(Expr::sym("to_int"), Expr::extern_fun(|args, ctx| {
        match eval_first(args, ctx) {
            Expr::Int(n) => Expr::Int(n),
            Expr::Float(f) => Expr::Int(f as i64),
            Expr::Str(s) => match s.parse::<i64>() {
                Ok(n) => Expr::Int(n),
                Err(_) => Expr::Nil
            },
            _ => Expr::Nil
        }
    }, "to_int", "Convert to int"));

    reflect_exports.insert(Expr::sym("to_float"), Expr::extern_fun(|args, ctx| {
        match eval_first(args, ctx) {
            Expr::Int(n) => Expr::Float(n as f64),
            Expr::Float(f) => Expr::Float(f),
             Expr::Str(s) => match s.parse::<f64>() {
                Ok(n) => Expr::Float(n),
                Err(_) => Expr::Nil
            },
            _ => Expr::Nil
        }
    }, "to_float", "Convert to float"));
    
    reflect_exports.insert(Expr::sym("to_str"), Expr::extern_fun(|args, ctx| {
        Expr::Str(eval_first(args, ctx).to_string())
    }, "to_str", "Convert to string"));
    
    reflect_exports.insert(Expr::sym("to_sym"), Expr::extern_fun(|args, ctx| {
        match eval_first(args, ctx) {
            Expr::Str(s) => Expr::sym(s.as_str()),
            Expr::Sym(s) => Expr::sym(s.as_str()),
            _ => Expr::Nil
        }
    }, "to_sym", "Convert to symbol"));

    let mod_val = Expr::Ref(Arc::new(RwLock::new(Expr::Map(reflect_exports))));
    ctx.define(Expr::sym("Type"), mod_val);
}

fn eval_first(args: &[Expr], ctx: &mut Context) -> Expr {
    if args.len() != 1 {
        Expr::Nil
    } else {
        crate::context::eval(args[0].clone(), ctx)
    }
}
