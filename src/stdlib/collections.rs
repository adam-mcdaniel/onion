use crate::context::{Context, eval};
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
                    crate::stop!("push requires 2 arguments, got {}", args.len());
                }
                let list = crate::context::eval(args[0].clone(), ctx);
                let val = crate::context::eval(args[1].clone(), ctx);

                match list {
                    Expr::List(mut v) => {
                        v.push(val);
                        Expr::List(v)
                    }
                    other => crate::stop!("push expected List as first argument, got {:?}", other),
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
                other => crate::stop!("pop expected List, got {:?}", other),
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
                other => crate::stop!("peek expected List, got {:?}", other),
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
                other => crate::stop!("reverse expected List, got {:?}", other),
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
                    crate::stop!("sort requires at least 1 argument");
                }
                let list_expr = crate::context::eval(args[0].clone(), ctx);

                match list_expr {
                    Expr::List(mut v) => {
                        v.sort_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal));
                        Expr::List(v)
                    }
                    other => crate::stop!("sort expected List, got {:?}", other),
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
                    crate::stop!("range requires at least 2 arguments (start, end, [step])");
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
                            crate::stop!("range step cannot be 0");
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
                    (s, e, st) => crate::stop!(
                        "range arguments must be integers, got start={:?}, end={:?}, step={:?}",
                        s,
                        e,
                        st
                    ),
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
                    crate::stop!("zip requires 2 arguments, got {}", args.len());
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
                    (a, b) => crate::stop!("zip expected two Lists, got {:?} and {:?}", a, b),
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
                other => crate::stop!("flatten expected List, got {:?}", other),
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
                other => crate::stop!("dedup expected List, got {:?}", other),
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
                other => crate::stop!("keys expected Map or HashMap, got {:?}", other),
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
                other => crate::stop!("values expected Map or HashMap, got {:?}", other),
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
                    crate::stop!("contains_key requires 2 arguments, got {}", args.len());
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
                    other => crate::stop!("contains_key expected Map, got {:?}", other),
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
                    crate::stop!("merge requires 2 arguments, got {}", args.len());
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
                    (a, b) => crate::stop!(
                        "merge expected two Maps/HashMaps of same type, got {:?} and {:?}",
                        a,
                        b
                    ),
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
                    crate::stop!("map requires 2 arguments (list, function)");
                }
                // let list = crate::context::eval(args[0].clone(), ctx);
                crate::context::eval_in_place(&mut args[0], ctx);
                crate::context::eval_in_place(&mut args[1], ctx);
                // let func = crate::context::eval(args[1].clone(), ctx); // Func should optimize to self or extern

                match args[0].clone() {
                    Expr::List(v) => {
                        let mut res = Vec::with_capacity(v.len());
                        for item in v {
                            // Call function
                            let mut call_args = vec![item];
                            let val = call_fn(&args[1], &mut call_args, ctx);
                            res.push(val);
                        }
                        Expr::List(res)
                    }
                    other => crate::stop!("map expected List, got {:?}", other),
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
                    crate::stop!("filter requires 2 arguments (list, function)");
                }
                let list = crate::context::eval(args[0].clone(), ctx);
                let func = crate::context::eval(args[1].clone(), ctx);
                match list {
                    Expr::List(v) => {
                        let mut res = Vec::new();
                        for item in v {
                            let mut call_args = vec![item.clone()];
                            let val = call_fn(&func, &mut call_args, ctx);
                            // Truthy check
                            if !matches!(val, Expr::Int(0) | Expr::Nil) {
                                res.push(item);
                            }
                        }
                        Expr::List(res)
                    }
                    other => crate::stop!("filter expected List, got {:?}", other),
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
                    crate::stop!("fold requires 3 arguments (list, init, function)");
                }
                let list = crate::context::eval(args[0].clone(), ctx);
                let mut acc = crate::context::eval(args[1].clone(), ctx);
                let func = crate::context::eval(args[2].clone(), ctx);

                match list {
                    Expr::List(v) => {
                        for item in v {
                            let mut call_args = vec![acc, item];
                            acc = call_fn(&func, &mut call_args, ctx);
                        }
                        acc
                    }
                    other => crate::stop!("fold expected List, got {:?}", other),
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
                    crate::stop!("find requires 2 arguments (list, function)");
                }
                let list = crate::context::eval(args[0].clone(), ctx);
                let func = crate::context::eval(args[1].clone(), ctx);

                match list {
                    Expr::List(v) => {
                        for item in v {
                            let mut call_args = vec![item.clone()];
                            let val = call_fn(&func, &mut call_args, ctx);
                            if !matches!(val, Expr::Int(0) | Expr::Nil) {
                                return item;
                            }
                        }
                        Expr::Nil
                    }
                    other => crate::stop!("find expected List, got {:?}", other),
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
                    crate::stop!("any requires 2 arguments (list, function)");
                }
                let list = crate::context::eval(args[0].clone(), ctx);
                let func = crate::context::eval(args[1].clone(), ctx);

                match list {
                    Expr::List(v) => {
                        for item in v {
                            let mut call_args = vec![item.clone()];
                            let val = call_fn(&func, &mut call_args, ctx);
                            if !matches!(val, Expr::Int(0) | Expr::Nil) {
                                return Expr::Int(1);
                            }
                        }
                        Expr::Int(0)
                    }
                    other => crate::stop!("any expected List, got {:?}", other),
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
                    crate::stop!("all requires 2 arguments (list, function)");
                }
                let list = crate::context::eval(args[0].clone(), ctx);
                let func = crate::context::eval(args[1].clone(), ctx);

                match list {
                    Expr::List(v) => {
                        for item in v {
                            let mut call_args = vec![item.clone()];
                            let val = call_fn(&func, &mut call_args, ctx);
                            if matches!(val, Expr::Int(0) | Expr::Nil) {
                                return Expr::Int(0);
                            }
                        }
                        Expr::Int(1)
                    }
                    other => crate::stop!("all expected List, got {:?}", other),
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
                    crate::stop!("get requires 2 arguments (collection, key/index)");
                }
                let col = crate::context::eval(args[0].clone(), ctx);
                let key = crate::context::eval(args[1].clone(), ctx);

                match (col, key) {
                    (Expr::List(v), Expr::Int(i)) => {
                        let idx = if i < 0 { v.len() as i64 + i } else { i };
                        if idx >= 0 && idx < v.len() as i64 {
                            v[idx as usize].clone()
                        } else {
                            crate::stop!("Index out of bounds: {} (len {})", i, v.len());
                        }
                    }
                    (Expr::Str(s), Expr::Int(i)) => {
                        let idx = if i < 0 { s.len() as i64 + i } else { i };
                        if idx >= 0 && idx < s.len() as i64 {
                            // Very inefficient char access but consistent
                            s.chars()
                                .nth(idx as usize)
                                .map(|c| Expr::Str(c.to_string()))
                                .unwrap_or(Expr::Nil) // Should be unreachable with bounds check
                        } else {
                            crate::stop!("Index out of bounds: {} (len {})", i, s.len());
                        }
                    }
                    (Expr::Map(m), key) => m
                        .get(&key)
                        .cloned()
                        .unwrap_or_else(|| crate::stop!("Key not found in Map: {:?}", key)),
                    (Expr::HashMap(m), key) => m
                        .get(&key)
                        .cloned()
                        .unwrap_or_else(|| crate::stop!("Key not found in HashMap: {:?}", key)),
                    (c, k) => crate::stop!(
                        "get expected List/Str/Map and valid key, got {:?} and {:?}",
                        c,
                        k
                    ),
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
                    crate::stop!("enumerate requires 1 argument (list)");
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
                    other => crate::stop!("enumerate expected List, got {:?}", other),
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
        crate::stop!("Expected exactly 1 argument, got {}", args.len());
    } else {
        crate::context::eval(args[0].clone(), ctx)
    }
}

fn call_fn(func: &Expr, args: &mut [Expr], ctx: &mut Context) -> Expr {
    match func {
        Expr::Function {
            params: _,
            body: _,
            env: _,
            name: _,
        } => {
            // let mut call_list = Vec::new();
            // call_list.push(func.clone());
            // // for arg in args {
            // //     call_list.push(Expr::Quoted(Box::new(arg.clone())));
            // // }
            // call_list.extend_from_slice(args);
            // // crate::context::eval(Expr::List(call_list), ctx)
            // let mut call_expr = Expr::List(call_list);
            // crate::context::eval_in_place(&mut call_expr, ctx);
            // call_expr
            let mut func_expr = func.clone();
            crate::context::apply_in_place(&mut func_expr, args, ctx);
            func_expr
        }
        Expr::Extern(ext) => {
            // let mut call_args = Vec::new();
            // for arg in args {
            //     call_args.push(Expr::Quoted(Box::new(arg.clone())));
            // }
            ext.call(args, ctx)
        }
        _ => Expr::Nil,
    }
}
