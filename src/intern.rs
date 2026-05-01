use std::cell::RefCell;
use std::collections::HashMap;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord, Default)]
pub struct Symbol(u32);

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

struct Interner {
    map: HashMap<String, u32>,
    vec: Vec<String>,
}

impl Interner {
    fn new() -> Self {
        let mut interner = Interner {
            map: HashMap::new(),
            vec: Vec::new(),
        };
        interner.vec.push(String::new());
        interner.map.insert(String::new(), 0);
        interner
    }

    fn intern(&mut self, name: &str) -> Symbol {
        if let Some(&id) = self.map.get(name) {
            return Symbol(id);
        }
        let id = self.vec.len() as u32;
        self.vec.push(name.to_string());
        self.map.insert(name.to_string(), id);
        Symbol(id)
    }

    fn resolve(&self, sym: Symbol) -> &str {
        &self.vec[sym.0 as usize]
    }
}

thread_local! {
    static INTERNER: RefCell<Interner> = RefCell::new(Interner::new());
}
