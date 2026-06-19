use crate::hir::ir::{FnId, HirLiteral, HirType, VarId};
use crate::intern::Symbol;
use crate::mir::ir::MirLocal;
use crate::parser::ast::BinaryOp;
use std::collections::HashMap;

use super::ir::*;

/// Serialize LirProgram to compact binary.
pub fn program_to_bytes(p: &LirProgram) -> Vec<u8> {
    let mut buf = Vec::new();
    // Header
    buf.extend_from_slice(b"LIR2");

    // Strings
    put_u32(&mut buf, p.strings.len() as u32);
    for s in &p.strings { put_str(&mut buf, s); }

    // fn_names
    put_u32(&mut buf, p.fn_names.len() as u32);
    for (k, v) in &p.fn_names {
        put_u32(&mut buf, k.0 as u32);
        put_str(&mut buf, v);
    }

    // Functions
    put_u32(&mut buf, p.functions.len() as u32);
    for f in &p.functions { put_fn(&mut buf, f); }

    // Vtables
    put_u32(&mut buf, p.vtables.len() as u32);
    for v in &p.vtables {
        put_str(&mut buf, &v.name);
        put_u32(&mut buf, v.fn_ids.len() as u32);
        for id in &v.fn_ids { put_u32(&mut buf, id.0 as u32); }
    }

    // struct_defs
    put_u32(&mut buf, p.struct_defs.len() as u32);
    for (name, fields) in &p.struct_defs {
        put_str(&mut buf, &name.as_str());
        put_u32(&mut buf, fields.len() as u32);
        for (fn_name, ty) in fields {
            put_str(&mut buf, &fn_name.as_str());
            put_type(&mut buf, ty);
        }
    }

    // generic_struct_params
    put_u32(&mut buf, p.generic_struct_params.len() as u32);
    for (name, params) in &p.generic_struct_params {
        put_str(&mut buf, &name.as_str());
        put_u32(&mut buf, params.len() as u32);
        for (gp_name, constraint) in params {
            put_str(&mut buf, &gp_name.as_str());
            put_u32(&mut buf, constraint.map(|_| 1u32).unwrap_or(0));
            if let Some(c) = constraint {
                put_str(&mut buf, &c.as_str());
            }
        }
    }

    // imported_fn_ids
    put_u32(&mut buf, p.imported_fn_ids.len() as u32);
    for id in &p.imported_fn_ids { put_u32(&mut buf, id.0 as u32); }

    buf
}

/// Deserialize LirProgram from binary.
pub fn program_from_bytes(data: &[u8]) -> Result<LirProgram, String> {
    if data.len() < 4 || &data[0..4] != b"LIR2" {
        return Err("invalid LIR data".into());
    }
    let mut pos = 4;
    let mut r = Reader { data, pos: &mut pos };

    let str_count = r.u32()?;
    let mut strings = Vec::new();
    for _ in 0..str_count { strings.push(r.str()?); }

    let fn_count = r.u32()?;
    let mut fn_names = HashMap::new();
    for _ in 0..fn_count {
        let k = FnId(r.u32()? as usize);
        let v = r.str()?;
        fn_names.insert(k, v);
    }

    let func_count = r.u32()?;
    let mut functions = Vec::new();
    for _ in 0..func_count { functions.push(r.read_fn()?); }

    let vt_count = r.u32()?;
    let mut vtables = Vec::new();
    for _ in 0..vt_count {
        let name = r.str()?;
        let id_count = r.u32()?;
        let mut fn_ids = Vec::new();
        for _ in 0..id_count { fn_ids.push(FnId(r.u32()? as usize)); }
        vtables.push(VtableDesc { name, fn_ids });
    }

    let sd_count = r.u32()?;
    let mut struct_defs = HashMap::new();
    for _ in 0..sd_count {
        let name = Symbol::intern(&r.str()?);
        let f_count = r.u32()?;
        let mut fields = Vec::new();
        for _ in 0..f_count {
            let fn_name = Symbol::intern(&r.str()?);
            let ty = r.ty()?;
            fields.push((fn_name, ty));
        }
        struct_defs.insert(name, fields);
    }

    let gsp_count = r.u32()?;
    let mut generic_struct_params = std::collections::HashMap::new();
    for _ in 0..gsp_count {
        let name = Symbol::intern(&r.str()?);
        let p_count = r.u32()?;
        let mut params = Vec::new();
        for _ in 0..p_count {
            let gp_name = Symbol::intern(&r.str()?);
            let has_constraint = r.u32()?;
            let constraint = if has_constraint != 0 { Some(Symbol::intern(&r.str()?)) } else { None };
            params.push((gp_name, constraint));
        }
        generic_struct_params.insert(name, params);
    }

    let imp_count = r.u32()?;
    let mut imported_fn_ids = std::collections::HashSet::new();
    for _ in 0..imp_count { imported_fn_ids.insert(FnId(r.u32()? as usize)); }

    Ok(LirProgram { strings, fn_names, functions, vtables, struct_defs, generic_struct_params, imported_fn_ids })
}

// ============================================================
//  Writer helpers
// ============================================================

fn put_inst(buf: &mut Vec<u8>, inst: &LirNodeBox) {
    inst.serialize(buf);
}

