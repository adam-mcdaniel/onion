use crate::context::Context;
use crate::expr::Expr;
use std::collections::BTreeMap;
use std::sync::{Arc, RwLock};
use std::time::{SystemTime, UNIX_EPOCH, Duration};

pub fn register(ctx: &mut Context) {
    let mut time_exports = BTreeMap::new();

    time_exports.insert(Expr::sym("now"), Expr::extern_fun(|_args, _ctx| {
        let start = SystemTime::now();
        let since_the_epoch = start.duration_since(UNIX_EPOCH)
            .expect("Time went backwards");
        Expr::Float(since_the_epoch.as_secs_f64())
    }, "now", "Current unix timestamp (seconds)"));

    time_exports.insert(Expr::sym("sleep"), Expr::extern_fun(|args, ctx| {
        match eval_first(args, ctx) {
            Expr::Int(n) => {
                std::thread::sleep(Duration::from_millis(n as u64));
                Expr::Nil
            }
            Expr::Float(f) => {
                 std::thread::sleep(Duration::from_secs_f64(f));
                 Expr::Nil
            }
            _ => Expr::Nil
        }
    }, "sleep", "Sleep for N milliseconds (int) or N seconds (float)"));

    let mod_val = Expr::Ref(Arc::new(RwLock::new(Expr::Map(time_exports))));
    ctx.define(Expr::sym("Time"), mod_val);
}

fn eval_first(args: &[Expr], ctx: &mut Context) -> Expr {
    if args.len() != 1 {
        Expr::Nil
    } else {
        crate::context::eval(args[0].clone(), ctx)
    }
}
