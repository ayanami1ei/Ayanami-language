use std::{
    clone,
    collections::HashMap,
    fs::{self, OpenOptions},
    io::Write,
    mem::transmute,
    path::Path,
    process::Command,
};

use inkwell::{
    AddressSpace, OptimizationLevel,
    basic_block::BasicBlock,
    context::Context,
    module::Module,
    targets::{
        CodeModel, FileType, InitializationConfig, RelocMode, Target, TargetMachine, TargetTriple,
    },
    types::{BasicMetadataTypeEnum, BasicTypeEnum},
    values::{
        BasicMetadataValueEnum, BasicValue, BasicValueEnum, FunctionValue, IntValue, PointerValue,
    },
};
use llvm_sys::target_machine;
use macro_lib::{bin_operator_fn, call_bin_operator_fn, make_fn};

use crate::{
    hir::{BlockId, FloatKey, FuncId, HIR, HIRInst, HirFuncSymbol, SlotId, Value, VarId},
    lir::LirGenerator,
    types::VarType,
};

impl<'ctx> LirGenerator<'ctx> {
    fn init_env(&mut self) {
        let obj_ptr = self.context.i8_type().ptr_type(AddressSpace::default());

        let err_fn = self.module.add_function(
            "err",
            self.context.void_type().fn_type(&[obj_ptr.into()], false),
            None,
        );
        self.runtime_fn.insert("err", err_fn);

        let get_int_value_fn = self.module.add_function(
            "get_int_value",
            self.context.i32_type().fn_type(&[obj_ptr.into()], false),
            None,
        );
        self.runtime_fn.insert("get_int_value", get_int_value_fn);

        let is_truth_fn = self.module.add_function(
            "is_truth",
            self.context.bool_type().fn_type(&[obj_ptr.into()], false),
            None,
        );
        self.runtime_fn.insert("is_truth", is_truth_fn);

        let write_fn = self.module.add_function(
            "write",
            self.context.void_type().fn_type(&[obj_ptr.into()], false),
            None,
        );
        self.llvm_func_registry.insert(FuncId(1), write_fn);

        let is_type_fn = self.module.add_function(
            "is_type",
            self.context
                .bool_type()
                .fn_type(&[self.context.i64_type().into(), obj_ptr.into()], false),
            None,
        );
        self.runtime_fn.insert("is_type", is_type_fn);

        let del_obj_fn = self.module.add_function(
            "del_obj",
            self.context.void_type().fn_type(&[obj_ptr.into()], false),
            None,
        );
        self.runtime_fn.insert("del_obj", del_obj_fn);

        let dec_ref_fn = self.module.add_function(
            "dec_ref",
            self.context.void_type().fn_type(&[obj_ptr.into()], false),
            None,
        );
        self.runtime_fn.insert("dec_ref", dec_ref_fn);

        let inc_ref_fn = self.module.add_function(
            "inc_ref",
            self.context.void_type().fn_type(&[obj_ptr.into()], false),
            None,
        );
        self.runtime_fn.insert("inc_ref", inc_ref_fn);

        let alloc_int_fn = self.module.add_function(
            "alloc_int",
            obj_ptr.fn_type(&[self.context.i64_type().into()], false),
            None,
        );
        self.runtime_fn.insert("alloc_int", alloc_int_fn);

        let alloc_float_fn = self.module.add_function(
            "alloc_float",
            obj_ptr.fn_type(&[self.context.f64_type().into()], false),
            None,
        );
        self.runtime_fn.insert("alloc_float", alloc_float_fn);

        let alloc_bool_fn = self.module.add_function(
            "alloc_bool",
            obj_ptr.fn_type(&[self.context.bool_type().into()], false),
            None,
        );
        self.runtime_fn.insert("alloc_bool", alloc_bool_fn);

        let alloc_char_fn = self.module.add_function(
            "alloc_char",
            obj_ptr.fn_type(&[self.context.i32_type().into()], false),
            None,
        );
        self.runtime_fn.insert("alloc_char", alloc_char_fn);

        let alloc_string_fn = self.module.add_function(
            "alloc_string",
            obj_ptr.fn_type(
                &[
                    self.context
                        .i8_type()
                        .ptr_type(AddressSpace::default())
                        .into(),
                    self.context.i32_type().into(),
                ],
                false,
            ),
            None,
        );
        self.runtime_fn.insert("alloc_string", alloc_string_fn);
    }

