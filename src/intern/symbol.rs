use std::cell::RefCell;

use super::interner::Interner;

thread_local! {
    static INTERNER: RefCell<Interner> = RefCell::new(Interner::new());
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord, Default)]
pub struct Symbol(pub(crate) u32);

impl Symbol {
    pub fn intern(name: &str) -> Self {
        INTERNER.with(|i| i.borrow_mut().intern(name))
    }

    pub fn as_str(&self) -> String {
        INTERNER.with(|i| i.borrow().resolve(*self).to_string())
    }
}

impl std::fmt::Display for Symbol {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.as_str())
    }
}
