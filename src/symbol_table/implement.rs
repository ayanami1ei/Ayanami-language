use crate::{
    symbol_table::{Scope, Symbol, SymbolTable},
    types::VarType,
};
use std::{cell::RefCell, collections::HashSet, rc::Rc, usize};

static mut NEXT_ID: i32 = 0;

impl Symbol {
    pub(crate) fn new_var(name: String, scope: Rc<RefCell<Scope>>, level: i32) -> Symbol {
        unsafe {
            let res = Symbol {
                name: name,
                is_func: false,
                args: Vec::<Symbol>::new(),
                is_argc: false,
                is_ref: false,
                is_var: true,
                its_type: HashSet::new(),
                id: NEXT_ID,
                level,
                scope_id: scope.borrow().id,
                area: Rc::downgrade(&scope),
                body_scope_id: None,
                is_arr: false,
                elem_type: Vec::new(),
            };
            NEXT_ID += 1;
            res
        }
    }
    pub(crate) fn new_func(
        name: String,
        args: Vec<Symbol>,
        scope: Rc<RefCell<Scope>>,
        level: i32,
    ) -> Symbol {
        unsafe {
            let res = Symbol {
                name: name,
                is_func: true,
                args: args,
                is_argc: false,
                is_ref: false,
                is_var: false,
                its_type: HashSet::new(),
                id: NEXT_ID,
                level,
                scope_id: scope.borrow().id,
                area: Rc::downgrade(&scope),
                body_scope_id: None,
                is_arr: false,
                elem_type: Vec::new(),
            };
            NEXT_ID += 1;
            res
        }
    }
    pub(crate) fn new_argc(
        name: String,
        is_ref: bool,
        scope: Rc<RefCell<Scope>>,
        level: i32,
    ) -> Symbol {
        unsafe {
            let res = Symbol {
                name: name,
                is_func: false,
                args: Vec::<Symbol>::new(),
                is_argc: true,
                is_ref: is_ref,
                is_var: false,
                its_type: HashSet::new(),
                id: NEXT_ID,
                level,
                scope_id: scope.borrow().id,
                area: Rc::downgrade(&scope),
                body_scope_id: None,
                is_arr: false,
                elem_type: Vec::new(),
            };
            NEXT_ID += 1;
            res
        }
    }
    pub(crate) fn new_array(name: String, scope: Rc<RefCell<Scope>>, level: i32) -> Symbol {
        unsafe {
            let res = Symbol {
                name: name,
                is_func: false,
                args: Vec::<Symbol>::new(),
                is_argc: false,
                is_ref: false,
                is_var: true,
                its_type: HashSet::new(),
                id: NEXT_ID,
                level,
                scope_id: scope.borrow().id,
                area: Rc::downgrade(&scope),
                body_scope_id: None,
                is_arr: true,
                elem_type: Vec::new(),
            };
            NEXT_ID += 1;
            res
        }
    }
}