    fn init_operation(&mut self) {
        bin_operator_fn!(
            self,
            add,
            sub,
            mul,
            div_op,
            equal,
            greater,
            less,
            greater_equal,
            less_equal,
            and_op,
            or_op
        );

        let obj_ptr = self.context.i8_type().ptr_type(AddressSpace::default());
        let not_fn =
            self.module
                .add_function("not", obj_ptr.fn_type(&[obj_ptr.into()], false), None);
        self.runtime_fn.insert("not", not_fn);
    }

    fn init_runtime(&mut self) {
        self.init_env();
        self.init_operation();
    }

    pub(crate) fn new(
        context: &'ctx Context,
        hirs: &Vec<HIR>,
        func_registry: HashMap<FuncId, HirFuncSymbol>,
        var_to_slot: HashMap<VarId, SlotId>,
    ) -> LirGenerator<'ctx> {
        let module = context.create_module("ayanami_modlue");
        let builder = context.create_builder();
        let object_type = context.i8_type().ptr_type(AddressSpace::default());

        let mut res = LirGenerator {
            context,
            module,
            builder,
            hirs: hirs.to_vec(),
            func_registry,
            llvm_func_registry: HashMap::new(),
            block_registry: HashMap::new(),
            slot_registry: HashMap::new(),
            var_registry: HashMap::new(),
            var_to_slot,
            runtime_fn: HashMap::<&str, FunctionValue>::new(),
            main_id: FuncId(0),
            i: 0,
        };

        res.init_runtime();

