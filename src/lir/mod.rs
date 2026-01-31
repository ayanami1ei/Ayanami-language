use std::collections::HashMap;

use crate::{
    hir::{BlockId, FuncId, HIR, HirFuncSymbol, SlotId, VarId},
    types::VarType,
};
use inkwell::{
    basic_block::BasicBlock,
    builder::Builder,
    context::Context,
    module::Module,
    types::StructType,
    values::{FunctionValue, PointerValue},
};

pub(crate) mod implement;
pub(super) mod types;

pub(crate) struct LirGenerator<'ctx> {
    context: &'ctx Context,
    module: Module<'ctx>,
    builder: Builder<'ctx>,

    func_registry: HashMap<FuncId, HirFuncSymbol>,
    llvm_func_registry: HashMap<FuncId, FunctionValue<'ctx>>,
    block_registry: HashMap<BlockId, BasicBlock<'ctx>>,
    slot_registry: HashMap<SlotId, PointerValue<'ctx>>,
    var_registry: HashMap<VarId, PointerValue<'ctx>>,

    runtime_fn: HashMap<&'ctx str, FunctionValue<'ctx>>,

    main_id: FuncId,

    hirs: Vec<HIR>,
    i: usize,
}

struct Object {
    ty: VarType,
    refcnt: i32,
    // fields...
}
