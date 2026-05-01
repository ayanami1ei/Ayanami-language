use std::collections::HashMap;

pub(crate) struct Interner {
    map: HashMap<String, u32>,
    vec: Vec<String>,
}

impl Interner {
    pub(crate) fn new() -> Self {
        let mut interner = Interner {
            map: HashMap::new(),
            vec: Vec::new(),
        };
        interner.vec.push(String::new());
        interner.map.insert(String::new(), 0);
        interner
    }

    pub(crate) fn intern(&mut self, name: &str) -> super::symbol::Symbol {
        if let Some(&id) = self.map.get(name) {
            return super::symbol::Symbol(id);
        }
        let id = self.vec.len() as u32;
        self.vec.push(name.to_string());
        self.map.insert(name.to_string(), id);
        super::symbol::Symbol(id)
    }

    pub(crate) fn resolve(&self, sym: super::symbol::Symbol) -> &str {
        &self.vec[sym.0 as usize]
    }
}
