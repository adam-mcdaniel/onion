use crate::context::Context;
use crate::expr::Expr;
use std::collections::BTreeMap;
use std::sync::{Arc, RwLock};

pub fn register(ctx: &mut Context) {
    let mut col_exports = BTreeMap::new();

    // List Operations
    col_exports.insert(
        Expr::sym("push"),
        Expr::extern_fun(
            |args, ctx| {
                if args.len() != 2 {
                    return Expr::Nil;
                }
                let list = crate::context::eval(args[0].clone(), ctx);
                let val = crate::context::eval(args[1].clone(), ctx);

                match list {
                    Expr::List(mut v) => {
                        v.push(val);
                        Expr::List(v)
                    }
                    _ => Expr::Nil,
                }
            },
            "push",
            "Append element to list/vector",
        ),
    );

    col_exports.insert(
        Expr::sym("pop"),
        Expr::extern_fun(
            |args, ctx| match eval_first(args, ctx) {
                Expr::List(mut v) => {
                    v.pop();
                    Expr::List(v)
                }
                _ => Expr::Nil,
            },
            "pop",
            "Remove last element",
        ),
    );

    col_exports.insert(
        Expr::sym("peek"),
        Expr::extern_fun(
            |args, ctx| match eval_first(args, ctx) {
                Expr::List(v) => v.last().cloned().unwrap_or(Expr::Nil),
                _ => Expr::Nil,
            },
            "peek",
            "Get last element",
        ),
    );

    col_exports.insert(
        Expr::sym("reverse"),
        Expr::extern_fun(
            |args, ctx| match eval_first(args, ctx) {
                Expr::List(mut v) => {
                    v.reverse();
                    Expr::List(v)
                }
                _ => Expr::Nil,
            },
            "reverse",
            "Reverse list",
        ),
    );

    col_exports.insert(
        Expr::sym("sort"),
        Expr::extern_fun(
            |args, ctx| {
                if args.len() < 1 {
                    return Expr::Nil;
                }
                let list_expr = crate::context::eval(args[0].clone(), ctx);

                match list_expr {
                    Expr::List(mut v) => {
                        v.sort_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal));
                        Expr::List(v)
                    }
                    _ => Expr::Nil,
                }
            },
            "sort",
            "Sort list",
        ),
    );

    col_exports.insert(
        Expr::sym("range"),
        Expr::extern_fun(
            |args, ctx| {
                if args.len() < 2 {
                    return Expr::Nil;
                }
                let start = crate::context::eval(args[0].clone(), ctx);
                let end = crate::context::eval(args[1].clone(), ctx);
                let step = if args.len() > 2 {
                    crate::context::eval(args[2].clone(), ctx)
                } else {
                    Expr::Int(1)
                };

                match (start, end, step) {
                    (Expr::Int(s), Expr::Int(e), Expr::Int(st)) => {
                        let mut res = Vec::new();
                        if st == 0 {
                            return Expr::Nil;
                        }
                        let mut i = s;
                        if st > 0 {
                            while i < e {
                                res.push(Expr::Int(i));
                                i += st;
                            }
                        } else {
                            while i > e {
                                res.push(Expr::Int(i));
                                i += st;
                            }
                        }
                        Expr::List(res)
                    }
                    _ => Expr::Nil,
                }
            },
            "range",
            "Generate range of integers",
        ),
    );

    col_exports.insert(
        Expr::sym("zip"),
        Expr::extern_fun(
            |args, ctx| {
                if args.len() != 2 {
                    return Expr::Nil;
                }
                let l1 = crate::context::eval(args[0].clone(), ctx);
                let l2 = crate::context::eval(args[1].clone(), ctx);

                match (l1, l2) {
                    (Expr::List(v1), Expr::List(v2)) => {
                        let len = std::cmp::min(v1.len(), v2.len());
                        let mut res = Vec::with_capacity(len);
                        for i in 0..len {
                            res.push(Expr::List(vec![v1[i].clone(), v2[i].clone()]));
                        }
                        Expr::List(res)
                    }
                    _ => Expr::Nil,
                }
            },
            "zip",
            "Zip two lists",
        ),
    );

    col_exports.insert(
        Expr::sym("flatten"),
        Expr::extern_fun(
            |args, ctx| match eval_first(args, ctx) {
                Expr::List(v) => {
                    fn flatten_recursively(list: Vec<Expr>, out: &mut Vec<Expr>) {
                        for item in list {
                            match item {
                                Expr::List(nested) => flatten_recursively(nested, out),
                                _ => out.push(item),
                            }
                        }
                    }
                    let mut res = Vec::new();
                    flatten_recursively(v, &mut res);
                    Expr::List(res)
                }
                _ => Expr::Nil,
            },
            "flatten",
            "Recursive flatten",
        ),
    );

    col_exports.insert(
        Expr::sym("dedup"),
        Expr::extern_fun(
            |args, ctx| match eval_first(args, ctx) {
                Expr::List(v) => {
                    let mut res = Vec::new();
                    for item in v {
                        if !res.contains(&item) {
                            res.push(item);
                        }
                    }
                    Expr::List(res)
                }
                _ => Expr::Nil,
            },
            "dedup",
            "Remove duplicates",
        ),
    );

    // Map Operations
    col_exports.insert(
        Expr::sym("keys"),
        Expr::extern_fun(
            |args, ctx| match eval_first(args, ctx) {
                Expr::Map(m) => Expr::List(m.keys().cloned().collect()),
                Expr::HashMap(m) => Expr::List(m.keys().cloned().collect()),
                _ => Expr::Nil,
            },
            "keys",
            "Get map keys",
        ),
    );

    col_exports.insert(
        Expr::sym("values"),
        Expr::extern_fun(
            |args, ctx| match eval_first(args, ctx) {
                Expr::Map(m) => Expr::List(m.values().cloned().collect()),
                Expr::HashMap(m) => Expr::List(m.values().cloned().collect()),
                _ => Expr::Nil,
            },
            "values",
            "Get map values",
        ),
    );

    col_exports.insert(
        Expr::sym("contains_key"),
        Expr::extern_fun(
            |args, ctx| {
                if args.len() != 2 {
                    return Expr::Nil;
                }
                let map = crate::context::eval(args[0].clone(), ctx);
                let key = crate::context::eval(args[1].clone(), ctx);

                match map {
                    Expr::Map(m) => {
                        if m.contains_key(&key) {
                            Expr::Int(1)
                        } else {
                            Expr::Nil
                        }
                    }
                    Expr::HashMap(m) => {
                        if m.contains_key(&key) {
                            Expr::Int(1)
                        } else {
                            Expr::Nil
                        }
                    }
                    _ => Expr::Nil,
                }
            },
            "contains_key",
            "Check if key exists",
        ),
    );

    col_exports.insert(
        Expr::sym("merge"),
        Expr::extern_fun(
            |args, ctx| {
                if args.len() != 2 {
                    return Expr::Nil;
                }
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
                    _ => Expr::Nil,
                }
            },
            "merge",
            "Merge two maps",
        ),
    );

    // Functional Helpers
    col_exports.insert(
        Expr::sym("map"),
        Expr::extern_fun(
            |args, ctx| {
                if args.len() != 2 {
                    return Expr::List(vec![]);
                }
                let list = crate::context::eval(args[0].clone(), ctx);
                let func = crate::context::eval(args[1].clone(), ctx); // Func should optimize to self or extern

                match list {
                    Expr::List(v) => {
                        let mut res = Vec::with_capacity(v.len());
                        for item in v {
                            // Call function
                            let call_args = vec![item];
                            let val = call_fn(&func, &call_args, ctx);
                            res.push(val);
                        }
                        Expr::List(res)
                    }
                    _ => Expr::List(vec![]),
                }
            },
            "map",
            "Apply function to list",
        ),
    );

    col_exports.insert(
        Expr::sym("filter"),
        Expr::extern_fun(
            |args, ctx| {
                if args.len() != 2 {
                    return Expr::List(vec![]);
                }
                let list = crate::context::eval(args[0].clone(), ctx);
                let func = crate::context::eval(args[1].clone(), ctx);

                match list {
                    Expr::List(v) => {
                        let mut res = Vec::new();
                        for item in v {
                            let call_args = vec![item.clone()];
                            let val = call_fn(&func, &call_args, ctx);
                            // Truthy check
                            if val != Expr::Nil {
                                res.push(item);
                            }
                        }
                        Expr::List(res)
                    }
                    _ => Expr::List(vec![]),
                }
            },
            "filter",
            "Filter list",
        ),
    );

    col_exports.insert(
        Expr::sym("fold"),
        Expr::extern_fun(
            |args, ctx| {
                if args.len() != 3 {
                    return Expr::Nil;
                }
                let list = crate::context::eval(args[0].clone(), ctx);
                let mut acc = crate::context::eval(args[1].clone(), ctx);
                let func = crate::context::eval(args[2].clone(), ctx);

                match list {
                    Expr::List(v) => {
                        for item in v {
                            let call_args = vec![acc, item];
                            acc = call_fn(&func, &call_args, ctx);
                        }
                        acc
                    }
                    _ => acc, // Return init if not list
                }
            },
            "fold",
            "Reduce list",
        ),
    );

    col_exports.insert(
        Expr::sym("find"),
        Expr::extern_fun(
            |args, ctx| {
                if args.len() != 2 {
                    return Expr::Nil;
                }
                let list = crate::context::eval(args[0].clone(), ctx);
                let func = crate::context::eval(args[1].clone(), ctx);

                match list {
                    Expr::List(v) => {
                        for item in v {
                            let call_args = vec![item.clone()];
                            let val = call_fn(&func, &call_args, ctx);
                            if val != Expr::Nil {
                                return item;
                            }
                        }
                        Expr::Nil
                    }
                    _ => Expr::Nil,
                }
            },
            "find",
            "Find first matching element",
        ),
    );

    col_exports.insert(
        Expr::sym("any"),
        Expr::extern_fun(
            |args, ctx| {
                if args.len() != 2 {
                    return Expr::Nil;
                }
                let list = crate::context::eval(args[0].clone(), ctx);
                let func = crate::context::eval(args[1].clone(), ctx);

                match list {
                    Expr::List(v) => {
                        for item in v {
                            let call_args = vec![item.clone()];
                            let val = call_fn(&func, &call_args, ctx);
                            if val != Expr::Nil {
                                return Expr::Int(1);
                            }
                        }
                        Expr::Nil
                    }
                    _ => Expr::Nil,
                }
            },
            "any",
            "Check if any match",
        ),
    );

    col_exports.insert(
        Expr::sym("all"),
        Expr::extern_fun(
            |args, ctx| {
                if args.len() != 2 {
                    return Expr::Nil;
                }
                let list = crate::context::eval(args[0].clone(), ctx);
                let func = crate::context::eval(args[1].clone(), ctx);

                match list {
                    Expr::List(v) => {
                        for item in v {
                            let call_args = vec![item.clone()];
                            let val = call_fn(&func, &call_args, ctx);
                            if val == Expr::Nil {
                                return Expr::Nil;
                            }
                        }
                        Expr::Int(1)
                    }
                    _ => Expr::Nil,
                }
            },
            "all",
            "Check if all match",
        ),
    );

    // Access
    col_exports.insert(
        Expr::sym("get"),
        Expr::extern_fun(
            |args, ctx| {
                if args.len() != 2 {
                    return Expr::Nil;
                }
                let col = crate::context::eval(args[0].clone(), ctx);
                let key = crate::context::eval(args[1].clone(), ctx);

                match (col, key) {
                    (Expr::List(v), Expr::Int(i)) => {
                        let idx = if i < 0 { v.len() as i64 + i } else { i };
                        if idx >= 0 && idx < v.len() as i64 {
                            v[idx as usize].clone()
                        } else {
                            Expr::Nil
                        }
                    }
                    (Expr::Str(s), Expr::Int(i)) => {
                        let idx = if i < 0 { s.len() as i64 + i } else { i };
                        if idx >= 0 && idx < s.len() as i64 {
                            // Very inefficient char access but consistent
                            s.chars()
                                .nth(idx as usize)
                                .map(|c| Expr::Str(c.to_string()))
                                .unwrap_or(Expr::Nil)
                        } else {
                            Expr::Nil
                        }
                    }
                    (Expr::Map(m), key) => m.get(&key).cloned().unwrap_or(Expr::Nil),
                    (Expr::HashMap(m), key) => m.get(&key).cloned().unwrap_or(Expr::Nil),
                    _ => Expr::Nil,
                }
            },
            "get",
            "Get element by index (List/Str) or key (Map)",
        ),
    );

    col_exports.insert(
        Expr::sym("enumerate"),
        Expr::extern_fun(
            |args, ctx| {
                if args.len() != 1 {
                    return Expr::Nil;
                }
                let col = crate::context::eval(args[0].clone(), ctx);

                match col {
                    Expr::List(v) => {
                        let mut result = Vec::new();
                        for (i, item) in v.iter().enumerate() {
                            result.push(Expr::List(vec![Expr::Int(i as i64), item.clone()]));
                        }
                        Expr::List(result)
                    }
                    _ => Expr::Nil,
                }
            },
            "enumerate",
            "Enumerate list",
        ),
    );

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

fn call_fn(func: &Expr, args: &[Expr], ctx: &mut Context) -> Expr {
    match func {
        Expr::Function {
            params: _,
            body: _,
            env: _,
            name: _,
        } => {
            let mut call_list = Vec::new();
            call_list.push(func.clone());
            for arg in args {
                call_list.push(Expr::Quoted(Box::new(arg.clone())));
            }
            crate::context::eval(Expr::List(call_list), ctx)
        }
        Expr::Extern(ext) => {
            let mut call_args = Vec::new();
            for arg in args {
                call_args.push(Expr::Quoted(Box::new(arg.clone())));
            }
            ext.call(&call_args, ctx)
        }
        _ => Expr::Nil,
    }
}
