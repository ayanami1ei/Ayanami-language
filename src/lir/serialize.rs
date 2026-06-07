use crate::hir::ir::{FnId, HirLiteral, HirType, VarId};
use crate::intern::Symbol;
use crate::mir::ir::MirLocal;
use crate::parser::ast::BinaryOp;
use std::collections::HashMap;

use super::ir::{ConvKind, LirBlock, LirFn, LirInst, LirProgram, LirValue, VtableDesc};

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

    let imp_count = r.u32()?;
    let mut imported_fn_ids = std::collections::HashSet::new();
    for _ in 0..imp_count { imported_fn_ids.insert(FnId(r.u32()? as usize)); }

    Ok(LirProgram { strings, fn_names, functions, vtables, struct_defs, imported_fn_ids })
}

// ============================================================
//  Writer helpers
// ============================================================

fn put_u32(buf: &mut Vec<u8>, v: u32) { buf.extend_from_slice(&v.to_le_bytes()); }
fn put_u64(buf: &mut Vec<u8>, v: u64) { buf.extend_from_slice(&v.to_le_bytes()); }
fn put_str(buf: &mut Vec<u8>, s: &str) {
    let b = s.as_bytes();
    put_u32(buf, b.len() as u32);
    buf.extend_from_slice(b);
}

fn put_type(buf: &mut Vec<u8>, ty: &HirType) {
    match ty {
        HirType::Int => buf.push(0),
        HirType::Float => buf.push(1),
        HirType::Char => buf.push(2),
        HirType::Void => buf.push(3),
        HirType::Bool => buf.push(4),
        HirType::Named(s) => { buf.push(5); put_str(buf, &s.as_str()); }
        HirType::Unique(inner) => { buf.push(6); put_type(buf, inner); }
        HirType::Shared(inner) => { buf.push(7); put_type(buf, inner); }
        HirType::Weak(inner) => { buf.push(8); put_type(buf, inner); }
        HirType::FatPtr { name, kind } => {
            buf.push(9);
            put_str(buf, &name.as_str());
            put_type(buf, kind);
        }
        HirType::Array(inner) => { buf.push(10); put_type(buf, inner); }
        HirType::Ref(inner, mutable) => { buf.push(11); put_type(buf, inner); buf.push(if *mutable { 1 } else { 0 }); }
    }
}

fn put_literal(buf: &mut Vec<u8>, lit: &HirLiteral) {
    match lit {
        HirLiteral::Int(n) => { buf.push(0); put_u64(buf, *n as u64); }
        HirLiteral::Float(n) => { buf.push(1); buf.extend_from_slice(&n.to_le_bytes()); }
        HirLiteral::Char(c) => { buf.push(2); put_u32(buf, *c as u32); }
        HirLiteral::String(s) => { buf.push(3); put_str(buf, s); }
        HirLiteral::Bool(b) => { buf.push(4); buf.push(if *b { 1 } else { 0 }); }
    }
}

fn put_value(buf: &mut Vec<u8>, v: &LirValue) {
    match v {
        LirValue::Var(vid) => { buf.push(0); put_u32(buf, vid.0 as u32); }
        LirValue::Tmp(t) => { buf.push(1); put_u64(buf, *t); }
        LirValue::Param(i) => { buf.push(2); put_u64(buf, *i); }
        LirValue::Literal(lit, ty) => {
            buf.push(3);
            put_literal(buf, lit);
            put_type(buf, ty);
        }
    }
}

