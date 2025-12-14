use crate::context::Context;
use crate::expr::Expr;
use std::collections::BTreeMap;
use std::sync::{Arc, RwLock};

pub fn register(ctx: &mut Context) {
    let mut col_exports = BTreeMap::new();

    // List Operations
    col_exports.insert(Expr::sym("push"), Expr::extern_fun(|args, ctx| {
        if args.len() != 2 { return Expr::Nil; }
        let list = crate::context::eval(args[0].clone(), ctx);
        let val = crate::context::eval(args[1].clone(), ctx);
        
        match list {
            Expr::List(mut v) => {
                v.push(val);
                Expr::List(v)
            }
            Expr::Vector(mut v) => {
                v.push(val);
                Expr::Vector(v)
            }
            _ => Expr::Nil
        }
    }, "push", "Append element to list/vector"));

    col_exports.insert(Expr::sym("pop"), Expr::extern_fun(|args, ctx| {
        match eval_first(args, ctx) {
            Expr::List(mut v) => {
                v.pop();
                Expr::List(v)
            }
            Expr::Vector(mut v) => {
                v.pop();
                Expr::Vector(v)
            }
            _ => Expr::Nil
        }
    }, "pop", "Remove last element"));
    
    col_exports.insert(Expr::sym("peek"), Expr::extern_fun(|args, ctx| {
        match eval_first(args, ctx) {
            Expr::List(v) => v.last().cloned().unwrap_or(Expr::Nil),
            Expr::Vector(v) => v.last().cloned().unwrap_or(Expr::Nil),
            _ => Expr::Nil
        }
    }, "peek", "Get last element"));
    
    col_exports.insert(Expr::sym("reverse"), Expr::extern_fun(|args, ctx| {
        match eval_first(args, ctx) {
            Expr::List(mut v) => {
                v.reverse();
                Expr::List(v)
            }
            Expr::Vector(mut v) => {
                v.reverse();
                Expr::Vector(v)
            }
            _ => Expr::Nil
        }
    }, "reverse", "Reverse list"));

    col_exports.insert(Expr::sym("sort"), Expr::extern_fun(|args, ctx| {
         if args.len() < 1 { return Expr::Nil; }
         let list_expr = crate::context::eval(args[0].clone(), ctx);
         
         match list_expr {
             Expr::List(mut v) => {
                 v.sort_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal));
                 Expr::List(v)
             }
             Expr::Vector(mut v) => {
                 v.sort_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal));
                 Expr::Vector(v)
             }
             _ => Expr::Nil
         }
    }, "sort", "Sort list"));
    
    // Map Operations
    col_exports.insert(Expr::sym("keys"), Expr::extern_fun(|args, ctx| {
        match eval_first(args, ctx) {
            Expr::Map(m) => Expr::List(m.keys().cloned().collect()),
            Expr::HashMap(m) => Expr::List(m.keys().cloned().collect()),
            _ => Expr::Nil
        }
    }, "keys", "Get map keys"));

    col_exports.insert(Expr::sym("values"), Expr::extern_fun(|args, ctx| {
         match eval_first(args, ctx) {
            Expr::Map(m) => Expr::List(m.values().cloned().collect()),
            Expr::HashMap(m) => Expr::List(m.values().cloned().collect()),
            _ => Expr::Nil
        }
    }, "values", "Get map values"));
    
    col_exports.insert(Expr::sym("contains_key"), Expr::extern_fun(|args, ctx| {
        if args.len() != 2 { return Expr::Nil; }
        let map = crate::context::eval(args[0].clone(), ctx);
        let key = crate::context::eval(args[1].clone(), ctx);
        
        match map {
            Expr::Map(m) => if m.contains_key(&key) { Expr::Int(1) } else { Expr::Nil },
            Expr::HashMap(m) => if m.contains_key(&key) { Expr::Int(1) } else { Expr::Nil },
            _ => Expr::Nil
        }
    }, "contains_key", "Check if key exists"));

    col_exports.insert(Expr::sym("merge"), Expr::extern_fun(|args, ctx| {
        if args.len() != 2 { return Expr::Nil; }
        let map1 = crate::context::eval(args[0].clone(), ctx);
        let map2 = crate::context::eval(args[1].clone(), ctx);
        
        match (map1, map2) {
            (Expr::Map(mut m1), Expr::Map(m2)) => {
                m1.extend(m2);
                Expr::Map(m1)
            }
             (Expr::HashMap(mut m1), Expr::HashMap(m2)) => {
                m1.extend(m2);
                Expr::HashMap(m1)
            }
            _ => Expr::Nil
        }
    }, "merge", "Merge two maps"));

    // Functional Helpers
    col_exports.insert(Expr::sym("map"), Expr::extern_fun(|args, ctx| {
        if args.len() != 2 { return Expr::List(vec![]); }
        let list = crate::context::eval(args[0].clone(), ctx);
        let func = crate::context::eval(args[1].clone(), ctx); // Func should optimize to self or extern

        match list {
            Expr::List(v) | Expr::Vector(v) => {
                let mut res = Vec::with_capacity(v.len());
                for item in v {
                    // Call function
                    let call_args = vec![item];
                    let val = call_fn(&func, &call_args, ctx);
                    res.push(val);
                }
                Expr::Vector(res)
            }
            _ => Expr::List(vec![])
        }
    }, "map", "Apply function to list"));

    col_exports.insert(Expr::sym("filter"), Expr::extern_fun(|args, ctx| {
        if args.len() != 2 { return Expr::List(vec![]); }
        let list = crate::context::eval(args[0].clone(), ctx);
        let func = crate::context::eval(args[1].clone(), ctx);

        match list {
            Expr::List(v) | Expr::Vector(v) => {
                let mut res = Vec::new();
                for item in v {
                    let call_args = vec![item.clone()];
                    let val = call_fn(&func, &call_args, ctx);
                    // Truthy check
                    if val != Expr::Nil {
                         res.push(item);
                    }
                }
                Expr::Vector(res)
            }
            _ => Expr::List(vec![])
        }
    }, "filter", "Filter list"));

    col_exports.insert(Expr::sym("fold"), Expr::extern_fun(|args, ctx| {
        if args.len() != 3 { return Expr::Nil; }
        let list = crate::context::eval(args[0].clone(), ctx);
        let mut acc = crate::context::eval(args[1].clone(), ctx);
        let func = crate::context::eval(args[2].clone(), ctx);

        match list {
            Expr::List(v) | Expr::Vector(v) => {
                for item in v {
                    let call_args = vec![acc, item];
                    acc = call_fn(&func, &call_args, ctx);
                }
                acc
            }
            _ => acc // Return init if not list?
        }
    }, "fold", "Reduce list"));

    let mod_val = Expr::Ref(Arc::new(RwLock::new(Expr::Map(col_exports))));
    ctx.define(Expr::sym("Collections"), mod_val);
}

