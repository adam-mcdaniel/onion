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
