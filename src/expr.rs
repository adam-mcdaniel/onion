use super::Context;
use super::Symbol;
use std::collections::{BTreeMap, HashMap};
use std::sync::{Arc, RwLock};

#[derive(Clone)]
pub struct ExternFunc {
    func: Arc<dyn Fn(&[Expr], &mut Context) -> Expr + Send + Sync>,
    short_desc: String,
    long_desc: String,
}

impl ExternFunc {
    pub fn new<F, S1, S2>(func: F, short_desc: S1, long_desc: S2) -> Self
    where
        F: Fn(&[Expr], &mut Context) -> Expr + Send + Sync + 'static,
        S1: Into<String>,
        S2: Into<String>,
    {
        Self {
            func: Arc::new(func),
            short_desc: short_desc.into(),
            long_desc: long_desc.into(),
        }
    }

    pub fn call(&self, args: &[Expr], ctx: &mut Context) -> Expr {
        (self.func)(args, ctx)
    }

    pub fn short_desc(&self) -> &str {
        &self.short_desc
    }

    pub fn long_desc(&self) -> &str {
        &self.long_desc
    }
}

impl std::fmt::Debug for ExternFunc {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "ExternFunc({{...}})")
    }
}

impl PartialEq for ExternFunc {
    fn eq(&self, other: &Self) -> bool {
        Arc::ptr_eq(&self.func, &other.func)
    }
}

impl Eq for ExternFunc {}

impl std::hash::Hash for ExternFunc {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        let ptr = Arc::as_ptr(&self.func);
        ptr.hash(state);
    }
}

impl PartialOrd for ExternFunc {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        let ptr1 = Arc::as_ptr(&self.func);
        let ptr2 = Arc::as_ptr(&other.func);
        ptr1.cast::<()>().partial_cmp(&ptr2.cast::<()>())
    }
}

impl Ord for ExternFunc {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        let ptr1 = Arc::as_ptr(&self.func);
        let ptr2 = Arc::as_ptr(&other.func);
        ptr1.cast::<()>().cmp(&ptr2.cast::<()>())
    }
}

#[derive(Clone, Debug)]
pub enum Expr {
    Nil,
    Int(i64),
    Float(f64),
    Str(String),
    Sym(Symbol),
    List(Vec<Expr>),
    Vector(Vec<Expr>),
    Map(BTreeMap<Expr, Expr>),
    HashMap(HashMap<Expr, Expr>),
    Tagged {
        tag: Symbol,
        value: Box<Expr>,
    },
    Extern(ExternFunc),
    Quoted(Box<Expr>),
    Function {
        params: Vec<Symbol>,
        body: Box<Expr>,
        env: Context,
        name: Option<Symbol>,
    },
    Ref(Arc<RwLock<Expr>>),
}

impl Eq for Expr {}

impl Expr {
    pub fn extern_fun<F, S1, S2>(func: F, short_desc: S1, long_desc: S2) -> Self
    where
        F: Fn(&[Expr], &mut Context) -> Expr + Send + Sync + 'static,
        S1: Into<String>,
        S2: Into<String>,
    {
        Expr::Extern(ExternFunc::new(func, short_desc, long_desc))
    }

    pub fn sym<S: Into<Symbol>>(s: S) -> Self {
        Expr::Sym(s.into())
    }

    pub fn str<S: Into<String>>(s: S) -> Self {
        Expr::Str(s.into())
    }

    pub fn is_int(&self) -> bool {
        matches!(self, Expr::Int(_))
    }

    pub fn is_float(&self) -> bool {
        matches!(self, Expr::Float(_))
    }

    pub fn is_number(&self) -> bool {
        self.is_int() || self.is_float()
    }

    pub fn is_str(&self) -> bool {
        matches!(self, Expr::Str(_))
    }

    pub fn is_sym(&self) -> bool {
        matches!(self, Expr::Sym(_))
    }

    pub fn is_list(&self) -> bool {
        matches!(self, Expr::List(_))
    }

    pub fn is_map(&self) -> bool {
        matches!(self, Expr::Map(_))
    }

    pub fn as_int(&self) -> Option<i64> {
        if let Expr::Int(i) = self {
            Some(*i)
        } else {
            None
        }
    }

    pub fn as_float(&self) -> Option<f64> {
        if let Expr::Float(f) = self {
            Some(*f)
        } else {
            None
        }
    }

    pub fn as_number(&self) -> Option<f64> {
        match self {
            Expr::Int(i) => Some(*i as f64),
            Expr::Float(f) => Some(*f),
            _ => None,
        }
    }

    pub fn as_str(&self) -> Option<&str> {
        if let Expr::Str(s) = self {
            Some(s)
        } else {
            None
        }
    }

