use super::*;

impl LirNode for SLirAlloca {
    fn clone_node(&self) -> Box<dyn LirNode> { Box::new(self.clone()) }
    fn kind(&self) -> &'static str { "Alloca" }
    fn as_any(&self) -> &dyn std::any::Any { self }
    fn emit(&self, ctx: &mut LirEmitCtx) -> Vec<String> {
        let llvm_ty = ctx.llvm_type(&self.ty);
        vec![format!("%v{} = alloca {}, align 8", self.var.0, llvm_ty)]
    }
    fn display(&self, f: &mut dyn Write) -> std::fmt::Result {
        writeln!(f, "    alloca v{} : {:?}", self.var.0, self.ty)
    }
    fn serialize(&self, buf: &mut Vec<u8>) {
        buf.push(0);
        put_u32(buf, self.var.0 as u32);
        put_type(buf, &self.ty);
    }
}

impl LirNode for SLirStore {
    fn clone_node(&self) -> Box<dyn LirNode> { Box::new(self.clone()) }
    fn kind(&self) -> &'static str { "Store" }
    fn as_any(&self) -> &dyn std::any::Any { self }
    fn emit(&self, ctx: &mut LirEmitCtx) -> Vec<String> {
        let mut lines = Vec::new();
        let llvm_ty = ctx.llvm_type(&self.ty);
        let src_str = match &self.src {
            LirValue::Param(i) => format!("%{}", i),
            LirValue::Tmp(t) => format!("%t{}", t),
            LirValue::Var(v) => {
                let tmp = ctx.tmp();
                lines.push(format!("%e{} = load {}, ptr %v{}, align 8", tmp, llvm_ty, v.0));
                format!("%e{}", tmp)
            }
            LirValue::Literal(lit, _) => lit_to_string(lit, &self.ty),
        };
        lines.push(format!("store {} {}, ptr %v{}, align 8", llvm_ty, src_str, self.dest.0));
        lines
    }
    fn display(&self, f: &mut dyn Write) -> std::fmt::Result {
        writeln!(f, "    store {:?} -> v{} : {:?}", self.src, self.dest.0, self.ty)
    }
    fn serialize(&self, buf: &mut Vec<u8>) {
        buf.push(1);
        put_u32(buf, self.dest.0 as u32);
        put_value(buf, &self.src);
        put_type(buf, &self.ty);
    }
}

impl LirNode for SLirLoad {
    fn clone_node(&self) -> Box<dyn LirNode> { Box::new(self.clone()) }
    fn kind(&self) -> &'static str { "Load" }
    fn as_any(&self) -> &dyn std::any::Any { self }
    fn emit(&self, ctx: &mut LirEmitCtx) -> Vec<String> {
        let llvm_ty = ctx.llvm_type(&self.ty);
        vec![format!("%t{} = load {}, ptr %v{}, align 8", self.dest, llvm_ty, self.src.0)]
    }
    fn display(&self, f: &mut dyn Write) -> std::fmt::Result {
        writeln!(f, "    t{} = load v{} : {:?}", self.dest, self.src.0, self.ty)
    }
    fn serialize(&self, buf: &mut Vec<u8>) {
        buf.push(2);
        put_u64(buf, self.dest);
        put_u32(buf, self.src.0 as u32);
        put_type(buf, &self.ty);
    }
}