impl Scope {
    pub(super) fn new(id: i32) -> Scope {
        Scope {
            parent: None,
            sons: Vec::<Rc<RefCell<Scope>>>::new(),
            id,
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
        let _area = Rc::new(RefCell::new(Scope::new(0)));
        let _area_ptr = Rc::downgrade(&_area);
        SymbolTable {
            area: _area,
            area_ptr: _area_ptr,
            next_scope_id: 1,
            now_level: 1,
        }
    }

    pub(crate) fn add_symbol(&mut self, symbol: Symbol) {
        if let Some(scope_ptr) = self.area_ptr.upgrade() {
            let mut binding = scope_ptr.borrow_mut();
            binding.add_symbol(symbol);
        }
    }

    pub(crate) fn into_new_scope(&mut self) {
        let scope = Rc::new(RefCell::new(Scope::new(self.next_scope_id)));
        self.next_scope_id += 1;

        if let Some(scope_ptr) = self.area_ptr.upgrade() {
            let mut binding = scope_ptr.borrow_mut();
            binding.add_son_scope(scope.clone());

            let mut son_binding = scope.borrow_mut();
            son_binding.set_parent_scope(scope_ptr.clone());
        }

        self.area_ptr = Rc::downgrade(&scope);
        self.now_level += 1;
    }

    pub(crate) fn ret_to_parent_scope(&mut self) {
        if let Some(scope_ptr) = self.area_ptr.upgrade() {
            let binding = scope_ptr.borrow();
            if let Some(ref parent_ptr) = binding.parent {
                self.area_ptr = parent_ptr.clone();
            }
        }

        self.now_level -= 1;
    }

    fn find_in_vec(vec: &Vec<Symbol>, name: &String) -> usize {
        for i in 0..vec.len() {
            if vec[i].name == *name {
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

    pub(crate) fn set_func_body_scope(&mut self, name: &String, body_id: i32) {
        // search scopes recursively from root
        if Self::find_and_set_body_scope(&self.area, name, body_id) {
            return;
        }
    }

    pub(crate) fn add_func_arg_type(
        &mut self,
        name: &String,
        arg_index: usize,
        ty: HashSet<VarType>,
    ) {
        // search scopes recursively from root and merge the type into the function symbol's arg
        let _ = Self::find_and_add_func_arg_type(&self.area, name, arg_index, ty);
    }

    fn find_and_set_body_scope(scope: &Rc<RefCell<Scope>>, name: &String, body_id: i32) -> bool {
        let mut binding = scope.borrow_mut();
        for i in 0..binding.symbol.len() {
            if binding.symbol[i].name == *name && binding.symbol[i].is_func {
                binding.symbol[i].body_scope_id = Some(body_id);
                return true;
            }
        }

        // collect sons first to avoid nested borrow while recursing
        let sons = binding.sons.clone();
        drop(binding);

        for son in &sons {
            if Self::find_and_set_body_scope(son, name, body_id) {
                return true;
            }
        }

        false
    }

    pub(crate) fn get_scope(&self) -> Rc<RefCell<Scope>> {
        self.area_ptr.upgrade().unwrap().clone()
    }

    pub(crate) fn reset(&mut self) {
        self.area_ptr = Rc::downgrade(&self.area)
    }

    pub(crate) fn set_area_ptr_by_id(&mut self, id: i32) {
        if let Some(scope) = self.find_scope_by_id(&self.area, id) {
            self.area_ptr = Rc::downgrade(&scope);
        } else {
            eprintln!(
                "Warning: scope id {} not found, keeping current area_ptr",
                id
            );
        }

        self.now_level += 1;
    }

    fn find_scope_by_id(&self, scope: &Rc<RefCell<Scope>>, id: i32) -> Option<Rc<RefCell<Scope>>> {
        let binding = scope.borrow();
        if binding.id == id {
            return Some(scope.clone());
        }
        for son in &binding.sons {
            if let Some(found) = self.find_scope_by_id(son, id) {
                return Some(found);
            }
        }
        None
    }

    fn find_and_add_func_arg_type(
        scope: &Rc<RefCell<Scope>>,
        name: &String,
        arg_index: usize,
        ty: HashSet<VarType>,
    ) -> bool {
        let mut binding = scope.borrow_mut();
        for i in 0..binding.symbol.len() {
            if binding.symbol[i].name == *name && binding.symbol[i].is_func {
                if arg_index < binding.symbol[i].args.len() {
                    binding.symbol[i].args[arg_index].its_type = binding.symbol[i].args[arg_index]
                        .its_type
                        .union(&ty)
                        .cloned()
                        .collect();
                }
                return true;
            }
        }

        let sons = binding.sons.clone();
        drop(binding);

        for son in &sons {
            if Self::find_and_add_func_arg_type(son, name, arg_index, ty.clone()) {
                return true;
            }
        }

        false
    }

    pub(crate) fn add_symbol_type(&mut self, name: &String, ty: HashSet<VarType>) {
        let mut cur_opt = self.area_ptr.upgrade();

        while let Some(cur_rc) = cur_opt {
            {
                let mut binding = cur_rc.borrow_mut();
                let index = Self::find_in_vec(&binding.symbol, name);
                if index != usize::MAX {
                    binding.symbol[index].its_type =
                        binding.symbol[index].its_type.union(&ty).cloned().collect();
                    return;
                }
                cur_opt = binding.parent.as_ref().and_then(|w| w.upgrade());
            }
        }
    }

    pub(crate) fn update_array_elem_type(&mut self, name: &String, elem_ty: HashSet<VarType>) {
        let mut cur_opt = self.area_ptr.upgrade();

        while let Some(cur_rc) = cur_opt {
            {
                let mut binding = cur_rc.borrow_mut();
                let index = Self::find_in_vec(&binding.symbol, name);
                if index != usize::MAX {
                    // ensure the symbol is marked as array and carries Array in its type set
                    binding.symbol[index].is_arr = true;
                    binding.symbol[index].its_type.insert(VarType::Array);

                    if binding.symbol[index].elem_type.is_empty() {
                        binding.symbol[index].elem_type.push(elem_ty.clone());
                    } else {
                        for set in binding.symbol[index].elem_type.iter_mut() {
                            *set = set.union(&elem_ty).cloned().collect();
                        }
                    }
                    return;
                }
                cur_opt = binding.parent.as_ref().and_then(|w| w.upgrade());
            }
        }
    }

    pub(crate) fn get_level(&mut self) -> i32 {
        self.now_level
    }
}