    pub fn as_sym(&self) -> Option<&Symbol> {
        if let Expr::Sym(s) = self {
            Some(s)
        } else {
            None
        }
    }

    pub fn as_list(&self) -> Option<&[Expr]> {
        if let Expr::List(l) = self {
            Some(l)
        } else {
            None
        }
    }

    pub fn as_map(&self) -> Option<&BTreeMap<Expr, Expr>> {
        if let Expr::Map(m) = self {
            Some(m)
        } else {
            None
        }
    }

    fn discriminant(&self) -> u8 {
        match self {
            Expr::Nil => 0,
            Expr::Int(_) => 1,
            Expr::Float(_) => 2,
            Expr::Str(_) => 3,
            Expr::Sym(_) => 4,
            Expr::List(_) => 5,
            Expr::Map(_) => 6,
            Expr::HashMap(_) => 7,
            Expr::Tagged { .. } => 8,
            Expr::Extern(_) => 9,
            Expr::Function { .. } => 10,
            Expr::Quoted(_) => 11,
            Expr::Ref(_) => 12,
            Expr::Vector(_) => 13,
        }
    }
}

impl std::hash::Hash for Expr {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.discriminant().hash(state);
        match self {
            Expr::Nil => {}
            Expr::Int(i) => {
                i.hash(state);
            }
            Expr::Float(f) => {
                // Hash the bits of the float
                f.to_bits().hash(state);
            }
            Expr::Str(s) => {
                s.hash(state);
            }
            Expr::Sym(s) => {
                s.hash(state);
            }
            Expr::List(l) => {
                for item in l {
                    item.hash(state);
                }
            }
            Expr::Map(m) => {
                for (k, v) in m {
                    k.hash(state);
                    v.hash(state);
                }
            }
            Expr::HashMap(hm) => {
                // To ensure order-independent hashing, we can hash the length and then each key-value pair sorted by key
                let mut items: Vec<(&Expr, &Expr)> = hm.iter().collect();
                items.sort_by(|a, b| a.0.cmp(b.0));
                for (k, v) in items {
                    k.hash(state);
                    v.hash(state);
                }
            }
            Expr::Tagged { tag, value } => {
                tag.hash(state);
                value.hash(state);
            }
            Expr::Extern(f) => {
                // Hash the function pointer address
                f.hash(state);
            }
            Expr::Function {
                params,
                body,
                env: _,
                name: _,
            } => {
                for param in params {
                    param.hash(state);
                }
                body.hash(state);
            }
            Expr::Quoted(expr) => {
                expr.hash(state);
            }
            Expr::Ref(r) => {
                // Hash the pointer address of the Arc
                Arc::as_ptr(r).hash(state);
            }
            Expr::Vector(v) => {
                for item in v {
                    item.hash(state);
                }
            }
        }
    }
}

impl PartialOrd for Expr {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for Expr {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        use std::cmp::Ordering;
        match (self, other) {
            (Expr::Nil, Expr::Nil) => Ordering::Equal,
            (Expr::Int(a), Expr::Int(b)) => a.cmp(b),
            (Expr::Float(a), Expr::Float(b)) => a.partial_cmp(b).unwrap_or(Ordering::Equal),
            (Expr::Str(a), Expr::Str(b)) => a.cmp(b),
            (Expr::Sym(a), Expr::Sym(b)) => a.cmp(b),
            (Expr::List(a), Expr::List(b)) => a.cmp(b),
            (Expr::Map(a), Expr::Map(b)) => a.cmp(b),
            (Expr::HashMap(a), Expr::HashMap(b)) => {
                let mut a_items: Vec<(&Expr, &Expr)> = a.iter().collect();
                let mut b_items: Vec<(&Expr, &Expr)> = b.iter().collect();
                a_items.sort_by(|x, y| x.0.cmp(y.0));
                b_items.sort_by(|x, y| x.0.cmp(y.0));
                a_items.cmp(&b_items)
            }
            (
                Expr::Tagged {
                    tag: atag,
                    value: aval,
                },
                Expr::Tagged {
                    tag: btag,
                    value: bval,
                },
            ) => match atag.cmp(btag) {
                Ordering::Equal => aval.cmp(bval),
                other => other,
            },
            (Expr::Extern(a), Expr::Extern(b)) => a.cmp(b),
            (
                Expr::Function {
                    params: aparams,
                    body: abody,
                    ..
                },
                Expr::Function {
                    params: bparams,
                    body: bbody,
                    ..
                },
            ) => match aparams.cmp(bparams) {
                Ordering::Equal => abody.cmp(bbody),
                other => other,
            },
            (Expr::Quoted(a), Expr::Quoted(b)) => a.cmp(b),
            (Expr::Ref(a), Expr::Ref(b)) => Arc::as_ptr(a).cmp(&Arc::as_ptr(b)),
            // Different variants are ordered by their discriminant
            _ => self.discriminant().cmp(&other.discriminant()),
        }
    }
}

