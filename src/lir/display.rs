use std::fmt::Write;

use super::ir::*;

pub fn lir_program_to_string(prog: &LirProgram) -> String {
    let mut s = String::new();
    writeln!(s, "LIR Program").unwrap();
    writeln!(s, "Strings: {} entries", prog.strings.len()).unwrap();
    for (i, str) in prog.strings.iter().enumerate() {
        writeln!(s, "  @__str_{} = \"{}\"", i, str).unwrap();
    }
    writeln!(s, "").unwrap();
    for func in &prog.functions {
        write_fn(func, &mut s).unwrap();
    }
    s
}

fn write_fn(f: &LirFn, w: &mut impl Write) -> std::fmt::Result {
    writeln!(w, "fn {} (fn_id={}) -> {:?} [{} blocks]",
        f.name.as_str(), f.fn_id.0, f.return_type, f.blocks.len())?;
    for local in &f.locals {
        writeln!(w, "  local {}: {:?}", local.name.as_str(), local.ty)?;
    }
    for block in &f.blocks {
        writeln!(w, "{}:", block.label)?;
        for inst in &block.insts {
            write_inst(inst, w)?;
        }
    }
    Ok(())
}

fn write_inst(inst: &LirInst, w: &mut impl Write) -> std::fmt::Result {
    match inst {
        LirInst::Alloca(vid, ty) => {
            writeln!(w, "    alloca v{} : {:?}", vid.0, ty)?;
        }
        LirInst::Store { dest, src, ty } => {
            writeln!(w, "    store {:?} -> v{} : {:?}", src, dest.0, ty)?;
        }
        LirInst::Load { dest, src, ty } => {
            writeln!(w, "    t{} = load v{} : {:?}", dest, src.0, ty)?;
        }
        LirInst::BinOp { dest, op, lhs, rhs, ty } => {
            writeln!(w, "    t{} = {:?} {:?} {:?} : {:?}", dest, op, lhs, rhs, ty)?;
        }
        LirInst::UnaryOp { dest, op, src, ty } => {
            writeln!(w, "    t{} = {:?} {:?} : {:?}", dest, op, src, ty)?;
        }
        LirInst::Call { dest, fn_id, args, ret_ty } => {
            let dest_str = dest.map(|d| format!("t{}", d)).unwrap_or("_".into());
            let args_str: Vec<String> = args.iter().map(|(v, t)| format!("{:?}:{:?}", v, t)).collect();
            writeln!(w, "    {} = call fn{} ({}) : {:?}", dest_str, fn_id.0, args_str.join(", "), ret_ty)?;
        }
        LirInst::StrGlobal { dest, str_idx } => {
            writeln!(w, "    t{} = str_global @__str_{}", dest, str_idx)?;
        }
        LirInst::Conv { dest, kind, src, ty, .. } => {
            writeln!(w, "    t{} = conv {:?} -> {:?} : {:?}", dest, kind, src, ty)?;
        }
        LirInst::DropValue(vid, ty) => {
            writeln!(w, "    drop v{} : {:?}", vid.0, ty)?;
        }
        LirInst::RetainValue(vid, ty) => {
            writeln!(w, "    retain v{} : {:?}", vid.0, ty)?;
        }
        LirInst::ReleaseValue(vid, ty) => {
            writeln!(w, "    release v{} : {:?}", vid.0, ty)?;
        }
        LirInst::Br(label) => {
            writeln!(w, "    br %{}", label)?;
        }
        LirInst::BrCond { cond, true_block, false_block } => {
            writeln!(w, "    br_cond {:?} %{} %{}", cond, true_block, false_block)?;
        }
        LirInst::Ret(val) => {
            match val {
                Some((v, _)) => writeln!(w, "    ret {:?}", v)?,
                None => writeln!(w, "    ret void")?,
            }
        }
        LirInst::MakeFatPtr { dest, vtable_name, .. } => {
            writeln!(w, "    t{} = make_fatptr vtable={}", dest, vtable_name)?;
        }
        LirInst::FieldAccess { dest, field_index, field_ty, .. } => {
            writeln!(w, "    t{} = field_access field={} : {:?}", dest, field_index, field_ty)?;
        }
        LirInst::StructLit { dest, struct_name, fields, .. } => {
            writeln!(w, "    t{} = struct_lit {} ({} fields)", dest, struct_name, fields.len())?;
        }
        LirInst::VirtualCall { fn_dest, receiver_tmp, method_index, .. } => {
            let d = fn_dest.map(|d| format!("t{}", d)).unwrap_or("_".into());
            writeln!(w, "    {} = virtual_call [receiver=t{}, slot={}]", d, receiver_tmp, 1 + method_index)?;
        }
        LirInst::ArrayLit { dest, elems, elem_ty, .. } => {
            writeln!(w, "    t{} = array_lit ({} elems, elem_ty={:?})", dest, elems.len(), elem_ty)?;
        }
        LirInst::ArraySized { dest, elem_count, elem_ty, .. } => {
            writeln!(w, "    t{} = array_sized ({} elems, elem_ty={:?})", dest, elem_count, elem_ty)?;
        }
        LirInst::IndexAccess { dest, elem_ty, .. } => {
            writeln!(w, "    t{} = index_access elem_ty={:?}", dest, elem_ty)?;
        }
        LirInst::FieldStore { dest, field_index, field_ty, .. } => {
            writeln!(w, "    t{} = field_store field={} : {:?}", dest, field_index, field_ty)?;
        }
        LirInst::IndexStore { dest, elem_ty, .. } => {
            writeln!(w, "    t{} = index_store elem_ty={:?}", dest, elem_ty)?;
        }
    }
    Ok(())
}
