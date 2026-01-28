use std::{collections::HashMap, fs::{self, OpenOptions}, io::Write};

use inkwell::{
    AddressSpace,
    basic_block::BasicBlock,
    context::Context,
    module::Module,
    types::{BasicMetadataTypeEnum, BasicTypeEnum},
    values::{
        BasicMetadataValueEnum, BasicValue, BasicValueEnum, FunctionValue, IntValue, PointerValue,
    },
};
use macro_lib::{bin_operator_fn, call_bin_operator_fn, make_fn};

use crate::{
    hir::{BlockId, FloatKey, FuncId, HIR, HIRInst, HirFuncSymbol, SlotId, Value},
    lir::LirGenerator,
    types::VarType,
};

impl<'ctx> LirGenerator<'ctx> {
    fn init_env(&mut self) {
        let obj_ptr = self.context.i8_type().ptr_type(AddressSpace::default());

        let err_fn =
            self.module
                .add_function("err", self.context.void_type().fn_type(&[obj_ptr.into()], false), None);
        self.runtime_fn.insert("err", err_fn);

        let is_type_fn = self.module.add_function(
            "is_type",
            self.context
                .bool_type()
                .fn_type(&[self.context.i64_type().into(), obj_ptr.into()], false),
            None,
        );
        self.runtime_fn.insert("is_type", is_type_fn);

        let del_obj_fn =
            self.module
                .add_function("del_obj", self.context.void_type().fn_type(&[obj_ptr.into()], false), None);
        self.runtime_fn.insert("del_obj", del_obj_fn);

        let dec_ref_fn =
            self.module
                .add_function("dec_ref", self.context.void_type().fn_type(&[obj_ptr.into()], false), None);
        self.runtime_fn.insert("dec_ref", dec_ref_fn);

        let inc_ref_fn =
            self.module
                .add_function("inc_ref", self.context.void_type().fn_type(&[obj_ptr.into()], false), None);
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

    fn init_operation_mod(&mut self) {
        bin_operator_fn!(
            self,
            add,
            sub,
            mul,
            div,
            equal,
            greater,
            less,
            greater_equal,
            less_equal,
            and,
            or
        );

        let obj_ptr = self.context.i8_type().ptr_type(AddressSpace::default());
        let not_fn =
            self.module
                .add_function("not", obj_ptr.fn_type(&[obj_ptr.into()], false), None);
        self.runtime_fn.insert("not", not_fn);
    }

    fn init_runtime(&mut self) {
        self.init_env();
        self.init_operation_mod();
    }

    pub(crate) fn new(
        context: &'ctx Context,
        hirs: &Vec<HIR>,
        func_registry: HashMap<FuncId, HirFuncSymbol>,
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
            runtime_fn: HashMap::<&str, FunctionValue>::new(),
            bool_object_type: context.struct_type(
                &[
                    object_type.into(),         // Object header
                    context.bool_type().into(), // value 字段
                ],
                false, // packed?
            ),
            i: 0,
        };

        res.init_runtime();

        res
    }

