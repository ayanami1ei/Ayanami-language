use crate::error::{Error, Result};
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
pub fn program_from_bytes(data: &[u8]) -> Result<LirProgram> {
    if data.len() < 4 || &data[0..4] != b"LIR2" {
        return Err(Error::Serialize("invalid LIR data".into()));
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
//  Reader helpers
// ============================================================

struct Reader<'a> {
    data: &'a [u8],
    pos: &'a mut usize,
}

mod decode;
mod reader;
mod write;

use write::{put_fn, put_inst};