impl LirNode for SLirBinOp {
    fn clone_node(&self) -> Box<dyn LirNode> { Box::new(self.clone()) }
    fn kind(&self) -> &'static str { "BinOp" }
    fn as_any(&self) -> &dyn std::any::Any { self }
    fn emit(&self, ctx: &mut LirEmitCtx) -> Vec<String> {
        let l = ctx.value_ref(&self.lhs, &self.ty);
        let r = ctx.value_ref(&self.rhs, &self.ty);
        let is_ptr = l == "null" || r == "null" || l.contains("ptr") || r.contains("ptr");
        let icmp_llvm = |ty: &HirType| -> String {
            match ty {
                HirType::Char => "i8".into(),
                HirType::Bool => "i1".into(),
                HirType::Int => "i64".into(),
                HirType::IntN { bits, .. } => format!("i{}", bits),
                _ => if is_ptr { "ptr".into() } else { "i64".into() },
            }
        };
        let unsigned = matches!(&self.ty, HirType::IntN { signed: false, .. });
        match (&self.op, &self.ty) {
            (BinaryOp::Add, HirType::Int) => vec![format!("%t{} = add i64 {}, {}", self.dest, l, r)],
            (BinaryOp::Sub, HirType::Int) => vec![format!("%t{} = sub i64 {}, {}", self.dest, l, r)],
            (BinaryOp::Mul, HirType::Int) => vec![format!("%t{} = mul i64 {}, {}", self.dest, l, r)],
            (BinaryOp::Div, HirType::Int) => vec![format!("%t{} = sdiv i64 {}, {}", self.dest, l, r)],
            (BinaryOp::Mod, HirType::Int) => vec![format!("%t{} = srem i64 {}, {}", self.dest, l, r)],
            (BinaryOp::Add, HirType::Float) => vec![format!("%t{} = fadd double {}, {}", self.dest, l, r)],
            (BinaryOp::Sub, HirType::Float) => vec![format!("%t{} = fsub double {}, {}", self.dest, l, r)],
            (BinaryOp::Mul, HirType::Float) => vec![format!("%t{} = fmul double {}, {}", self.dest, l, r)],
            (BinaryOp::Div, HirType::Float) => vec![format!("%t{} = fdiv double {}, {}", self.dest, l, r)],
            (BinaryOp::Mod, HirType::Float) => vec![format!("%t{} = frem double {}, {}", self.dest, l, r)],
            (BinaryOp::Add, HirType::Char) => vec![format!("%t{} = add i8 {}, {}", self.dest, l, r)],
            (BinaryOp::Sub, HirType::Char) => vec![format!("%t{} = sub i8 {}, {}", self.dest, l, r)],
            (BinaryOp::Add, HirType::IntN { bits, .. }) => vec![format!("%t{} = add i{} {}, {}", self.dest, bits, l, r)],
            (BinaryOp::Sub, HirType::IntN { bits, .. }) => vec![format!("%t{} = sub i{} {}, {}", self.dest, bits, l, r)],
            (BinaryOp::Mul, HirType::IntN { bits, .. }) => vec![format!("%t{} = mul i{} {}, {}", self.dest, bits, l, r)],
            (BinaryOp::Div, HirType::IntN { bits, signed }) => {
                let op = if *signed { "sdiv" } else { "udiv" };
                vec![format!("%t{} = {} i{} {}, {}", self.dest, op, bits, l, r)]
            }
            (BinaryOp::Mod, HirType::IntN { bits, signed }) => {
                let op = if *signed { "srem" } else { "urem" };
                vec![format!("%t{} = {} i{} {}, {}", self.dest, op, bits, l, r)]
            }
            (BinaryOp::BitAnd | BinaryOp::BitOr | BinaryOp::BitXor,
             HirType::Int | HirType::Char | HirType::Bool | HirType::IntN { .. }) => {
                let w = icmp_llvm(&self.ty);
                let op = match self.op { BinaryOp::BitAnd => "and", BinaryOp::BitOr => "or", _ => "xor" };
                vec![format!("%t{} = {} {} {}, {}", self.dest, op, w, l, r)]
            }
            (BinaryOp::Shl, HirType::Int | HirType::Char | HirType::IntN { .. }) => {
                let w = icmp_llvm(&self.ty);
                vec![format!("%t{} = shl {} {}, {}", self.dest, w, l, r)]
            }
            (BinaryOp::Shr, HirType::Int | HirType::Char | HirType::IntN { .. }) => {
                let w = icmp_llvm(&self.ty);
                let unsigned = matches!(&self.ty, HirType::Char | HirType::IntN { signed: false, .. });
                let op = if unsigned { "lshr" } else { "ashr" };
                vec![format!("%t{} = {} {} {}, {}", self.dest, op, w, l, r)]
            }
            (BinaryOp::Eq, _) if self.ty != HirType::Float => {
                let llvm_int = icmp_llvm(&self.ty);
                if is_ptr { vec![format!("%t{} = icmp eq ptr {}, {}", self.dest, l, r)] }
                else { vec![format!("%t{} = icmp eq {} {}, {}", self.dest, llvm_int, l, r)] }
            }
            (BinaryOp::Neq, _) if self.ty != HirType::Float => {
                let llvm_int = icmp_llvm(&self.ty);
                if is_ptr { vec![format!("%t{} = icmp ne ptr {}, {}", self.dest, l, r)] }
                else { vec![format!("%t{} = icmp ne {} {}, {}", self.dest, llvm_int, l, r)] }
            }
            (BinaryOp::Lt, _) if self.ty != HirType::Float => {
                let llvm_int = icmp_llvm(&self.ty);
                if is_ptr { vec![format!("%t{} = icmp ult ptr {}, {}", self.dest, l, r)] }
                else { let p = if unsigned { "ult" } else { "slt" }; vec![format!("%t{} = icmp {} {} {}, {}", self.dest, p, llvm_int, l, r)] }
            }
            (BinaryOp::Gt, _) if self.ty != HirType::Float => {
                let llvm_int = icmp_llvm(&self.ty);
                if is_ptr { vec![format!("%t{} = icmp ugt ptr {}, {}", self.dest, l, r)] }
                else { let p = if unsigned { "ugt" } else { "sgt" }; vec![format!("%t{} = icmp {} {} {}, {}", self.dest, p, llvm_int, l, r)] }
            }
            (BinaryOp::Le, _) if self.ty != HirType::Float => {
                let llvm_int = icmp_llvm(&self.ty);
                if is_ptr { vec![format!("%t{} = icmp ule ptr {}, {}", self.dest, l, r)] }
                else { let p = if unsigned { "ule" } else { "sle" }; vec![format!("%t{} = icmp {} {} {}, {}", self.dest, p, llvm_int, l, r)] }
            }
            (BinaryOp::Ge, _) if self.ty != HirType::Float => {
                let llvm_int = icmp_llvm(&self.ty);
                if is_ptr { vec![format!("%t{} = icmp uge ptr {}, {}", self.dest, l, r)] }
                else { let p = if unsigned { "uge" } else { "sge" }; vec![format!("%t{} = icmp {} {} {}, {}", self.dest, p, llvm_int, l, r)] }
            }
            (BinaryOp::Eq, HirType::Float) => vec![format!("%t{} = fcmp oeq double {}, {}", self.dest, l, r)],
            (BinaryOp::Neq, HirType::Float) => vec![format!("%t{} = fcmp one double {}, {}", self.dest, l, r)],
            (BinaryOp::Lt, HirType::Float) => vec![format!("%t{} = fcmp olt double {}, {}", self.dest, l, r)],
            (BinaryOp::Gt, HirType::Float) => vec![format!("%t{} = fcmp ogt double {}, {}", self.dest, l, r)],
            (BinaryOp::Le, HirType::Float) => vec![format!("%t{} = fcmp ole double {}, {}", self.dest, l, r)],
            (BinaryOp::Ge, HirType::Float) => vec![format!("%t{} = fcmp oge double {}, {}", self.dest, l, r)],
            (BinaryOp::And, _) => vec![format!("%t{} = and i1 {}, {}", self.dest, l, r)],
            (BinaryOp::Or, _) => vec![format!("%t{} = or i1 {}, {}", self.dest, l, r)],
            _ => vec![],
        }
    }
    fn display(&self, f: &mut dyn Write) -> std::fmt::Result {
        writeln!(f, "    t{} = {:?} {:?} {:?} : {:?}", self.dest, self.op, self.lhs, self.rhs, self.ty)
    }
    fn serialize(&self, buf: &mut Vec<u8>) {
        buf.push(3);
        put_u64(buf, self.dest);
        put_u32(buf, self.op as u32);
        put_value(buf, &self.lhs);
        put_value(buf, &self.rhs);
        put_type(buf, &self.ty);
        put_type(buf, &self.result_ty);
    }
}

