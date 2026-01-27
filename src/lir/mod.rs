use std::collections::HashMap;

use crate::{
    hir::{BlockId, FuncId, HIR, HirFuncSymbol, SlotId},
    types::VarType,
};
use inkwell::{
    basic_block::BasicBlock, builder::Builder, context::Context, module::Module,
    values::PointerValue,
};

pub(crate) mod implement;

pub(crate) struct LirGenerator<'ctx> {
    context: &'ctx Context,
    module: Module<'ctx>,
    builder: Builder<'ctx>,

    func_registry: HashMap<FuncId, HirFuncSymbol>,
    block_registry: HashMap<BlockId, BasicBlock<'ctx>>,
    slot_registry: HashMap<SlotId, PointerValue<'ctx>>,

    hirs: Vec<HIR>,
    i: usize,
}

struct Object {
    ty: VarType,
    refcnt: i32,
    // fields...
}
