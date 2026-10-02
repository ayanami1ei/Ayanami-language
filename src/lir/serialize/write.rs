use super::*;

// ============================================================
//  Writer helpers
// ============================================================

pub(super) fn put_inst(buf: &mut Vec<u8>, inst: &LirNodeBox) {
    inst.serialize(buf);
}

pub(super) fn put_fn(buf: &mut Vec<u8>, f: &LirFn) {
    put_u32(buf, f.fn_id.0 as u32);
    buf.push(if f.is_inline { 1 } else { 0 });
    buf.push(if f.extern_c { 1 } else { 0 });
    put_u32(buf, f.attrs.len() as u32);
    for a in &f.attrs {
        put_str(buf, &a.name);
        put_u32(buf, a.args.len() as u32);
        for arg in &a.args { put_str(buf, arg); }
    }
    put_u32(buf, f.param_attrs.len() as u32);
    for pv in &f.param_attrs {
        put_u32(buf, pv.len() as u32);
        for a in pv {
            put_str(buf, &a.name);
            put_u32(buf, a.args.len() as u32);
            for arg in &a.args { put_str(buf, arg); }
        }
    }
    buf.push(if f.effects.no_throws { 1 } else { 0 });
    buf.push(if f.effects.no_effects { 1 } else { 0 });
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