impl LirNode for SLirUnaryOp {
    fn clone_node(&self) -> Box<dyn LirNode> { Box::new(self.clone()) }
    fn kind(&self) -> &'static str { "UnaryOp" }
    fn as_any(&self) -> &dyn std::any::Any { self }
    fn emit(&self, ctx: &mut LirEmitCtx) -> Vec<String> {
        let s = ctx.value_ref(&self.src, &self.ty);
        match (&self.op, &self.ty) {
            (UnaryOp::Neg, HirType::Int) => vec![format!("%t{} = sub i64 0, {}", self.dest, s)],
            (UnaryOp::Neg, HirType::IntN { bits, .. }) => vec![format!("%t{} = sub i{} 0, {}", self.dest, bits, s)],
            (UnaryOp::Neg, HirType::Float) => vec![format!("%t{} = fsub double -0.0, {}", self.dest, s)],
            (UnaryOp::Not, _) => vec![format!("%t{} = xor i1 1, {}", self.dest, s)],
            (UnaryOp::BitNot, HirType::Int) => vec![format!("%t{} = xor i64 -1, {}", self.dest, s)],
            (UnaryOp::BitNot, HirType::Char) => vec![format!("%t{} = xor i8 -1, {}", self.dest, s)],
            (UnaryOp::BitNot, HirType::IntN { bits, .. }) => vec![format!("%t{} = xor i{} -1, {}", self.dest, bits, s)],
            _ => vec![],
        }
    }
    fn display(&self, f: &mut dyn Write) -> std::fmt::Result {
        writeln!(f, "    t{} = {:?} {:?} : {:?}", self.dest, self.op, self.src, self.ty)
    }
    fn serialize(&self, buf: &mut Vec<u8>) {
        buf.push(4);
        put_u64(buf, self.dest);
        put_u32(buf, self.op as u32);
        put_value(buf, &self.src);
        put_type(buf, &self.ty);
    }
}

