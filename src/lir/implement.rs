use std::{
    collections::HashMap,
    fs::{self, OpenOptions},
    io::Write,
    path::{self, Path},
    process::Command,
};

use inkwell::{
    AddressSpace, OptimizationLevel,
    basic_block::BasicBlock,
    context::Context,
    targets::{CodeModel, FileType, InitializationConfig, RelocMode, Target, TargetMachine},
    types::BasicMetadataTypeEnum,
    values::{BasicMetadataValueEnum, BasicValue, FunctionValue, PointerValue},
};
use macro_lib::{bin_operator_fn, call_bin_operator_fn, make_fn};

use crate::{
    hir::{BlockId, FloatKey, FuncId, HIR, HIRInst, HirFuncSymbol, SlotId, UnaryOperation, Value},
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

        #[cfg(debug_assertions)]
        {
            let debug_ref_fn = self.module.add_function(
                "runtime_debug_ref",
                self.context.void_type().fn_type(
                    &[
                        self.context.i32_type().into(),
                        self.context.i32_type().into(),
                        obj_ptr.into(),
                    ],
                    false,
                ),
                None,
            );
            self.runtime_fn.insert("runtime_debug_ref", debug_ref_fn);
        }

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

        let alloc_array_fn = self.module.add_function(
            "alloc_array",
            obj_ptr.fn_type(&[self.context.i64_type().into()], false),
            None,
        );
        self.runtime_fn.insert("alloc_array", alloc_array_fn);

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
    ) -> LirGenerator<'ctx> {
        let module = context.create_module("ayanami_modlue");
        let builder = context.create_builder();

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

        // 将 alloca 插入到 entry 块的第一个指令之前（若存在），
        // 保证 alloca 位于块开始且支配该函数内所有使用点。
        if let Some(first_instr) = entry_bb.get_first_instruction() {
            self.builder.position_before(&first_instr);
        } else {
            self.builder.position_at_end(entry_bb);
        }

        let obj_ptr_ty = self.context.i8_type().ptr_type(AddressSpace::default());
        let slot_ptr = self
            .builder
            .build_alloca(obj_ptr_ty, &format!("slot{}", slot))
            .unwrap();

        // 初始化为 null，避免未初始化 slot 被 dec_ref/dec_ref 使用时产生栈垃圾地址
        self.builder
            .build_store(slot_ptr, obj_ptr_ty.const_null())
            .unwrap();

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
            // Mangle user function names to avoid collisions with libc/runtime symbols
            name = format!("user_{}_{}", name, id.0);
            let ret_type = self.context.i8_type().ptr_type(AddressSpace::default());

            let mut args = Vec::<BasicMetadataTypeEnum<'ctx>>::new();
            for _ in 0..sym.param_id.len() {
                args.push(ret_type.into());
            }
            let fn_type = ret_type.fn_type(&args, false);
            function = self.module.add_function(&name, fn_type, None);
            self.llvm_func_registry.insert(id, function);
        }

        let entry_bb = self.context.append_basic_block(function, "entry");
        self.builder.position_at_end(entry_bb);

        // 初始化参数的 alloca 并写入参数值
        if !sym.is_main {
            let obj_ptr_type = self.context.i8_type().ptr_type(AddressSpace::default());
            for (i, param_id) in sym.param_id.iter().enumerate() {
                let param = function
                    .get_nth_param(i as u32)
                    .expect("param missing")
                    .into_pointer_value();

                if let Some(first_instr) = entry_bb.get_first_instruction() {
                    self.builder.position_before(&first_instr);
                } else {
                    self.builder.position_at_end(entry_bb);
                }

                let var_alloc = self
                    .builder
                    .build_alloca(obj_ptr_type, &format!("var{}", param_id))
                    .unwrap();
                self.builder.build_store(var_alloc, param).unwrap();

                self.var_registry.insert(*param_id, var_alloc);
            }

            self.builder.position_at_end(entry_bb);
        }

        function
    }

    fn new_llvm_block(&mut self, function: FunctionValue<'ctx>, id: &BlockId) {
        let mut name = "block".to_string();
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
                    VarType::String => self
                        .runtime_fn
                        .get("alloc_string")
                        .expect("runtime not found"),
                    _ => panic!("unknown type"),
                };

                let call_res;
                match obj_type {
                    VarType::Int => {
                        let args = self
                            .context
                            .i64_type()
                            .const_int(val.parse().unwrap(), false);
                        call_res = self
                            .builder
                            .build_call(new_fn, &[args.into()], "new int")
                            .unwrap();
                    }
                    VarType::Float => {
                        let args = self.context.f64_type().const_float(val.parse().unwrap());
                        call_res = self
                            .builder
                            .build_call(new_fn, &[args.into()], "new int")
                            .unwrap();
                    }
                    VarType::Bool => {
                        let args = self
                            .context
                            .bool_type()
                            .const_int(val.parse().unwrap(), false);
                        call_res = self
                            .builder
                            .build_call(new_fn, &[args.into()], "new int")
                            .unwrap();
                    }
                    VarType::String => {
                        // val is the string content
                        let s = val;
                        let bytes = s.as_bytes();
                        let array_ty = self.context.i8_type().array_type(bytes.len() as u32);
                        let gname = format!(".str.{}", self.i);
                        let global = self.module.add_global(array_ty, None, &gname);
                        let mut elems: Vec<inkwell::values::IntValue> = Vec::new();
                        for b in bytes.iter() {
                            elems.push(self.context.i8_type().const_int(*b as u64, false));
                        }
                        let const_array = self.context.i8_type().const_array(&elems);
                        global.set_initializer(&const_array);
                        global.set_constant(true);

                        let gv_ptr = global.as_pointer_value();
                        let gep = self
                            .builder
                            .build_pointer_cast(gv_ptr, object_ptr_type, "str.ptr")
                            .unwrap();

                        let len_val = self.context.i32_type().const_int(bytes.len() as u64, false);

                        call_res = self
                            .builder
                            .build_call(new_fn, &[gep.into(), len_val.into()], "new string")
                            .unwrap();
                        self.i += 1;
                    }
                    VarType::Char => {
                        let args = self
                            .context
                            .i32_type()
                            .const_int(val.parse::<char>().unwrap() as u64, false);
                        call_res = self
                            .builder
                            .build_call(new_fn, &[args.into()], "new int")
                            .unwrap();
                    }
                    _ => panic!("unknown type"),
                }

                let ret_val = call_res
                    .try_as_basic_value()
                    .left() // 有返回值才会是 Some
                    .expect("call must return");

                let entry_bb = function
                    .get_first_basic_block()
                    .expect("function has no entry block");
                self.get_or_create_slot(*dst, entry_bb);
                let dst_slot_ptr = *self.slot_registry.get(&dst).unwrap();
                self.builder.build_store(dst_slot_ptr, ret_val).unwrap();
            }
            HIRInst::Load { var, obj: slot } => {
                // 从 var_registry 中取出 var 的指针（指向 Object*），加载其指向的值，
                // 然后把该值存入目标 slot（slot_registry 中的 alloca）。
                let var_ptr = match self.var_registry.get(var) {
                    Some(p) => *p,
                    None => panic!(
                        "var {} not found: parameter should be initialized in gen_func_ir",
                        var
                    ),
                };

                let loaded = self
                    .builder
                    .build_load(object_ptr_type, var_ptr, "param.load")
                    .unwrap();

                let entry_bb = function
                    .get_first_basic_block()
                    .expect("function has no entry block");
                self.get_or_create_slot(*slot, entry_bb);
                let dst_ptr = *self.slot_registry.get(slot).expect("slot not created");
                self.builder.build_store(dst_ptr, loaded).unwrap();
            }
            HIRInst::ArrayNew { elems, dst } => {
                let len = elems.len();
                let alloc_array_fn = *self
                    .runtime_fn
                    .get("alloc_array")
                    .expect("runtime not found");

                // call alloc_arr(len: i64) -> Object*
                let len_val = self.context.i64_type().const_int(len as u64, false);
                let call_res = self
                    .builder
                    .build_call(alloc_array_fn, &[len_val.into()], "alloc_array")
                    .unwrap();
                let ret_val = call_res
                    .try_as_basic_value()
                    .left()
                    .expect("alloc_arr must return value");

                // store returned array object into dst slot
                let entry_bb = function
                    .get_first_basic_block()
                    .expect("function has no entry block");
                self.get_or_create_slot(*dst, entry_bb);
                let dst_slot_ptr = *self.slot_registry.get(&dst).unwrap();
                self.builder.build_store(dst_slot_ptr, ret_val).unwrap();

                // cast to ArrayObject* to set len/data
                let array_struct = self.context.struct_type(
                    &[
                        self.context.i32_type().into(),                           // VarType
                        self.context.i32_type().into(),                           // refcnt
                        self.context.i32_type().into(),                           // len
                        object_ptr_type.ptr_type(AddressSpace::default()).into(), // data: Object **
                    ],
                    false,
                );
                let arr_ptr = self
                    .builder
                    .build_pointer_cast(
                        ret_val.into_pointer_value(),
                        array_struct.ptr_type(AddressSpace::default()),
                        "arr.cast",
                    )
                    .unwrap();

                // set len field
                let len_ptr = self
                    .builder
                    .build_struct_gep(array_struct, arr_ptr, 2, "arr.len")
                    .unwrap();
                self.builder
                    .build_store(
                        len_ptr,
                        self.context.i32_type().const_int(len as u64, false),
                    )
                    .unwrap();

                // get data pointer (Object**)
                let data_ptr_ptr = self
                    .builder
                    .build_struct_gep(array_struct, arr_ptr, 3, "arr.data.ptr")
                    .unwrap();
                let data_ptr = self
                    .builder
                    .build_load(
                        object_ptr_type.ptr_type(AddressSpace::default()),
                        data_ptr_ptr,
                        "arr.data",
                    )
                    .unwrap()
                    .into_pointer_value();

                for (i, elem_slot_id) in elems.iter().enumerate() {
                    // load element value
                    let elem_ptr = *self
                        .slot_registry
                        .get(elem_slot_id)
                        .expect("elem slot not found");
                    let elem_val = self
                        .builder
                        .build_load(object_ptr_type, elem_ptr, "arr.elem.load")
                        .unwrap();

                    // store into data[i]
                    let idx_val = self.context.i64_type().const_int(i as u64, false);
                    let elem_dst_ptr = unsafe {
                        self.builder
                            .build_in_bounds_gep(
                                object_ptr_type,
                                data_ptr,
                                &[idx_val],
                                "arr.elem.ptr",
                            )
                            .unwrap()
                    };
                    self.builder.build_store(elem_dst_ptr, elem_val).unwrap();
                }
            }
            HIRInst::ArrayGet { arr, idx, dst } => {
                // Load array object pointer from slot.
                let arr_ptr_ptr = *self.slot_registry.get(arr).expect("array slot not found");
                let arr_obj = self
                    .builder
                    .build_load(object_ptr_type, arr_ptr_ptr, "arr.load")
                    .unwrap()
                    .into_pointer_value();

                // Cast to ArrayObject*; layout assumed: { i32 type, i32 refcnt, i32 len, Object** data }.
                let array_struct = self.context.struct_type(
                    &[
                        self.context.i32_type().into(),
                        self.context.i32_type().into(),
                        self.context.i32_type().into(),
                        object_ptr_type.ptr_type(AddressSpace::default()).into(),
                    ],
                    false,
                );
                let arr_typed = self
                    .builder
                    .build_pointer_cast(
                        arr_obj,
                        array_struct.ptr_type(AddressSpace::default()),
                        "arr.cast",
                    )
                    .unwrap();

                // Load data pointer (Object**).
                let data_ptr_ptr = self
                    .builder
                    .build_struct_gep(array_struct, arr_typed, 3, "arr.data.ptr")
                    .unwrap();
                let data_ptr = self
                    .builder
                    .build_load(
                        object_ptr_type.ptr_type(AddressSpace::default()),
                        data_ptr_ptr,
                        "arr.data",
                    )
                    .unwrap()
                    .into_pointer_value();

                // Compute index: load Object* then extract the int value via runtime helper.
                let idx_ptr = *self.slot_registry.get(idx).expect("index slot not found");
                let idx_obj = self
                    .builder
                    .build_load(object_ptr_type, idx_ptr, "idx.obj")
                    .unwrap();

                let get_fn = *self
                    .runtime_fn
                    .get("get_int_value")
                    .expect("get_int_value not found");
                let idx_val_i32 = self
                    .builder
                    .build_call(get_fn, &[idx_obj.into()], "idx.value")
                    .unwrap()
                    .try_as_basic_value()
                    .left()
                    .expect("get_int_value must return int")
                    .into_int_value();

                let idx_val = self
                    .builder
                    .build_int_s_extend(idx_val_i32, self.context.i64_type(), "idx.i64")
                    .unwrap();

                // data[idx]
                let elem_ptr = unsafe {
                    self.builder
                        .build_in_bounds_gep(object_ptr_type, data_ptr, &[idx_val], "elem.ptr")
                        .unwrap()
                };
                let elem_val = self
                    .builder
                    .build_load(object_ptr_type, elem_ptr, "elem.load")
                    .unwrap();

                // Store result into dst slot.
                let entry_bb = function
                    .get_first_basic_block()
                    .expect("function has no entry block");
                self.get_or_create_slot(*dst, entry_bb);
                let dst_ptr = *self.slot_registry.get(dst).expect("dst slot not created");
                self.builder.build_store(dst_ptr, elem_val).unwrap();
            }
            HIRInst::ArraySet { arr, idx, src } => {
                let arr_ptr_ptr = *self.slot_registry.get(arr).expect("array slot not found");
                let arr_obj = self
                    .builder
                    .build_load(object_ptr_type, arr_ptr_ptr, "arr.load")
                    .unwrap()
                    .into_pointer_value();

                let array_struct = self.context.struct_type(
                    &[
                        self.context.i32_type().into(),
                        self.context.i32_type().into(),
                        self.context.i32_type().into(),
                        object_ptr_type.ptr_type(AddressSpace::default()).into(),
                    ],
                    false,
                );
                let arr_typed = self
                    .builder
                    .build_pointer_cast(
                        arr_obj,
                        array_struct.ptr_type(AddressSpace::default()),
                        "arr.cast",
                    )
                    .unwrap();

                let data_ptr_ptr = self
                    .builder
                    .build_struct_gep(array_struct, arr_typed, 3, "arr.data.ptr")
                    .unwrap();
                let data_ptr = self
                    .builder
                    .build_load(
                        object_ptr_type.ptr_type(AddressSpace::default()),
                        data_ptr_ptr,
                        "arr.data",
                    )
                    .unwrap()
                    .into_pointer_value();

                let idx_ptr = *self.slot_registry.get(idx).expect("index slot not found");
                let idx_obj = self
                    .builder
                    .build_load(object_ptr_type, idx_ptr, "idx.obj")
                    .unwrap();

                let get_fn = *self
                    .runtime_fn
                    .get("get_int_value")
                    .expect("get_int_value not found");
                let idx_val_i32 = self
                    .builder
                    .build_call(get_fn, &[idx_obj.into()], "idx.value")
                    .unwrap()
                    .try_as_basic_value()
                    .left()
                    .expect("get_int_value must return int")
                    .into_int_value();
                let idx_val = self
                    .builder
                    .build_int_s_extend(idx_val_i32, self.context.i64_type(), "idx.i64")
                    .unwrap();

                let elem_ptr = unsafe {
                    self.builder
                        .build_in_bounds_gep(object_ptr_type, data_ptr, &[idx_val], "elem.ptr")
                        .unwrap()
                };

                // store new value (array holds shared references; lifetime managed elsewhere)
                let src_ptr = *self.slot_registry.get(src).expect("src slot not found");
                let src_val = self
                    .builder
                    .build_load(object_ptr_type, src_ptr, "src.load")
                    .unwrap();
                self.builder.build_store(elem_ptr, src_val).unwrap();
            }
            HIRInst::Delete { dst } => {
                // Lifetime pass marked this slot as no longer needed; drop one reference.
                // Runtime dec_ref will free the object if refcnt reaches zero.
                let dec_ref_fn = match self.module.get_function("dec_ref") {
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

                #[cfg(debug_assertions)]
                {
                    let debug_ref_fn = *self
                        .runtime_fn
                        .get("runtime_debug_ref")
                        .expect("runtime_debug_ref not init");
                    let op_val = self.context.i32_type().const_int(0, false);
                    let slot_val = self
                        .context
                        .i32_type()
                        .const_int(dst.raw_id() as u64, false);
                    self.builder
                        .build_call(
                            debug_ref_fn,
                            &[op_val.into(), slot_val.into(), load_res.into()],
                            "dbg_dec_ref",
                        )
                        .unwrap();
                }

                match self
                    .builder
                    .build_call(dec_ref_fn, &[load_res.into()], "dec_ref")
                {
                    Err(e) => panic!("{}", e),
                    Ok(_) => {}
                }

                self.builder
                    .build_store(*ptr, object_ptr_type.const_null())
                    .unwrap();
            }
            HIRInst::Store { from, to } => {
                let entry_bb = function
                    .get_first_basic_block()
                    .expect("function has no entry block");
                self.get_or_create_slot(*to, entry_bb);
                let to_ptr = match self.slot_registry.get(to) {
                    None => match self.builder.build_alloca(object_ptr_type, "tmp") {
                        Err(e) => panic!("{}", e),
                        Ok(x) => x,
                    },
                    Some(x) => *x,
                };
                match from {
                    Value::Const(c) => {
                        let val = match c {
                            crate::hir::Const::Int(x) => {
                                let alloc_int = *self
                                    .runtime_fn
                                    .get("alloc_int")
                                    .expect("runtime alloc_int not found");
                                let arg = self.context.i64_type().const_int(*x as u64, false);
                                self.builder
                                    .build_call(alloc_int, &[arg.into()], "new int")
                                    .unwrap()
                                    .try_as_basic_value()
                                    .left()
                                    .expect("alloc_int must return")
                            }
                            crate::hir::Const::Float(float_key) => {
                                let FloatKey(x) = *float_key;
                                let alloc_float = *self
                                    .runtime_fn
                                    .get("alloc_float")
                                    .expect("runtime alloc_float not found");
                                let arg = self.context.f64_type().const_float(x);
                                self.builder
                                    .build_call(alloc_float, &[arg.into()], "new float")
                                    .unwrap()
                                    .try_as_basic_value()
                                    .left()
                                    .expect("alloc_float must return")
                            }
                            crate::hir::Const::Char(c) => {
                                let alloc_char = *self
                                    .runtime_fn
                                    .get("alloc_char")
                                    .expect("runtime alloc_char not found");
                                let arg = self.context.i32_type().const_int(*c as u64, false);
                                self.builder
                                    .build_call(alloc_char, &[arg.into()], "new char")
                                    .unwrap()
                                    .try_as_basic_value()
                                    .left()
                                    .expect("alloc_char must return")
                            }
                            crate::hir::Const::Bool(b) => {
                                let alloc_bool = *self
                                    .runtime_fn
                                    .get("alloc_bool")
                                    .expect("runtime alloc_bool not found");
                                let arg = self.context.bool_type().const_int(*b as u64, false);
                                self.builder
                                    .build_call(alloc_bool, &[arg.into()], "new bool")
                                    .unwrap()
                                    .try_as_basic_value()
                                    .left()
                                    .expect("alloc_bool must return")
                            }
                            crate::hir::Const::Null => {
                                object_ptr_type.const_null().as_basic_value_enum()
                            }
                            crate::hir::Const::String(s) => {
                                // create a global constant byte array for the string
                                let bytes = s.as_bytes();
                                let array_ty =
                                    self.context.i8_type().array_type(bytes.len() as u32);
                                let gname = format!(".str.{}", self.i);
                                let global = self.module.add_global(array_ty, None, &gname);
                                // build initializer
                                let mut elems: Vec<inkwell::values::IntValue> = Vec::new();
                                for b in bytes.iter() {
                                    elems.push(self.context.i8_type().const_int(*b as u64, false));
                                }
                                let const_array = self.context.i8_type().const_array(&elems);
                                global.set_initializer(&const_array);
                                global.set_constant(true);

                                // get i8* pointer to the first element by pointer-casting
                                let gv_ptr = global.as_pointer_value();
                                let gep = self
                                    .builder
                                    .build_pointer_cast(gv_ptr, object_ptr_type, "str.ptr")
                                    .unwrap();

                                // call runtime alloc_string(ptr, len) -> Object*
                                let alloc_str = *self
                                    .runtime_fn
                                    .get("alloc_string")
                                    .expect("runtime alloc_string not found");
                                let len_val =
                                    self.context.i32_type().const_int(bytes.len() as u64, false);
                                let call_res = self
                                    .builder
                                    .build_call(
                                        alloc_str,
                                        &[gep.into(), len_val.into()],
                                        "new string",
                                    )
                                    .unwrap();
                                self.i += 1;

                                call_res
                                    .try_as_basic_value()
                                    .left()
                                    .expect("alloc_string must return")
                            }
                        };

                        match self.builder.build_store(to_ptr, val) {
                            Err(e) => panic!("{}", e),
                            Ok(_) => {}
                        };
                    }
                    Value::Obj(_) | Value::Null => {
                        self.builder
                            .build_store(to_ptr, object_ptr_type.const_null())
                            .unwrap();
                    }
                }
            }
            HIRInst::Br {
                cond,
                then_block,
                else_block,
            } => {
                // 1. cond_ir 返回 i1
                let is_truth_fn = *self.runtime_fn.get("is_truth").expect("runtime not found");
                let cond_slot_ptr = self.slot_registry.get(cond).expect("");
                let slot_val = self
                    .builder
                    .build_load(object_ptr_type, *cond_slot_ptr, "cond.load")
                    .unwrap();
                let cond_i1 = self
                    .builder
                    .build_call(is_truth_fn, &[slot_val.into()], "call is_truth")
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
            HIRInst::Call {
                id,
                args: call_args,
                dst,
            } => {
                let func = *self
                    .llvm_func_registry
                    .get(id)
                    .expect(&format!("no such func, id:{}", id));

                let mut args = Vec::<BasicMetadataValueEnum>::new();

                // 直接使用调用处给定的参数 slot，加载其 Object* 并传入被调用函数
                for slot_id in call_args.iter() {
                    let slot_ptr = *self.slot_registry.get(slot_id).expect("arg slot not found");
                    let arg_val = self
                        .builder
                        .build_load(object_ptr_type, slot_ptr, "call.arg")
                        .unwrap()
                        .into();
                    args.push(arg_val);
                }

                let call_res = self
                    .builder
                    .build_call(func, args.as_slice(), "call")
                    .unwrap();

                // 如果调用有返回值（Some），则写入目标 slot；否则跳过。
                if let Some(ret_val) = call_res.try_as_basic_value().left() {
                    let entry_bb = function
                        .get_first_basic_block()
                        .expect("function has no entry block");
                    self.get_or_create_slot(*dst, entry_bb);
                    let dst_slot_ptr = *self.slot_registry.get(dst).unwrap();
                    self.builder.build_store(dst_slot_ptr, ret_val).unwrap();
                }
            }
            HIRInst::BinOp {
                left,
                op,
                right,
                dst,
            } => {
                let entry_bb = function
                    .get_first_basic_block()
                    .expect("function has no entry block");
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

                let ret_val = call_res
                    .try_as_basic_value()
                    .left() // 有返回值才会是 Some
                    .expect("binop must return");
                let dst_slot_ptr = *self.slot_registry.get(&dst).unwrap();
                self.builder.build_store(dst_slot_ptr, ret_val).unwrap();
            }
            HIRInst::UnaryOp { op, expr, dst } => {
                let entry_bb = function
                    .get_first_basic_block()
                    .expect("function has no entry block");
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

                let call_res = match op {
                    UnaryOperation::Not => self
                        .builder
                        .build_call(not_fn, &[expr_obj.into()], "not")
                        .unwrap(),
                };

                let ret_val = call_res
                    .try_as_basic_value()
                    .left()
                    .expect("binop must return");
                let dst_slot_ptr = *self.slot_registry.get(&dst).unwrap();
                self.builder.build_store(dst_slot_ptr, ret_val).unwrap();
            }
            HIRInst::IncRef { obj } => {
                let object_ptr_type = self.context.i8_type().ptr_type(AddressSpace::default());

                let slot_ptr: &PointerValue<'_> =
                    self.slot_registry.get(&obj).expect("sloy not found");
                let slot_val = self
                    .context
                    .i32_type()
                    .const_int(obj.raw_id() as u64, false);
                let obj_val = self
                    .builder
                    .build_load(object_ptr_type, *slot_ptr, "left_obj")
                    .unwrap();

                #[cfg(debug_assertions)]
                {
                    let debug_ref_fn = *self
                        .runtime_fn
                        .get("runtime_debug_ref")
                        .expect("runtime_debug_ref not init");
                    let op_val = self.context.i32_type().const_int(1, false);

                    self.builder
                        .build_call(
                            debug_ref_fn,
                            &[op_val.into(), slot_val.into(), obj_val.into()],
                            "dbg_inc_ref",
                        )
                        .unwrap();
                }
                let inc_ref_fn = *self.runtime_fn.get("inc_ref").expect("runtime not init");
                self.builder
                    .build_call(inc_ref_fn, &[obj_val.into()], "inc_ref")
                    .unwrap();
            }
            HIRInst::DecRef { obj } => {
                let object_ptr_type = self.context.i8_type().ptr_type(AddressSpace::default());

                let slot_ptr: &PointerValue<'_> =
                    self.slot_registry.get(&obj).expect("sloy not found");
                let slot_val = self
                    .context
                    .i32_type()
                    .const_int(obj.raw_id() as u64, false);
                let obj_val = self
                    .builder
                    .build_load(object_ptr_type, *slot_ptr, "left_obj")
                    .unwrap();

                #[cfg(debug_assertions)]
                {
                    let debug_ref_fn = *self
                        .runtime_fn
                        .get("runtime_debug_ref")
                        .expect("runtime_debug_ref not init");

                    let op_val = self.context.i32_type().const_int(0, false);

                    self.builder
                        .build_call(
                            debug_ref_fn,
                            &[op_val.into(), slot_val.into(), obj_val.into()],
                            "dbg_dec_ref",
                        )
                        .unwrap();
                }

                let dec_ref_fn = *self.runtime_fn.get("dec_ref").expect("runtime not init");
                self.builder
                    .build_call(dec_ref_fn, &[obj_val.into()], "dec_ref")
                    .unwrap();
            }
            HIRInst::Bind { var, obj } => {
                let slot_ptr = self.slot_registry[&obj]; // Object**

                // 1. 将 slot 中的 Object* 值加载出来（供存入 var 用）
                let obj_val = self
                    .builder
                    .build_load(object_ptr_type, slot_ptr, "bind.load")
                    .unwrap();

                // 2. 如果 var 已有对应的 alloca（如参数或之前创建的变量），
                //    将值写入该 alloca；否则在函数入口创建一个新的 alloca 并写入。
                if let Some(existing_var_ptr) = self.var_registry.get(var) {
                    self.builder
                        .build_store(*existing_var_ptr, obj_val)
                        .unwrap();
                } else {
                    // 在函数 entry block 中创建 var 的 alloca（但不要在 entry 中执行 store）
                    let entry_bb = function
                        .get_first_basic_block()
                        .expect("function has no entry block");
                    let cur_bb = self.builder.get_insert_block();

                    // 将插入点定位到 entry 块的第一个指令之前（若存在），用于创建 alloca
                    if let Some(first_instr) = entry_bb.get_first_instruction() {
                        self.builder.position_before(&first_instr);
                    } else {
                        self.builder.position_at_end(entry_bb);
                    }

                    let var_alloc = self
                        .builder
                        .build_alloca(object_ptr_type, &format!("var{}", var))
                        .unwrap();

                    // 立即把 var_alloc 注册到 var_registry
                    self.var_registry.insert(*var, var_alloc);

                    // 恢复到原来的插入点，在当前块执行 store（使用在当前块中定义的 obj_val）
                    if let Some(bb) = cur_bb {
                        self.builder.position_at_end(bb);
                    }

                    self.builder.build_store(var_alloc, obj_val).unwrap();
                }
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

    pub fn gen_lir(&mut self) {
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
                    // 不再清空 `var_registry`，以允许嵌套函数访问作用域外的变量。
                    // 之前清空会导致无法在内部函数中读取外层变量。
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

        // Ensure main has a return; insert `ret i32 0` into any unterminated basic block.
        if let Some(main_fn) = self.llvm_func_registry.get(&self.main_id) {
            let i32_type = self.context.i32_type();
            for bb in main_fn.get_basic_blocks() {
                if bb.get_terminator().is_none() {
                    let cur = self.builder.get_insert_block();
                    self.builder.position_at_end(bb);
                    self.builder
                        .build_return(Some(&i32_type.const_int(0, false)))
                        .unwrap();
                    if let Some(c) = cur {
                        self.builder.position_at_end(c);
                    }
                }
            }
        }

        let lir = self.module.print_to_string().to_string();
        let lir_path = Path::new("./build/lir.txt");
        if let Some(parent) = lir_path.parent() {
            let _ = fs::create_dir_all(parent);
        }
        let _ = fs::File::create(lir_path);
        fs::write(lir_path, "").unwrap();
        let mut lir_file = OpenOptions::new().append(true).open(lir_path).unwrap();
        lir_file.write(&lir.to_string().as_bytes()).unwrap();
        lir_file.write("\n".as_bytes()).unwrap();
    }

    pub fn to_llvm_bc(&mut self, path_str: &str) {
        let path = Path::new(path_str);
        if let Some(parent) = path.parent() {
            let _ = fs::create_dir_all(parent);
        }
        let _ = fs::File::create(path);
        self.module.write_bitcode_to_path(path);
    }
}
