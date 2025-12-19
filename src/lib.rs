#[macro_export]
macro_rules! stop {
    ($($arg:tt)*) => {{
        eprintln!("Runtime Error: {}", format!($($arg)*));
        // Print a backtrace if available
        let bt = std::backtrace::Backtrace::capture();
        if bt.status() == std::backtrace::BacktraceStatus::Captured {
             eprintln!("{}", bt);
        }
        std::process::exit(1);
    }};
}

mod symbol;
pub use symbol::Symbol;
pub mod parser;

pub mod expr;
pub use expr::Expr;

pub mod context;
pub use context::Context;

pub mod stdlib;
