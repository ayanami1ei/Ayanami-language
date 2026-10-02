use super::*;

impl<'a> Reader<'a> {
    pub(super) fn literal(&mut self) -> Result<HirLiteral> {
        let tag = self.read(1)?[0];
        match tag {
            0 => Ok(HirLiteral::Int(self.u64()? as i64)),
            1 => { let b = self.read(8)?; Ok(HirLiteral::Float(f64::from_le_bytes([b[0], b[1], b[2], b[3], b[4], b[5], b[6], b[7]]))) }
            2 => Ok(HirLiteral::Char(char::from_u32(self.u32()?).unwrap_or('\0'))),
            3 => Ok(HirLiteral::String(self.str()?)),
            4 => Ok(HirLiteral::Bool(self.read(1)?[0] != 0)),
            _ => Err(Error::Serialize(format!("unknown literal tag: {}", tag))),
        }
    }
    pub(super) fn value(&mut self) -> Result<LirValue> {
        let tag = self.read(1)?[0];
        match tag {
            0 => Ok(LirValue::Var(VarId(self.u32()? as usize))),
            1 => Ok(LirValue::Tmp(self.u64()?)),
            2 => Ok(LirValue::Param(self.u64()?)),
            3 => { let l = self.literal()?; let t = self.ty()?; Ok(LirValue::Literal(l, t)) }
            _ => Err(Error::Serialize(format!("unknown value tag: {}", tag))),
        }
    }
    pub(super) fn inst(&mut self) -> Result<LirNodeBox> {
        let tag = self.read(1)?[0];
        match tag {
            0 => Ok(SLirAlloca { var: VarId(self.u32()? as usize), ty: self.ty()? }.into()),
            1 => {
                let d = VarId(self.u32()? as usize);
                let s = self.value()?;
                let t = self.ty()?;
                Ok(SLirStore { dest: d, src: s, ty: t }.into())
            }
            2 => {
                let d = self.u64()?;
                let s = VarId(self.u32()? as usize);
                let t = self.ty()?;
                Ok(SLirLoad { dest: d, src: s, ty: t }.into())
            }
            3 => {
                let d = self.u64()?; let o = self.u32()?; let l = self.value()?; let r = self.value()?; let t = self.ty()?; let rt = self.ty()?;
                let op = match o { 0 => BinaryOp::Add, 1 => BinaryOp::Sub, 2 => BinaryOp::Mul, 3 => BinaryOp::Div, 4 => BinaryOp::Mod, 5 => BinaryOp::Eq, 6 => BinaryOp::Neq, 7 => BinaryOp::Lt, 8 => BinaryOp::Gt, 9 => BinaryOp::Le, 10 => BinaryOp::Ge, 11 => BinaryOp::And, 12 => BinaryOp::Or, _ => return Err(Error::Serialize("unknown BinaryOp".into())) };
                Ok(SLirBinOp { dest: d, op, lhs: l, rhs: r, ty: t, result_ty: rt }.into())
            }
            4 => {
                let d = self.u64()?; let o = self.u32()?; let s = self.value()?; let t = self.ty()?;
                let op = match o { 0 => crate::parser::ast::UnaryOp::Neg, 1 => crate::parser::ast::UnaryOp::Not, _ => return Err(Error::Serialize("unknown UnaryOp".into())) };
                Ok(SLirUnaryOp { dest: d, op, src: s, ty: t }.into())
            }
            5 => {
                let d_raw = self.u32()?;
                let dest = if d_raw == 0xFFFFFFFF { None } else { Some(d_raw as u64) };
                let fid = FnId(self.u32()? as usize);
                let ac = self.u32()?;
                let mut args = Vec::new();
                for _ in 0..ac { args.push((self.value()?, self.ty()?)); }
                let rt = self.ty()?;
                Ok(SLirCall { dest, fn_id: fid, args, ret_ty: rt }.into())
            }
            6 => Ok(SLirStrGlobal { dest: self.u64()?, str_idx: self.u64()? }.into()),
            7 => {
                let d = self.u64()?; let at = self.u64()?; let mt = self.u64()?;
                let s = self.value()?; let k = self.u32()?; let st = self.ty()?; let t = self.ty()?;
                let kind = match k { 0 => ConvKind::ToUnique, 1 => ConvKind::ToShared, 2 => ConvKind::ToWeak, _ => return Err(Error::Serialize("unknown ConvKind".into())) };
                Ok(SLirConv { dest: d, alloca_tmp: at, malloc_tmp: mt, src: s, kind, src_ty: st, ty: t }.into())
            }
            8 => Ok(SLirDropValue { var: VarId(self.u32()? as usize), ty: self.ty()? }.into()),
            9 => Ok(SLirRetainValue { var: VarId(self.u32()? as usize), ty: self.ty()? }.into()),
            10 => Ok(SLirReleaseValue { var: VarId(self.u32()? as usize), ty: self.ty()? }.into()),
            11 => Ok(SLirBr { label: self.str()? }.into()),
            12 => { let c = self.value()?; Ok(SLirBrCond { cond: c, true_block: self.str()?, false_block: self.str()? }.into()) }
            13 => {
                if self.read(1)?[0] == 0 { Ok(SLirRet { val: None }.into()) }
                else { let v = self.value()?; let t = self.ty()?; Ok(SLirRet { val: Some((v, t)) }.into()) }
            }
            14 => {
                let d = self.u64()?; let mt = self.u64()?; let bt = self.u64()?;
                let vgt = self.u64()?; let ivt = self.u64()?;
                let vs = self.value()?; let vt = self.ty()?; let vn = self.str()?; let t = self.ty()?;
                Ok(SLirMakeFatPtr { dest: d, malloc_tmp: mt, bc_tmp: bt, vtable_gep_tmp: vgt, iv_tmp: ivt,
                    value_src: vs, value_ty: vt, vtable_name: vn, ty: t }.into())
            }
            15 => {
                let fd_raw = self.u32()?;
                let fn_dest = if fd_raw == 0xFFFFFFFF { None } else { Some(fd_raw as u64) };
                let rt = self.u64()?; let dt = self.u64()?; let vt = self.u64()?;
                let gt = self.u64()?; let fpt = self.u64()?; let mi = self.u32()? as usize;
                let ac = self.u32()?;
                let mut args = Vec::new();
                for _ in 0..ac { args.push((self.value()?, self.ty()?)); }
                let rt2 = self.ty()?;
                Ok(SLirVirtualCall { fn_dest, receiver_tmp: rt, data_tmp: dt, vtable_tmp: vt,
                    gep_tmp: gt, fn_ptr_tmp: fpt, method_index: mi, args, ret_ty: rt2 }.into())
            }
            16 => {
                let d = self.u64()?;
                let gt = self.u64()?;
                let s = self.value()?; let fi = self.u32()? as usize;
                let ft = self.ty()?; let st = self.ty()?;
                Ok(SLirFieldAccess { dest: d, gep_tmp: gt, src: s, field_index: fi, field_ty: ft, struct_ty: st }.into())
            }
            17 => {
                let d = self.u64()?; let at = self.u64()?;
                let fgc = self.u32()?;
                let mut fgs = Vec::new();
                for _ in 0..fgc { fgs.push(self.u64()?); }
                let fc = self.u32()?;
                let mut fields = Vec::new();
                for _ in 0..fc { fields.push((self.value()?, self.ty()?)); }
                let sn = Symbol::intern(&self.str()?); let st = self.ty()?;
                Ok(SLirStructLit { dest: d, alloca_tmp: at, field_geps: fgs, fields, struct_name: sn, struct_ty: st }.into())
            }
            18 => {
                let d = self.u64()?; let mt = self.u64()?;
                let egc = self.u32()?;
                let mut egs = Vec::new();
                for _ in 0..egc { egs.push(self.u64()?); }
                let ec = self.u32()?;
                let mut elems = Vec::new();
                for _ in 0..ec { elems.push((self.value()?, self.ty()?)); }
                let et = self.ty()?; let t = self.ty()?;
                Ok(SLirArrayLit { dest: d, malloc_tmp: mt, elem_geps: egs, elems, elem_ty: et, ty: t }.into())
            }
            19 => {
                let d = self.u64()?; let gt = self.u64()?; let lt = self.u64()?;
                let a = self.value()?; let i = self.value()?;
                let et = self.ty()?; let t = self.ty()?;
                Ok(SLirIndexAccess { dest: d, gep_tmp: gt, load_tmp: lt, arr: a, index: i, elem_ty: et, ty: t }.into())
            }
            20 => {
                let d = self.u64()?; let mt = self.u64()?;
                let ct = self.u64()?; let st = self.u64()?;
                let ec = self.value()?; let es = self.u64()?;
                let et = self.ty()?; let t = self.ty()?;
                Ok(SLirArraySized { dest: d, malloc_tmp: mt, count_tmp: ct, size_tmp: st, elem_count: ec, elem_size: es, elem_ty: et, ty: t }.into())
            }
            21 => {
                let d = self.u64()?; let vr = self.u32()?;
                let var_id = if vr == 0xFFFFFFFF { None } else { Some(VarId(vr as usize)) };
                let gt = self.u64()?; let iv = self.u64()?;
                let s = self.value()?; let fi = self.u32()? as usize;
                let ft = self.ty()?; let st = self.ty()?;
                Ok(SLirFieldStore { dest: d, var_id, gep_tmp: gt, iv_tmp: iv, src: s, field_index: fi, field_ty: ft, struct_ty: st }.into())
            }
            22 => {
                let d = self.u64()?; let gt = self.u64()?;
                let s = self.value()?; let idx = self.value()?;
                let et = self.ty()?; let at = self.ty()?;
                Ok(SLirIndexStore { dest: d, gep_tmp: gt, src: s, index: idx, elem_ty: et, array_ty: at }.into())
            }
            23 => { let d = self.u64()?; let vr = VarId(self.u32()? as usize); let m = self.read(1)?[0] != 0; let t = self.ty()?; Ok(SLirRefInst { dest: d, var_id: vr, mutable: m, ty: t }.into()) }
            24 => {
                let d_raw = self.u32()?;
                let dest = if d_raw == 0xFFFFFFFF { None } else { Some(d_raw as u64) };
                let template = self.str()?;
                let oc_count = self.u32()?;
                let mut output_constraints = Vec::new();
                for _ in 0..oc_count { output_constraints.push(self.str()?); }
                let io_count = self.u32()?;
                let mut input_operands = Vec::new();
                for _ in 0..io_count { input_operands.push((self.value()?, self.ty()?)); }
                let ic_count = self.u32()?;
                let mut input_constraints = Vec::new();
                for _ in 0..ic_count { input_constraints.push(self.str()?); }
                let ret_ty = self.ty()?;
                Ok(SLirAsm { dest, template, output_constraints, input_operands, input_constraints, ret_ty }.into())
            }
            25 => {
                let d = self.u64()?;
                let fp = self.value()?;
                let ac = self.u32()?;
                let mut args = Vec::new();
                for _ in 0..ac { args.push((self.value()?, self.ty()?)); }
                let rt = self.ty()?;
                Ok(SLirCallPtr { dest: d, fn_ptr: fp, args, ret_ty: rt }.into())
            }
            26 => {
                let d = self.u64()?;
                let fid = FnId(self.u32()? as usize);
                Ok(SLirFnAddr { dest: d, fn_id: fid }.into())
            }
            _ => Err(Error::Serialize(format!("unknown inst tag: {}", tag))),
        }
    }
    pub(super) fn read_fn(&mut self) -> Result<LirFn> {
        let fid = FnId(self.u32()? as usize);
        let is_inline = self.read(1)?[0] != 0;
        let extern_c = self.read(1)?[0] != 0;
        let name = Symbol::intern(&self.str()?);
        let ret = self.ty()?;
        let pc = self.u32()?;
        let mut params = Vec::new();
        for _ in 0..pc { params.push((Symbol::intern(&self.str()?), self.ty()?)); }
        let lc = self.u32()?;
        let mut locals = Vec::new();
        for _ in 0..lc {
            let n = Symbol::intern(&self.str()?);
            let t = self.ty()?;
            let m = self.read(1)?[0] != 0;
            locals.push(MirLocal { name: n, ty: t, mutable: m });
        }
        let bc = self.u32()?;
        let mut blocks = Vec::new();
        for _ in 0..bc {
            let label = self.str()?;
            let ic = self.u32()?;
            let mut insts = Vec::new();
            for _ in 0..ic { insts.push(self.inst()?); }
            blocks.push(LirBlock { label, insts });
        }
        Ok(LirFn { fn_id: fid, is_inline, extern_c, name, params, return_type: ret, locals, blocks, custom: Vec::new() })
    }
}
