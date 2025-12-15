use crate::context::Context;
use crate::expr::Expr;
use std::collections::BTreeMap;
use std::sync::{Arc, RwLock};

pub fn register(ctx: &mut Context) {
    let mut string_exports = BTreeMap::new();

    // Inspection
    string_exports.insert(Expr::sym("len"), Expr::extern_fun(|args, ctx| {
        match eval_first(args, ctx) {
            Expr::Str(s) => Expr::Int(s.len() as i64),
            Expr::Nil => Expr::Int(0),
            _ => Expr::Nil
        }
    }, "len", "Length of string"));

    string_exports.insert(Expr::sym("is_empty"), Expr::extern_fun(|args, ctx| {
        match eval_first(args, ctx) {
            Expr::Str(s) => if s.is_empty() { Expr::Int(1) } else { Expr::Nil },
            Expr::Nil => Expr::Int(1),
            _ => Expr::Nil
        }
    }, "is_empty", "Check if empty"));

    // Manipulation
    string_exports.insert(Expr::sym("trim"), Expr::extern_fun(|args, ctx| {
        match eval_first(args, ctx) {
            Expr::Str(s) => Expr::Str(s.trim().to_string()),
            _ => Expr::Nil
        }
    }, "trim", "Trim whitespace"));

    string_exports.insert(Expr::sym("to_upper"), Expr::extern_fun(|args, ctx| {
        match eval_first(args, ctx) {
            Expr::Str(s) => Expr::Str(s.to_uppercase()),
            _ => Expr::Nil
        }
    }, "to_upper", "To Uppercase"));

    string_exports.insert(Expr::sym("to_lower"), Expr::extern_fun(|args, ctx| {
        match eval_first(args, ctx) {
            Expr::Str(s) => Expr::Str(s.to_lowercase()),
            _ => Expr::Nil
        }
    }, "to_lower", "To Lowercase"));

    string_exports.insert(Expr::sym("split"), Expr::extern_fun(|args, ctx| {
        if args.len() != 2 { return Expr::Nil; }
        let s_expr = crate::context::eval(args[0].clone(), ctx);
        let sep_expr = crate::context::eval(args[1].clone(), ctx);
        
        match (s_expr, sep_expr) {
            (Expr::Str(s), Expr::Str(sep)) => {
                Expr::List(s.split(&sep).map(|part| Expr::Str(part.to_string())).collect())
            }
            _ => Expr::Nil
        }
    }, "split", "Split string by separator"));

    string_exports.insert(Expr::sym("join"), Expr::extern_fun(|args, ctx| {
        if args.len() != 2 { return Expr::Nil; }
        let list_expr = crate::context::eval(args[0].clone(), ctx);
        let sep_expr = crate::context::eval(args[1].clone(), ctx);

        match (list_expr, sep_expr) {
            (Expr::List(l), Expr::Str(sep)) => {
                let parts: Vec<String> = l.iter().map(|e| {
                   match e {
                       Expr::Str(s) => s.clone(),
                       _ => e.to_string()
                   }
                }).collect();
                Expr::Str(parts.join(&sep))
            }
            _ => Expr::Nil
        }
    }, "join", "Join list of strings with separator"));
    
    string_exports.insert(Expr::sym("replace"), Expr::extern_fun(|args, ctx| {
        if args.len() != 3 { return Expr::Nil; }
        let s = crate::context::eval(args[0].clone(), ctx);
        let old = crate::context::eval(args[1].clone(), ctx);
        let new = crate::context::eval(args[2].clone(), ctx);
        
        match (s, old, new) {
            (Expr::Str(s), Expr::Str(o), Expr::Str(n)) => Expr::Str(s.replace(&o, &n)),
            _ => Expr::Nil
        }
    }, "replace", "Replace occurrences"));
    
    string_exports.insert(Expr::sym("substring"), Expr::extern_fun(|args, ctx| {
         if args.len() < 2 { return Expr::Nil; }
         let s = crate::context::eval(args[0].clone(), ctx);
         let start = crate::context::eval(args[1].clone(), ctx);
         let len_opt = if args.len() > 2 { Some(crate::context::eval(args[2].clone(), ctx)) } else { None };

         match (s, start) {
             (Expr::Str(s), Expr::Int(start_idx)) => {
                 let start_idx = start_idx.max(0) as usize;
                 if start_idx >= s.len() { return Expr::Str("".to_string()); }
                 
                 let end_idx = if let Some(Expr::Int(l)) = len_opt {
                     (start_idx + l.max(0) as usize).min(s.len())
                 } else {
                     s.len()
                 };
                 
                 Expr::Str(s[start_idx..end_idx].to_string())
             }
             _ => Expr::Nil
         }
    }, "substring", "Get substring"));


    string_exports.insert(Expr::sym("chars"), Expr::extern_fun(|args, ctx| {
        match eval_first(args, ctx) {
            Expr::Str(s) => {
                let chars: Vec<Expr> = s.chars().map(|c| Expr::Str(c.to_string())).collect();
                Expr::List(chars)
            },
            _ => Expr::Nil
        }
    }, "chars", "Split string into list of characters"));

    string_exports.insert(Expr::sym("lines"), Expr::extern_fun(|args, ctx| {
        match eval_first(args, ctx) {
            Expr::Str(s) => {
                let lines: Vec<Expr> = s.lines().map(|line| Expr::Str(line.to_string())).collect();
                Expr::List(lines)
            },
            _ => Expr::Nil
        }
    }, "lines", "Split string into lines"));

    // Manipulation
    string_exports.insert(Expr::sym("repeat"), Expr::extern_fun(|args, ctx| {
        if args.len() != 2 { return Expr::Nil; }
        let s = crate::context::eval(args[0].clone(), ctx);
        let n = crate::context::eval(args[1].clone(), ctx);
        match (s, n) {
            (Expr::Str(s), Expr::Int(n)) => {
                if n <= 0 { Expr::Str("".to_string()) }
                else { Expr::Str(s.repeat(n as usize)) }
            }
            _ => Expr::Nil
        }
    }, "repeat", "Repeat string n times"));

    string_exports.insert(Expr::sym("pad_left"), Expr::extern_fun(|args, ctx| {
        if args.len() < 2 { return Expr::Nil; }
        let s = crate::context::eval(args[0].clone(), ctx).as_str().unwrap_or("").to_string();
        let width = crate::context::eval(args[1].clone(), ctx).as_int().unwrap_or(0) as usize;
        let pad_char = if args.len() > 2 {
            crate::context::eval(args[2].clone(), ctx).as_str().unwrap_or(" ").chars().next().unwrap_or(' ')
        } else { ' ' };
        
        if s.len() >= width {
            Expr::Str(s)
        } else {
            let padding = width - s.len();
            let mut res = String::new();
            for _ in 0..padding { res.push(pad_char); }
            res.push_str(&s);
            Expr::Str(res)
        }
    }, "pad_left", "Pad string on left to width"));

    string_exports.insert(Expr::sym("pad_right"), Expr::extern_fun(|args, ctx| {
        if args.len() < 2 { return Expr::Nil; }
        let s = crate::context::eval(args[0].clone(), ctx).as_str().unwrap_or("").to_string();
        let width = crate::context::eval(args[1].clone(), ctx).as_int().unwrap_or(0) as usize;
        let pad_char = if args.len() > 2 {
            crate::context::eval(args[2].clone(), ctx).as_str().unwrap_or(" ").chars().next().unwrap_or(' ')
        } else { ' ' };
        
        if s.len() >= width {
            Expr::Str(s)
        } else {
            let padding = width - s.len();
            let mut res = s;
            for _ in 0..padding { res.push(pad_char); }
            Expr::Str(res)
        }
    }, "pad_right", "Pad string on right to width"));

    // Predicates
    string_exports.insert(Expr::sym("starts_with"), Expr::extern_fun(|args, ctx| {
        if args.len() != 2 { return Expr::Nil; }
        let s = crate::context::eval(args[0].clone(), ctx);
        let prefix = crate::context::eval(args[1].clone(), ctx);
        match (s, prefix) {
            (Expr::Str(s), Expr::Str(p)) => if s.starts_with(&p) { Expr::Int(1) } else { Expr::Nil },
            _ => Expr::Nil
        }
    }, "starts_with", "Check prefix"));

    string_exports.insert(Expr::sym("ends_with"), Expr::extern_fun(|args, ctx| {
        if args.len() != 2 { return Expr::Nil; }
        let s = crate::context::eval(args[0].clone(), ctx);
        let suffix = crate::context::eval(args[1].clone(), ctx);
        match (s, suffix) {
            (Expr::Str(s), Expr::Str(p)) => if s.ends_with(&p) { Expr::Int(1) } else { Expr::Nil },
            _ => Expr::Nil
        }
    }, "ends_with", "Check suffix"));

    string_exports.insert(Expr::sym("contains"), Expr::extern_fun(|args, ctx| {
        if args.len() != 2 { return Expr::Nil; }
        let s = crate::context::eval(args[0].clone(), ctx);
        let sub = crate::context::eval(args[1].clone(), ctx);
        match (s, sub) {
            (Expr::Str(s), Expr::Str(sub)) => if s.contains(&sub) { Expr::Int(1) } else { Expr::Nil },
            _ => Expr::Nil
        }
    }, "contains", "Check contains"));

    // Formatting
    string_exports.insert(Expr::sym("fmt"), Expr::extern_fun(|args, ctx| {
         if args.is_empty() { return Expr::Nil; }
         let template_val = crate::context::eval(args[0].clone(), ctx);
         let template = match template_val {
             Expr::Str(s) => s,
             _ => return Expr::Nil
         };
         
         let mut result = String::new();
         let mut arg_idx = 1;
         let mut chars = template.chars().peekable();
         
         while let Some(c) = chars.next() {
             if c == '{' {
                 if let Some(&'}') = chars.peek() {
                     chars.next(); // Consume '}'
                     if arg_idx < args.len() {
                         let val = crate::context::eval(args[arg_idx].clone(), ctx);
                         match val {
                             Expr::Str(s) => result.push_str(&s),
                             _ => result.push_str(&val.to_string())
                         }
                         arg_idx += 1;
                     } else {
                         result.push_str("{}"); // Not enough args
                     }
                 } else {
                     result.push(c);
                 }
             } else {
                 result.push(c);
             }
         }
         Expr::Str(result)
    }, "fmt", "Format string with {} placeholders"));

    let mod_val = Expr::Ref(Arc::new(RwLock::new(Expr::Map(string_exports))));
    ctx.define(Expr::sym("String"), mod_val);
}

fn eval_first(args: &[Expr], ctx: &mut Context) -> Expr {
    if args.len() != 1 {
        Expr::Nil
    } else {
        crate::context::eval(args[0].clone(), ctx)
    }
}
