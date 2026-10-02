use super::*;

pub(super) fn resolve_dependencies(
    _src_path: &Path,
    base_dir: &Path,
    out_dir: &Path,
    compiling: &mut HashSet<PathBuf>,
    cache: &mut HashMap<PathBuf, CompiledFile>,
    program: &Program,
) -> Result<(Vec<PathBuf>, Vec<String>, Vec<PathBuf>, Vec<Stmt>)> {
    let mut dep_obj_paths = Vec::new();
    let mut dep_link_flags = Vec::new();
    let mut dep_lcl_paths = Vec::new();
    let mut new_stmts = Vec::new();

    for stmt in &program.stmts {
        if let Stmt::Import { path, .. } = stmt {
            let resolved = resolve_import_path(path, base_dir);
            let dep_path = match resolved {
                Some(p) => p,
                None => {
                    let direct = base_dir.join(path);
                    if direct.exists() {
                        direct
                    } else {
                        return Err(Error::Compile(format!(
                            "cannot resolve import '{}' from {}: not found",
                            path,
                            base_dir.display()
                        )));
                    }
                }
            };
            let dep_target = load_config_for_file(&dep_path, out_dir);
            if dep_path.to_string_lossy().ends_with(".aya") {
                let dep_target_or_static = if dep_target.as_deref() == Some("dynamic-lib") {
                    "dynamic-lib"
                } else {
                    "static-lib"
                };
                let dep = compile_file(
                    &dep_path,
                    base_dir,
                    out_dir,
                    compiling,
                    cache,
                    Some(dep_target_or_static),
                )?;
                if dep_target_or_static == "dynamic-lib" {
                    let dep_stem = dep_path.file_stem().unwrap_or_default().to_string_lossy();
                    let canon = out_dir.canonicalize().unwrap_or_else(|_| out_dir.to_path_buf());
                    dep_link_flags.push(format!("-L{}", canon.display()));
                    dep_link_flags.push(format!("-l{}", dep_stem));
                    dep_link_flags.push(format!("-Wl,-rpath,{}", canon.display()));
                } else {
                    for p in &dep.obj_paths {
                        if !dep_obj_paths.contains(p) {
                            dep_obj_paths.push(p.clone());
                        }
                    }
                }
                dep_link_flags.extend(dep.link_flags.clone());
                let lcl_name = dep.lcl_path.to_string_lossy().into_owned();
                if !dep_lcl_paths.contains(&dep.lcl_path) {
                    dep_lcl_paths.push(dep.lcl_path.clone());
                }
                new_stmts.push(Stmt::Import {
                    path: lcl_name,
                    span: crate::span::Span::default(),
                });
            } else {
                let lcl_str = dep_path.to_string_lossy().into_owned();
                new_stmts.push(Stmt::Import {
                    path: lcl_str,
                    span: crate::span::Span::default(),
                });

                // 编译该 .lcl 及其同目录下所有 .lcl 文件为 .o
                // （因为 std.lcl 依赖 string.lcl、io.lcl 等，它们的函数必须参与链接）
                let lcl_dir = dep_path.parent().unwrap_or(std::path::Path::new("."));
                if let Ok(entries) = std::fs::read_dir(lcl_dir) {
                    for entry in entries.flatten() {
                        let path = entry.path();
                        if path.extension().map(|e| e == "lcl") != Some(true) { continue; }
                        let o_name = path.file_stem().unwrap_or(std::ffi::OsStr::new("a"));
                        let o_path = out_dir.join(o_name).with_extension("o");
                        // 缓存失效：.o 不存在，或 .lcl 比 .o 新
                        let stale = match (path.metadata().and_then(|m| m.modified()), o_path.metadata().and_then(|m| m.modified())) {
                            (Ok(lcl_t), Ok(o_t)) => lcl_t > o_t,
                            _ => true,
                        };
                        if stale {
                            if let Ok((_, _, lir_binary, _)) =
                                crate::package::load_package(&path.to_string_lossy())
                            {
                                if let Ok(lir_prog) =
                                    crate::lir::serialize::program_from_bytes(&lir_binary)
                                {
                                    let llvm_ir = crate::lir::emit_program(&lir_prog);
                                    crate::driver::ir_to_object(&llvm_ir, &o_path)
                                        .map_err(|e| Error::Compile(format!("llc failed for {}: {}", path.display(), e)))?;
                                }
                            }
                        }
                        if o_path.exists() && !dep_obj_paths.contains(&o_path) {
                            dep_obj_paths.push(o_path);
                        }
                    }
                }
                if !dep_lcl_paths.contains(&dep_path) {
                    dep_lcl_paths.push(dep_path.clone());
                }
            }
        } else {
            new_stmts.push(stmt.clone());
        }
    }

    Ok((dep_obj_paths, dep_link_flags, dep_lcl_paths, new_stmts))
}

pub(super) fn merge_dep_struct_defs(
    lir_program: &mut crate::lir::ir::LirProgram,
    dep_lcl_paths: &[PathBuf],
) {
    for lcl_path in dep_lcl_paths {
        if let Ok((_, _, lir_binary, _)) =
            crate::package::load_package(&lcl_path.to_string_lossy())
        {
            if let Ok(dep_lir) = crate::lir::serialize::program_from_bytes(&lir_binary) {
                for (name, fields) in dep_lir.struct_defs {
                    lir_program.struct_defs.entry(name).or_insert(fields);
                }
            }
        }
    }
}

pub(super) fn merge_symbols(
    pkg: &mut crate::package::Package,
    dep_lcl_paths: &[PathBuf],
) {
    use crate::package::PackageSymbol;
    for dep_lcl in dep_lcl_paths {
        if let Ok((syms, sources, _, _)) = crate::package::load_package(&dep_lcl.to_string_lossy())
        {
            for sym in syms {
                let pkg_sym = match sym {
                    crate::package::ImportedSymbol::Fn { name, sig } => {
                        PackageSymbol::Fn {
                            name,
                            signature: sig,
                        }
                    }
                    crate::package::ImportedSymbol::Struct { name } => {
                        PackageSymbol::Struct { name }
                    }
                    crate::package::ImportedSymbol::Namespace { name } => {
                        PackageSymbol::Namespace { name }
                    }
                    crate::package::ImportedSymbol::Interface { name } => {
                        PackageSymbol::Interface { name }
                    }
                };
                if !pkg.symbols.contains(&pkg_sym) {
                    pkg.symbols.push(pkg_sym);
                }
            }
            for src in &sources {
                if !pkg.generic_sources.contains(src) {
                    pkg.generic_sources.push(src.clone());
                }
            }
        }
    }
}
