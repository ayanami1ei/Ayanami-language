use std::collections::HashMap;

use inkwell::{
    AddressSpace,
    basic_block::BasicBlock,
    context::Context,
    module::Module,
    types::BasicMetadataTypeEnum,
    values::{BasicValue, FunctionValue},
};

use crate::{
    hir::{BlockId, FloatKey, FuncDef, FuncId, HIR, HIRInst, HirFuncSymbol, Value},
    lir::LirGenerator,
};

impl<'ctx> LirGenerator<'ctx> {
    pub(crate) fn new(
        context: &'ctx Context,
        hirs: &Vec<HIR>,
        func_registry: HashMap<FuncId, HirFuncSymbol>,
    ) -> LirGenerator<'ctx> {
        let module = context.create_module("ayanami_modlue");
        let builder = context.create_builder();
        LirGenerator {
            context,
            module,
            builder,
            hirs: hirs.to_vec(),
            func_registry,
            block_registry: HashMap::new(),
            slot_registry: HashMap::new(),
            i: 0,
        }
    }

    fn bind_block(&mut self, block_id: BlockId, llvm_block: BasicBlock<'ctx>) {
        self.block_registry.insert(block_id, llvm_block);
    }

    fn gen_func_ir(&mut self, func_def: &FuncDef) -> FunctionValue {
        let id = func_def.get_id();

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
        function
    }
    fn new_llvm_block(&mut self, function: FunctionValue<'ctx>, id: &BlockId){
        let name = id.to_string();
        let block=self.context.append_basic_block(function, &name);
        self.block_registry.insert(*id, block);
    }
    fn emit_llvm_block(&mut self, id:&BlockId){
        let block=match self.block_registry.get(id){
            None=>panic!("block no found"),
            Some(x)=>*x
        };
        self.builder.position_at_end(block);
    }

    fn gen_inst_ir(&mut self, function: FunctionValue, inst: &HIRInst, cur_func:Option<String>) {
        let cur_function=match cur_func{
            None=>panic!(""),
            Some(x)=>x
        };
        match inst {
            HIRInst::New {
                obj_type: _,
                dst: _,
            } => todo!(),
            HIRInst::Load { var: _, obj: _ } => todo!(),
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
                            let FloatKey(x)=*float_key;
                            self
                            .context
                            .f64_type()
                            .const_float(x)
                            .as_basic_value_enum()
                        },
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

                    match self.builder.build_store(to_ptr, val){
                        Err(e)=>panic!("e"),
                        Ok(_)=>{}
                    };
                }
            }
            HIRInst::Br {
                cond,
                then_block,
                else_block,
            } => {
                let comparison=match self.slot_registry.get(cond){
                    None=>panic!(),
                    Some(x)=>*x
                };

                let loaded = self.builder.build_load(comparison, "tmp");

                self.new_llvm_block(function, then_block);
                self.new_llvm_block(function, else_block);

                let llvm_then_block=match self.block_registry.get(then_block){
                    None=>panic!(),
                    Some(x)=>*x
                };
                let llvm_else_block=match self.block_registry.get(else_block){
                    None=>panic!(),
                    Some(x)=>*x
                };

                self.builder.build_conditional_branch(comparison, llvm_then_block, llvm_else_block);
            },
            HIRInst::Jmp { target } => todo!(),
            HIRInst::Call { id, ret } => todo!(),
            HIRInst::BinOp {
                left,
                op,
                right,
                dst,
            } => todo!(),
            HIRInst::UnaryOp { op, expr, dst } => todo!(),
            HIRInst::IncRef { obj } => todo!(),
            HIRInst::DecRef { obj } => todo!(),
            HIRInst::Bind { var, obj } => todo!(),
            HIRInst::Ret { ret_obj } => todo!(),
        }
    }

    pub(crate) fn gen_lir(&mut self) -> Module<'ctx> {
        let mut cur_func: Option<String> = None;

        let hirs = self.hirs.clone();
        let mut i = 0;
        while i < hirs.len() {
            match hirs[i].clone() {
                HIR::Inst(hirinst) => {}
                HIR::Block(block_id) => {
                    self.emit_llvm_block(&block_id);
                }
                HIR::FuncLabel(ref func_def) => {
                    let t = self.gen_func_ir(func_def);
                    cur_func = Some(func_def.get_id().to_string());
                }
            }

            i += 1;
        }

        self.module.clone()
    }
}
