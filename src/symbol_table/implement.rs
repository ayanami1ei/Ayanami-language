use crate::{
    symbol_table::{Scope, Symbol, SymbolTable},
    types::VarType,
};
use std::{cell::RefCell, rc::Rc, usize};

impl Symbol {
    pub(crate) fn new_var(name: String, scope: Rc<RefCell<Scope>>) -> Symbol {
        Symbol {
            name: name,
            is_func: false,
            args: Vec::<Symbol>::new(),
            is_argc: false,
            is_ref: false,
            is_var: true,
            its_type: VarType::Unknown,
            area: Rc::downgrade(&scope),
        }
    }
    pub(crate) fn new_func(name: String, args: Vec<Symbol>, scope: Rc<RefCell<Scope>>) -> Symbol {
        Symbol {
            name: name,
            is_func: true,
            args: args,
            is_argc: false,
            is_ref: false,
            is_var: false,
            its_type: VarType::Unknown,
            area: Rc::downgrade(&scope),
        }
    }
    pub(crate) fn new_argc(name: String, is_ref: bool, scope: Rc<RefCell<Scope>>) -> Symbol {
        Symbol {
            name: name,
            is_func: false,
            args: Vec::<Symbol>::new(),
            is_argc: true,
            is_ref: is_ref,
            is_var: false,
            its_type: VarType::Unknown,
            area: Rc::downgrade(&scope),
        }
    }
}

impl Scope {
    pub(super) fn new() -> Scope {
        Scope {
            parent: None,
            sons: Vec::<Rc<RefCell<Scope>>>::new(),
            symbol: Vec::<Symbol>::new(),
        }
    }

    pub(super) fn add_symbol(&mut self, symbol: Symbol) {
        self.symbol.push(symbol);
    }

    pub(super) fn add_son_scope(&mut self, scope: Rc<RefCell<Scope>>) {
        self.sons.push(scope);
    }

    pub(super) fn set_parent_scope(&mut self, scope: Rc<RefCell<Scope>>) {
        self.parent = Some(Rc::downgrade(&scope));
    }
}

impl SymbolTable {
    pub(crate) fn new() -> SymbolTable {
        let _area = Rc::new(RefCell::new(Scope::new()));
        let _area_ptr = Rc::downgrade(&_area);
        SymbolTable {
            area: _area,
            area_ptr: _area_ptr,
        }
    }

    pub(crate) fn add_symbol(&mut self, symbol: Symbol) {
        if let Some(scope_ptr) = self.area_ptr.upgrade() {
            #[cfg(debug_assertions)]
            {
                println!("add type {}", symbol.its_type);
            }
            let mut binding = scope_ptr.borrow_mut();
            binding.add_symbol(symbol);
        }
    }

    pub(crate) fn into_new_scope(&mut self) {
        let scope = Rc::new(RefCell::new(Scope::new()));

        if let Some(scope_ptr) = self.area_ptr.upgrade() {
            let mut binding = scope_ptr.borrow_mut();
            binding.add_son_scope(scope.clone());

            let mut son_binding = scope.borrow_mut();
            son_binding.set_parent_scope(scope_ptr.clone());
        }

        self.area_ptr = Rc::downgrade(&scope);
    }

    pub(crate) fn ret_to_parent_scope(&mut self) {
        if let Some(scope_ptr) = self.area_ptr.upgrade() {
            let binding = scope_ptr.borrow();
            if let Some(ref parent_ptr) = binding.parent {
                self.area_ptr = parent_ptr.clone();
            }
        }
    }

    fn find_in_vec(vec: &Vec<Symbol>, name: &String) -> usize {
        for i in 0..vec.len() {
            if vec[i].name == *name  && vec[i].is_var{
                return i;
            }
        }

        return usize::MAX;
    }

    pub(crate) fn find_symbol(&self, name: &String) -> Option<Symbol> {
        // 从当前 scope 开始
        let mut cur_opt = self.area_ptr.upgrade(); // Option<Rc<RefCell<Scope>>>

        while let Some(cur_rc) = cur_opt {
            // 这里单独开一个作用域，确保 borrow 在本轮结束前就被释放
            {
                let binding = cur_rc.borrow();

                let index = Self::find_in_vec(&binding.symbol, name);
                // 先在当前这一层作用域里找
                if index != usize::MAX {
                    return Some(binding.symbol[index].clone());
                }

                // 为下一轮准备：父作用域（可能是 None）
                cur_opt = binding.parent.as_ref().and_then(|w| w.upgrade());
            }
            // 离开这个 block，binding 的借用被释放，下一轮可以重新 borrow
        }

        None
    }

    pub(crate) fn get_scope(&self) -> Rc<RefCell<Scope>> {
        self.area_ptr.upgrade().unwrap().clone()
    }

    pub(crate) fn reset(&mut self) {
        self.area_ptr = Rc::downgrade(&self.area)
    }

    pub(crate) fn set_symbol_type(&mut self, name: &String, ty: VarType) {
        let mut cur_opt = self.area_ptr.upgrade();

        while let Some(cur_rc) = cur_opt {
            {
                let mut binding = cur_rc.borrow_mut();
                let index = Self::find_in_vec(&binding.symbol, name);
                if index != usize::MAX {
                    binding.symbol[index].its_type = ty.clone();
                    return;
                }
                cur_opt = binding.parent.as_ref().and_then(|w| w.upgrade());
            }
        }
    }
}
