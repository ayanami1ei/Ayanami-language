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
    // Header（LIR4：M-opt.2 起函数带 is_pub；LIR3：#129 带 specialized_fns 段）
    buf.extend_from_slice(b"LIR4");

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
        for (gp_name, constraints) in params {
            put_str(&mut buf, &gp_name.as_str());
            put_u32(&mut buf, constraints.len() as u32);
            for c in constraints {
                put_str(&mut buf, &c.as_str());
            }
        }
    }

    // extern_decls（真实签名 + 标注）
    put_u32(&mut buf, p.extern_decls.len() as u32);
    for d in &p.extern_decls {
        put_str(&mut buf, &d.name);
        put_u32(&mut buf, d.params.len() as u32);
        for t in &d.params { put_type(&mut buf, t); }
        put_type(&mut buf, &d.return_type);
        put_u32(&mut buf, d.attrs.len() as u32);
        for a in &d.attrs {
            put_str(&mut buf, &a.name);
            put_u32(&mut buf, a.args.len() as u32);
            for arg in &a.args { put_str(&mut buf, arg); }
        }
        put_u32(&mut buf, d.param_attrs.len() as u32);
        for pv in &d.param_attrs {
            put_u32(&mut buf, pv.len() as u32);
            for a in pv {
                put_str(&mut buf, &a.name);
                put_u32(&mut buf, a.args.len() as u32);
                for arg in &a.args { put_str(&mut buf, arg); }
            }
        }
        buf.push(if d.effects.no_throws { 1 } else { 0 });
        buf.push(if d.effects.no_effects { 1 } else { 0 });
    }

    // #129：泛型特化集合（.lcl 重新发射时保持 linkonce_odr 弱链接）；排序保证可复现
    let mut spec_ids: Vec<u32> = p.specialized_fns.iter().map(|id| id.0 as u32).collect();
    spec_ids.sort_unstable();
    put_u32(&mut buf, spec_ids.len() as u32);
    for id in spec_ids { put_u32(&mut buf, id); }

    buf
}

/// Deserialize LirProgram from binary.
pub fn program_from_bytes(data: &[u8]) -> Result<LirProgram> {
    // LIR4：带 is_pub 字节（M-opt.2）；LIR3：带 specialized_fns 段；LIR2：旧格式
    if data.len() < 4 || (&data[0..4] != b"LIR4" && &data[0..4] != b"LIR3" && &data[0..4] != b"LIR2") {
        return Err(Error::Serialize("invalid LIR data".into()));
    }
    let has_spec = &data[0..4] != b"LIR2";
    let has_pub = &data[0..4] == b"LIR4";
    let mut pos = 4;
    let mut r = Reader { data, pos: &mut pos, has_pub };

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
            let c_count = r.u32()?;
            let mut constraints = Vec::new();
            for _ in 0..c_count {
                constraints.push(Symbol::intern(&r.str()?));
            }
            params.push((gp_name, constraints));
        }
        generic_struct_params.insert(name, params);
    }

    let ed_count = r.u32()?;
    let mut extern_decls = Vec::new();
    for _ in 0..ed_count {
        let name = r.str()?;
        let pc = r.u32()?;
        let mut params = Vec::new();
        for _ in 0..pc { params.push(r.ty()?); }
        let return_type = r.ty()?;
        let ac = r.u32()?;
        let mut attrs = Vec::new();
        for _ in 0..ac {
            let an = r.str()?;
            let argc = r.u32()?;
            let mut args = Vec::new();
            for _ in 0..argc { args.push(r.str()?); }
            attrs.push(LirAttr { name: an, args });
        }
        let pac = r.u32()?;
        let mut param_attrs = Vec::new();
        for _ in 0..pac {
            let pvc = r.u32()?;
            let mut pv = Vec::new();
            for _ in 0..pvc {
                let an = r.str()?;
                let argc = r.u32()?;
                let mut args = Vec::new();
                for _ in 0..argc { args.push(r.str()?); }
                pv.push(LirAttr { name: an, args });
            }
            param_attrs.push(pv);
        }
        let no_throws = r.read(1)?[0] != 0;
        let no_effects = r.read(1)?[0] != 0;
        extern_decls.push(ExternDecl {
            name, params, return_type, attrs, param_attrs,
            effects: LirEffects { no_throws, no_effects },
            extern_c: false, // 序列化来自包导入（Ayanami 函数）；源码 extern C 声明不进 .lcl
        });
    }

    // #129：特化集合（保持弱链接）；旧 LIR2 无此段
    let mut specialized_fns = std::collections::HashSet::new();
    if has_spec {
        let spec_count = r.u32()?;
        for _ in 0..spec_count {
            specialized_fns.insert(FnId(r.u32()? as usize));
        }
    }

    Ok(LirProgram {
        specialized_fns,
        strings,
        globals: Vec::new(),
        extern_globals: Vec::new(),
        fn_names, functions, vtables, struct_defs, generic_struct_params, extern_decls,
        effect_summaries: std::collections::HashMap::new(),
    })
}


// ============================================================
//  Reader helpers
// ============================================================

struct Reader<'a> {
    data: &'a [u8],
    pos: &'a mut usize,
    /// M-opt.2：LIR4 起每个函数带 is_pub 字节
    has_pub: bool,
}

mod decode;
mod reader;
mod write;

use write::put_fn;
