use lazy_static::lazy_static;
use std::sync::Arc;

/// An interned symbol.
#[derive(Clone, Eq, Hash, PartialOrd, Ord)]
pub struct Symbol(Arc<str>);

lazy_static! {
    static ref SYMBOL_TABLE: std::sync::RwLock<std::collections::HashSet<Arc<str>>> =
        std::sync::RwLock::new(std::collections::HashSet::new());
}

fn get_interned(s: &str) -> Arc<str> {
    let existing = {
        let table = SYMBOL_TABLE.read().unwrap();
        table.get(s).cloned()
    };
    if let Some(existing) = existing {
        existing
    } else {
        let mut table = SYMBOL_TABLE.write().unwrap();
        // Double-check if it was inserted while we were waiting for the lock
        if let Some(existing) = table.get(s) {
            return existing.clone();
        }
        let arc_str: Arc<str> = Arc::from(s);
        table.insert(arc_str.clone());
        arc_str
    }
}

impl Symbol {
    /// Creates a new `Symbol` from the given string slice.
    pub fn new(s: &str) -> Self {
        Self(get_interned(s))
    }

    /// Returns the string slice representation of the symbol.
    pub fn as_str(&self) -> &str {
        &self.0
    }

    /// Returns the length of the symbol.
    pub fn len(&self) -> usize {
        self.0.len()
    }

    /// Checks if the symbol is empty.
    pub fn is_empty(&self) -> bool {
        self.0.is_empty()
    }
}

impl std::fmt::Display for Symbol {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", &self.0)
    }
}

impl std::fmt::Debug for Symbol {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}", &self.0)
    }
}

impl From<&str> for Symbol {
    fn from(value: &str) -> Self {
        Symbol::new(value)
    }
}

impl From<String> for Symbol {
    fn from(value: String) -> Self {
        Symbol::new(&value)
    }
}

impl AsRef<str> for Symbol {
    fn as_ref(&self) -> &str {
        &self.0
    }
}

impl std::ops::Deref for Symbol {
    type Target = str;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl std::borrow::Borrow<str> for Symbol {
    fn borrow(&self) -> &str {
        &self.0
    }
}

impl PartialEq for Symbol {
    fn eq(&self, other: &Self) -> bool {
        Arc::ptr_eq(&self.0, &other.0)
    }
}

#[cfg(test)]
mod tests {
    use super::Symbol;
    #[test]
    fn test_symbol_interning() {
        let sym1 = Symbol::new("example");
        let sym2 = Symbol::new("example");
        assert!(sym1 == sym2);
        assert!(std::ptr::eq(&*sym1.0, &*sym2.0));
        let sym3 = Symbol::new("different");
        assert!(sym1 != sym3);
    }

    #[test]
    fn test_symbol_display() {
        let sym = Symbol::new("display_test");
        assert_eq!(format!("{}", sym), "display_test");
    }

    #[test]
    fn test_symbol_debug() {
        let sym = Symbol::new("debug_test");
        assert_eq!(format!("{:?}", sym), "\"debug_test\"");
    }

    #[test]
    fn test_symbol_as_str() {
        let sym = Symbol::new("as_str_test");
        assert_eq!(sym.as_str(), "as_str_test");
    }
}
