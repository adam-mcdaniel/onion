use crate::context::Context;
use crate::expr::Expr;
use std::collections::BTreeMap;
use std::sync::{Arc, RwLock};
use std::env;

pub fn register(ctx: &mut Context) {
    let mut os_exports = BTreeMap::new();

    os_exports.insert(Expr::sym("args"), Expr::extern_fun(|_args, _ctx| {
        let args: Vec<Expr> = env::args().map(Expr::Str).collect();
        Expr::List(args)
    }, "args", "Command line arguments"));

    os_exports.insert(Expr::sym("env"), Expr::extern_fun(|args, ctx| {
        match eval_first(args, ctx) {
            Expr::Str(key) => match env::var(key) {
                Ok(val) => Expr::Str(val),
                Err(_) => Expr::Nil
            },
           _ => {
               // Return all env vars as Map if no arg or wrong arg?
               // Let's stick to get.
               Expr::Nil
           }
        }
    }, "env", "Get environment variable"));

    os_exports.insert(Expr::sym("exit"), Expr::extern_fun(|args, ctx| {
        let code = match eval_first(args, ctx) {
            Expr::Int(n) => n as i32,
            _ => 0
        };
        std::process::exit(code);
    }, "exit", "Exit process"));
    
    // Command execution
    os_exports.insert(Expr::sym("exec"), Expr::extern_fun(|args, ctx| {
         if args.is_empty() { return Expr::Nil; }
         
         let mut cmd_args = Vec::new();
         for arg in args {
             match crate::context::eval(arg.clone(), ctx) {
                 Expr::Str(s) => cmd_args.push(s),
                 _ => return Expr::Nil
             }
         }
         
         if cmd_args.is_empty() { return Expr::Nil; }
         
         let command = &cmd_args[0];
         let mut cmd = std::process::Command::new(command);
         if cmd_args.len() > 1 {
             cmd.args(&cmd_args[1..]);
         }
         
         match cmd.output() {
             Ok(output) => {
                 let stdout = String::from_utf8_lossy(&output.stdout).to_string();
                 Expr::Str(stdout)
             }
             Err(_) => Expr::Nil
         }
    }, "exec", "Execute command"));

    let mod_val = Expr::Ref(Arc::new(RwLock::new(Expr::Map(os_exports))));
    ctx.define(Expr::sym("OS"), mod_val);
}

fn eval_first(args: &[Expr], ctx: &mut Context) -> Expr {
    if args.len() != 1 {
        Expr::Nil
    } else {
        crate::context::eval(args[0].clone(), ctx)
    }
}
