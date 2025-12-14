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
    
    // ... rest of stdlib ...

    // Define standard operators
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

                // Check if LHS is a dot-access chain: (. obj key)
                if let Expr::List(list) = lhs {
                    if !list.is_empty() {
                        if let Expr::Sym(op) = &list[0] {
                            if op.as_str() == "." && list.len() >= 3 {
                                // It is a dot set!
                                // LHS is: (. obj key) or (. obj key1 key2...)
                                // We want to transform: (= (. A B) V) -> (. A B V)

                                // Evaluate RHS first
                                let val = eval(rhs.clone(), ctx);

                                // Construct new list for dot-setter: (. obj key val)
                                // We take the existing list elements and append val.
                                let mut new_list = list.clone();
                                new_list.push(val);

                                // Evaluate the new list as a function call
                                return eval(Expr::List(new_list), ctx);
                            }
                        }
                    }
                }
                // Handle variable assignment (def alias behavior?)
                // If LHS is a Symbol, maybe we update it?
                else if let Expr::Sym(s) = lhs {
                    let val = eval(rhs.clone(), ctx);
                    println!("DEBUG: Assigning {} = {:?} in scope", s, val);
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
                    Expr::Nil => Expr::Int(1), // !nil -> true (1)
                    _ => Expr::Nil,            // !truthy -> false (nil)
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
                    Expr::Vector(v) => Expr::Int(v.len() as i64),
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
                    Expr::Vector(v) => v.first().cloned().unwrap_or(Expr::Nil),
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

    // Implement `new` operator for creating references
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

    // Implement `.` operator for property access and setting
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
                    // Hybrid Lookup: Literal Symbol First, then Evaluated
                    let key_sym = if let Expr::Sym(s) = &args[1] {
                        Some(s.clone())
                    } else {
                        None
                    };

                    // 1. Try Literal Lookup if symbol
                    if let Some(key) = &key_sym {
                        let guard = obj_ref.read().unwrap();
                        let found = match &*guard {
                            Expr::Map(m) => m.get(&Expr::Sym(key.clone())).cloned(),
                            Expr::HashMap(m) => m.get(&Expr::Sym(key.clone())).cloned(),
                            _ => None,
                        };
                        if let Some(v) = found {
                            // Method binding check
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

                    // 2. Try Evaluated Lookup
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
                    // SET
                    let attr_expr = eval(args[1].clone(), ctx);
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

    // Define (def symbol value)
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
    ctx.define_op(
        "fun",
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
                let params_expr = &args[0];
                let body = &args[1];

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
                    Expr::Sym(s) => vec![s.clone()], // Allow single param without parens? Standard lisp usually requires list.
                    _ => return Expr::Nil,
                };

                Expr::Function {
                    params,
                    body: Box::new(body.clone()),
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

                // Create new scope linked to current scope
                // Create new scope linked to current scope
                let module_scope = Scope {
                    vars: RwLock::new(HashMap::new()),
                    parent: Some(ctx.scope.clone()),
                };

                let mut module_ctx = Context {
                    parsing: ctx.parsing.clone(),
                    scope: Arc::new(module_scope),
                };

                // Execute body in module context
                let mut last = Expr::Nil;
                for expr in &args[1..] {
                    last = eval(expr.clone(), &mut module_ctx);
                }

                // Collect definitions
                let mut map = BTreeMap::new();
                // We iterate module_ctx.scope.vars directly.
                {
                    let vars = module_ctx.scope.vars.read().unwrap();
                    println!("Harvesting module {}: found {} vars", name_sym, vars.len());
                    for (k, v) in vars.iter() {
                        println!("Exporting: {:?}", k);
                        map.insert(k.clone(), v.clone());
                    }
                }

                let mod_val = Expr::Ref(Arc::new(RwLock::new(Expr::Map(map))));

                // Define module in OUTER context
                ctx.define(Expr::Sym(name_sym), mod_val.clone());

                mod_val
            },
            "module",
            "Define a new module with its own scope. (module Name (def x 1)...)",
        ),
    );

    // Define (defun name (params) body)
    ctx.define(
        Expr::sym("defun"),
        Expr::extern_fun(
            |args, ctx| {
                if args.len() != 3 {
                    return Expr::Nil;
                }
                let name_expr = &args[0];
                let params_expr = &args[1];
                let body = &args[2];

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

                let func = Expr::Function {
                    params,
                    body: Box::new(body.clone()),
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

    // Define (struct Name (fields) (method1 ...) ...)
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

                // Parse methods
                // Each method def is (Name (Params) Body...)
                // We want to pre-create Expr::Function templates
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
                            // Implicit DO block if multiple expressions in body
                            Box::new(Expr::List(l[2..].iter().cloned().map(|e| e).collect()))
                            // Wait, implicit DO is handled by DO macro usually, or body is just one expr.
                            // Standard lisp `defun` usually allows multiple exprs as implied `do`.
                            // My `defun` takes 3 args: name, params, body (single expr).
                            // Let's stick to single expr body for now to match `defun`.
                        };

                        // Create the function template.
                        // IMPORTANT: The environment `env` should be the one where the struct is defined.
                        // When the method is called, `self` will be bound.
                        let method_func = Expr::Function {
                            params: m_params,
                            body: m_body,
                            env: ctx.clone(), // Capture definition context
                            name: Some(m_name.clone()),
                        };
                        methods.insert(m_name, method_func);
                    } else {
                        return Expr::Nil;
                    }
                }

                // Create Constructor Function
                let fields_clone = fields.clone();
                let methods_clone = methods.clone();

                let constructor = Expr::extern_fun(
                    move |ctor_args, _ctx| {
                        if ctor_args.len() != fields_clone.len() {
                            return Expr::Nil;
                        }

                        let mut obj_map = BTreeMap::new();

                        // Bind fields
                        for (i, field) in fields_clone.iter().enumerate() {
                            obj_map.insert(field.clone().into(), eval(ctor_args[i].clone(), _ctx));
                            // Wait, ctor_args are already evaluated? checking call site...
                            // Yes, `eval` evaluates args before calling ExternFunc.
                            // So `ctor_args[i]` is the value. We don't need `eval`.
                            // BUT, `Expr::extern_fun` signature: `args: &[Expr], ctx: &mut Context`.
                            // Who calls this? `eval` loop match Expr::List.
                            // `eval(func_expr)` -> Extern.
                            // `expr = f.call(&args, ctx)`.
                            // The `args` passed to `call` are RAW expressions from the `Expr::List`.
                            // `eval` creates the args vector: `let args = list[1..].to_vec();`.
                            // It does NOT evaluate them!
                            // Wait, let me check `eval` again.
                        }

                        // Re-check eval logic relative to ExternFunc args.
                        // In `eval`:
                        // `match eval(func_expr, ctx) { Expr::Extern(f) => expr = f.call(&args, ctx) }`
                        // `args` is `list[1..]`. Un-evaluated expressions.
                        // So `+` iterates args and calls `eval(arg)`.
                        // So here, we MUST call `eval(ctor_args[i])`. Correct.

                        for (i, field) in fields_clone.iter().enumerate() {
                            obj_map.insert(field.clone().into(), eval(ctor_args[i].clone(), _ctx));
                        }

                        // Bind methods
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
                    (Expr::Int(b), Expr::Int(e)) => Expr::Float((b as f64).powf(e as f64)), // Using powf for simplicity
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

    // --- List Extensions ---

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
                    _ => Expr::Int(0),
                }
            },
            "length",
            "Return length of list or string.",
        ),
    );

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
        // Test precedence: * (20) > + (10)
        let input_precedence = "1 + 2 * 3"; // 1 + 6 = 7
        let (_, expr) = parse_expr(input_precedence, &ctx).unwrap();
        let evaluated = eval(expr, &mut ctx.clone());
        assert_eq!(evaluated, Expr::Int(7));

        // Test associativity: - (Left)
        let input_assoc_left = "10 - 5 - 2"; // (10 - 5) - 2 = 3
        let (_, expr) = parse_expr(input_assoc_left, &ctx).unwrap();
        let evaluated = eval(expr, &mut ctx.clone());
        assert_eq!(evaluated, Expr::Int(3));

        // Test parentheses grouping
        let input_parens = "(1 + 2) * 3"; // 3 * 3 = 9
        let (_, expr) = parse_expr(input_parens, &ctx).unwrap();
        let evaluated = eval(expr, &mut ctx.clone());
        assert_eq!(evaluated, Expr::Int(9));

        // Test complex mixed operators
        // 10 + 3 * 2 - 4 / 2
        // 10 + 6 - 2 = 14
        let input_complex = "10 + 3 * 2 - 4 / 2";
        let (_, expr) = parse_expr(input_complex, &ctx).unwrap();
        let evaluated = eval(expr, &mut ctx.clone());
        assert_eq!(evaluated, Expr::Int(14));

        // Test mixed precedence
        // 1 + 2 > 2 -> 3 > 2 -> 1 (True)
        // If precedence of > was higher (unlikely) or equal to +, it might be different.
        // Actually > (5) < + (10). So + binds tighter.
        // (1 + 2) > 2.
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
        // 10 + 5 * 2 = 10 + 10 = 20
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
        // 15 > 10 -> true -> 15 - 5 = 10
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
                // Stirling's approx for 5 is ~118.019
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
        let builder = std::thread::Builder::new().stack_size(32 * 1024 * 1024); // 32MB

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
        // Test infix dot usage:
        // root.child.val  <-- (.(.root child ) val )
        // Using + with it: root.child.val + 10
        // Method: (obj.method) ()
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
        // Demonstrating nested mutation:
        // We have reference p.
        // We have reference r which has 'origin pointing to p.
        // We want to mutate p via r.
        // (. (r.origin) 'x 999)
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
        // ... (existing test content kept as is for now, actually I need to append, not replace if I want to keep it)
        // I will restart from view_file location.
        // Actually, let's just add the test at the end.
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
                // Distance (Float 5.0)
                if let Expr::Float(d) = l[2] {
                    assert!((d - 5.0).abs() < 0.001);
                } else {
                    panic!("Expected float for distance, got {:?}", l[2]);
                }

                // Add result (Point Ref)
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