fn put_fn(buf: &mut Vec<u8>, f: &LirFn) {
    put_u32(buf, f.fn_id.0 as u32);
    buf.push(if f.is_inline { 1 } else { 0 });
    buf.push(if f.extern_c { 1 } else { 0 });
    put_str(buf, &f.name.as_str());
    put_type(buf, &f.return_type);
    put_u32(buf, f.params.len() as u32);
    for (n, t) in &f.params {
        put_str(buf, &n.as_str());
        put_type(buf, t);
    }
    put_u32(buf, f.locals.len() as u32);
    for l in &f.locals {
        put_str(buf, &l.name.as_str());
        put_type(buf, &l.ty);
        buf.push(if l.mutable { 1 } else { 0 });
    }
    put_u32(buf, f.blocks.len() as u32);
    for b in &f.blocks {
        put_str(buf, &b.label);
        put_u32(buf, b.insts.len() as u32);
        for inst in &b.insts { put_inst(buf, inst); }
    }
}

// ============================================================
//  Reader helpers
// ============================================================

struct Reader<'a> {
    data: &'a [u8],
    pos: &'a mut usize,
}

impl<'a> Reader<'a> {
    fn read(&mut self, n: usize) -> Result<&'a [u8], String> {
        if *self.pos + n > self.data.len() {
            return Err("unexpected EOF".into());
        }
        let slice = &self.data[*self.pos..*self.pos + n];
        *self.pos += n;
        Ok(slice)
    }
    fn u32(&mut self) -> Result<u32, String> {
        let b = self.read(4)?;
        Ok(u32::from_le_bytes([b[0], b[1], b[2], b[3]]))
    }
    fn u64(&mut self) -> Result<u64, String> {
        let b = self.read(8)?;
        Ok(u64::from_le_bytes([b[0], b[1], b[2], b[3], b[4], b[5], b[6], b[7]]))
    }
    fn str(&mut self) -> Result<String, String> {
        let len = self.u32()? as usize;
        let b = self.read(len)?;
        Ok(String::from_utf8(b.to_vec()).map_err(|e| format!("invalid string: {}", e))?)
    }
    fn ty(&mut self) -> Result<HirType, String> {
        let tag = self.read(1)?[0];
        match tag {
            0 => Ok(HirType::Int),
            1 => Ok(HirType::Float),
            2 => Ok(HirType::Char),
            3 => Ok(HirType::Void),
            4 => Ok(HirType::Bool),
            5 => { let s = Symbol::intern(&self.str()?); Ok(HirType::Named(s)) }
            6 => Ok(HirType::Unique(Box::new(self.ty()?))),
            7 => Ok(HirType::Shared(Box::new(self.ty()?))),
            8 => Ok(HirType::Weak(Box::new(self.ty()?))),
            9 => {
                let name = Symbol::intern(&self.str()?);
                let kind = Box::new(self.ty()?);
                Ok(HirType::FatPtr { name, kind })
            }
            10 => Ok(HirType::Array(Box::new(self.ty()?))),
            11 => { let inner = Box::new(self.ty()?); let mutable = self.read(1)?[0] != 0; Ok(HirType::Ref(inner, mutable)) }
            12 => {
                let pc = self.u32()? as usize;
                let mut params = Vec::with_capacity(pc);
                for _ in 0..pc { params.push(self.ty()?); }
                let ret = Box::new(self.ty()?);
                Ok(HirType::FnPtr(params, ret))
            }
            _ => Err(format!("unknown type tag: {}", tag)),
        }
    }
    fn literal(&mut self) -> Result<HirLiteral, String> {
        let tag = self.read(1)?[0];
        match tag {
            0 => Ok(HirLiteral::Int(self.u64()? as i64)),
            1 => { let b = self.read(8)?; Ok(HirLiteral::Float(f64::from_le_bytes([b[0], b[1], b[2], b[3], b[4], b[5], b[6], b[7]]))) }
            2 => Ok(HirLiteral::Char(char::from_u32(self.u32()?).unwrap_or('\0'))),
            3 => Ok(HirLiteral::String(self.str()?)),
            4 => Ok(HirLiteral::Bool(self.read(1)?[0] != 0)),
            _ => Err(format!("unknown literal tag: {}", tag)),
        }
    }
    fn value(&mut self) -> Result<LirValue, String> {
        let tag = self.read(1)?[0];
        match tag {
            0 => Ok(LirValue::Var(VarId(self.u32()? as usize))),
            1 => Ok(LirValue::Tmp(self.u64()?)),
            2 => Ok(LirValue::Param(self.u64()?)),
            3 => { let l = self.literal()?; let t = self.ty()?; Ok(LirValue::Literal(l, t)) }
            _ => Err(format!("unknown value tag: {}", tag)),
        }
    }
    fn inst(&mut self) -> Result<LirNodeBox, String> {
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
                let op = match o { 0 => BinaryOp::Add, 1 => BinaryOp::Sub, 2 => BinaryOp::Mul, 3 => BinaryOp::Div, 4 => BinaryOp::Mod, 5 => BinaryOp::Eq, 6 => BinaryOp::Neq, 7 => BinaryOp::Lt, 8 => BinaryOp::Gt, 9 => BinaryOp::Le, 10 => BinaryOp::Ge, 11 => BinaryOp::And, 12 => BinaryOp::Or, _ => return Err("unknown BinaryOp".into()) };
                Ok(SLirBinOp { dest: d, op, lhs: l, rhs: r, ty: t, result_ty: rt }.into())
            }
            4 => {
                let d = self.u64()?; let o = self.u32()?; let s = self.value()?; let t = self.ty()?;
                let op = match o { 0 => crate::parser::ast::UnaryOp::Neg, 1 => crate::parser::ast::UnaryOp::Not, _ => return Err("unknown UnaryOp".into()) };
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
                let kind = match k { 0 => ConvKind::ToUnique, 1 => ConvKind::ToShared, 2 => ConvKind::ToWeak, _ => return Err("unknown ConvKind".into()) };
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
            _ => Err(format!("unknown inst tag: {}", tag)),
        }
    }
    fn read_fn(&mut self) -> Result<LirFn, String> {
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
