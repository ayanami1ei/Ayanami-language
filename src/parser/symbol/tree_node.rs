use std::{
    cell::RefCell,
    rc::{Rc, Weak},
};

#[derive(Default, Clone)]
pub struct TreeNode<T: Default + Clone> {
    pub(crate) node: T,
    pub(crate) id: usize,
    pub(crate) sons: Vec<Rc<RefCell<TreeNode<T>>>>,
    pub(crate) parent: Weak<RefCell<TreeNode<T>>>,
}