fn put_inst(buf: &mut Vec<u8>, inst: &LirInst) {
    use LirInst::*;
    match inst {
        Alloca(vid, ty) => { buf.push(0); put_u32(buf, vid.0 as u32); put_type(buf, ty); }
        Store { dest, src, ty } => { buf.push(1); put_u32(buf, dest.0 as u32); put_value(buf, src); put_type(buf, ty); }
        Load { dest, src, ty } => { buf.push(2); put_u64(buf, *dest); put_u32(buf, src.0 as u32); put_type(buf, ty); }
        BinOp { dest, op, lhs, rhs, ty } => { buf.push(3); put_u64(buf, *dest); put_u32(buf, *op as u32); put_value(buf, lhs); put_value(buf, rhs); put_type(buf, ty); }
        UnaryOp { dest, op, src, ty } => { buf.push(4); put_u64(buf, *dest); put_u32(buf, *op as u32); put_value(buf, src); put_type(buf, ty); }
        Call { dest, fn_id, args, ret_ty } => {
            buf.push(5);
            put_u32(buf, dest.map_or(0xFFFFFFFF, |d| d as u32));
            put_u32(buf, fn_id.0 as u32);
            put_u32(buf, args.len() as u32);
            for (v, t) in args { put_value(buf, v); put_type(buf, t); }
            put_type(buf, ret_ty);
        }
        StrGlobal { dest, str_idx } => { buf.push(6); put_u64(buf, *dest); put_u64(buf, *str_idx); }
        Conv { dest, alloca_tmp, malloc_tmp, src, kind, src_ty, ty } => {
            buf.push(7);
            put_u64(buf, *dest); put_u64(buf, *alloca_tmp); put_u64(buf, *malloc_tmp);
            put_value(buf, src); put_u32(buf, *kind as u32); put_type(buf, src_ty); put_type(buf, ty);
        }
        DropValue(vid, ty) => { buf.push(8); put_u32(buf, vid.0 as u32); put_type(buf, ty); }
        RetainValue(vid, ty) => { buf.push(9); put_u32(buf, vid.0 as u32); put_type(buf, ty); }
        ReleaseValue(vid, ty) => { buf.push(10); put_u32(buf, vid.0 as u32); put_type(buf, ty); }
        Br(label) => { buf.push(11); put_str(buf, label); }
        BrCond { cond, true_block, false_block } => { buf.push(12); put_value(buf, cond); put_str(buf, true_block); put_str(buf, false_block); }
        Ret(val) => {
            buf.push(13);
            match val {
                Some((v, t)) => { buf.push(1); put_value(buf, v); put_type(buf, t); }
                None => { buf.push(0); }
            }
        }
        MakeFatPtr { dest, malloc_tmp, bc_tmp, vtable_gep_tmp, iv_tmp, value_src, value_ty, vtable_name, ty } => {
            buf.push(14);
            put_u64(buf, *dest); put_u64(buf, *malloc_tmp); put_u64(buf, *bc_tmp);
            put_u64(buf, *vtable_gep_tmp); put_u64(buf, *iv_tmp);
            put_value(buf, value_src); put_type(buf, value_ty); put_str(buf, vtable_name); put_type(buf, ty);
        }
        VirtualCall { fn_dest, receiver_tmp, data_tmp, vtable_tmp, gep_tmp, fn_ptr_tmp, method_index, args, ret_ty } => {
            buf.push(15);
            put_u32(buf, fn_dest.map_or(0xFFFFFFFF, |d| d as u32));
            put_u64(buf, *receiver_tmp); put_u64(buf, *data_tmp); put_u64(buf, *vtable_tmp);
            put_u64(buf, *gep_tmp); put_u64(buf, *fn_ptr_tmp);
            put_u32(buf, *method_index as u32);
            put_u32(buf, args.len() as u32);
            for (v, t) in args { put_value(buf, v); put_type(buf, t); }
            put_type(buf, ret_ty);
        }
        FieldAccess { dest, gep_tmp, src, field_index, field_ty, struct_ty } => {
            buf.push(16);
            put_u64(buf, *dest); put_u64(buf, *gep_tmp);
            put_value(buf, src); put_u32(buf, *field_index as u32);
            put_type(buf, field_ty); put_type(buf, struct_ty);
        }
        FieldStore { dest, var_id, gep_tmp, iv_tmp, src, field_index, field_ty, struct_ty } => {
            buf.push(21);
            put_u64(buf, *dest);
            put_u32(buf, var_id.map_or(0xFFFFFFFF, |v| v.0 as u32));
            put_u64(buf, *gep_tmp); put_u64(buf, *iv_tmp);
            put_value(buf, src); put_u32(buf, *field_index as u32);
            put_type(buf, field_ty); put_type(buf, struct_ty);
        }
        StructLit { dest, alloca_tmp, field_geps, fields, struct_name, struct_ty } => {
            buf.push(17);
            put_u64(buf, *dest); put_u64(buf, *alloca_tmp);
            put_u32(buf, field_geps.len() as u32);
            for g in field_geps { put_u64(buf, *g); }
            put_u32(buf, fields.len() as u32);
            for (v, t) in fields { put_value(buf, v); put_type(buf, t); }
            put_str(buf, &struct_name.as_str()); put_type(buf, struct_ty);
        }
        ArrayLit { dest, malloc_tmp, elem_geps, elems, elem_ty, ty } => {
            buf.push(18);
            put_u64(buf, *dest); put_u64(buf, *malloc_tmp);
            put_u32(buf, elem_geps.len() as u32);
            for g in elem_geps { put_u64(buf, *g); }
            put_u32(buf, elems.len() as u32);
            for (v, t) in elems { put_value(buf, v); put_type(buf, t); }
            put_type(buf, elem_ty); put_type(buf, ty);
        }
        IndexAccess { dest, gep_tmp, load_tmp, arr, index, elem_ty, ty } => {
            buf.push(19);
            put_u64(buf, *dest); put_u64(buf, *gep_tmp); put_u64(buf, *load_tmp);
            put_value(buf, arr); put_value(buf, index);
            put_type(buf, elem_ty); put_type(buf, ty);
        }
        ArraySized { dest, malloc_tmp, count_tmp, size_tmp, elem_count, elem_size, elem_ty, ty } => {
            buf.push(20);
            put_u64(buf, *dest); put_u64(buf, *malloc_tmp);
            put_u64(buf, *count_tmp); put_u64(buf, *size_tmp);
            put_value(buf, elem_count); put_u64(buf, *elem_size);
            put_type(buf, elem_ty); put_type(buf, ty);
        }
        Asm { dest, template, output_constraints, input_operands, input_constraints, ret_ty } => {
            buf.push(24);
            put_u32(buf, dest.map_or(0xFFFFFFFF, |d| d as u32));
            put_str(buf, template);
            put_u32(buf, output_constraints.len() as u32);
            for c in output_constraints { put_str(buf, c); }
            put_u32(buf, input_operands.len() as u32);
            for (v, t) in input_operands { put_value(buf, v); put_type(buf, t); }
            put_u32(buf, input_constraints.len() as u32);
            for c in input_constraints { put_str(buf, c); }
            put_type(buf, ret_ty);
        }
        RefInst { dest, var_id, mutable, ty } => { buf.push(23); put_u64(buf, *dest); put_u32(buf, var_id.0 as u32); buf.push(if *mutable { 1 } else { 0 }); put_type(buf, ty); }
        IndexStore { dest, gep_tmp, src, index, elem_ty, array_ty } => {
            buf.push(22);
            put_u64(buf, *dest); put_u64(buf, *gep_tmp);
            put_value(buf, src); put_value(buf, index);
            put_type(buf, elem_ty); put_type(buf, array_ty);
        }
    }
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
    fn inst(&mut self) -> Result<LirInst, String> {
        use LirInst::*;
        let tag = self.read(1)?[0];
        match tag {
            0 => Ok(Alloca(VarId(self.u32()? as usize), self.ty()?)),
            1 => { let d = VarId(self.u32()? as usize); let s = self.value()?; let t = self.ty()?; Ok(Store { dest: d, src: s, ty: t }) }
            2 => { let d = self.u64()?; let s = VarId(self.u32()? as usize); let t = self.ty()?; Ok(Load { dest: d, src: s, ty: t }) }
            3 => { let d = self.u64()?; let o = self.u32()?; let l = self.value()?; let r = self.value()?; let t = self.ty()?;
                   let op = match o { 0 => BinaryOp::Add, 1 => BinaryOp::Sub, 2 => BinaryOp::Mul, 3 => BinaryOp::Div, 4 => BinaryOp::Mod, 5 => BinaryOp::Eq, 6 => BinaryOp::Neq, 7 => BinaryOp::Lt, 8 => BinaryOp::Gt, 9 => BinaryOp::Le, 10 => BinaryOp::Ge, 11 => BinaryOp::And, 12 => BinaryOp::Or, _ => return Err("unknown BinaryOp".into()) };
                   Ok(BinOp { dest: d, op, lhs: l, rhs: r, ty: t }) }
            4 => { let d = self.u64()?; let o = self.u32()?; let s = self.value()?; let t = self.ty()?;
                    let op = match o { 0 => crate::parser::ast::UnaryOp::Neg, 1 => crate::parser::ast::UnaryOp::Not, _ => return Err("unknown UnaryOp".into()) };
                   Ok(UnaryOp { dest: d, op, src: s, ty: t }) }
            5 => {
        let d_raw = self.u32()?;
        let dest = if d_raw == 0xFFFFFFFF { None } else { Some(d_raw as u64) };
        let fid = FnId(self.u32()? as usize);
                let ac = self.u32()?;
                let mut args = Vec::new();
                for _ in 0..ac { args.push((self.value()?, self.ty()?)); }
                let rt = self.ty()?;
                Ok(Call { dest, fn_id: fid, args, ret_ty: rt })
            }
            6 => Ok(StrGlobal { dest: self.u64()?, str_idx: self.u64()? }),
            7 => {
                let d = self.u64()?; let at = self.u64()?; let mt = self.u64()?;
                let s = self.value()?; let k = self.u32()?; let st = self.ty()?; let t = self.ty()?;
                let kind = match k { 0 => ConvKind::ToUnique, 1 => ConvKind::ToShared, 2 => ConvKind::ToWeak, _ => return Err("unknown ConvKind".into()) };
                Ok(Conv { dest: d, alloca_tmp: at, malloc_tmp: mt, src: s, kind, src_ty: st, ty: t })
            }
            8 => Ok(DropValue(VarId(self.u32()? as usize), self.ty()?)),
            9 => Ok(RetainValue(VarId(self.u32()? as usize), self.ty()?)),
            10 => Ok(ReleaseValue(VarId(self.u32()? as usize), self.ty()?)),
            11 => Ok(Br(self.str()?)),
            12 => { let c = self.value()?; Ok(BrCond { cond: c, true_block: self.str()?, false_block: self.str()? }) }
            13 => {
                if self.read(1)?[0] == 0 { Ok(Ret(None)) }
                else { let v = self.value()?; let t = self.ty()?; Ok(Ret(Some((v, t)))) }
            }
            14 => {
                let d = self.u64()?; let mt = self.u64()?; let bt = self.u64()?;
                let vgt = self.u64()?; let ivt = self.u64()?;
                let vs = self.value()?; let vt = self.ty()?; let vn = self.str()?; let t = self.ty()?;
                Ok(MakeFatPtr { dest: d, malloc_tmp: mt, bc_tmp: bt, vtable_gep_tmp: vgt, iv_tmp: ivt,
                    value_src: vs, value_ty: vt, vtable_name: vn, ty: t })
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
                Ok(VirtualCall { fn_dest, receiver_tmp: rt, data_tmp: dt, vtable_tmp: vt,
                    gep_tmp: gt, fn_ptr_tmp: fpt, method_index: mi, args, ret_ty: rt2 })
            }
            16 => {
                let d = self.u64()?;
                let s = self.value()?; let fi = self.u32()? as usize;
                let ft = self.ty()?; let st = self.ty()?;
                // Old FieldStore format (no var_id); discard this path
                let _ = d; let _ = s; let _ = fi; let _ = ft; let _ = st;
                return Err("deprecated FieldStore format".into());
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
                Ok(StructLit { dest: d, alloca_tmp: at, field_geps: fgs, fields, struct_name: sn, struct_ty: st })
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
                Ok(ArrayLit { dest: d, malloc_tmp: mt, elem_geps: egs, elems, elem_ty: et, ty: t })
            }
            19 => {
                let d = self.u64()?; let gt = self.u64()?; let lt = self.u64()?;
                let a = self.value()?; let i = self.value()?;
                let et = self.ty()?; let t = self.ty()?;
                Ok(IndexAccess { dest: d, gep_tmp: gt, load_tmp: lt, arr: a, index: i, elem_ty: et, ty: t })
            }
             20 => {
                let d = self.u64()?; let mt = self.u64()?;
                let ct = self.u64()?; let st = self.u64()?;
                let ec = self.value()?; let es = self.u64()?;
                let et = self.ty()?; let t = self.ty()?;
                Ok(ArraySized { dest: d, malloc_tmp: mt, count_tmp: ct, size_tmp: st, elem_count: ec, elem_size: es, elem_ty: et, ty: t })
            }
            21 => {
                let d = self.u64()?; let vr = self.u32()?;
                let var_id = if vr == 0xFFFFFFFF { None } else { Some(VarId(vr as usize)) };
                let gt = self.u64()?; let iv = self.u64()?;
                let s = self.value()?; let fi = self.u32()? as usize;
                let ft = self.ty()?; let st = self.ty()?;
                Ok(FieldStore { dest: d, var_id, gep_tmp: gt, iv_tmp: iv, src: s, field_index: fi, field_ty: ft, struct_ty: st })
            }
            22 => {
                let d = self.u64()?; let gt = self.u64()?;
                let s = self.value()?; let idx = self.value()?;
                let et = self.ty()?; let at = self.ty()?;
                Ok(IndexStore { dest: d, gep_tmp: gt, src: s, index: idx, elem_ty: et, array_ty: at })
            }
            23 => { let d = self.u64()?; let vr = VarId(self.u32()? as usize); let m = self.read(1)?[0] != 0; let t = self.ty()?; Ok(RefInst { dest: d, var_id: vr, mutable: m, ty: t }) }
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
                Ok(Asm { dest, template, output_constraints, input_operands, input_constraints, ret_ty })
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
        Ok(LirFn { fn_id: fid, is_inline, extern_c, name, params, return_type: ret, locals, blocks })
    }
}
