use crate::context::Context;
use crate::expr::Expr;
use std::collections::BTreeMap;
use std::sync::{Arc, RwLock};
use std::fs;
use std::io::Write;

pub fn register(ctx: &mut Context) {
    let mut io_exports = BTreeMap::new();

    io_exports.insert(Expr::sym("read_file"), Expr::extern_fun(|args, ctx| {
        match eval_first(args, ctx) {
            Expr::Str(path) => match fs::read_to_string(path) {
                Ok(content) => Expr::Str(content),
                Err(_) => Expr::Nil
            },
            _ => Expr::Nil
        }
    }, "read_file", "Read file content"));

    io_exports.insert(Expr::sym("write_file"), Expr::extern_fun(|args, ctx| {
        if args.len() != 2 { return Expr::Nil; }
        let path = crate::context::eval(args[0].clone(), ctx);
        let content = crate::context::eval(args[1].clone(), ctx);
        
        match (path, content) {
            (Expr::Str(p), Expr::Str(c)) => {
                 match fs::write(p, c) {
                     Ok(_) => Expr::Int(1),
                     Err(_) => Expr::Nil
                 }
            }
            _ => Expr::Nil
        }
    }, "write_file", "Write content to file"));
    
    // Append
    io_exports.insert(Expr::sym("append_file"), Expr::extern_fun(|args, ctx| {
        if args.len() != 2 { return Expr::Nil; }
        let path = crate::context::eval(args[0].clone(), ctx);
        let content = crate::context::eval(args[1].clone(), ctx);
        
        match (path, content) {
            (Expr::Str(p), Expr::Str(c)) => {
                 match std::fs::OpenOptions::new().append(true).create(true).open(p) {
                     Ok(mut file) => {
                         if let Ok(_) = write!(file, "{}", c) {
                             Expr::Int(1)
                         } else {
                             Expr::Nil
                         }
                     }
                     Err(_) => Expr::Nil
                 }
            }
            _ => Expr::Nil
        }
    }, "append_file", "Append content to file"));

    io_exports.insert(Expr::sym("read_line"), Expr::extern_fun(|_args, _ctx| {
        let mut input = String::new();
        match std::io::stdin().read_line(&mut input) {
            Ok(_) => Expr::Str(input.trim_end().to_string()),
            Err(_) => Expr::Nil
        }
    }, "read_line", "Read line from stdin"));

    io_exports.insert(Expr::sym("exists"), Expr::extern_fun(|args, ctx| {
        match eval_first(args, ctx) {
            Expr::Str(p) => if std::path::Path::new(&p).exists() { Expr::Int(1) } else { Expr::Nil },
            _ => Expr::Nil
        }
    }, "exists", "Check if path exists"));

    io_exports.insert(Expr::sym("remove_file"), Expr::extern_fun(|args, ctx| {
        match eval_first(args, ctx) {
            Expr::Str(p) => match std::fs::remove_file(p) {
                Ok(_) => Expr::Int(1),
                Err(_) => Expr::Nil
            },
            _ => Expr::Nil
        }
    }, "remove_file", "Remove file"));

    io_exports.insert(Expr::sym("is_dir"), Expr::extern_fun(|args, ctx| {
        match eval_first(args, ctx) {
            Expr::Str(p) => if std::path::Path::new(&p).is_dir() { Expr::Int(1) } else { Expr::Nil },
            _ => Expr::Nil
        }
    }, "is_dir", "Check if path is directory"));

    io_exports.insert(Expr::sym("is_file"), Expr::extern_fun(|args, ctx| {
        match eval_first(args, ctx) {
            Expr::Str(p) => if std::path::Path::new(&p).is_file() { Expr::Int(1) } else { Expr::Nil },
            _ => Expr::Nil
        }
    }, "is_file", "Check if path is file"));

    let mod_val = Expr::Ref(Arc::new(RwLock::new(Expr::Map(io_exports))));
    ctx.define(Expr::sym("IO"), mod_val);
}

fn eval_first(args: &[Expr], ctx: &mut Context) -> Expr {
    if args.len() != 1 {
        Expr::Nil
    } else {
        crate::context::eval(args[0].clone(), ctx)
    }
}
