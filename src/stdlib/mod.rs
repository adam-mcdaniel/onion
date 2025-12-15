use super::context::{Assoc, Context, OpInfo};
use super::*;
use crate::context::eval;

use crate::context::Scope;
use std::collections::{BTreeMap, HashMap};
use std::sync::{Arc, RwLock};

pub mod math;
pub mod string;
pub mod collections;
pub mod time;
pub mod os;
pub mod io;
pub mod reflect;
pub mod game;

pub fn stdlib() -> Context {
    let mut ctx = Context::new();
    
    // Register Modules
    math::register(&mut ctx);
    string::register(&mut ctx);
    reflect::register(&mut ctx);
    collections::register(&mut ctx);
    time::register(&mut ctx);
    os::register(&mut ctx);
    io::register(&mut ctx);
    game::register(&mut ctx);
    
    ctx.define_op(
        "+",
        OpInfo {
            precedence: 10,
            associativity: Assoc::Left,
            unary: false,
        },
        Expr::extern_fun(
            |args, ctx| {
                let mut sum = Expr::Nil;

                for arg in args {
                    match eval(arg.clone(), ctx) {
                        Expr::Int(n) => {
                            sum = match sum {
                                Expr::Nil => Expr::Int(n),
                                Expr::Int(m) => Expr::Int(m + n),
                                Expr::Float(f) => Expr::Float(f + (n as f64)),
                                _ => return Expr::Nil.into(),
                            };
                        }
                        Expr::Float(f) => {
                            sum = match sum {
                                Expr::Nil => Expr::Float(f),
                                Expr::Int(n) => Expr::Float((n as f64) + f),
                                Expr::Float(g) => Expr::Float(g + f),
                                _ => return Expr::Nil.into(),
                            };
                        }
                        Expr::Str(s) => {
                            sum = match sum {
                                Expr::Nil => Expr::Str(s.clone()),
                                Expr::Str(existing) => {
                                    let mut new_str = existing.clone();
                                    new_str.push_str(&s);
                                    Expr::Str(new_str)
                                }
                                _ => return Expr::Nil.into(),
                            };
                        }
                        Expr::List(lst) => {
                            sum = match sum {
                                Expr::Nil => Expr::List(lst.clone()),
                                Expr::List(existing) => {
                                    let mut new_list = existing.clone();
                                    new_list.extend_from_slice(&lst);
                                    Expr::List(new_list)
                                }

                                _ => return Expr::Nil.into(),
                            };
                        }
                        Expr::Map(m) => {
                            sum = match sum {
                                Expr::Nil => Expr::Map(m.clone()),
                                Expr::HashMap(existing) => {
                                    let mut new_map = existing.clone();
                                    for (k, v) in m {
                                        new_map.insert(k.clone(), v.clone());
                                    }
                                    Expr::HashMap(new_map)
                                }
                                Expr::Map(existing) => {
                                    let mut new_map = existing.clone();
                                    for (k, v) in m {
                                        new_map.insert(k.clone(), v.clone());
                                    }
                                    Expr::Map(new_map)
                                }
                                _ => return Expr::Nil.into(),
                            };
                        }
                        Expr::HashMap(m) => {
                            sum = match sum {
                                Expr::Nil => Expr::HashMap(m.clone()),
                                Expr::HashMap(existing) => {
                                    let mut new_map = existing.clone();
                                    for (k, v) in m {
                                        new_map.insert(k.clone(), v.clone());
                                    }
                                    Expr::HashMap(new_map)
                                }
                                Expr::Map(existing) => {
                                    let mut new_map = existing.clone();
                                    for (k, v) in m {
                                        new_map.insert(k.clone(), v.clone());
                                    }
                                    Expr::Map(new_map)
                                }
                                _ => return Expr::Nil.into(),
                            };
                        }
                        _ => return Expr::Nil.into(),
                    }
                }

                sum
            },
            "+",
            "Add two numbers, concatenate lists, strings, or maps.",
        ),
    );

    // Define standard operators
    ctx.define_op(
        "-",
        OpInfo {
            precedence: 10,
            associativity: Assoc::Left,
            unary: false,
        },
        Expr::extern_fun(
            |args, ctx| {
                let mut sum = Expr::Nil;

                for arg in args {
                    match eval(arg.clone(), ctx) {
                        Expr::Int(n) => {
                            sum = match sum {
                                Expr::Nil => Expr::Int(n),
                                Expr::Int(m) => Expr::Int(m - n),
                                Expr::Float(f) => Expr::Float(f - (n as f64)),
                                _ => return Expr::Nil.into(),
                            };
                        }
                        Expr::Float(f) => {
                            sum = match sum {
                                Expr::Nil => Expr::Float(f),
                                Expr::Int(n) => Expr::Float((n as f64) - f),
                                Expr::Float(g) => Expr::Float(g - f),
                                _ => return Expr::Nil.into(),
                            };
                        }
                        _ => return Expr::Nil.into(),
                    }
                }

                sum
            },
            "-",
            "Subtract two numbers.",
        ),
    );

    // Equality / Assignment
    ctx.define_op(
        "=",
        OpInfo {
            precedence: 5,
            associativity: Assoc::Right,
            unary: false,
        },
        Expr::extern_fun(
            |args, ctx| {
                if args.len() != 2 {
                    return Expr::Nil;
                }

                let lhs = &args[0];
                let rhs = &args[1];

                if let Expr::List(list) = lhs {
                    if !list.is_empty() {
                        if let Expr::Sym(op) = &list[0] {
                            if op.as_str() == "." && list.len() >= 3 {
                                let mut new_list = list.clone();
                                new_list.push(rhs.clone());

                                return eval(Expr::List(new_list), ctx);
                            }
                        }
                    }
                }
                else if let Expr::Sym(s) = lhs {
                    let val = eval(rhs.clone(), ctx);
                    ctx.define(Expr::Sym(s.clone()), val.clone());
                    return val;
                }

                Expr::Nil
            },
            "=",
            "Assign value to property or variable. (x = 10) or (obj.key = 20)",
        ),
    );

    // Multiplication
    ctx.define_op(
        "*",
        OpInfo {
            precedence: 20,
            associativity: Assoc::Left,
            unary: false,
        },
        Expr::extern_fun(
            |args, ctx| {
                let mut prod = Expr::Nil;
                for arg in args {
                    match eval(arg.clone(), ctx) {
                        Expr::Int(n) => {
                            prod = match prod {
                                Expr::Nil => Expr::Int(n),
                                Expr::Int(m) => Expr::Int(m * n),
                                Expr::Float(f) => Expr::Float(f * (n as f64)),
                                _ => return Expr::Nil.into(),
                            };
                        }
                        Expr::Float(f) => {
                            prod = match prod {
                                Expr::Nil => Expr::Float(f),
                                Expr::Int(n) => Expr::Float((n as f64) * f),
                                Expr::Float(g) => Expr::Float(g * f),
                                _ => return Expr::Nil.into(),
                            };
                        }
                        _ => return Expr::Nil.into(),
                    }
                }
                if prod == Expr::Nil {
                    Expr::Int(1)
                } else {
                    prod
                }
            },
            "*",
            "Multiply numbers.",
        ),
    );

    // Division
    ctx.define_op(
        "/",
        OpInfo {
            precedence: 20,
            associativity: Assoc::Left,
            unary: false,
        },
        Expr::extern_fun(
            |args, ctx| {
                let mut res = Expr::Nil;
                let mut first = true;
                for arg in args {
                    match eval(arg.clone(), ctx) {
                        Expr::Int(n) => {
                            if first {
                                res = Expr::Int(n);
                                first = false;
                            } else {
                                res = match res {
                                    Expr::Int(m) => {
                                        if n != 0 {
                                            Expr::Int(m / n)
                                        } else {
                                            Expr::Nil
                                        }
                                    }
                                    Expr::Float(f) => {
                                        if n != 0 {
                                            Expr::Float(f / (n as f64))
                                        } else {
                                            Expr::Nil
                                        }
                                    }
                                    _ => return Expr::Nil.into(),
                                };
                            }
                        }
                        Expr::Float(f) => {
                            if first {
                                res = Expr::Float(f);
                                first = false;
                            } else {
                                res = match res {
                                    Expr::Int(n) => {
                                        if f != 0.0 {
                                            Expr::Float((n as f64) / f)
                                        } else {
                                            Expr::Nil
                                        }
                                    }
                                    Expr::Float(g) => {
                                        if f != 0.0 {
                                            Expr::Float(g / f)
                                        } else {
                                            Expr::Nil
                                        }
                                    }
                                    _ => return Expr::Nil.into(),
                                };
                            }
                        }
                        _ => return Expr::Nil.into(),
                    }
                }
                res
            },
            "/",
            "Divide numbers.",
        ),
    );

    ctx.define_op(
        "%",
        OpInfo {
            precedence: 20,
            associativity: Assoc::Left,
            unary: false,
        },
        Expr::extern_fun(
            |args, ctx| {
                if args.len() != 2 {
                    return Expr::Nil;
                }
                let first = eval(args[0].clone(), ctx);
                let second = eval(args[1].clone(), ctx);
                match (first, second) {
                    (Expr::Int(a), Expr::Int(b)) => Expr::Int(a % b),
                    (Expr::Float(a), Expr::Float(b)) => Expr::Float(a % b),
                    _ => Expr::Nil,
                }
            },
            "%",
            "Modulo operation.",
        ),
    );

    // Equality
    ctx.define_op(
        "==",
        OpInfo {
            precedence: 5,
            associativity: Assoc::Left,
            unary: false,
        },
        Expr::extern_fun(
            |args, ctx| {
                if args.len() < 2 {
                    return Expr::Int(1); // True for 0 or 1 arg
                }
                let first = eval(args[0].clone(), ctx);
                for arg in &args[1..] {
                    if eval(arg.clone(), ctx) != first {
                        return Expr::Nil; // False
                    }
                }
                Expr::Int(1) // True
            },
            "=",
            "Check equality.",
        ),
    );

    // Less Than
    ctx.define_op(
        "<",
        OpInfo {
            precedence: 5,
            associativity: Assoc::Left,
            unary: false,
        },
        Expr::extern_fun(
            |args, ctx| {
                if args.len() < 2 {
                    return Expr::Int(1);
                }
                let mut prev = eval(args[0].clone(), ctx);
                for arg in &args[1..] {
                    let curr = eval(arg.clone(), ctx);
                    match (&prev, &curr) {
                        (Expr::Int(a), Expr::Int(b)) => {
                            if !(*a < *b) {
                                return Expr::Nil;
                            }
                        }
                        (Expr::Float(a), Expr::Float(b)) => {
                            if !(*a < *b) {
                                return Expr::Nil;
                            }
                        }
                        (Expr::Int(a), Expr::Float(b)) => {
                            if !((*a as f64) < *b) {
                                return Expr::Nil;
                            }
                        }
                        (Expr::Float(a), Expr::Int(b)) => {
                            if !(*a < (*b as f64)) {
                                return Expr::Nil;
                            }
                        }
                        _ => return Expr::Nil,
                    }
                    prev = curr;
                }
                Expr::Int(1)
            },
            "<",
            "Less than.",
        ),
    );

    // Greater Than
    ctx.define_op(
        ">",
        OpInfo {
            precedence: 5,
            associativity: Assoc::Left,
            unary: false,
        },
        Expr::extern_fun(
            |args, ctx| {
                if args.len() < 2 {
                    return Expr::Int(1);
                }
                let mut prev = eval(args[0].clone(), ctx);
                for arg in &args[1..] {
                    let curr = eval(arg.clone(), ctx);
                    match (&prev, &curr) {
                        (Expr::Int(a), Expr::Int(b)) => {
                            if !(*a > *b) {
                                return Expr::Nil;
                            }
                        }
                        (Expr::Float(a), Expr::Float(b)) => {
                            if !(*a > *b) {
                                return Expr::Nil;
                            }
                        }
                        (Expr::Int(a), Expr::Float(b)) => {
                            if !((*a as f64) > *b) {
                                return Expr::Nil;
                            }
                        }
                        (Expr::Float(a), Expr::Int(b)) => {
                            if !(*a > (*b as f64)) {
                                return Expr::Nil;
                            }
                        }
                        _ => return Expr::Nil,
                    }
                    prev = curr;
                }
                Expr::Int(1)
            },
            ">",
            "Greater than.",
        ),
    );

    ctx.define_op(
        "<=",
        OpInfo {
            precedence: 10,
            associativity: Assoc::Left,
            unary: false,
        },
        Expr::extern_fun(
            |args, ctx| {
                if args.len() != 2 {
                    return Expr::Nil;
                }
                let left = eval(args[0].clone(), ctx);
                let right = eval(args[1].clone(), ctx);
                match (left, right) {
                    (Expr::Int(a), Expr::Int(b)) => Expr::Int(if a <= b { 1 } else { 0 }),
                    (Expr::Float(a), Expr::Float(b)) => Expr::Int(if a <= b { 1 } else { 0 }),
                    (Expr::Int(a), Expr::Float(b)) => Expr::Int(if (a as f64) <= b { 1 } else { 0 }),
                    (Expr::Float(a), Expr::Int(b)) => Expr::Int(if a <= (b as f64) { 1 } else { 0 }),
                    _ => Expr::Nil,
                }
            },
            "<=",
            "Less than or equal to.",
        ),
    );

    ctx.define_op(
        ">=",
        OpInfo {
            precedence: 10,
            associativity: Assoc::Left,
            unary: false,
        },
        Expr::extern_fun(
            |args, ctx| {
                if args.len() != 2 {
                    return Expr::Nil;
                }
                let left = eval(args[0].clone(), ctx);
                let right = eval(args[1].clone(), ctx);
                match (left, right) {
                    (Expr::Int(a), Expr::Int(b)) => Expr::Int(if a >= b { 1 } else { 0 }),
                    (Expr::Float(a), Expr::Float(b)) => Expr::Int(if a >= b { 1 } else { 0 }),
                    (Expr::Int(a), Expr::Float(b)) => Expr::Int(if (a as f64) >= b { 1 } else { 0 }),
                    (Expr::Float(a), Expr::Int(b)) => Expr::Int(if a >= (b as f64) { 1 } else { 0 }),
                    _ => Expr::Nil,
                }
            },
            ">=",
            "Greater than or equal to.",
        ),
    );

    // Unary Negation / Not
    ctx.define_op(
        "!",
        OpInfo {
            precedence: 15,
            associativity: Assoc::Right,
            unary: true,
        },
        Expr::extern_fun(
            |args, ctx| {
                if args.len() != 1 {
                    return Expr::Nil;
                }
                match eval(args[0].clone(), ctx) {
                    Expr::Int(n) => Expr::Int(-n),
                    Expr::Float(f) => Expr::Float(-f),
                    Expr::Nil => Expr::Int(1),
                    _ => Expr::Nil,
                }
            },
            "!",
            "Unary negation or logical NOT.",
        ),
    );

    ctx.define(
        Expr::sym("not"),
        Expr::extern_fun(
            |args, ctx| {
                if args.len() != 1 {
                    return Expr::Nil;
                }
                match eval(args[0].clone(), ctx) {
                    Expr::Nil => Expr::Int(1),
                    _ => Expr::Nil,
                }
            },
            "not",
            "Logical NOT.",
        ),
    );

    // Logic AND (short-circuit)
    ctx.define(
        Expr::sym("and"),
        Expr::extern_fun(
            |args, ctx| {
                let mut last = Expr::Int(1);
                for arg in args {
                    last = eval(arg.clone(), ctx);
                    if matches!(last, Expr::Nil) {
                        return Expr::Nil;
                    }
                }
                last
            },
            "and",
            "Logical AND (short-circuiting).",
        ),
    );

    // Logic OR (short-circuit)
    ctx.define(
        Expr::sym("or"),
        Expr::extern_fun(
            |args, ctx| {
                for arg in args {
                    let val = eval(arg.clone(), ctx);
                    if !matches!(val, Expr::Nil) {
                        return val;
                    }
                }
                Expr::Nil
            },
            "or",
            "Logical OR (short-circuiting).",
        ),
    );

    // List functions
    ctx.define(
        Expr::sym("list"),
        Expr::extern_fun(
            |args, ctx| {
                let mut vals = Vec::new();
                for arg in args {
                    vals.push(eval(arg.clone(), ctx));
                }
                Expr::List(vals)
            },
            "list",
            "Create a list.",
        ),
    );

    ctx.define(
        Expr::sym("len"),
        Expr::extern_fun(
            |args, ctx| {
                if args.len() != 1 {
                    return Expr::Nil;
                }
                match crate::context::eval(args[0].clone(), ctx) {
                    Expr::List(l) => Expr::Int(l.len() as i64),
                    Expr::Str(s) => Expr::Int(s.len() as i64),
                    Expr::Map(m) => Expr::Int(m.len() as i64),
                    Expr::HashMap(m) => Expr::Int(m.len() as i64),
                    _ => Expr::Nil,
                }
            },
            "len",
            "Returns the length of a list, vector, string, or map.",
        ),
    );

    ctx.define(
        Expr::sym("first"),
        Expr::extern_fun(
            |args, ctx| {
                if args.is_empty() {
                    return Expr::Nil;
                }
                match crate::context::eval(args[0].clone(), ctx) {
                    Expr::List(l) => l.first().cloned().unwrap_or(Expr::Nil),
                    _ => Expr::Nil,
                }
            },
            "first",
            "Return the first element of a list/vector.",
        ),
    );

    ctx.define(
        Expr::sym("rest"),
        Expr::extern_fun(
            |args, ctx| {
                if args.is_empty() {
                    return Expr::Nil;
                }
                match eval(args[0].clone(), ctx) {
                    Expr::List(l) => {
                        if l.is_empty() {
                            Expr::List(vec![])
                        } else {
                            Expr::List(l[1..].to_vec())
                        }
                    }
                    _ => Expr::Nil,
                }
            },
            "rest",
            "Return the rest of a list.",
        ),
    );

    ctx.define(
        Expr::sym("cons"),
        Expr::extern_fun(
            |args, ctx| {
                if args.len() != 2 {
                    return Expr::Nil;
                }
                let head = eval(args[0].clone(), ctx);
                match eval(args[1].clone(), ctx) {
                    Expr::List(tail) => {
                        let mut new_list = vec![head];
                        new_list.extend(tail);
                        Expr::List(new_list)
                    }
                    Expr::Nil => Expr::List(vec![head]),
                    _ => Expr::Nil,
                }
            },
            "cons",
            "Add an element to the front of a list.",
        ),
    );

    // Control Flow
    ctx.define(
        Expr::sym("if"),
        Expr::extern_fun(
            |args, ctx| {
                if args.len() < 2 || args.len() > 3 {
                    return Expr::Nil;
                }
                let cond = eval(args[0].clone(), ctx);
                if !matches!(cond, Expr::Nil | Expr::Int(0)) {
                    eval(args[1].clone(), ctx)
                } else if args.len() == 3 {
                    eval(args[2].clone(), ctx)
                } else {
                    Expr::Nil
                }
            },
            "if",
            "Conditional execution: (if cond then else).",
        ),
    );

    ctx.define(
        Expr::sym("while"),
        Expr::extern_fun(
            |args, ctx| {
                let cond = &args[0];
                let body = &args[1..];

                let mut last = Expr::Nil;
                loop {
                    let cond_val = eval(cond.clone(), ctx);
                    if matches!(cond_val, Expr::Nil | Expr::Int(0)) {
                        break;
                    }
                    for expr in body {
                        last = eval(expr.clone(), ctx);
                    }
                }
                last
            },
            "while",
            "While loop: (while cond body).",
        ),
    );

    ctx.define(
        Expr::sym("for"),
        Expr::extern_fun(
            |args, ctx| {
                let var = &args[0];
                let iterator = &args[1];
                let body = &args[2..];

                let mut last = Expr::Nil;
                match iterator {
                    Expr::List(lst) => {
                        for item in lst {
                            ctx.define(var.clone(), item.clone());
                            for expr in body {
                                last = eval(expr.clone(), ctx);
                            }
                        }
                    }
                    _ => return Expr::Nil,
                }
                last
            },
            "for",
            "For loop: (for var iterator body).",
        ),
    );

    ctx.define(
        Expr::sym("do"),
        Expr::extern_fun(
            |args, ctx| {
                let mut result = Expr::Nil;
                for expr in args {
                    result = eval(expr.clone(), ctx);
                }
                result
            },
            "do",
            "Execute a sequence of expressions, returning the value of the last one.",
        ),
    );

    fn print(args: &[Expr], ctx: &mut Context) -> Expr {
        let mut result = Expr::Nil;
        let mut first = true;
        for expr in args {
            if !first {
                print!(" ");
            } else {
                first = false;
            }
            result = eval(expr.clone(), ctx);
            print!("{}", result);
        }
        result
    }
    ctx.define(
        Expr::sym("print"),
        Expr::extern_fun(
            print,
            "print",
            "Prints the given expressions to standard output without a newline.",
        ),
    );
    ctx.define(
        Expr::sym("println"),
        Expr::extern_fun(
            |args, ctx| {
                let result = print(args, ctx);
                println!();
                result
            },
            "println",
            "Prints the given expressions to standard output with a newline.",
        ),
    );

    ctx.define_op(
        "new",
        OpInfo {
            precedence: 0,
            associativity: Assoc::Right,
            unary: true,
        },
        Expr::extern_fun(
            |args, ctx| {
                if args.len() != 1 {
                    return Expr::Nil;
                }
                let val = eval(args[0].clone(), ctx);
                Expr::Ref(Arc::new(RwLock::new(val)))
            },
            "new",
            "Create a new mutable reference to a value.",
        ),
    );

    ctx.define_op(
        ".",
        OpInfo {
            precedence: 100,
            associativity: Assoc::Left,
            unary: false,
        },
        Expr::extern_fun(
            |args, ctx| {
                if args.len() < 2 || args.len() > 3 {
                    return Expr::Nil;
                }

                // First arg must be a reference
                let obj_expr = eval(args[0].clone(), ctx);
                let obj_ref = if let Expr::Ref(r) = &obj_expr {
                    r
                } else {
                    return Expr::Nil; // Can only use dot on refs
                };

                let result = if args.len() == 2 {
                    let key_sym = if let Expr::Sym(s) = &args[1] {
                        Some(s.clone())
                    } else {
                        None
                    };

                    if let Some(key) = &key_sym {
                        let guard = obj_ref.read().unwrap();
                        let found = match &*guard {
                            Expr::Map(m) => m.get(&Expr::Sym(key.clone())).cloned(),
                            Expr::HashMap(m) => m.get(&Expr::Sym(key.clone())).cloned(),
                            _ => None,
                        };
                        if let Some(v) = found {
                            match v {
                                Expr::Function {
                                    params,
                                    body,
                                    env,
                                    name,
                                } => {
                                    let mut new_ctx = env.clone();
                                    new_ctx.define(Expr::sym("self"), obj_expr.clone());
                                    return Expr::Function {
                                        params,
                                        body,
                                        env: new_ctx,
                                        name,
                                    };
                                }
                                _ => return v,
                            }
                        }
                    }

                    let attr_expr = eval(args[1].clone(), ctx);
                    let val_opt = {
                        let guard = obj_ref.read().unwrap();
                        match &*guard {
                            Expr::Map(m) => m.get(&attr_expr).cloned(),
                            Expr::HashMap(m) => m.get(&attr_expr).cloned(),
                            _ => None,
                        }
                    };

                    if let Some(val) = val_opt {
                        match val {
                            Expr::Function {
                                params,
                                body,
                                env,
                                name,
                            } => {
                                let mut new_ctx = env.clone();
                                new_ctx.define(Expr::sym("self"), obj_expr.clone());
                                Expr::Function {
                                    params,
                                    body,
                                    env: new_ctx,
                                    name,
                                }
                            }
                            _ => val,
                        }
                    } else {
                        Expr::Nil
                    }
                } else {
                    let attr_expr = if let Expr::Sym(s) = &args[1] {
                        Expr::Sym(s.clone())
                    } else {
                        eval(args[1].clone(), ctx)
                    };
                    let val_expr = eval(args[2].clone(), ctx);

                    let mut guard = obj_ref.write().unwrap();
                    match &mut *guard {
                        Expr::Map(m) => {
                            m.insert(attr_expr, val_expr.clone());
                        }
                        Expr::HashMap(m) => {
                            m.insert(attr_expr, val_expr.clone());
                        }
                        _ => return Expr::Nil,
                    };
                    val_expr
                };
                result
            },
            ".",
            "Access or set properties of a reference. (obj.attr) or (obj.attr val).",
        ),
    );

    ctx.define_op(
        "?",
        OpInfo {
            precedence: 0,
            associativity: Assoc::Left,
            unary: false,
        },
        Expr::extern_fun(
            |args, ctx| {
                if args.len() < 2 || args.len() > 3 {
                    return Expr::Nil;
                }
                let obj = eval(args[0].clone(), ctx);
                let key = match &args[1] {
                    Expr::Sym(s) => Expr::Sym(s.clone()),
                    _ => eval(args[1].clone(), ctx),
                };

                if args.len() == 3 {
                    let val = eval(args[2].clone(), ctx);
                    if let Expr::Ref(r) = obj {
                        let mut guard = r.write().unwrap();
                        match &mut *guard {
                            Expr::Map(m) => {
                                m.insert(key, val.clone());
                            }
                            Expr::HashMap(m) => {
                                m.insert(key, val.clone());
                            }
                            Expr::List(l) => {
                                if let Expr::Int(i) = key {
                                    if i >= 0 && (i as usize) < l.len() {
                                        l[i as usize] = val.clone();
                                    }
                                }
                            }
                            _ => return Expr::Nil,
                        }
                        return val;
                    }
                    return Expr::Nil;
                } else {
                    match obj {
                        Expr::List(l) => {
                            if let Expr::Int(i) = key {
                                if i >= 0 && (i as usize) < l.len() {
                                    return l[i as usize].clone();
                                }
                            }
                            Expr::Nil
                        }
                        Expr::Map(m) => m.get(&key).cloned().unwrap_or(Expr::Nil),
                        Expr::HashMap(m) => m.get(&key).cloned().unwrap_or(Expr::Nil),
                         Expr::Ref(r) => {
                            let guard = r.read().unwrap();
                            match &*guard {
                                Expr::List(l) => {
                                    if let Expr::Int(i) = key {
                                        if i >= 0 && (i as usize) < l.len() {
                                            return l[i as usize].clone();
                                        }
                                    }
                                    Expr::Nil
                                }
                                Expr::Map(m) => m.get(&key).cloned().unwrap_or(Expr::Nil),
                                Expr::HashMap(m) => m.get(&key).cloned().unwrap_or(Expr::Nil),
                                _ => Expr::Nil
                            }
                        }
                        _ => Expr::Nil,
                    }
                }
            },
            "?",
            "Index access (collection ? key) or set (collection ? key value).",
        ),
    );

    ctx.define_op(
        "def",
        OpInfo {
            precedence: 0,
            associativity: Assoc::Right,
            unary: false,
        },
        Expr::extern_fun(
            |args, ctx| {
                if args.len() != 2 {
                    return Expr::Nil;
                }
                let sym = &args[0];
                let val = eval(args[1].clone(), ctx);
                ctx.define(sym.clone(), val.clone());
                val
            },
            "def",
            "Define a value in the current context.",
        ),
    );

    // Define (fun (params) body)
    ctx.define(
        Expr::sym("fun"),
        Expr::extern_fun(
            |args, ctx| {
                if args.len() < 2 {
                    return Expr::Nil;
                }
                let params_expr = &args[0];
                
                let body = if args.len() == 2 {
                    Box::new(args[1].clone())
                } else {
                    let mut do_block = vec![Expr::sym("do")];
                    for i in 1..args.len() {
                        do_block.push(args[i].clone());
                    }
                    Box::new(Expr::List(do_block))
                };

                let params = match params_expr {
                    Expr::List(lst) => {
                        let mut syms = Vec::new();
                        for item in lst {
                            if let Expr::Sym(s) = item {
                                syms.push(s.clone());
                            } else {
                                return Expr::Nil;
                            }
                        }
                        syms
                    }
                    Expr::Sym(s) => vec![s.clone()], 
                    _ => return Expr::Nil,
                };

                Expr::Function {
                    params,
                    body,
                    env: ctx.clone(),
                    name: None,
                }
            },
            "fun",
            "Create a function (closure).",
        ),
    );

    // Module Definition
    ctx.define(
        Expr::sym("module"),
        Expr::extern_fun(
            |args, ctx| {
                if args.len() < 2 {
                    return Expr::Nil;
                }
                let name_sym = match &args[0] {
                    Expr::Sym(s) => s.clone(),
                    _ => return Expr::Nil,
                };

                let module_scope = Scope {
                    vars: RwLock::new(HashMap::new()),
                    parent: Some(ctx.scope.clone()),
                };

                let mut module_ctx = Context {
                    parsing: ctx.parsing.clone(),
                    scope: Arc::new(module_scope),
                };

                let mut last = Expr::Nil;
                for expr in &args[1..] {
                    last = eval(expr.clone(), &mut module_ctx);
                }

                let mut map = BTreeMap::new();
                {
                    let vars = module_ctx.scope.vars.read().unwrap();
                    println!("Harvesting module {}: found {} vars", name_sym, vars.len());
                    for (k, v) in vars.iter() {
                        println!("Exporting: {:?}", k);
                        map.insert(k.clone(), v.clone());
                    }
                }

                let mod_val = Expr::Ref(Arc::new(RwLock::new(Expr::Map(map))));

                ctx.define(Expr::Sym(name_sym), mod_val.clone());

                mod_val
            },
            "module",
            "Define a new module with its own scope. (module Name (def x 1)...)",
        ),
    );

    ctx.define(
        Expr::sym("defun"),
        Expr::extern_fun(
            |args, ctx| {
                if args.len() < 3 {
                    return Expr::Nil;
                }
                let name_expr = &args[0];
                let params_expr = &args[1];
                let body_exprs = &args[2..];

                let fn_name_sym = match name_expr {
                    Expr::Sym(s) => s.clone(),
                    _ => return Expr::Nil,
                };

                let params = match params_expr {
                    Expr::List(lst) => {
                        let mut syms = Vec::new();
                        for item in lst {
                            if let Expr::Sym(s) = item {
                                syms.push(s.clone());
                            } else {
                                return Expr::Nil;
                            }
                        }
                        syms
                    }
                    Expr::Sym(s) => vec![s.clone()],
                    Expr::Nil => vec![],
                    _ => return Expr::Nil,
                };

                let body = if body_exprs.len() == 1 {
                    body_exprs[0].clone()
                } else {
                    let mut do_block = vec![Expr::sym("do")];
                    do_block.extend(body_exprs.iter().cloned());
                    Expr::List(do_block)
                };

                let func = Expr::Function {
                    params,
                    body: Box::new(body),
                    env: ctx.clone(),
                    name: Some(fn_name_sym.clone()),
                };

                ctx.define(fn_name_sym.into(), func.clone());
                func
            },
            "defun",
            "Define a recursive function.",
        ),
    );

    ctx.define(
        Expr::sym("struct"),
        Expr::extern_fun(
            |args, ctx| {
                if args.len() < 2 {
                    return Expr::Nil;
                }
                let name_expr = &args[0];
                let fields_expr = &args[1];
                let method_defs = &args[2..];

                let struct_name = match name_expr {
                    Expr::Sym(s) => s.clone(),
                    _ => return Expr::Nil,
                };

                let fields: Vec<crate::symbol::Symbol> = match fields_expr {
                    Expr::List(lst) => {
                        let mut syms = Vec::new();
                        for item in lst {
                            if let Expr::Sym(s) = item {
                                syms.push(s.clone());
                            } else {
                                return Expr::Nil;
                            }
                        }
                        syms
                    }
                    Expr::Nil => vec![],
                    _ => return Expr::Nil,
                };

                let mut methods = BTreeMap::new();
                for method_def in method_defs {
                    if let Expr::List(l) = method_def {
                        if l.len() < 3 {
                            return Expr::Nil;
                        } // Name, Params, Body
                        let m_name = match &l[0] {
                            Expr::Sym(s) => s.clone(),
                            _ => return Expr::Nil,
                        };

                        let m_params = match &l[1] {
                            Expr::List(pl) => {
                                let mut ps = Vec::new();
                                for p in pl {
                                    if let Expr::Sym(s) = p {
                                        ps.push(s.clone());
                                    } else {
                                        return Expr::Nil;
                                    }
                                }
                                ps
                            }
                            Expr::Nil => vec![],
                            _ => return Expr::Nil,
                        };

                        let m_body = if l.len() == 3 {
                            Box::new(l[2].clone())
                        } else {
                            Box::new(Expr::List(l[2..].iter().cloned().map(|e| e).collect()))
                        };

                        let method_func = Expr::Function {
                            params: m_params,
                            body: m_body,
                            env: ctx.clone(),
                            name: Some(m_name.clone()),
                        };
                        methods.insert(m_name, method_func);
                    } else {
                        return Expr::Nil;
                    }
                }

                let fields_clone = fields.clone();
                let methods_clone = methods.clone();

                let constructor = Expr::extern_fun(
                    move |ctor_args, _ctx| {
                        if ctor_args.len() != fields_clone.len() {
                            return Expr::Nil;
                        }

                        let mut obj_map = BTreeMap::new();

                        for (i, field) in fields_clone.iter().enumerate() {
                            obj_map.insert(field.clone().into(), eval(ctor_args[i].clone(), _ctx));
                        }

                        for (i, field) in fields_clone.iter().enumerate() {
                            obj_map.insert(field.clone().into(), eval(ctor_args[i].clone(), _ctx));
                        }

                        for (m_name, m_func) in &methods_clone {
                            obj_map.insert(Expr::Sym(m_name.clone()), m_func.clone());
                        }

                        Expr::Ref(Arc::new(RwLock::new(Expr::Map(obj_map))))
                    },
                    struct_name.to_string(),
                    "Struct Constructor",
                );

                ctx.define(struct_name.into(), constructor);
                Expr::Nil
            },
            "struct",
            "Define a struct with fields and methods.",
        ),
    );

    use std::f64::consts::{E, PI};
    ctx.define(Expr::sym("PI"), Expr::Float(PI));
    ctx.define(Expr::sym("E"), Expr::Float(E));

    ctx.define(
        Expr::sym("sqrt"),
        Expr::extern_fun(
            |args, ctx| {
                if args.len() != 1 {
                    return Expr::Nil;
                }
                match eval(args[0].clone(), ctx) {
                    Expr::Int(n) => Expr::Float((n as f64).sqrt()),
                    Expr::Float(f) => Expr::Float(f.sqrt()),
                    _ => Expr::Nil,
                }
            },
            "sqrt",
            "Calculate square root.",
        ),
    );

    ctx.define(
        Expr::sym("pow"),
        Expr::extern_fun(
            |args, ctx| {
                if args.len() != 2 {
                    return Expr::Nil;
                }
                let base = eval(args[0].clone(), ctx);
                let exp = eval(args[1].clone(), ctx);
                match (base, exp) {
                    (Expr::Int(b), Expr::Int(e)) => Expr::Float((b as f64).powf(e as f64)),
                    (Expr::Float(b), Expr::Float(e)) => Expr::Float(b.powf(e)),
                    (Expr::Int(b), Expr::Float(e)) => Expr::Float((b as f64).powf(e)),
                    (Expr::Float(b), Expr::Int(e)) => Expr::Float(b.powf(e as f64)),
                    _ => Expr::Nil,
                }
            },
            "pow",
            "Calculate power (base^exponent).",
        ),
    );

    ctx.define(
        Expr::sym("length"),
        Expr::extern_fun(
            |args, ctx| {
                if args.len() != 1 {
                    return Expr::Nil;
                }
                match eval(args[0].clone(), ctx) {
                    Expr::List(l) => Expr::Int(l.len() as i64),
                    Expr::Str(s) => Expr::Int(s.len() as i64),
                    Expr::Map(m) => Expr::Int(m.len() as i64),
                    Expr::HashMap(hm) => Expr::Int(hm.len() as i64),
                    _ => Expr::Int(0),
                }
            },
            "length",
            "Return length of list or string.",
        ),
    );

    let len_fn = ctx.resolve(&Expr::sym("length")).unwrap();
    ctx.define(Expr::sym("len"), len_fn);

    ctx.define(
        Expr::sym("nth"),
        Expr::extern_fun(
            |args, ctx| {
                if args.len() != 2 {
                    return Expr::Nil;
                }
                let idx_expr = eval(args[0].clone(), ctx);
                let list_expr = eval(args[1].clone(), ctx);

                match (idx_expr, list_expr) {
                    (Expr::Int(idx), Expr::List(l)) => {
                        if idx < 0 || idx >= l.len() as i64 {
                            Expr::Nil
                        } else {
                            l[idx as usize].clone()
                        }
                    }
                    _ => Expr::Nil,
                }
            },
            "nth",
            "Get element at index.",
        ),
    );

    ctx.define(
        Expr::sym("take"),
        Expr::extern_fun(
            |args, ctx| {
                if args.len() != 2 {
                    return Expr::Nil;
                }
                let n_expr = eval(args[0].clone(), ctx);
                let list_expr = eval(args[1].clone(), ctx);

                match (n_expr, list_expr) {
                    (Expr::Int(n), Expr::List(l)) => {
                        let n = n.max(0) as usize;
                        let count = n.min(l.len());
                        Expr::List(l[0..count].to_vec())
                    }
                    _ => Expr::Nil,
                }
            },
            "take",
            "Take first n elements.",
        ),
    );

    ctx.define(
        Expr::sym("drop"),
        Expr::extern_fun(
            |args, ctx| {
                if args.len() != 2 {
                    return Expr::Nil;
                }
                let n_expr = eval(args[0].clone(), ctx);
                let list_expr = eval(args[1].clone(), ctx);

                match (n_expr, list_expr) {
                    (Expr::Int(n), Expr::List(l)) => {
                        let n = n.max(0) as usize;
                        if n >= l.len() {
                            Expr::List(vec![])
                        } else {
                            Expr::List(l[n..].to_vec())
                        }
                    }
                    _ => Expr::Nil,
                }
            },
            "drop",
            "Drop first n elements.",
        ),
    );

    ctx
}