impl PartialEq for Expr {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (Expr::Nil, Expr::Nil) => true,
            (Expr::Int(a), Expr::Int(b)) => a == b,
            (Expr::Float(a), Expr::Float(b)) => a == b,
            (Expr::Str(a), Expr::Str(b)) => a == b,
            (Expr::Sym(a), Expr::Sym(b)) => a == b,
            (Expr::List(a), Expr::List(b)) => a == b,
            (Expr::Map(a), Expr::Map(b)) => a == b,
            (Expr::HashMap(a), Expr::HashMap(b)) => a == b,
            (
                Expr::Tagged {
                    tag: atag,
                    value: aval,
                },
                Expr::Tagged {
                    tag: btag,
                    value: bval,
                },
            ) => atag == btag && aval == bval,
            (Expr::Quoted(a), Expr::Quoted(b)) => a == b,
            (Expr::Extern(a), Expr::Extern(b)) => a == b,
            (
                Expr::Function {
                    params: aparams,
                    body: abody,
                    ..
                },
                Expr::Function {
                    params: bparams,
                    body: bbody,
                    ..
                },
            ) => aparams == bparams && abody == bbody,
            (Expr::Ref(a), Expr::Ref(b)) => Arc::ptr_eq(a, b),
            _ => false,
        }
    }
}

impl From<i64> for Expr {
    fn from(i: i64) -> Self {
        Expr::Int(i)
    }
}

impl From<f64> for Expr {
    fn from(f: f64) -> Self {
        Expr::Float(f)
    }
}

impl From<String> for Expr {
    fn from(s: String) -> Self {
        Expr::Str(s)
    }
}

impl From<&str> for Expr {
    fn from(s: &str) -> Self {
        Expr::Str(s.to_string())
    }
}

impl From<Symbol> for Expr {
    fn from(s: Symbol) -> Self {
        Expr::Sym(s)
    }
}

impl From<Vec<Expr>> for Expr {
    fn from(v: Vec<Expr>) -> Self {
        Expr::List(v)
    }
}

impl From<BTreeMap<Expr, Expr>> for Expr {
    fn from(m: BTreeMap<Expr, Expr>) -> Self {
        Expr::Map(m)
    }
}

impl From<HashMap<Expr, Expr>> for Expr {
    fn from(m: HashMap<Expr, Expr>) -> Self {
        Expr::HashMap(m)
    }
}

impl std::fmt::Display for Expr {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Expr::Nil => write!(f, "nil"),
            Expr::Int(i) => write!(f, "{}", i),
            Expr::Float(fl) => write!(f, "{}", fl),
            Expr::Str(s) => write!(f, "{}", s),
            Expr::Sym(s) => write!(f, "{}", s),
            Expr::List(l) => {
                write!(f, "(")?;
                let mut first = true;
                for item in l {
                    if !first {
                        write!(f, " ")?;
                    }
                    write!(f, "{}", item)?;
                    first = false;
                }
                write!(f, ")")
            }
            Expr::Vector(l) => {
                write!(f, "[")?;
                let mut first = true;
                for item in l {
                    if !first {
                        write!(f, " ")?;
                    }
                    write!(f, "{}", item)?;
                    first = false;
                }
                write!(f, "]")
            }
            Expr::Map(m) => {
                write!(f, "#[")?; // Map uses #[ now? Or keep [ for Map?
                // Parser uses #[ for map?
                // Let's use #[ for Map to avoid confusing with Vector [
                let mut first = true;
                for (k, v) in m {
                    if !first {
                        write!(f, " ")?;
                    }
                    write!(f, "{} {}", k, v)?;
                    first = false;
                }
                write!(f, "]")
            }
            Expr::HashMap(hm) => {
                write!(f, "#[")?;
                let mut first = true;
                for (k, v) in hm {
                    if !first {
                        write!(f, " ")?;
                    }
                    write!(f, "{} {}", k, v)?;
                    first = false;
                }
                write!(f, "]")
            }
            Expr::Tagged { tag, value } => {
                write!(f, "{} {}", tag, value)
            }
            Expr::Extern(ext) => {
                write!(f, "<extern: {}>", ext.short_desc())
            }
            Expr::Function { params, body, .. } => {
                write!(f, "<function params: (")?;
                let mut first = true;
                for param in params {
                    if !first {
                        write!(f, " ")?;
                    }
                    write!(f, "{}", param)?;
                    first = false;
                }
                write!(f, ") body: {}>", body)
            }
            Expr::Quoted(expr) => {
                write!(f, "'{}", expr)
            }
            Expr::Ref(inner) => {
                // Try printing inner
                write!(f, "{}", inner.read().unwrap())
            }
        }
    }
}
