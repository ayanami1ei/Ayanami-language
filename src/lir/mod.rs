use crate::types::VarType;
use inkwell::{builder::Builder, context::Context, module::Module};

pub(crate) mod implement;

struct LirGenerator<'a> {
    context: &'a Context,
    module: Module<'a>,
    builder: Builder<'a>,
}

struct Object {
    ty: VarType,
    refcnt: i32,
    // fields...
}