pub fn call_anon_fn(func: &Expr, args: &[Expr], ctx: &mut Context) -> Expr {
    match func {
        Expr::Function { params: _, body: _, env: _, name: _ } => {
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
        _ => Expr::Nil
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::parser::parse_expr;

    #[test]
    fn test_stdlib_add() {
        let ctx = stdlib();
        let input = "1 + 2 + 3 - 1 + (! 1) + {
            (println \"Hello, World!\")
            0
        }";
        let (_, expr) = parse_expr(input, &ctx).unwrap();
        let evaluated = eval(expr, &mut ctx.clone());

        assert_eq!(evaluated, Expr::Int(4));
    }

    #[test]
    fn test_stdlib_extended() {
        let ctx = stdlib();
        let input = "{
            (if (= 1 1) (println \"Equality works\") (println \"Equality broken\"))
            (println \"List:\" (list 1 2 3))
            (println \"First:\" (first (list 10 20)))

            (println \"Rest:\" (rest (list 10 20)))
            (println \"Cons:\" (cons 1 (list 2 3)))
            (println \"Logic True:\" (and 1 1))
            (println \"Logic False:\" (and 1 nil))
            (println \"Logic Or:\" (or nil 5))
            (println \"Math:\" (* 2 3))
            (println \"Div:\" (/ 10 2))
            (if (< 1 2) (println \"Less than works\") (println \"Less than broken\"))
            (+ 1 2)
        }";
        let (_, expr) = parse_expr(input, &ctx).unwrap();
        let evaluated = eval(expr, &mut ctx.clone());
        assert_eq!(evaluated, Expr::Int(3));
    }

    #[test]
    fn test_infix_notation() {
        let ctx = stdlib();
        let input_precedence = "1 + 2 * 3";
        let (_, expr) = parse_expr(input_precedence, &ctx).unwrap();
        let evaluated = eval(expr, &mut ctx.clone());
        assert_eq!(evaluated, Expr::Int(7));

        let input_assoc_left = "10 - 5 - 2";
        let (_, expr) = parse_expr(input_assoc_left, &ctx).unwrap();
        let evaluated = eval(expr, &mut ctx.clone());
        assert_eq!(evaluated, Expr::Int(3));

        let input_parens = "(1 + 2) * 3";
        let (_, expr) = parse_expr(input_parens, &ctx).unwrap();
        let evaluated = eval(expr, &mut ctx.clone());
        assert_eq!(evaluated, Expr::Int(9));

        let input_complex = "10 + 3 * 2 - 4 / 2";
        let (_, expr) = parse_expr(input_complex, &ctx).unwrap();
        let evaluated = eval(expr, &mut ctx.clone());
        assert_eq!(evaluated, Expr::Int(14));

        let input_mixed = "1 + 2 > 2";
        let (_, expr) = parse_expr(input_mixed, &ctx).unwrap();
        let evaluated = eval(expr, &mut ctx.clone());
        assert_eq!(evaluated, Expr::Int(1));
    }

    #[test]
    fn test_infix_within_functions() {
        let ctx = stdlib();
        let input = "{
            (def add_infix (fun (a b) {
                a + b * 2
            }))
            (println \"Result:\" (add_infix 10 5))
            (add_infix 10 5)
        }";
        let (_, expr) = parse_expr(input, &ctx).unwrap();
        let evaluated = eval(expr, &mut ctx.clone());
        assert_eq!(evaluated, Expr::Int(20));

        let input2 = "{
             (def nested_infix (fun (x) {
                 (if (x > 10) 
                     {x - 5}
                     {x + 5})
             }))
             (nested_infix 15)
        }";
        let (_, expr2) = parse_expr(input2, &ctx).unwrap();
        let evaluated2 = eval(expr2, &mut ctx.clone());
        assert_eq!(evaluated2, Expr::Int(10));
    }

    #[test]
    fn test_algorithms() {
        let ctx = stdlib();
        let input = "{
            (defun factorial (n)
                (if (< n 2) 1 (* n (factorial (- n 1)))))
            
            (defun stirling (n)
                (* (sqrt (* 2 (* PI n))) (pow (/ n E) n)))

            (defun merge (left right)
                (if (== (length left) 0) right
                  (if (== (length right) 0) left
                    (if (< (first left) (first right))
                        (cons (first left) (merge (rest left) right))
                        (cons (first right) (merge left (rest right)))))))

            (defun mergesort (lst)
                (if (< (length lst) 2) lst
                    {
                        (def len (length lst))
                        (def mid (/ len 2))
                        (def left (take mid lst))
                        (def right (drop mid lst))
                        (merge (mergesort left) (mergesort right))
                    }))

            (def fact5 (factorial 5))
            (println \"Factorial 5:\" fact5)
            
            (def stir5 (stirling 5))
            (println \"Stirling 5:\" stir5)

            (def sorted (mergesort (list 3 1 4 1 5 9 2 6)))
            (println \"Sorted:\" sorted)
            
            (list fact5 stir5 sorted)
        }";

        let (_, expr) = parse_expr(input, &ctx).unwrap();
        let evaluated = eval(expr, &mut ctx.clone());

        match evaluated {
            Expr::List(res) => {
                assert_eq!(res.len(), 3);
                assert_eq!(res[0], Expr::Int(120));
                if let Expr::Float(f) = res[1] {
                    assert!((f - 118.019).abs() < 0.1);
                } else {
                    panic!("Expected float for Stirling result");
                }

                // Sorted list
                if let Expr::List(l) = &res[2] {
                    assert_eq!(l.len(), 8);
                    assert_eq!(l[0], Expr::Int(1));
                    assert_eq!(l[1], Expr::Int(1));
                    assert_eq!(l[2], Expr::Int(2));
                    assert_eq!(l[3], Expr::Int(3));
                    assert_eq!(l[4], Expr::Int(4));
                    assert_eq!(l[5], Expr::Int(5));
                    assert_eq!(l[6], Expr::Int(6));
                    assert_eq!(l[7], Expr::Int(9));
                } else {
                    panic!("Expected list for Mergesort result");
                }
            }
            _ => panic!("Expected list result"),
        }
    }

    #[test]
    fn test_mergesort_large() {
        let builder = std::thread::Builder::new().stack_size(32 * 1024 * 1024);

        let handler = builder
            .spawn(|| {
                let ctx = stdlib();
                let input = "{
                (defun merge (left right)
                    (if ((length left) == 0) right
                      (if ((length right) == 0) left
                        (if ((first left) < (first right))
                            (cons (first left) (merge (rest left) right))
                            (cons (first right) (merge left (rest right)))))))

                (defun mergesort (lst)
                    (if ((length lst) < 2) lst
                        {
                            (def len (length lst))
                            (def mid (len / 2))
                            (def left (take mid lst))
                            (def right (drop mid lst))
                            (merge (mergesort left) (mergesort right))
                        }))

                (defun gen_descending (n)
                    (if (n < 1) {} (cons n (gen_descending (n - 1)))))
                
                (def list_size 200)
                (def my_list (gen_descending list_size))
                (def sorted (mergesort my_list))
                
                (length sorted)
            }";

                // Measure time
                let start = std::time::Instant::now();
                let (_, expr) = parse_expr(input, &ctx).unwrap();
                let evaluated = eval(expr, &mut ctx.clone());
                let duration = start.elapsed();

                println!("Mergesort of 200 items took: {:?}", duration);

                assert_eq!(evaluated, Expr::Int(200));
            })
            .unwrap();

        handler.join().unwrap();
    }

    #[test]
    fn test_oop_features() {
        let ctx = stdlib();
        let input = "{
            (def obj (new #[]))
            (println \"Initial obj:\" obj) 
            
            (. obj 'x 10)
            (println \"X is:\" (. obj 'x))
            
            (. obj 'x 20)
            (println \"X is now:\" (. obj 'x))
            
            (def counter_cls (new #[
                'count 0
                'inc (fun () (. self 'count (+ (. self 'count) 1)))
                'get (fun () (. self 'count))
            ]))
            
            (println \"Counter:\" counter_cls)
            ((. counter_cls 'inc))
            ((. counter_cls 'inc))
            (println \"Count after inc:\" ((. counter_cls 'get)))
            ((. counter_cls 'get))
        }";

        let (_, expr) = parse_expr(input, &ctx).unwrap();
        let evaluated = eval(expr, &mut ctx.clone());

        assert_eq!(evaluated, Expr::Int(2));
    }

    #[test]
    fn test_oop_infix_nested() {
        let ctx = stdlib();
        let input = "{
            (def point (new #[ 'x 10 'y 20 ]))
            (def rect (new #[ 
                'origin point 
                'area (fun () (* (self.origin.x) (self.origin.y)))
            ]))

            (println \"Origin X:\" (rect.origin.x))
            
            (def val (rect.origin.x + rect.origin.y))
            (println \"Sum coordinates:\" val)

            (println \"Area method:\" ((rect.area)))
            
            val
        }";

        let (_, expr) = parse_expr(input, &ctx).unwrap();
        let evaluated = eval(expr, &mut ctx.clone());

        assert_eq!(evaluated, Expr::Int(30)); // 10 + 20
    }

    #[test]
    fn test_oop_nested_mutation() {
        let ctx = stdlib();
        let input = "{
            (def p (new #[ 'x 1 'y 2 ]))
            (def r (new #[ 'origin p ]))
            
            (println \"Initial X:\" (r.origin.x))
            
            ;; Nested set: (. (r.origin) 'x 999)
            (. (r.origin) 'x 999)
            
            (println \"New X:\" (r.origin.x))
            (r.origin.x)
        }";

        let (_, expr) = parse_expr(input, &ctx).unwrap();
        let evaluated = eval(expr, &mut ctx.clone());
        assert_eq!(evaluated, Expr::Int(999));
    }

    #[test]
    fn test_oop_assignment() {
        let ctx = stdlib();
        let input = "{
            (def p (new #[ 'x 0 'y 0 ]))
            val = 42
            p.x = 100
            (def r (new #[ 'origin p ]))
            r.origin.y = 200
            (val + p.x + r.origin.y)
        }";
        let (_, expr) = parse_expr(input, &ctx).unwrap();
        let evaluated = eval(expr, &mut ctx.clone());
        assert_eq!(evaluated, Expr::Int(342));
    }

    #[test]
    fn test_oop_struct() {
        let ctx = stdlib();
        let input = "{
            (struct Point (x y)
                (area () self.x * self.y)
                (move (dx dy) {
                    (self.x = (self.x) + dx)
                    (self.y = (self.y) + dy)
                })
            )

            (def p (Point 10 20))
            (println \"Created Point:\" p)
            (println \"Area:\" (p.area))
            
            (p.move 5 5)
            (println \"Moved Point:\" p)
            
            (list (p.x) (p.y) (p.area))
        }";

        let (_, expr) = parse_expr(input, &ctx).unwrap();
        let evaluated = eval(expr, &mut ctx.clone());

        match evaluated {
            Expr::List(l) => {
                assert_eq!(l.len(), 3);
                assert_eq!(l[0], Expr::Int(15));
                assert_eq!(l[1], Expr::Int(25));
                assert_eq!(l[2], Expr::Int(375));
            }
            _ => panic!("Expected list result"),
        }
    }

    #[test]
    fn test_oop_struct_method() {
        let ctx = stdlib();
        let input = "{
            (struct Counter (count)
                (inc () (self.count = self.count + 1))
                (get () self.count)
            )

            (def c (Counter 0))
            (println \"Created Counter:\" c)
            (println \"Count:\" (c.get))
            
            (c.inc)
            (c.inc)
            (c.inc)
            (println \"Count:\" (c.get))
            
            (list (c.get))
        }";

        let (_, expr) = parse_expr(input, &ctx).unwrap();
        let evaluated = eval(expr, &mut ctx.clone());

        match evaluated {
            Expr::List(l) => {
                assert_eq!(l.len(), 1);
                assert_eq!(l[0], Expr::Int(3));
            }
            _ => panic!("Expected list result"),
        }
    }

    #[test]
    fn test_modules() {
        let ctx = stdlib();
        let input = "{
            (module MathLib 
                (def PI 3.14159)
                (defun square (x) (* x x))
                (defun area (r) (* PI (square r)))

                (struct Point (x y)
                    (distance (other) {
                        (sqrt 
                            (self.x - other.x) * (self.x - other.x)
                            + (self.y - other.y) * (self.y - other.y)
                        )
                    })
                
                (add (other) {
                    (Point
                        self.x + other.x
                        self.y + other.y)
                }))
            )
            
            (println \"Module loaded:\" MathLib)
            (println \"PI:\" MathLib.PI)
            
            (def sq MathLib.square)
            (println \"Square func:\" sq)
            (println \"Square 3:\" (sq 3))
            
            (def ar MathLib.area)
            (println \"Area 2:\" (ar 2))
            
            (def p1 (MathLib.Point 1 2))
            (def p2 (MathLib.Point 4 6))
            (println \"Distance:\" (p1.distance p2))
            (println \"Add:\" (p1.add p2))

            (list MathLib.PI (sq 4) (p1.distance p2) (p1.add p2))
        }";

        let (_, expr) = parse_expr(input, &ctx).unwrap();
        let evaluated = eval(expr, &mut ctx.clone());

        match evaluated {
            Expr::List(l) => {
                assert_eq!(l.len(), 4);
                if let Expr::Float(f) = l[0] {
                    assert!((f - 3.14159).abs() < 0.001);
                } else {
                    panic!("Expected float for PI, got {:?}", l[0]);
                }
                assert_eq!(l[1], Expr::Int(16));
                if let Expr::Float(d) = l[2] {
                    assert!((d - 5.0).abs() < 0.001);
                } else {
                    panic!("Expected float for distance, got {:?}", l[2]);
                }

                if let Expr::Ref(_) = l[3] {
                    // Success
                } else {
                    panic!("Expected Ref for Add result, got {:?}", l[3]);
                }
            }
            _ => panic!("Expected list result"),
        }
    }
}
