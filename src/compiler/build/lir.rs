use super::*;

pub(super) fn lower_to_lir(program: &Program, src_path: &Path) -> Result<(
    crate::lir::ir::LirProgram,
    Vec<(crate::intern::Symbol, crate::hir::ir::HirType, crate::hir::ir::HirLiteral)>,
    Vec<crate::hir::HirStatic>,
)> {
    // A5b-2：先做用户宏展开（import 已重写为 .lcl 绝对路径）
    let program = crate::compiler::macro_expand::expand(program, src_path)?;
    crate::hir::lower::set_source_path(src_path);
    let mut hir_program = crate::hir::lower_program(&program)
        .map_err(|e| Error::Compile(format!("{}: {}", src_path.display(), e)))?;
    check_hir_returns(&hir_program, src_path)?;
    crate::hir::effects::analyze(&mut hir_program, &program, src_path)?;
    let mut mir_program = crate::mir::lower_program(&hir_program)?;
    // A5d-2：应用 MIR 优化注解（#[pass]）
    super::passes::apply_passes(&mut mir_program, &hir_program, &program, src_path)?;
    let follow_table = crate::mir::borrow::build_follow_table(&mir_program);
    for item in &mir_program.items {
        if let crate::mir::ir::MirItem::Fn(f) = item {
            crate::mir::borrow::check_borrows(f, &follow_table)
                .map_err(|e| Error::Compile(format!("{}: {}", src_path.display(), e)))?;
        }
    }
    let consts = hir_program.consts.clone();
    // 导出列表只含本模块定义的 static（导入的外部符号不再导出）
    let statics: Vec<crate::hir::HirStatic> = hir_program.statics.iter()
        .filter(|s| !s.is_external)
        .cloned()
        .collect();
    Ok((crate::lir::lower_program(&mir_program), consts, statics))
}

pub(super) fn build_target_artifact(
    target_override: Option<&str>,
    dep_obj_paths: &[PathBuf],
    dep_link_flags: &[String],
    own_obj: &Path,
    out_dir: &Path,
    stem: &std::ffi::OsStr,
    runtime: Option<&Path>,
) -> Result<()> {
    if dep_obj_paths.is_empty() && dep_link_flags.is_empty() {
        let target = target_override.unwrap_or("static-lib");
        let stem_str = stem.to_string_lossy();
        match target {
            "executable" => {
                let exe_path = out_dir.join(&*stem_str);
                crate::driver::objects_to_exe_with_runtime(
                    &[own_obj.to_path_buf()],
                    &[],
                    &exe_path,
                    runtime,
                )?;
            }
            "static-lib" => {
                let lib_path = out_dir.join(format!("lib{}.a", stem_str));
                crate::driver::object_to_static_lib(own_obj, &lib_path)?;
            }
            "dynamic-lib" => {
                let so_path = out_dir.join(format!("lib{}.so", stem_str));
                crate::driver::object_to_shared_lib(own_obj, &so_path)?;
            }
            _ => {}
        }
    }
    Ok(())
}