fn eval_first(args: &[Expr], ctx: &mut Context) -> Expr {
    if args.len() != 1 {
        Expr::Nil
    } else {
        crate::context::eval(args[0].clone(), ctx)
    }
}

// Helper to call generic function from Rust
fn call_fn(func: &Expr, args: &[Expr], ctx: &mut Context) -> Expr {
    match func {
        Expr::Function { params: _, body: _, env: _, name: _ } => {
            // Function requires explicit call handling or eval
            // We use eval of a list form (func arg1 arg2) but we need to Quote args
            // OR use context::eval loop logic explicitly?
            // Reusing eval is safer for consistency.
            
            let mut call_list = Vec::new();
            call_list.push(func.clone());
            for arg in args {
                // Args are already evaluated values. We need to Quote them to prevent re-eval in eval()
                call_list.push(Expr::Quoted(Box::new(arg.clone())));
            }
            crate::context::eval(Expr::List(call_list), ctx)
        }
        Expr::Extern(ext) => {
            // Extern call expects UNEVALUATED args in current architecture
            // But we have EVALUATED values.
            // And stdlib functions now call eval() on their args.
            // If we pass Quoted(Val), eval() returns Val.
            // So we should Quote args here too!
             let mut call_args = Vec::new();
             for arg in args {
                 call_args.push(Expr::Quoted(Box::new(arg.clone())));
             }
            ext.call(&call_args, ctx)
        }
        _ => Expr::Nil
    }
}
