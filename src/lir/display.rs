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

fn write_inst(inst: &LirNodeBox, w: &mut impl Write) -> std::fmt::Result {
    inst.display(w)
}
