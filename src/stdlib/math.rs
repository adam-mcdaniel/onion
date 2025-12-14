use crate::context::Context;
use crate::expr::Expr;
use std::collections::BTreeMap;
use std::sync::{Arc, RwLock};

pub fn register(ctx: &mut Context) {
    let mut math_exports = BTreeMap::new();

    // Constants
    math_exports.insert(Expr::sym("PI"), Expr::Float(std::f64::consts::PI));
    math_exports.insert(Expr::sym("E"), Expr::Float(std::f64::consts::E));
    math_exports.insert(Expr::sym("TAU"), Expr::Float(std::f64::consts::TAU));
    math_exports.insert(Expr::sym("INF"), Expr::Float(f64::INFINITY));
    math_exports.insert(Expr::sym("NAN"), Expr::Float(f64::NAN));

    // Basic Functions
    math_exports.insert(Expr::sym("abs"), Expr::extern_fun(|args, ctx| {
        match eval_first(args, ctx) {
            Expr::Int(n) => Expr::Int(n.abs()),
            Expr::Float(f) => Expr::Float(f.abs()),
            _ => Expr::Nil
        }
    }, "abs", "Absolute value"));

    math_exports.insert(Expr::sym("ceil"), Expr::extern_fun(|args, ctx| {
         match eval_first(args, ctx) {
            Expr::Int(n) => Expr::Int(n),
            Expr::Float(f) => Expr::Int(f.ceil() as i64),
            _ => Expr::Nil
        }
    }, "ceil", "Ceiling"));

    math_exports.insert(Expr::sym("floor"), Expr::extern_fun(|args, ctx| {
         match eval_first(args, ctx) {
            Expr::Int(n) => Expr::Int(n),
            Expr::Float(f) => Expr::Int(f.floor() as i64),
            _ => Expr::Nil
        }
    }, "floor", "Floor"));
    
    math_exports.insert(Expr::sym("round"), Expr::extern_fun(|args, ctx| {
         match eval_first(args, ctx) {
             Expr::Int(n) => Expr::Int(n),
             Expr::Float(f) => Expr::Int(f.round() as i64),
             _ => Expr::Nil
         }
    }, "round", "Round to nearest integer"));

    // Trigonometry
    math_exports.insert(Expr::sym("sin"), Expr::extern_fun(|args, ctx| {
         match eval_first(args, ctx) {
             Expr::Int(n) => Expr::Float((n as f64).sin()),
             Expr::Float(f) => Expr::Float(f.sin()),
             _ => Expr::Nil
         }
    }, "sin", "Sine"));
    
    math_exports.insert(Expr::sym("cos"), Expr::extern_fun(|args, ctx| {
         match eval_first(args, ctx) {
             Expr::Int(n) => Expr::Float((n as f64).cos()),
             Expr::Float(f) => Expr::Float(f.cos()),
             _ => Expr::Nil
         }
    }, "cos", "Cosine"));
    
    math_exports.insert(Expr::sym("tan"), Expr::extern_fun(|args, ctx| {
         match eval_first(args, ctx) {
             Expr::Int(n) => Expr::Float((n as f64).tan()),
             Expr::Float(f) => Expr::Float(f.tan()),
             _ => Expr::Nil
         }
    }, "tan", "Tangent"));

    math_exports.insert(Expr::sym("sqrt"), Expr::extern_fun(|args, ctx| {
         match eval_first(args, ctx) {
             Expr::Int(n) => Expr::Float((n as f64).sqrt()),
             Expr::Float(f) => Expr::Float(f.sqrt()),
             _ => Expr::Nil
         }
    }, "sqrt", "Square root"));

    math_exports.insert(Expr::sym("pow"), Expr::extern_fun(|args, ctx| {
         if args.len() != 2 { return Expr::Nil; }
         let base = crate::context::eval(args[0].clone(), ctx);
         let exp = crate::context::eval(args[1].clone(), ctx);
         match (base, exp) {
             (Expr::Int(b), Expr::Int(e)) => Expr::Float((b as f64).powf(e as f64)),
             (Expr::Float(b), Expr::Float(e)) => Expr::Float(b.powf(e)),
             (Expr::Int(b), Expr::Float(e)) => Expr::Float((b as f64).powf(e)),
             (Expr::Float(b), Expr::Int(e)) => Expr::Float(b.powf(e as f64)),
             _ => Expr::Nil
         }
    }, "pow", "Power"));

    // Min/Max
    math_exports.insert(Expr::sym("min"), Expr::extern_fun(|args, ctx| {
        let mut min_val = Expr::Nil;
        for arg in args {
             let val = crate::context::eval(arg.clone(), ctx);
             if min_val == Expr::Nil {
                 min_val = val; 
             } else {
                 match (&min_val, &val) {
                     (Expr::Int(a), Expr::Int(b)) => if b < a { min_val = val; },
                     (Expr::Float(a), Expr::Float(b)) => if b < a { min_val = val; },
                     (Expr::Int(a), Expr::Float(b)) => if *b < (*a as f64) { min_val = val; },
                     (Expr::Float(a), Expr::Int(b)) => if (*b as f64) < *a { min_val = val; },
                     _ => {}
                 }
             }
        }
        min_val
    }, "min", "Minimum value"));
    
    math_exports.insert(Expr::sym("max"), Expr::extern_fun(|args, ctx| {
         let mut max_val = Expr::Nil;
         for arg in args {
              let val = crate::context::eval(arg.clone(), ctx);
              if max_val == Expr::Nil {
                  max_val = val; 
              } else {
                  match (&max_val, &val) {
                      (Expr::Int(a), Expr::Int(b)) => if b > a { max_val = val; },
                      (Expr::Float(a), Expr::Float(b)) => if b > a { max_val = val; },
                      (Expr::Int(a), Expr::Float(b)) => if *b > (*a as f64) { max_val = val; },
                      (Expr::Float(a), Expr::Int(b)) => if (*b as f64) > *a { max_val = val; },
                      _ => {}
                  }
              }
         }
         max_val
     }, "max", "Maximum value"));


    // Random
    math_exports.insert(Expr::sym("rand"), Expr::extern_fun(|_args, _ctx| {
        Expr::Float(rand::random())
    }, "rand", "Random float 0.0..1.0"));

    // Define 'Math' module in context
    let math_mod = Expr::Ref(Arc::new(RwLock::new(Expr::Map(math_exports))));
    ctx.define(Expr::sym("Math"), math_mod);
}

fn eval_first(args: &[Expr], ctx: &mut Context) -> Expr {
    if args.len() != 1 {
        Expr::Nil
    } else {
        crate::context::eval(args[0].clone(), ctx)
    }
}
