use super::*;

// ═══════════════════════════════════════════════════════════════════
//  28 struct types for each LirInst variant
// ═══════════════════════════════════════════════════════════════════

s_lir!(SLirAlloca { var: VarId, ty: HirType });
s_lir!(SLirStore { dest: VarId, src: LirValue, ty: HirType });
s_lir!(SLirLoad { dest: u64, src: VarId, ty: HirType });
s_lir!(SLirLoadPtr { dest: u64, src: LirValue, ty: HirType });
s_lir!(SLirBinOp { dest: u64, op: BinaryOp, lhs: LirValue, rhs: LirValue, ty: HirType, result_ty: HirType });
s_lir!(SLirUnaryOp { dest: u64, op: UnaryOp, src: LirValue, ty: HirType });
s_lir!(SLirCall { dest: Option<u64>, fn_id: FnId, args: Vec<(LirValue, HirType)>, ret_ty: HirType });
s_lir!(SLirCallPtr { dest: u64, fn_ptr: LirValue, args: Vec<(LirValue, HirType)>, ret_ty: HirType });
s_lir!(SLirFnAddr { dest: u64, fn_id: FnId });
s_lir!(SLirStrGlobal { dest: u64, str_idx: u64 });
s_lir!(SLirConv { dest: u64, alloca_tmp: u64, malloc_tmp: u64, src: LirValue, kind: ConvKind, src_ty: HirType, ty: HirType });
s_lir!(SLirDropValue { var: VarId, ty: HirType });
s_lir!(SLirBr { label: String });
s_lir!(SLirBrCond { cond: LirValue, true_block: String, false_block: String });
s_lir!(SLirAssume { cond: LirValue });
s_lir!(SLirContractCheck { kind: crate::hir::ContractKind, cond: LirValue, line: u64, col: u64 });
s_lir!(SLirRet { val: Option<(LirValue, HirType)> });
s_lir!(SLirMakeFatPtr { dest: u64, malloc_tmp: u64, bc_tmp: u64, vtable_gep_tmp: u64, iv_tmp: u64, value_src: LirValue, value_ty: HirType, vtable_name: String, ty: HirType });
s_lir!(SLirFieldAccess { dest: u64, gep_tmp: u64, src: LirValue, field_index: usize, field_ty: HirType, struct_ty: HirType });
s_lir!(SLirAsm { dest: Option<u64>, template: String, output_constraints: Vec<String>, output_operands: Vec<(LirValue, HirType, Option<VarId>)>, input_operands: Vec<(LirValue, HirType)>, input_constraints: Vec<String>, ret_ty: HirType });
s_lir!(SLirRefInst { dest: u64, var_id: VarId, mutable: bool, ty: HirType });
s_lir!(SLirRefTmp { dest: u64, alloca_tmp: u64, src: LirValue, mutable: bool, ty: HirType });
s_lir!(SLirArraySized { dest: u64, malloc_tmp: u64, count_tmp: u64, size_tmp: u64, elem_count: LirValue, elem_size: u64, elem_ty: HirType, ty: HirType });
s_lir!(SLirArrayLit { dest: u64, malloc_tmp: u64, elem_geps: Vec<u64>, elems: Vec<(LirValue, HirType)>, elem_ty: HirType, ty: HirType });
s_lir!(SLirIndexAccess { dest: u64, gep_tmp: u64, load_tmp: u64, arr: LirValue, index: LirValue, elem_ty: HirType, ty: HirType });
s_lir!(SLirStructLit { dest: u64, alloca_tmp: u64, field_geps: Vec<u64>, fields: Vec<(LirValue, HirType)>, struct_name: Symbol, struct_ty: HirType });
s_lir!(SLirVirtualCall { fn_dest: Option<u64>, receiver_tmp: u64, data_tmp: u64, vtable_tmp: u64, gep_tmp: u64, fn_ptr_tmp: u64, method_index: usize, args: Vec<(LirValue, HirType)>, ret_ty: HirType });
s_lir!(SLirFieldAddr { dest: u64, obj: LirValue, field_index: usize, struct_ty: HirType });
// M2/FnOnce：字段移出（load + 源字段清零，避免源结构 drop 时双重释放）
s_lir!(SLirFieldTake { dest: u64, gep_tmp: u64, obj: LirValue, field_index: usize, field_ty: HirType, struct_ty: HirType });
s_lir!(SLirLocalTake { dest: u64, var: VarId, ty: HirType });
s_lir!(SLirDropPtr { ptr: LirValue, ty: HirType });
s_lir!(SLirClone { dest: u64, alloca_tmp: u64, src: LirValue, ty: HirType });
s_lir!(SLirDropArray { var: VarId, elem_ty: HirType, count_var: VarId });
// M6.2：全局变量地址（dest = getelementptr 取 @name 指针）
s_lir!(SLirGlobalAddr { dest: u64, name: Symbol });
// #130：数组元素地址（arr_tmp 为数组缓冲指针）
s_lir!(SLirIndexAddr { dest: u64, arr_tmp: u64, index: LirValue, elem_ty: HirType });
s_lir!(SLirFieldStorePtr { gep_tmp: u64, obj: LirValue, field_index: usize, field_ty: HirType, src: LirValue, struct_ty: HirType });
s_lir!(SLirFieldStore { dest: u64, var_id: Option<VarId>, gep_tmp: u64, iv_tmp: u64, src: LirValue, field_index: usize, field_ty: HirType, struct_ty: HirType });
s_lir!(SLirIndexStore { dest: u64, gep_tmp: u64, src: LirValue, index: LirValue, elem_ty: HirType, array_ty: HirType });
s_lir!(SLirCustom { node: IrNode });

// ═══════════════════════════════════════════════════════════════════
//  impl_into_lir_node_box! macro
// ═══════════════════════════════════════════════════════════════════

impl_into_lir_node_box!(
    SLirAlloca, SLirStore, SLirLoad, SLirLoadPtr, SLirBinOp, SLirUnaryOp,
    SLirCall, SLirCallPtr, SLirFnAddr, SLirStrGlobal, SLirConv,
    SLirDropValue,
    SLirBr, SLirBrCond, SLirAssume, SLirContractCheck, SLirRet,
    SLirMakeFatPtr, SLirFieldAccess, SLirAsm, SLirRefInst, SLirRefTmp,
    SLirArraySized, SLirArrayLit, SLirIndexAccess, SLirStructLit,
    SLirVirtualCall, SLirFieldStore, SLirIndexStore, SLirFieldAddr, SLirFieldStorePtr,
    SLirGlobalAddr, SLirIndexAddr, SLirFieldTake, SLirLocalTake, SLirDropPtr,
    SLirClone, SLirDropArray,
);

// ═══════════════════════════════════════════════════════════════════
//  Helper functions (moved from emit.rs)
// ═══════════════════════════════════════════════════════════════════