        res
    }

    fn get_or_create_slot(
        &mut self,
        slot: SlotId,
        entry_bb: BasicBlock<'ctx>,
    ) -> PointerValue<'ctx> {
        if let Some(ptr) = self.slot_registry.get(&slot) {
            return *ptr;
        }

        //  alloca 必须在函数 entry block
        let current_bb = self.builder.get_insert_block().unwrap();
        self.builder.position_at_end(entry_bb);

        let obj_ptr_ty = self.context.i8_type().ptr_type(AddressSpace::default());
        let slot_ptr = self
            .builder
            .build_alloca(obj_ptr_ty, &format!("slot{}", slot))
            .unwrap();

        // 可以初始化为 null
        /*self.builder
        .build_store(slot_ptr, obj_ptr_ty.const_null())
        .unwrap();*/

        self.builder.position_at_end(current_bb);

        self.slot_registry.insert(slot, slot_ptr);
        slot_ptr
    }

    fn gen_func_ir(&mut self, func_def: &FuncId) -> FunctionValue<'ctx> {
        let id = func_def.clone();

        let sym = match self.func_registry.get(&id) {
            None => panic!(""),
            Some(x) => x,
        };

        let mut name = sym.name.clone();
        let function;

        if sym.is_main {
            let ret_type = self.context.i32_type();
            name = "main".to_string();
            self.main_id = sym.id;

            let fn_type = ret_type.fn_type(&[], false);
            function = self.module.add_function(&name, fn_type, None);
            self.llvm_func_registry.insert(id, function);
        } else {
            let ret_type = self.context.i8_type().ptr_type(AddressSpace::default());

            let mut args = Vec::<BasicMetadataTypeEnum<'ctx>>::new();
            for _ in 0..sym.param_id.len() {
                args.push(ret_type.into());
            }
            let fn_type = ret_type.fn_type(&args, false);
            function = self.module.add_function(&name, fn_type, None);
            self.llvm_func_registry.insert(id, function);
        }

        // 为参数在 entry block 中写入传入的参数值，并建立 VarId -> LLVM 指针 的映射。
        // 入口块用于放置 slot 的 alloca（如果需要通过 slot 管理）。
        let entry_bb = self.context.append_basic_block(function, "entry");
        let cur_bb = self.builder.get_insert_block();
        self.builder.position_at_end(entry_bb);

        let object_ptr_ty = self.context.i8_type().ptr_type(AddressSpace::default());
        let param_ids = sym.param_id.clone();
        for (idx, var_id) in param_ids.iter().enumerate() {
            // 取函数第 idx 个参数
            let param_val = function
                .get_nth_param(idx as u32)
                .expect("missing function param");

            // 如果存在 var->slot 的静态映射，则在 slot 上创建/获取 alloca，并把参数写入该 slot，
            // 同时让 var_registry 指向该 slot。否则，为 var 分配一个临时 alloca 并写入参数。
            if let Some(slot_id) = self.var_to_slot.get(var_id) {
                let slot_ptr = self.get_or_create_slot(*slot_id, entry_bb);
                self.builder.build_store(slot_ptr, param_val).unwrap();
                self.var_registry.insert(*var_id, slot_ptr);
            } else {
                let var_ptr = self
                    .builder
                    .build_alloca(object_ptr_ty, &format!("var{}", var_id))
                    .unwrap();
                self.builder.build_store(var_ptr, param_val).unwrap();
                self.var_registry.insert(*var_id, var_ptr);
            }
        }

        // 恢复插入点
        if let Some(bb) = cur_bb {
            self.builder.position_at_end(bb);
        }

        function
    }
    fn new_llvm_block(&mut self, function: FunctionValue<'ctx>, id: &BlockId) {
        if let Some(_) = self.block_registry.get(id) {
            return;
        }
        let mut name = "block_".to_string();
        name.push_str(&id.to_string());
        let block = self.context.append_basic_block(function, &name);

        // 如果函数已有首个 basic block，且它没有 terminator，则在首块插入到新块的无条件分支。
        if let Some(first_bb) = function.get_first_basic_block() {
            if first_bb != block {
                if first_bb.get_terminator().is_none() {
                    // 保存当前插入点，切换到首块插入分支，随后恢复
                    let cur = self.builder.get_insert_block();
                    self.builder.position_at_end(first_bb);
                    self.builder.build_unconditional_branch(block).unwrap();
                    if let Some(bb) = cur {
                        self.builder.position_at_end(bb);
                    }
                }
            }
        }

        self.block_registry.insert(*id, block);
    }
    fn emit_llvm_block(&mut self, id: &BlockId) {
        let block = match self.block_registry.get(id) {
            None => panic!("block no found"),
            Some(x) => *x,
        };
        self.builder.position_at_end(block);
    }

    fn gen_inst_ir(&mut self, inst: &HIRInst, cur_func: Option<String>) {
        let function;
        if let Some(ref name) = cur_func
            && name == "main"
        {
            let id = self.main_id;
            function = *self.llvm_func_registry.get(&id).expect("");
        } else {
            function = *self
                .llvm_func_registry
                .get(&FuncId(cur_func.clone().expect("").parse::<i32>().unwrap()))
                .expect("");
        }

        let object_ptr_type = self.context.i8_type().ptr_type(AddressSpace::default());
        match inst {
            HIRInst::New { obj_type, val, dst } => {
                let new_fn = *match obj_type {
                    VarType::Int => self.runtime_fn.get("alloc_int").expect("runtime not found"),
                    VarType::Float => self
                        .runtime_fn
                        .get("alloc_float")
                        .expect("runtime not found"),
                    VarType::Bool => self
                        .runtime_fn
                        .get("alloc_bool")
                        .expect("runtime not found"),
                    VarType::Char => self
                        .runtime_fn
                        .get("alloc_char")
                        .expect("runtime not found"),
                    _ => panic!("unknown type"),
                };

                let call_res;
                match obj_type {
                    VarType::Int => {
                        let args = self.context.i64_type().const_int(*val as u64, false);
                        call_res = self
                            .builder
                            .build_call(new_fn, &[args.into()], "new int")
                            .unwrap();
                    }
                    VarType::Float => {
                        let args = self.context.f64_type().const_float(*val);
                        call_res = self
                            .builder
                            .build_call(new_fn, &[args.into()], "new int")
                            .unwrap();
                    }
                    VarType::Bool => {
                        let args = self.context.bool_type().const_int(*val as u64, false);
                        call_res = self
                            .builder
                            .build_call(new_fn, &[args.into()], "new int")
                            .unwrap();
                    }
                    VarType::Char => {
                        let args = self.context.i32_type().const_int(*val as u64, false);
                        call_res = self
                            .builder
                            .build_call(new_fn, &[args.into()], "new int")
                            .unwrap();
                    }
                    _ => panic!("unknown type"),
                }

                let result_obj_ptr = call_res
                    .try_as_basic_value()
                    .left() // 有返回值才会是 Some
                    .expect("binop must return")
                    .into_pointer_value(); // Object*

                let entry_bb = self
                    .builder
                    .get_insert_block()
                    .expect("builder has no insertion block");
                self.get_or_create_slot(*dst, entry_bb);
                let dst_slot_ptr = *self.slot_registry.get(&dst).unwrap();
                self.builder
                    .build_store(dst_slot_ptr, result_obj_ptr)
                    .unwrap();
            }
            HIRInst::Load { var, obj: slot } => {
                // 将 slot 别名到 var：slot_registry[slot] = var_registry[var]
                // var 的参数值在 gen_func_ir 已经写入到 var 对应的存储，因此这里只建立别名关系。
                let var_ptr = match self.var_registry.get(var) {
                    Some(p) => *p,
                    None => panic!(
                        "var {} not found: parameter should be initialized in gen_func_ir",
                        var
                    ),
                };

                self.slot_registry.insert(*slot, var_ptr);
            }
            HIRInst::Delete { dst } => {
                let del_ref_fn = match self.module.get_function("del_obj") {
                    None => panic!("ayanami_runtime not found"),
                    Some(x) => x,
                };
                let ptr = match self.slot_registry.get(dst) {
                    None => return,
                    Some(x) => x,
                };

                let load_res = self
                    .builder
                    .build_load(object_ptr_type, *ptr, "load")
                    .unwrap();

                match self
                    .builder
                    .build_call(del_ref_fn, &[load_res.into()], "del_obj")
                {
                    Err(e) => panic!("{}", e),
                    Ok(_) => {}
                }

                self.builder
                    .build_store(*ptr, object_ptr_type.const_null())
                    .unwrap();
            }
            HIRInst::Store { from, to } => {
                let entry_bb = self
                    .builder
                    .get_insert_block()
                    .expect("builder has no insertion block");
                self.get_or_create_slot(*to, entry_bb);
                let to_ptr = match self.slot_registry.get(to) {
                    None => match self.builder.build_alloca(self.context.i32_type(), "tmp") {
                        Err(e) => panic!("{}", e),
                        Ok(x) => x,
                    },
                    Some(x) => *x,
                };
                if let Value::Const(c) = from {
                    let val = match c {
                        crate::hir::Const::Int(x) => self
                            .context
                            .i64_type()
                            .const_int(*x as u64, false)
                            .as_basic_value_enum(),
                        crate::hir::Const::Float(float_key) => {
                            let FloatKey(x) = *float_key;
                            self.context.f64_type().const_float(x).as_basic_value_enum()
                        }
                        crate::hir::Const::Char(c) => self
                            .context
                            .i8_type()
                            .const_int(*c as u64, false)
                            .as_basic_value_enum(),
                        crate::hir::Const::Bool(b) => self
                            .context
                            .bool_type()
                            .const_int(*b as u64, false)
                            .as_basic_value_enum(),
                        crate::hir::Const::Null => todo!(),
                    };

                    match self.builder.build_store(to_ptr, val) {
                        Err(e) => panic!("e"),
                        Ok(_) => {}
                    };
                }
            }
            HIRInst::Br {
                cond,
                then_block,
                else_block,
            } => {
                // 1. cond_ir 返回 i1
                let is_truth_fn = *self.runtime_fn.get("is_truth").expect("runtime not found");
                let cond_slot = self.slot_registry.get(cond).expect("");
                let args = self
                    .builder
                    .build_load(object_ptr_type, *cond_slot, "cond_load")
                    .unwrap();
                let cond_i1 = self
                    .builder
                    .build_call(is_truth_fn, &[args.into()], "call is_truth")
                    .unwrap()
                    .try_as_basic_value()
                    .left()
                    .expect("is_truth has no return")
                    .into_int_value();

                // 2. 获取 LLVM 基本块
                self.new_llvm_block(function, then_block);
                let llvm_then = *self
                    .block_registry
                    .get(&then_block)
                    .expect("then block missing");
                self.new_llvm_block(function, else_block);
                let llvm_else = *self
                    .block_registry
                    .get(&else_block)
                    .expect("else block missing");

                // 3. 生成条件分支
                self.builder
                    .build_conditional_branch(cond_i1, llvm_then, llvm_else)
                    .unwrap();
            }
            HIRInst::Jmp { target } => {
                let target_bb = match self.block_registry.get(target) {
                    None => {
                        self.new_llvm_block(function, target);
                        self.block_registry.get(target).unwrap()
                    }
                    Some(x) => x,
                };
                self.builder.build_unconditional_branch(*target_bb).unwrap();
            }
            HIRInst::Call { id, ret } => {
                let func = *self
                    .llvm_func_registry
                    .get(id)
                    .expect(&format!("no such func, id:{}", id));
                let sym = self.func_registry.get(id).expect("");

                let mut args = Vec::<BasicMetadataValueEnum>::new();

                for i in sym.param_id.clone() {
                    let ptr = *self.var_registry.get(&i).expect("");
                    let obj = self
                        .builder
                        .build_load(object_ptr_type, ptr, "param")
                        .unwrap()
                        .into();

                    args.push(obj);
                }

                let call_res = self
                    .builder
                    .build_call(func, args.as_slice(), "call")
                    .unwrap();

                let result_obj_ptr = match call_res
                    .try_as_basic_value()
                    .left() // 有返回值才会是 Some
                    {
                        Some(x)=>x.into_pointer_value(),
                        None=>return
                    }; // Object*

                let entry_bb = self
                    .builder
                    .get_insert_block()
                    .expect("builder has no insertion block");
                self.get_or_create_slot(*ret, entry_bb);
                let dst_slot_ptr = *self.slot_registry.get(ret).unwrap();
                self.builder
                    .build_store(dst_slot_ptr, result_obj_ptr)
                    .unwrap();
            }
            HIRInst::BinOp {
                left,
                op,
                right,
                dst,
            } => {
                let entry_bb = self
                    .builder
                    .get_insert_block()
                    .expect("builder has no insertion block");
                self.get_or_create_slot(*dst, entry_bb);

                let left_ptr = match self.slot_registry.get(left) {
                    None => panic!(""),
                    Some(x) => x,
                };
                let right_ptr = match self.slot_registry.get(right) {
                    None => panic!(""),
                    Some(x) => x,
                };

                let left_obj = self
                    .builder
                    .build_load(object_ptr_type, *left_ptr, "left_obj")
                    .unwrap();
                let right_obj = self
                    .builder
                    .build_load(object_ptr_type, *right_ptr, "right_obj")
                    .unwrap();

                let call_res;

                match op {
                    crate::hir::BinOperator::Add => {
                        call_bin_operator_fn!(self, add, left_obj, right_obj, call_res)
                    }
                    crate::hir::BinOperator::Sub => {
                        call_bin_operator_fn!(self, sub, left_obj, right_obj, call_res)
                    }
                    crate::hir::BinOperator::Mul => {
                        call_bin_operator_fn!(self, mul, left_obj, right_obj, call_res)
                    }
                    crate::hir::BinOperator::Div => {
                        call_bin_operator_fn!(self, div, left_obj, right_obj, call_res)
                    }
                    crate::hir::BinOperator::Equal => {
                        call_bin_operator_fn!(self, equal, left_obj, right_obj, call_res)
                    }
                    crate::hir::BinOperator::Greater => {
                        call_bin_operator_fn!(self, greater, left_obj, right_obj, call_res)
                    }
                    crate::hir::BinOperator::Less => {
                        call_bin_operator_fn!(self, less, left_obj, right_obj, call_res)
                    }
                    crate::hir::BinOperator::GreaterEqual => {
                        call_bin_operator_fn!(self, greater_equal, left_obj, right_obj, call_res)
                    }
                    crate::hir::BinOperator::LessEqual => {
                        call_bin_operator_fn!(self, less_equal, left_obj, right_obj, call_res)
                    }
                    crate::hir::BinOperator::And => {
                        call_bin_operator_fn!(self, and, left_obj, right_obj, call_res)
                    }
                    crate::hir::BinOperator::Or => {
                        call_bin_operator_fn!(self, or, left_obj, right_obj, call_res)
                    }
                }

                let result_obj_ptr = call_res
                    .try_as_basic_value()
                    .left() // 有返回值才会是 Some
                    .expect("binop must return")
                    .into_pointer_value(); // Object*
                let dst_slot_ptr = *self.slot_registry.get(&dst).unwrap();
                self.builder
                    .build_store(dst_slot_ptr, result_obj_ptr)
                    .unwrap();
            }
            HIRInst::UnaryOp { op, expr, dst } => {
                let entry_bb = self
                    .builder
                    .get_insert_block()
                    .expect("builder has no insertion block");
                self.get_or_create_slot(*dst, entry_bb);

                let expr_ptr = match self.slot_registry.get(expr) {
                    None => panic!(""),
                    Some(x) => x,
                };

                let expr_obj = self
                    .builder
                    .build_load(object_ptr_type, *expr_ptr, "left_obj")
                    .unwrap();

                let not_fn = *self.runtime_fn.get("not").expect("runtime not init");

                let call_res = self
                    .builder
                    .build_call(not_fn, &[expr_obj.into()], "not")
                    .unwrap();

                let result_obj_ptr = call_res
                    .try_as_basic_value()
                    .left() // 有返回值才会是 Some
                    .expect("binop must return")
                    .into_pointer_value(); // Object*
                let dst_slot_ptr = *self.slot_registry.get(&dst).unwrap();
                self.builder
                    .build_store(dst_slot_ptr, result_obj_ptr)
                    .unwrap();
            }
            HIRInst::IncRef { obj } => {
                let object_ptr_type = self.context.i8_type().ptr_type(AddressSpace::default());

                let slot_ptr: &PointerValue<'_> =
                    self.slot_registry.get(&obj).expect("sloy not found");
                let obj = self
                    .builder
                    .build_load(object_ptr_type, *slot_ptr, "left_obj")
                    .unwrap();

                let inc_ref_fn = *self.runtime_fn.get("inc_ref").expect("runtime not init");
                self.builder
                    .build_call(inc_ref_fn, &[obj.into()], "inc_ref")
                    .unwrap();
            }
            HIRInst::DecRef { obj } => {
                let object_ptr_type = self.context.i8_type().ptr_type(AddressSpace::default());

                let slot_ptr: &PointerValue<'_> =
                    self.slot_registry.get(&obj).expect("sloy not found");
                let obj = self
                    .builder
                    .build_load(object_ptr_type, *slot_ptr, "left_obj")
                    .unwrap();

                let dec_ref_fn = *self.runtime_fn.get("dec_ref").expect("runtime not init");
                self.builder
                    .build_call(dec_ref_fn, &[obj.into()], "dec_ref")
                    .unwrap();
            }
            HIRInst::Bind { var, obj } => {
                let slot_ptr = self.slot_registry[&obj]; // Object**
                let var_ptr = self
                    .builder
                    .build_alloca(object_ptr_type, &format!("slot{}", obj))
                    .unwrap();

                // 1. 从 slot 中读 Object*
                let obj_val = self
                    .builder
                    .build_load(object_ptr_type, slot_ptr, "bind.load")
                    .unwrap();

                // 2. 存到变量对应的内存
                self.builder.build_store(var_ptr, obj_val).unwrap();

                self.var_registry.insert(*var, var_ptr);
            }
            HIRInst::Ret { ret_obj } => {
                let slot_ptr = *self.slot_registry.get(ret_obj).expect("ret slot not found");

                // 1. 从 slot 中 load 出 Object*
                let obj = self
                    .builder
                    .build_load(object_ptr_type, slot_ptr, "ret.load")
                    .unwrap();

                // 2. inc_ref —— 返回的是“共享引用”
                let inc_fn = *self.runtime_fn.get("inc_ref").expect("inc_ref not found");
                self.builder
                    .build_call(inc_fn, &[obj.into()], "ret.inc_ref")
                    .unwrap();

                // 3. main 特判：拆值 + 释放对象
                if let Some(func) = cur_func
                    && func == "main"
                {
                    let get_fn = *self.runtime_fn.get("get_int_value").expect("");
                    let ret_val = self
                        .builder
                        .build_call(get_fn, &[obj.into()], "get main ret")
                        .unwrap()
                        .try_as_basic_value()
                        .left()
                        .expect("get_int_value must return")
                        .into_int_value();

                    // main 是程序终点，可以直接释放
                    let del_fn = *self.runtime_fn.get("del_obj").expect("");
                    self.builder
                        .build_call(del_fn, &[obj.into()], "main.del_obj")
                        .unwrap();

                    self.builder.build_return(Some(&ret_val)).unwrap();
                    return;
                }

                // 4. 普通函数：ret Object*
                self.builder.build_return(Some(&obj)).unwrap();
            }
            HIRInst::Unreachable => {
                self.builder.build_unreachable().unwrap();
            }
        }
    }

    pub(crate) fn gen_lir(&mut self) {
        let mut cur_func: Option<String> = None;
        let mut cur_block_id = BlockId {
            id: 0,
            is_merge: false,
        };

        let hirs = self.hirs.clone();
        let mut i = 0;
        while i < hirs.len() {
            match hirs[i].clone() {
                HIR::Inst(hirinst) => {
                    self.gen_inst_ir(&hirinst, cur_func.clone());
                }
                HIR::Block(block_id) => {
                    // 如果前一个 block 存在且未加入到 LIR（block_registry），先在当前函数中声明它
                    if cur_block_id.id != 0 {
                        if !self.block_registry.contains_key(&cur_block_id) {
                            let function = *self
                                .llvm_func_registry
                                .get(&FuncId(cur_func.clone().expect("").parse::<i32>().unwrap()))
                                .expect("function not found when declaring previous block");
                            self.new_llvm_block(function, &cur_block_id);
                        }
                    }

                    // 如果当前 block 未被声明，则在当前函数中声明它
                    if !self.block_registry.contains_key(&block_id) {
                        if let Some(ref name) = cur_func
                            && name == "main"
                        {
                            let FuncId(id) = self.main_id;
                            let function = *self
                                .llvm_func_registry
                                .get(&FuncId(id))
                                .expect("function not found when declaring current block");
                            self.new_llvm_block(function, &block_id);
                        } else {
                            let function = *self
                                .llvm_func_registry
                                .get(&FuncId(cur_func.clone().expect("").parse::<i32>().unwrap()))
                                .expect("function not found when declaring current block");
                            self.new_llvm_block(function, &block_id);
                        }
                    }

                    // 将当前 block 切换为插入点
                    self.emit_llvm_block(&block_id);

                    // 更新 cur_block_id 为当前 block
                    cur_block_id = block_id;
                }
                HIR::FuncLabel(ref func_def) => {
                    let _t = self.gen_func_ir(func_def);
                    // 进入新函数，重置参数索引（HIR 中 load 的顺序对应参数顺序）
                    self.i = 0;
                    if *func_def == self.main_id {
                        cur_func = Some("main".to_string());
                    } else {
                        cur_func = Some(func_def.get_id().to_string());
                    }
                }
            }

            i += 1;
        }

        let lir = self.module.print_to_string().to_string();
        fs::write("./build/lir.txt", "").unwrap();
        let mut lir_file = OpenOptions::new()
            .append(true)
            .open("./build/lir.txt")
            .unwrap();
        lir_file.write(&lir.to_string().as_bytes()).unwrap();
        lir_file.write("\n".as_bytes()).unwrap();
    }

    pub(crate) fn to_asm(&mut self) {
        let init_config = InitializationConfig {
            asm_printer: true,
            asm_parser: true,
            base: true,
            disassembler: true,
            info: true,
            machine_code: true,
        };
        Target::initialize_all(&init_config);

        let triple = &TargetMachine::get_default_triple();
        let target = Target::from_triple(triple).unwrap();
        let target_machine = target
            .create_target_machine(
                triple,
                "generic",
                "",
                OptimizationLevel::Aggressive,
                RelocMode::PIC,
                CodeModel::Default,
            )
            .unwrap();

        let output_obj = Path::new("./build/test.o");
        target_machine
            .write_to_file(&self.module, FileType::Object, output_obj)
            .unwrap();

        let link = Command::new("g++")
            .arg("-g")
            .arg("-O0")
            .arg("./build/test.o")
            .arg("-L./runtime")
            .arg("-layanami_runtime")
            .arg("-o")
            .arg("./build/ayanami_test")
            .status()
            .unwrap();

        if !link.success() {
            println!("g++ not success, {:?}", link.code());
        }

        self.module.print_to_file("./build/output.ll").unwrap();
        Command::new("llc -O0 -print-after-all ./build/output.ll > ./build/debug.txt");
    }
}