    fn cond_ir(&mut self, current_fn: FunctionValue<'ctx>, cond_slot: SlotId) -> IntValue<'ctx> {
        // 1. 从 slot_registry 拿 Object* 指针
        let entry_bb = self
            .builder
            .get_insert_block()
            .expect("builder has no insertion block");
        self.get_or_create_slot(cond_slot, entry_bb);
        let loaded_ptr = *self.slot_registry.get(&cond_slot).expect("slot not found");

        // 2. Load Object*
        let loaded: BasicValueEnum<'ctx> = self
            .builder
            .build_load(
                self.context.i8_type().ptr_type(AddressSpace::default()),
                loaded_ptr,
                "cond_loaded",
            )
            .unwrap();

        // 3. 类型检查：调用 runtime is_type(Object*, VarType::Bool)
        let is_type_fn = *self.runtime_fn.get("is_type").expect("runtime not init");
        let type_tag = self
            .context
            .i32_type()
            .const_int(VarType::Bool as u64, false);

        let is_type_call = self
            .builder
            .build_call(
                is_type_fn,
                &[type_tag.into(), loaded.into()],
                "is_type_call",
            )
            .unwrap();

        let is_bool = is_type_call
            .try_as_basic_value()
            .left()
            .expect("is_type has no return")
            .into_int_value();

        // 4. 创建 ok / panic 分支
        let ok_bb = self.context.append_basic_block(current_fn, "ok");
        let panic_bb = self.context.append_basic_block(current_fn, "panic");

        self.builder
            .build_conditional_branch(is_bool, ok_bb, panic_bb)
            .unwrap();

        // 5. panic 分支
        self.builder.position_at_end(panic_bb);
        let panic_fn = *self.runtime_fn.get("err").expect("panic not init");

        let new_str_obj_fn = *self
            .runtime_fn
            .get("alloc_string")
            .expect("runtime not init");

        let hello_str = self
            .builder
            .build_global_string_ptr("type unsuitable", "err")
            .unwrap();
        let metadata_arg: BasicMetadataValueEnum = hello_str.as_pointer_value().into();

        self.builder
            .build_call(new_str_obj_fn, &[metadata_arg], "err")
            .unwrap();
        self.builder.build_call(panic_fn, &[], "panic").unwrap();
        self.builder.build_unreachable().unwrap();

        // 6. ok 分支
        self.builder.position_at_end(ok_bb);

        // 7. 强制类型转换 Object* -> BoolObject*
        let bool_ptr_type: BasicTypeEnum<'ctx> = self
            .bool_object_type
            .ptr_type(AddressSpace::default())
            .into();
        let bool_obj_ptr: PointerValue<'ctx> = self
            .builder
            .build_bitcast(
                loaded,
                bool_ptr_type, // 这里是 BasicTypeEnum
                "bool_obj",
            )
            .expect("bitcast failed")
            .into_pointer_value(); // 转成 PointerValue

        // 8. 获取 BoolObject.value 字段
        let value_ptr = self
            .builder
            .build_struct_gep(
                self.bool_object_type,
                bool_obj_ptr,
                1, // value 在第二个字段
                "value_ptr",
            )
            .expect("GEP failed");

        // 9. load value 字段
        let cond_i1 = self
            .builder
            .build_load(self.context.bool_type(), value_ptr, "cond_i1")
            .unwrap()
            .into_int_value();

        cond_i1
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
        self.builder
            .build_store(slot_ptr, obj_ptr_ty.const_null())
            .unwrap();

        self.builder.position_at_end(current_bb);

        self.slot_registry.insert(slot, slot_ptr);
        slot_ptr
    }

    fn gen_func_ir(&mut self, func_def: &FuncId) -> FunctionValue {
        let id = func_def.clone();

        let sym = match self.func_registry.get(&id) {
            None => panic!(""),
            Some(x) => x,
        };
        let ptr = self.context.i8_type().ptr_type(AddressSpace::default());
        let mut args = Vec::<BasicMetadataTypeEnum<'ctx>>::new();
        for _ in 0..sym.param_id.len() {
            args.push(ptr.into());
        }
        let fn_type = ptr.fn_type(&args, false);
        let function = self.module.add_function(&id.to_string(), fn_type, None);
        self.llvm_func_registry.insert(id, function);
        function
    }
    fn new_llvm_block(&mut self, function: FunctionValue<'ctx>, id: &BlockId) {
        if let Some(_) = self.block_registry.get(id) {
            return;
        }
        let name = id.to_string();
        let block = self.context.append_basic_block(function, &name);
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
        let function = *self
            .llvm_func_registry
            .get(&FuncId(cur_func.expect("").parse::<i32>().unwrap()))
            .expect("");

        let object_ptr_type = self.context.i8_type().ptr_type(AddressSpace::default());
        match inst {
            HIRInst::New {
                obj_type: _,
                dst: _,
            } => todo!(),
            HIRInst::Load { var, obj: slot } => {
                let slot_type = object_ptr_type; // alloca Object*
                let slot_ptr: PointerValue<'ctx> =
                    self.builder.build_alloca(slot_type, "slot").unwrap();

                let loaded: PointerValue<'ctx> = self
                    .builder
                    .build_load(object_ptr_type, slot_ptr, "load_slot_1")
                    .unwrap()
                    .into_pointer_value(); // Object*

                self.slot_registry.insert(*slot, loaded);
            }
            HIRInst::Delete { dst } => {
                let dec_ref_fn = match self.module.get_function("del_obj") {
                    None => panic!("ayanami_runtime not found"),
                    Some(x) => x,
                };
                let ptr = match self.slot_registry.get(dst) {
                    None => panic!(""),
                    Some(x) => x,
                };
                match self
                    .builder
                    .build_call(dec_ref_fn, &[(*ptr).into()], "del_obj")
                {
                    Err(e) => panic!("{}", e),
                    Ok(_) => {}
                }
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
                let cond_i1 = self.cond_ir(function, *cond);

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
                let func = *self.llvm_func_registry.get(id).expect("no such func");
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
                let slot_ptr = *self.slot_registry.get(ret_obj).expect(""); // Object**

                // 1. 从 slot 中 load 出 Object*
                let ret_val = self
                    .builder
                    .build_load(object_ptr_type, slot_ptr, "ret.load")
                    .unwrap();

                // 2. ret Object*
                self.builder.build_return(Some(&ret_val)).unwrap();
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
            println!("index: {}", i + 1);
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
                        let function = *self
                            .llvm_func_registry
                            .get(&FuncId(cur_func.clone().expect("").parse::<i32>().unwrap()))
                            .expect("function not found when declaring current block");
                        self.new_llvm_block(function, &block_id);
                    }

                    // 将当前 block 切换为插入点
                    self.emit_llvm_block(&block_id);

                    // 更新 cur_block_id 为当前 block
                    cur_block_id = block_id;
                }
                HIR::FuncLabel(ref func_def) => {
                    let t = self.gen_func_ir(func_def);
                    cur_func = Some(func_def.get_id().to_string());
                }
            }

            i += 1;
        }

        let lir = self.module.print_to_string().to_string();
        fs::write("./lir.txt", "").unwrap();
        let mut lir_file = OpenOptions::new().append(true).open("./lir.txt").unwrap();
        lir_file.write(&lir.to_string().as_bytes()).unwrap();
        lir_file.write("\n".as_bytes()).unwrap();
    }
}
