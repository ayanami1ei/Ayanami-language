use std::{
    cell::{RefCell},
    rc::{Rc, Weak},
};

use anyhow::{Error, anyhow};

use crate::parser::symbol::symbol::SemanticSymbol;

#[derive(Default, Clone)]
pub struct TreeNode<T: Default + Clone> {
    pub(super) node: T,
    pub(super) id: usize,
    pub(super) sons: Vec<Rc<RefCell<TreeNode<T>>>>,
    pub(super) parent: Weak<RefCell<TreeNode<T>>>,
}

pub struct Tree<T: Default + Clone> {
    head: TreeNode<T>,
    next_index: usize,
    cursor: Weak<RefCell<TreeNode<T>>>,
}

impl<T: Default + Clone> Default for Tree<T> {
    fn default() -> Self {
        Self::new()
    }
}

impl<T: Default + Clone> Tree<T> {
    pub fn new() -> Self {
        let head = TreeNode::default();
        Self {
            head: head.clone(),
            next_index: 1,
            cursor: Rc::downgrade(&Rc::new(RefCell::new(head))),
        }
    }

    pub fn with_inner(&self, func: fn(&mut T)) {
        func(&mut self.cursor.upgrade().unwrap().borrow_mut().node)
    }
}

impl Tree<Vec<SemanticSymbol>> {
    pub fn into_next(&mut self) -> usize {
        let new = Rc::new(RefCell::new(TreeNode::default()));
        new.borrow_mut().id = self.next_index;
        self.next_index += 1;

        self.cursor
            .upgrade()
            .expect("invaild tree node")
            .borrow_mut()
            .sons
            .push(new.clone());

        new.borrow_mut().parent = self.cursor.clone();

        self.cursor = Rc::downgrade(&new);

        self.next_index - 1
    }

    pub fn return_to_upper(&mut self) -> usize {
        let new = self
            .cursor
            .upgrade()
            .expect("there is no upper")
            .borrow()
            .parent
            .clone();

        self.cursor = new;
        self.cursor
            .upgrade()
            .expect("there is no upper")
            .borrow()
            .id
    }

    pub fn set_by_id(&mut self, id: usize) -> Result<(), Error> {
        if id >= self.next_index {
            return Err(anyhow!("no such id"));
        }

        let dummy = Rc::new(RefCell::new(self.head.clone()));
        if let Some(x) = Self::find_sons(dummy, id) {
            self.cursor = x;
            return Ok(());
        }

        Err(anyhow!("no such id"))
    }

    fn find_sons(
        dummy: Rc<RefCell<TreeNode<Vec<SemanticSymbol>>>>,
        id: usize,
    ) -> Option<Weak<RefCell<TreeNode<Vec<SemanticSymbol>>>>> {
        let son_len = dummy.borrow().sons.len();
        if son_len == 0 {
            return None;
        }

        if dummy.borrow().id == id {
            return Some(Rc::downgrade(&dummy));
        }

        for son in dummy.borrow().sons.clone() {
            let res = Self::find_sons(son, id);
            if res.is_none() {
                continue;
            } else {
                return res;
            }
        }

        None
    }
}