impl LirNode for SLirCall {
    fn clone_node(&self) -> Box<dyn LirNode> { Box::new(self.clone()) }
    fn kind(&self) -> &'static str { "Call" }
    fn as_any(&self) -> &dyn std::any::Any { self }
    fn emit(&self, ctx: &mut LirEmitCtx) -> Vec<String> {
        let fn_name = &ctx.prog.fn_names[&self.fn_id];
        let mut arg_strs = Vec::new();
        for (val, aty) in &self.args {
            let llvm_ty = ctx.llvm_type(aty);
            let val_str = ctx.value_ref(val, aty);
            arg_strs.push(format!("{} {}", llvm_ty, val_str));
        }
        let is_void = matches!(&self.ret_ty, HirType::Void);
        let dest_str = match (&self.dest, is_void) {
            (Some(d), false) => format!("%t{} = ", d),
            _ => String::new(),
        };
        let ret_llvm = ctx.llvm_type(&self.ret_ty);
        vec![format!("{}call {} @{}({})", dest_str, ret_llvm, fn_name, arg_strs.join(", "))]
    }
    fn display(&self, f: &mut dyn Write) -> std::fmt::Result {
        let dest_str = self.dest.map(|d| format!("t{}", d)).unwrap_or("_".into());
        let args_str: Vec<String> = self.args.iter().map(|(v, t)| format!("{:?}:{:?}", v, t)).collect();
        writeln!(f, "    {} = call fn{} ({}) : {:?}", dest_str, self.fn_id.0, args_str.join(", "), self.ret_ty)
    }
    fn serialize(&self, buf: &mut Vec<u8>) {
        buf.push(5);
        put_u32(buf, self.dest.map_or(0xFFFFFFFF, |d| d as u32));
        put_u32(buf, self.fn_id.0 as u32);
        put_u32(buf, self.args.len() as u32);
        for (v, t) in &self.args { put_value(buf, v); put_type(buf, t); }
        put_type(buf, &self.ret_ty);
    }
}

impl LirNode for SLirCallPtr {
    fn clone_node(&self) -> Box<dyn LirNode> { Box::new(self.clone()) }
    fn kind(&self) -> &'static str { "CallPtr" }
    fn as_any(&self) -> &dyn std::any::Any { self }
    fn emit(&self, ctx: &mut LirEmitCtx) -> Vec<String> {
        let fn_src = ctx.value_ref(&self.fn_ptr, &HirType::Int);
        let ret_llvm = ctx.llvm_type(&self.ret_ty);
        let call_args: Vec<String> = self.args.iter().map(|(v, t)| format!("{} {}", ctx.llvm_type(t), ctx.value_ref(v, t))).collect();
        let is_void = matches!(&self.ret_ty, HirType::Void);
        if is_void {
            vec![format!("call void {} ({})", fn_src, call_args.join(", "))]
        } else {
            vec![format!("%t{} = call {} {} ({})", self.dest, ret_llvm, fn_src, call_args.join(", "))]
        }
    }
    fn display(&self, f: &mut dyn Write) -> std::fmt::Result {
        let args_str: Vec<String> = self.args.iter().map(|(v, t)| format!("{:?}:{:?}", v, t)).collect();
        writeln!(f, "    t{} = callptr {:?} ({}) : {:?}", self.dest, self.fn_ptr, args_str.join(", "), self.ret_ty)
    }
    fn serialize(&self, buf: &mut Vec<u8>) {
        buf.push(25);
        put_u64(buf, self.dest);
        put_value(buf, &self.fn_ptr);
        put_u32(buf, self.args.len() as u32);
        for (v, t) in &self.args { put_value(buf, v); put_type(buf, t); }
        put_type(buf, &self.ret_ty);
    }
}

impl LirNode for SLirFnAddr {
    fn clone_node(&self) -> Box<dyn LirNode> { Box::new(self.clone()) }
    fn kind(&self) -> &'static str { "FnAddr" }
    fn as_any(&self) -> &dyn std::any::Any { self }
    fn emit(&self, ctx: &mut LirEmitCtx) -> Vec<String> {
        let fn_name = &ctx.prog.fn_names[&self.fn_id];
        vec![format!("%t{} = getelementptr i8, ptr @{}, i32 0", self.dest, fn_name)]
    }
    fn display(&self, f: &mut dyn Write) -> std::fmt::Result {
        writeln!(f, "    t{} = fnaddr fn{}", self.dest, self.fn_id.0)
    }
    fn serialize(&self, buf: &mut Vec<u8>) {
        buf.push(26);
        put_u64(buf, self.dest);
        put_u32(buf, self.fn_id.0 as u32);
    }
}
