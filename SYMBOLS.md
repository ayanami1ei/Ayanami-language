# SYMBOLS.md — Ayanami 符号地图

> 本文件由 scripts/gen_symbols.sh 自动生成，请勿手改。重新生成：./scripts/gen_symbols.sh
> 查询：rg "关键词" SYMBOLS.md

## Rust
src/cli/build.rs:3: pub(crate) fn cmd_build(args: &[String])
src/cli/build.rs:24: pub(crate) fn cmd_run(args: &[String])
src/cli/check.rs:3: fn do_check(path: &Path) -> ayanami::error::Result<()>
src/cli/check.rs:14: pub(crate) fn cmd_check(args: &[String])
src/cli/check.rs:33: fn watch_file(path: &Path)
src/cli/clean.rs:3: pub(crate) fn cmd_clean()
src/cli/defs.rs:3: pub(crate) fn cmd_defs(args: &[String])
src/cli/defs.rs:43: fn resolve_import_defs(stmts: &[ayanami::parser::ast::Stmt], base_path: &str, visited: &mut std::collections::HashSet<std::path::PathBuf>, defs: &mut Vec<ayanami::compiler::SymDef>)
src/cli/fmt.rs:3: pub(crate) fn cmd_fmt(args: &[String])
src/cli/mod.rs:7: pub(crate) mod build;
src/cli/mod.rs:8: pub(crate) mod check;
src/cli/mod.rs:9: pub(crate) mod clean;
src/cli/mod.rs:10: pub(crate) mod defs;
src/cli/mod.rs:11: pub(crate) mod fmt;
src/cli/mod.rs:12: pub(crate) mod new;
src/cli/mod.rs:13: pub(crate) mod package_install;
src/cli/mod.rs:24: pub(crate) struct CliFlags
src/cli/mod.rs:30: pub(crate) fn split_flags(args: &[String]) -> (Vec<String>, CliFlags)
src/cli/mod.rs:44: pub(crate) fn apply_mode(flags: &CliFlags)
src/cli/new.rs:3: pub(crate) fn cmd_new(args: &[String])
src/cli/package_install.rs:3: pub(crate) fn cmd_package(args: &[String])
src/cli/package_install.rs:18: pub(crate) fn cmd_install(args: &[String])
src/compiler/build/compile.rs:6: pub fn compile_file(
src/compiler/build/compile.rs:128: pub(super) fn parse_and_check(code: &str, src_path: &Path) -> Result<Program>
src/compiler/build/deps.rs:3: pub(super) fn resolve_dependencies(
src/compiler/build/deps.rs:124: pub(super) fn merge_dep_struct_defs(
src/compiler/build/deps.rs:141: pub(super) fn merge_symbols(
src/compiler/build/lir.rs:3: pub(super) fn lower_to_lir(program: &Program, src_path: &Path) -> Result<crate::lir::ir::LirProgram>
src/compiler/build/lir.rs:23: pub(super) fn build_target_artifact(
src/compiler/build/mod.rs:14: mod compile;
src/compiler/build/mod.rs:15: mod deps;
src/compiler/build/mod.rs:16: mod lir;
src/compiler/build/mod.rs:17: mod package;
src/compiler/build/mod.rs:18: mod target;
src/compiler/build/mod.rs:25: pub fn build_source(src_path: &str, code: &str) -> Result<()>
src/compiler/build/mod.rs:30: pub fn build_source_to(src_path: &str, _code: &str, out_dir: &str) -> Result<()>
src/compiler/build/package.rs:4: pub(super) fn emit_lcl_package(
src/compiler/build/package.rs:21: pub fn package_source(src_path: &str, _code: &str) -> Result<()>
src/compiler/build/package.rs:72: pub fn install_package(lcl_path: &str, target_type: Option<&str>) -> Result<()>
src/compiler/build/target.rs:4: pub fn build_source_with_target(
src/compiler/build/target.rs:105: pub fn run_executable(exe_name: &str) -> Result<i32>
src/compiler/check.rs:5: pub fn check_hir_returns(hir: &HirProgram, src_path: &Path) -> Result<()>
src/compiler/check.rs:36: fn has_return_in_item(item: &HirItem) -> bool
src/compiler/check.rs:75: fn has_return_in_stmt(s: &HirStmt) -> bool
src/compiler/debug/expr.rs:5: pub(super) fn write_expr(expr: &Expr, level: usize, w: &mut impl Write)
src/compiler/debug/format.rs:3: pub(super) fn pad(n: usize) -> String
src/compiler/debug/format.rs:7: pub(super) fn format_type(ty: &Type) -> String
src/compiler/debug/format.rs:37: pub(super) fn format_op(op: &BinaryOp) -> &str
src/compiler/debug/format.rs:55: pub(super) fn format_unary(op: &UnaryOp) -> &str
src/compiler/debug/format.rs:62: pub(super) fn format_literal(lit: &Literal) -> String
src/compiler/debug/mod.rs:5: mod expr;
src/compiler/debug/mod.rs:6: mod format;
src/compiler/debug/mod.rs:7: mod stmt;
src/compiler/debug/mod.rs:13: pub fn format_program(program: &Program) -> String
src/compiler/debug/stmt.rs:5: fn write_block(block: &Block, level: usize, w: &mut impl Write)
src/compiler/debug/stmt.rs:13: pub(super) fn write_stmt(stmt: &Stmt, level: usize, w: &mut impl Write)
src/compiler/import.rs:2: pub fn find_std_dir() -> Option<std::path::PathBuf>
src/compiler/import.rs:20: pub fn resolve_import_path(
src/compiler/import.rs:84: pub fn load_config_for_file(file_path: &std::path::Path, _out_dir: &std::path::Path) -> Option<String>
src/compiler/macro_expand/mod.rs:15: const MAX_DEPTH: usize = 32;
src/compiler/macro_expand/mod.rs:17: mod plugin;
src/compiler/macro_expand/mod.rs:20: pub fn expand(program: &Program, src_path: &Path) -> Result<Program>
src/compiler/macro_expand/mod.rs:29: struct MacroCtx
src/compiler/macro_expand/mod.rs:37: impl MacroCtx
src/compiler/macro_expand/mod.rs:38: fn collect(program: &Program, src_path: &Path) -> Result<Self>
src/compiler/macro_expand/mod.rs:45: fn resolve(&self, a: &Attr) -> Option<(String, String)>
src/compiler/macro_expand/mod.rs:60: fn collect_imports(
src/compiler/macro_expand/mod.rs:92: fn expand_stmts(stmts: &[Stmt], ctx: &MacroCtx, depth: usize) -> Result<Vec<Stmt>>
src/compiler/macro_expand/mod.rs:135: fn expand_one(stmt: &Stmt, ctx: &MacroCtx, depth: usize) -> Result<Vec<Stmt>>
src/compiler/macro_expand/mod.rs:175: fn parse_source(code: &str) -> Result<Program>
src/compiler/macro_expand/mod.rs:185: fn is_compiler_attr(a: &Attr) -> bool
src/compiler/macro_expand/mod.rs:196: fn attrs_of(stmt: &Stmt) -> Option<&Vec<Attr>>
src/compiler/macro_expand/mod.rs:207: fn attrs_of_mut(stmt: &mut Stmt) -> Option<&mut Vec<Attr>>
src/compiler/macro_expand/plugin.rs:10: mod dl
src/compiler/macro_expand/plugin.rs:14: pub fn dlopen(filename: *const c_char, flag: c_int) -> *mut c_void;
src/compiler/macro_expand/plugin.rs:15: pub fn dlsym(handle: *mut c_void, symbol: *const c_char) -> *mut c_void;
src/compiler/macro_expand/plugin.rs:16: pub fn dlclose(handle: *mut c_void) -> c_int;
src/compiler/macro_expand/plugin.rs:19: const RTLD_NOW: c_int = 2;
src/compiler/macro_expand/plugin.rs:25: pub(super) fn invoke_plugin(lcl_path: &str, macro_name: &str, input: &str) -> Result<String>
src/compiler/macro_expand/plugin.rs:50: fn build_plugin(lir: &crate::lir::ir::LirProgram, symbol: &str, arity: usize) -> Result<PathBuf>
src/compiler/macro_expand/plugin.rs:124: type ExpandFn = unsafe extern "C" fn(*const u8, usize, *mut usize) -> *mut u8;
src/compiler/macro_expand/plugin.rs:125: type FreeFn = unsafe extern "C" fn(*mut c_char);
src/compiler/macro_expand/plugin.rs:127: fn call_plugin(so_path: &Path, symbol: &str, _arity: usize, input: &str) -> Result<String>
src/compiler/mod.rs:1: pub mod build;
src/compiler/mod.rs:2: pub mod check;
src/compiler/mod.rs:3: pub mod debug;
src/compiler/mod.rs:4: pub mod import;
src/compiler/mod.rs:5: pub mod macro_expand;
src/compiler/mod.rs:6: pub mod symdef;
src/compiler/mod.rs:24: pub struct CompiledFile
src/compiler/mod.rs:36: pub struct CompileResult
src/compiler/mod.rs:64: pub struct CompilerPipeline
src/compiler/mod.rs:81: impl CompilerPipeline
src/compiler/mod.rs:83: pub fn new(code: &str) -> Self
src/compiler/mod.rs:96: pub fn lex(&mut self) -> Result<&mut Self>
src/compiler/mod.rs:109: pub fn parse(&mut self) -> Result<&mut Self>
src/compiler/mod.rs:116: pub fn lower_hir(&mut self) -> Result<&mut Self>
src/compiler/mod.rs:127: pub fn check_returns(&mut self) -> Result<&mut Self>
src/compiler/mod.rs:138: pub fn lower_mir(&mut self) -> Result<&mut Self>
src/compiler/mod.rs:147: pub fn check_borrows(&mut self) -> Result<&mut Self>
src/compiler/mod.rs:163: pub fn lower_lir(&mut self) -> &mut Self
src/compiler/mod.rs:172: pub fn emit(&mut self) -> &mut Self
src/compiler/mod.rs:180: pub fn compile(&mut self) -> Result<&mut Self>
src/compiler/mod.rs:191: pub fn result(&self) -> CompileResult
src/compiler/mod.rs:200: pub fn hir_program(&self) -> Option<&crate::hir::ir::HirProgram>
src/compiler/mod.rs:205: pub fn mir_program(&self) -> Option<&crate::mir::ir::MirProgram>
src/compiler/mod.rs:210: pub fn ast_program(&self) -> Option<&Program>
src/compiler/mod.rs:216: pub fn compile_source(code: &str) -> Result<CompileResult>
src/compiler/mod.rs:224: pub fn check_source(code: &str, _out_dir: &str) -> Result<()>
src/compiler/symdef.rs:5: pub struct SymDef
src/compiler/symdef.rs:18: fn stmt_effect_tokens(stmt: &Stmt) -> Vec<String>
src/compiler/symdef.rs:29: fn stmt_attr_names(stmt: &Stmt) -> Vec<String>
src/compiler/symdef.rs:42: pub fn collect_defs_from_stmts(
src/compiler/symdef.rs:53: fn collect_defs_from_stmt(stmt: &Stmt, file: &str, prefix: &str, defs: &mut Vec<SymDef>)
src/compiler/symdef.rs:177: pub fn defs_to_json(defs: &[SymDef]) -> String
src/diagnostics.rs:10: pub fn supports_color() -> bool
src/diagnostics.rs:14: pub fn error(msg: &str)
src/diagnostics.rs:22: pub fn warning(msg: &str)
src/diagnostics.rs:31: pub fn success(msg: &str)
src/diagnostics.rs:40: pub fn warning_at(file: &Path, line: usize, col: usize, msg: &str)
src/driver/mod.rs:11: pub(crate) fn find_llc() -> Result<(PathBuf, PathBuf)>
src/driver/mod.rs:29: fn find_opt() -> Option<(PathBuf, PathBuf)>
src/driver/mod.rs:46: pub fn ir_to_object(llvm_ir: &str, obj_path: impl AsRef<Path>) -> Result<()>
src/driver/mod.rs:97: pub fn objects_to_exe(obj_paths: &[PathBuf], exe_path: impl AsRef<Path>) -> Result<()>
src/driver/mod.rs:102: pub fn objects_to_exe_with_flags(obj_paths: &[PathBuf], extra_flags: &[String], exe_path: impl AsRef<Path>) -> Result<()>
src/driver/mod.rs:115: pub fn object_to_exe(obj_path: impl AsRef<Path>, exe_path: impl AsRef<Path>) -> Result<()>
src/driver/mod.rs:121: pub(crate) fn find_runtime_c() -> Result<String>
src/driver/mod.rs:149: pub fn object_to_static_lib(obj_path: impl AsRef<Path>, lib_path: impl AsRef<Path>) -> Result<()>
src/driver/mod.rs:164: pub fn objects_to_shared_lib(obj_paths: &[PathBuf], lib_path: impl AsRef<Path>) -> Result<()>
src/driver/mod.rs:175: pub fn object_to_shared_lib(obj_path: impl AsRef<Path>, lib_path: impl AsRef<Path>) -> Result<()>
src/driver/mod.rs:192: pub fn ir_to_library(llvm_ir: &str, lib_path: impl AsRef<Path>, lib_type: &str) -> Result<()>
src/driver/mod.rs:212: pub fn ir_to_object_keep(llvm_ir: &str, obj_path: impl AsRef<Path>) -> Result<()>
src/driver/mod.rs:217: pub fn ir_to_executable(llvm_ir: &str, exe_path: impl AsRef<Path>) -> Result<()>
src/error.rs:10: pub enum Error
src/error.rs:61: pub type Result<T> = std::result::Result<T, Error>;
src/formatter/expr.rs:4: pub(super) fn write_type(ty: &Type) -> String
src/formatter/expr.rs:38: pub(super) fn write_expr(expr: &Expr) -> String
src/formatter/expr.rs:136: pub(super) fn write_literal(lit: &Literal) -> String
src/formatter/expr.rs:156: pub(super) fn write_bin_op(op: &BinaryOp) -> &str
src/formatter/expr.rs:174: pub(super) fn vis_str(vis: &Visibility) -> &str
src/formatter/helpers.rs:4: pub(super) fn write_stmt_separator(out: &mut String, stmt: &Stmt)
src/formatter/helpers.rs:14: pub(super) fn indent(level: usize) -> String
src/formatter/helpers.rs:18: pub(super) fn write_block_same_line(out: &mut String, block: &Block, level: usize)
src/formatter/helpers.rs:32: pub(super) fn write_attrs(out: &mut String, attrs: &[Attr], level: usize)
src/formatter/helpers.rs:44: fn write_attr_arg(arg: &crate::parser::ast::AttrArg) -> String
src/formatter/helpers.rs:52: pub(super) fn write_generic_params(out: &mut String, params: &[(Symbol, Option<Symbol>)])
src/formatter/helpers.rs:65: pub(super) fn write_params(out: &mut String, params: &[(Symbol, Type)], param_attrs: &[Vec<crate::parser::ast::Attr>])
src/formatter/helpers.rs:94: pub(super) fn write_return_type(out: &mut String, ty: &Type)
src/formatter/mod.rs:7: const INDENT: &str = "    ";
src/formatter/mod.rs:9: pub fn format_program(program: &Program) -> String
src/formatter/mod.rs:21: mod expr;
src/formatter/mod.rs:22: mod helpers;
src/formatter/mod.rs:23: mod stmt;
src/formatter/mod.rs:26: pub(crate) fn format_expr(expr: &Expr) -> String
src/formatter/mod.rs:33: pub fn format_file(code: &str) -> crate::error::Result<String>
src/formatter/stmt.rs:5: pub(super) fn write_stmt(out: &mut String, stmt: &Stmt, level: usize)
src/hir/attrs.rs:9: pub const ALLOWED: &[&str] = &[
src/hir/attrs.rs:31: pub const PARAM_ALLOWED: &[&str] = &["noalias", "nonnull"];
src/hir/attrs.rs:35: pub struct Imports
src/hir/attrs.rs:42: impl Imports
src/hir/attrs.rs:43: pub fn collect(program: &Program) -> Self
src/hir/attrs.rs:51: fn collect_stmt(&mut self, stmt: &Stmt)
src/hir/attrs.rs:74: pub fn pkg_stem(path: &str) -> String
src/hir/attrs.rs:84: pub fn has(attrs: &[Attr], name: &str) -> bool
src/hir/attrs.rs:89: pub fn validate(attrs: &[Attr], imports: &Imports) -> Result<()>
src/hir/attrs.rs:97: fn resolve(a: &Attr, imports: &Imports) -> Result<()>
src/hir/attrs.rs:133: fn validate_follow_with(a: &Attr) -> Result<()>
src/hir/attrs.rs:153: fn validate_follow_with_attrs(attrs: &[Attr]) -> Result<()>
src/hir/attrs.rs:160: fn reject_follow_with(attrs: &[Attr], place: &str) -> Result<()>
src/hir/attrs.rs:172: pub fn validate_program(program: &Program) -> Result<()>
src/hir/attrs.rs:180: fn validate_param(attrs: &[Attr], ty: &crate::parser::ast::Type) -> Result<()>
src/hir/attrs.rs:203: fn validate_stmt(stmt: &Stmt, imports: &Imports) -> Result<()>
src/hir/attrs_macro.rs:12: fn is_compiler_attr(a: &Attr) -> bool
src/hir/attrs_macro.rs:24: pub fn validate_macros(
src/hir/attrs_macro.rs:37: fn validate_macros_stmt(
src/hir/cfg.rs:14: pub fn filter_program(program: &Program) -> Result<Program>
src/hir/cfg.rs:19: pub fn filter_stmts(stmts: &[Stmt]) -> Result<Vec<Stmt>>
src/hir/cfg.rs:29: fn filter_one(stmt: &Stmt) -> Result<Option<Stmt>>
src/hir/cfg.rs:68: pub fn stmt_enabled(stmt: &Stmt) -> bool
src/hir/cfg.rs:72: fn attrs_of(stmt: &Stmt) -> Option<&[Attr]>
src/hir/cfg.rs:84: fn cfg_enabled(attrs: Option<&[Attr]>) -> Result<bool>
src/hir/cfg.rs:98: fn eval_predicate(arg: &AttrArg, span: crate::span::Span) -> Result<bool>
src/hir/cfg.rs:121: fn eval_bare(e: &Expr, span: crate::span::Span) -> Result<bool>
src/hir/cfg.rs:138: fn unsupported(span: crate::span::Span) -> Error
src/hir/contracts.rs:7: pub fn validate_fn_attrs(attrs: &[Attr]) -> Result<()>
src/hir/contracts.rs:21: pub fn ensure_bool_condition(cond: &crate::hir::HirNodeBox, kind: &str, line: usize, col: usize) -> Result<()>
src/hir/contracts.rs:31: static RELEASE_MODE: std::sync::atomic::AtomicBool = std::sync::atomic::AtomicBool::new(false);
src/hir/contracts.rs:34: pub fn set_release(v: bool)
src/hir/contracts.rs:39: pub fn is_release() -> bool
src/hir/contracts.rs:45: pub fn checks_enabled() -> bool
src/hir/contracts.rs:53: pub fn requires_conditions(attrs: &[Attr]) -> Vec<(&Expr, usize, usize)>
src/hir/contracts.rs:64: pub fn validate_invariant_attrs(attrs: &[Attr]) -> Result<()>
src/hir/contracts.rs:79: pub fn invariant_conditions(attrs: &[Attr]) -> Vec<(&Expr, usize, usize)>
src/hir/contracts.rs:90: pub fn ensure_conditions(attrs: &[Attr]) -> Vec<(&Expr, usize, usize)>
src/hir/contracts.rs:101: pub fn assume_conditions(attrs: &[Attr]) -> Vec<&Expr>
src/hir/display.rs:5: pub fn display_hir_program(program: &HirProgram)
src/hir/display.rs:9: pub fn hir_program_to_string(program: &HirProgram) -> String
src/hir/display.rs:18: pub(crate) fn pad(n: usize) -> String
src/hir/display.rs:22: pub(crate) fn display_type(ty: &HirType) -> String
src/hir/display.rs:44: fn write_item(item: &HirItem, level: usize, w: &mut impl Write) -> std::fmt::Result
src/hir/display.rs:89: fn write_block(block: &HirBlock, level: usize, w: &mut impl Write) -> std::fmt::Result
src/hir/display.rs:99: pub(crate) fn write_expr(expr: &HirNodeBox, level: usize, w: &mut impl Write) -> std::fmt::Result
src/hir/display.rs:103: fn write_stmt(stmt: &HirStmt, level: usize, w: &mut impl Write) -> std::fmt::Result
src/hir/effects/diag.rs:9: pub(super) fn diagnostics(hir: &HirProgram, ast: &crate::parser::ast::Program, src_path: &Path) -> Result<()>
src/hir/effects/infer.rs:17: pub struct EffectSet
src/hir/effects/infer.rs:28: impl EffectSet
src/hir/effects/infer.rs:29: fn union(&mut self, o: &EffectSet)
src/hir/effects/infer.rs:38: static VERIFY_EFFECTS: std::sync::atomic::AtomicBool = std::sync::atomic::AtomicBool::new(false);
src/hir/effects/infer.rs:40: pub fn set_verify_effects(v: bool)
src/hir/effects/infer.rs:44: pub fn verify_effects() -> bool
src/hir/effects/infer.rs:49: pub fn analyze(hir: &mut HirProgram, ast: &crate::parser::ast::Program, src_path: &Path) -> Result<()>
src/hir/effects/infer.rs:62: pub fn summarize(hir: &HirProgram) -> HashMap<String, crate::hir::effects::EffectSummary>
src/hir/effects/infer.rs:77: fn compute(hir: &HirProgram) -> HashMap<crate::hir::ty::FnId, EffectSet>
src/hir/effects/infer.rs:170: fn declared_to_set(d: &EffectDecl) -> EffectSet
src/hir/effects/infer.rs:178: pub(super) fn collect_fns<'a>(items: &'a [HirItem], out: &mut Vec<&'a HirFn>)
src/hir/effects/infer.rs:188: fn for_each_fn_mut(items: &mut [HirItem], f: &mut impl FnMut(&mut HirFn))
src/hir/effects/infer.rs:199: struct CallInfo
src/hir/effects/infer.rs:206: fn walk_stmts(stmts: &[HirStmt], calls: &mut Vec<CallInfo>)
src/hir/effects/infer.rs:252: fn state_marker() -> CallInfo
src/hir/effects/infer.rs:256: fn is_observable_target(object: &crate::hir::HirNodeBox) -> bool
src/hir/effects/infer.rs:260: fn walk_expr(e: &dyn HirNode, calls: &mut Vec<CallInfo>)
src/hir/effects/mod.rs:15: pub const BUILTIN_EFFECTS: &[&str] = &["io", "state", "alloc"];
src/hir/effects/mod.rs:18: pub(crate) const IO_NAMES: &[&str] = &[
src/hir/effects/mod.rs:24: pub mod diag;
src/hir/effects/mod.rs:25: pub mod infer;
src/hir/effects/mod.rs:26: pub mod scan;
src/hir/effects/mod.rs:31: static EXTRA_EFFECTS: LazyLock<Mutex<Vec<String>>> = LazyLock::new(|| Mutex::new(Vec::new()));
src/hir/effects/mod.rs:33: pub fn register_effect(name: &str)
src/hir/effects/mod.rs:41: pub fn is_effect(name: &str) -> bool
src/hir/effects/mod.rs:47: pub enum ThrowsDecl
src/hir/effects/mod.rs:58: pub struct EffectDecl
src/hir/effects/mod.rs:69: impl EffectDecl
src/hir/effects/mod.rs:70: pub fn has_effect(&self, name: &str) -> bool
src/hir/effects/mod.rs:75: pub fn no_effects(&self) -> bool
src/hir/effects/mod.rs:80: pub fn no_throws(&self) -> bool
src/hir/effects/mod.rs:87: pub struct EffectSummary
src/hir/effects/mod.rs:92: impl EffectSummary
src/hir/effects/mod.rs:94: pub fn tokens(&self) -> Vec<String>
src/hir/effects/mod.rs:123: pub fn from_tokens(tokens: &[String]) -> Self
src/hir/effects/mod.rs:154: pub fn parse(attrs: &[Attr]) -> Result<EffectDecl>
src/hir/effects/mod.rs:175: fn parse_throws(a: &Attr) -> Result<ThrowsDecl>
src/hir/effects/mod.rs:196: fn merge_throws(decl: &mut EffectDecl, new: ThrowsDecl)
src/hir/effects/mod.rs:213: fn bad_throws(a: &Attr) -> Error
src/hir/effects/scan.rs:8: pub(super) enum Obs
src/hir/effects/scan.rs:15: impl Obs
src/hir/effects/scan.rs:17: pub(super) fn site_of(&self, kind: &str) -> Option<(usize, usize)>
src/hir/effects/scan.rs:28: pub(super) fn collect_ast_observations(program: &crate::parser::ast::Program) -> HashMap<String, Vec<Obs>>
src/hir/effects/scan.rs:36: fn qualify(prefix: &str, name: &str) -> String
src/hir/effects/scan.rs:40: fn scan_decl(stmt: &crate::parser::ast::Stmt, prefix: &str, map: &mut HashMap<String, Vec<Obs>>)
src/hir/effects/scan.rs:62: fn scan_body(stmt: &crate::parser::ast::Stmt, obs: &mut Vec<Obs>)
src/hir/effects/scan.rs:111: fn scan_expr(e: &crate::parser::ast::Expr, obs: &mut Vec<Obs>)
src/hir/item.rs:7: pub struct HirStructField
src/hir/item.rs:13: pub struct HirStructDef
src/hir/item.rs:19: pub struct HirLocal
src/hir/item.rs:25: impl HirLocal
src/hir/item.rs:26: pub fn new(name: Symbol, ty: HirType, mutable: bool) -> Self
src/hir/item.rs:34: pub struct VtableEntry
src/hir/item.rs:42: pub struct HirInterfaceMethod
src/hir/item.rs:50: pub struct HirFn
src/hir/item.rs:77: pub enum HirItem
src/hir/item.rs:93: pub struct ImportedFnSig
src/hir/item.rs:107: pub struct HirProgram
src/hir/lower/body/collect_enum.rs:5: impl crate::hir::lower::Ctx
src/hir/lower/body/collect_enum.rs:6: pub(crate) fn collect_enum_def(
src/hir/lower/body/collect_import.rs:3: impl crate::hir::lower::Ctx
src/hir/lower/body/collect_import.rs:4: pub(crate) fn collect_import(&mut self, path: &String, ns_prefix: &str) -> Result<()>
src/hir/lower/body/ensure.rs:8: impl crate::hir::lower::Ctx
src/hir/lower/body/ensure.rs:10: pub(crate) fn inject_ensures(
src/hir/lower/body/ensure.rs:57: impl crate::hir::lower::Ctx
src/hir/lower/body/ensure.rs:59: pub(crate) fn loop_check_stmts(
src/hir/lower/body/ensure.rs:83: fn result_node(var: VarId, ty: &HirType) -> HirNodeBox
src/hir/lower/body/ensure.rs:88: fn rewrite_returns(stmts: &mut Vec<HirStmt>, var: VarId, ty: &HirType, checks: &[HirStmt])
src/hir/lower/body/ensure.rs:119: fn always_returns(stmts: &[HirStmt]) -> bool
src/hir/lower/body/ensure.rs:132: fn default_literal(ty: &HirType) -> Option<HirLiteral>
src/hir/lower/body/ensure.rs:143: fn append_default_return(
src/hir/lower/body/expr_access.rs:3: impl crate::hir::lower::Ctx
src/hir/lower/body/expr_access.rs:4: pub(crate) fn lower_field_access(&mut self, object: &Box<Expr>, field: &Symbol, expr_span: &Span) -> Result<HirNodeBox>
src/hir/lower/body/expr_access.rs:17: pub(crate) fn lower_struct_literal(&mut self, type_name: &Symbol, generic_args: &Vec<Type>, fields: &Vec<(Symbol, Expr)>) -> Result<HirNodeBox>
src/hir/lower/body/expr_access.rs:94: pub(crate) fn lower_array_literal(&mut self, elems: &Vec<Expr>, span: &Span) -> Result<HirNodeBox>
src/hir/lower/body/expr_access.rs:115: pub(crate) fn lower_index(&mut self, object: &Box<Expr>, index: &Box<Expr>, span: &Span) -> Result<HirNodeBox>
src/hir/lower/body/expr_call.rs:3: impl crate::hir::lower::Ctx
src/hir/lower/body/expr_call.rs:4: pub(crate) fn lower_fn_call(&mut self, name: &Symbol, args: &Vec<Expr>, span: &Span) -> Result<HirNodeBox>
src/hir/lower/body/expr_call.rs:131: pub(crate) fn lower_method_call(&mut self, object: &Box<Expr>, method: &Symbol, args: &Vec<Expr>, span: &Span) -> Result<HirNodeBox>
src/hir/lower/body/expr_call_extra.rs:3: impl crate::hir::lower::Ctx
src/hir/lower/body/expr_call_extra.rs:4: pub(crate) fn lower_call_expr(&mut self, target: &Box<Expr>, args: &Vec<Expr>, span: &Span) -> Result<HirNodeBox>
src/hir/lower/body/expr_call_extra.rs:39: pub(crate) fn make_fatptr_arg(&self, arg: HirNodeBox, param_ty: &HirType, ct: Symbol, iface: Symbol) -> HirNodeBox
src/hir/lower/body/expr_enum.rs:3: impl crate::hir::lower::Ctx
src/hir/lower/body/expr_enum.rs:4: pub(crate) fn lower_enum_construct(&mut self, enum_name: &Symbol, variant_name: &Symbol, tuple_args: &Vec<Expr>, named_args: &Vec<(Symbol, Expr)>, span: &Span) -> Result<HirNodeBox>
src/hir/lower/body/expr_misc.rs:3: impl crate::hir::lower::Ctx
src/hir/lower/body/expr_misc.rs:4: pub(crate) fn lower_asm(&mut self, template: &String, outputs: &Vec<(String, Box<Expr>)>, inputs: &Vec<(String, Box<Expr>)>) -> Result<HirNodeBox>
src/hir/lower/body/expr_misc.rs:24: pub(crate) fn lower_lambda(&mut self, params: &Vec<(Symbol, Type)>, return_type: &Type, body: &Vec<Stmt>) -> Result<HirNodeBox>
src/hir/lower/body/expr_ops1.rs:3: impl crate::hir::lower::Ctx
src/hir/lower/body/expr_ops1.rs:4: pub(crate) fn lower_binary(&mut self, op: &BinaryOp, lhs: &Box<Expr>, rhs: &Box<Expr>, span: &Span) -> Result<HirNodeBox>
src/hir/lower/body/expr_ops1.rs:58: pub(crate) fn lower_unary(&mut self, op: &UnaryOp, arg: &Box<Expr>) -> Result<HirNodeBox>
src/hir/lower/body/expr_ops1.rs:93: pub(crate) fn lower_try_op(&mut self, inner: &Box<Expr>, span: &Span) -> Result<HirNodeBox>
src/hir/lower/body/mod.rs:10: mod part_01;
src/hir/lower/body/mod.rs:11: mod part_02;
src/hir/lower/body/mod.rs:12: mod part_03;
src/hir/lower/body/mod.rs:13: mod part_04;
src/hir/lower/body/mod.rs:14: mod part_05;
src/hir/lower/body/mod.rs:15: mod part_06;
src/hir/lower/body/mod.rs:16: mod part_07;
src/hir/lower/body/mod.rs:17: mod part_08;
src/hir/lower/body/mod.rs:18: mod part_09;
src/hir/lower/body/mod.rs:19: mod ensure;
src/hir/lower/body/mod.rs:20: mod part_10;
src/hir/lower/body/mod.rs:21: mod collect_enum;
src/hir/lower/body/mod.rs:22: mod collect_import;
src/hir/lower/body/mod.rs:23: mod expr_access;
src/hir/lower/body/mod.rs:24: mod expr_call;
src/hir/lower/body/mod.rs:25: mod expr_call_extra;
src/hir/lower/body/mod.rs:26: mod expr_enum;
src/hir/lower/body/mod.rs:27: mod expr_misc;
src/hir/lower/body/mod.rs:28: mod expr_ops1;
src/hir/lower/body/part_01.rs:3: impl crate::hir::lower::Ctx
src/hir/lower/body/part_01.rs:8: pub(crate) fn collect_fns(&mut self, stmts: &[Stmt]) -> Result<()>
src/hir/lower/body/part_02.rs:3: impl crate::hir::lower::Ctx
src/hir/lower/body/part_02.rs:4: pub(super) fn collect_fns_with_ns(&mut self, stmts: &[Stmt], ns_prefix: &str) -> Result<()>
src/hir/lower/body/part_03.rs:3: impl crate::hir::lower::Ctx
src/hir/lower/body/part_03.rs:6: pub(crate) fn build_vtables(&mut self) -> Result<()>
src/hir/lower/body/part_03.rs:72: pub(crate) fn try_match_interface(
src/hir/lower/body/part_03.rs:131: pub(crate) fn try_match_generic_interface(
src/hir/lower/body/part_04.rs:3: impl crate::hir::lower::Ctx
src/hir/lower/body/part_04.rs:4: pub(crate) fn infer_iface_generic(
src/hir/lower/body/part_04.rs:17: pub(crate) fn substitute_iface_type(ty: &HirType, subst: &HashMap<Symbol, HirType>, gp_names: &[Symbol]) -> HirType
src/hir/lower/body/part_04.rs:30: pub(crate) fn type_matches(a: &HirType, b: &HirType) -> bool
src/hir/lower/body/part_04.rs:43: pub(crate) fn find_fn_by_sig(&self, name: Symbol, param_types: &[HirType]) -> Option<FnId>
src/hir/lower/body/part_04.rs:56: pub(crate) fn extract_concrete_type_name(ty: &HirType) -> Option<Symbol>
src/hir/lower/body/part_04.rs:71: pub(crate) fn is_fatptr_compatible(&self, param_ty: &HirType, arg_ty: &HirType) -> bool
src/hir/lower/body/part_04.rs:109: pub(crate) fn check_generic_fns_for_iface(&self, type_name: &Symbol, iface_name: &Symbol) -> bool
src/hir/lower/body/part_04.rs:144: pub(crate) fn ensure_specialized_interface(&mut self, specialized_name: &Symbol) -> Result<()>
src/hir/lower/body/part_04.rs:181: pub(crate) fn register_generic_vtable(
src/hir/lower/body/part_05.rs:4: fn same_base_name(a: &HirType, b: &HirType) -> bool
src/hir/lower/body/part_05.rs:14: impl crate::hir::lower::Ctx
src/hir/lower/body/part_05.rs:15: pub(crate) fn param_compatible(&self, param_ty: &HirType, arg_ty: &HirType) -> bool
src/hir/lower/body/part_05.rs:52: pub(crate) fn resolve_fn_call(&self, name: &Symbol, arg_types: &[HirType]) -> Option<FnId>
src/hir/lower/body/part_05.rs:98: pub(crate) fn receiver_matches_param(receiver: &HirType, param: &HirType) -> bool
src/hir/lower/body/part_05.rs:137: pub(crate) fn resolve_method(&self, receiver_type: &HirType, method_name: &Symbol, arg_types: &[HirType]) -> Option<FnId>
src/hir/lower/body/part_06.rs:3: impl crate::hir::lower::Ctx
src/hir/lower/body/part_06.rs:6: pub(crate) fn specialize_generic_call(&mut self, name: &Symbol, arg_types: &[HirType], span: &crate::span::Span) -> Result<FnId>
src/hir/lower/body/part_07.rs:3: impl crate::hir::lower::Ctx
src/hir/lower/body/part_07.rs:4: pub(crate) fn lower_items(&mut self, stmts: &[Stmt]) -> Result<Vec<HirItem>>
src/hir/lower/body/part_07.rs:8: pub(crate) fn lower_items_with_ns(&mut self, stmts: &[Stmt], ns_prefix: &str) -> Result<Vec<HirItem>>
src/hir/lower/body/part_07.rs:115: pub(crate) fn collect_gp_from_type(ty: &HirType, out: &mut Vec<Symbol>)
src/hir/lower/body/part_07.rs:146: pub(crate) fn lower_fn(
src/hir/lower/body/part_07.rs:255: pub(crate) fn lower_block(&mut self, block: &Block) -> Result<HirBlock>
src/hir/lower/body/part_08.rs:3: impl crate::hir::lower::Ctx
src/hir/lower/body/part_08.rs:4: pub(crate) fn lower_stmt(&mut self, stmt: &Stmt) -> Result<HirStmt>
src/hir/lower/body/part_08.rs:184: pub(crate) fn lower_while(
src/hir/lower/body/part_08.rs:200: pub(crate) fn lower_for(
src/hir/lower/body/part_09.rs:3: impl crate::hir::lower::Ctx
src/hir/lower/body/part_09.rs:4: pub(crate) fn lower_expr(&mut self, expr: &Expr) -> Result<HirNodeBox>
src/hir/lower/body/part_10.rs:3: impl crate::hir::lower::Ctx
src/hir/lower/body/part_10.rs:4: pub(crate) fn lower_literal(&mut self, lit: &Literal) -> Result<HirNodeBox>
src/hir/lower/ctx.rs:21: pub(crate) struct Ctx
src/hir/lower/ctx.rs:58: impl Ctx
src/hir/lower/ctx.rs:60: pub fn new() -> Self
src/hir/lower/ctx.rs:96: pub fn push_scope(&mut self) { self.scopes.push(HashMap::new()); }
src/hir/lower/ctx.rs:99: pub fn pop_scope(&mut self) { self.scopes.pop(); }
src/hir/lower/ctx.rs:102: pub fn bind_var(&mut self, name: Symbol, id: VarId, ty: HirType, mutable: bool)
src/hir/lower/ctx.rs:107: pub fn lookup_var(&self, name: &Symbol) -> Option<(VarId, HirType, bool)>
src/hir/lower/ctx.rs:115: pub fn find_field_index(&self, struct_ty: &HirType, field: &Symbol, span: &Span) -> Result<usize>
src/hir/lower/ctx.rs:128: fn find_field_index_by_name(&self, type_name: &Symbol, field: &Symbol, span: &Span) -> Result<usize>
src/hir/lower/ctx.rs:146: pub fn find_field_type(&self, struct_ty: &HirType, field: &Symbol, span: &Span) -> Result<HirType>
src/hir/lower/ctx.rs:164: fn find_field_type_by_name(&self, type_name: &Symbol, field: &Symbol, span: &Span) -> Result<HirType>
src/hir/lower/ctx.rs:183: pub fn variant_payload_type(&self, enum_ty: &HirType, data_field: &Symbol, span: &Span) -> Result<HirType>
src/hir/lower/ctx.rs:205: pub(crate) fn build_generic_subst(&self, type_name: &Symbol, base: &Symbol) -> HashMap<Symbol, HirType>
src/hir/lower/ctx.rs:224: pub fn collected_generic_params(&self, type_name: &Symbol) -> Vec<(Symbol, Option<Symbol>)>
src/hir/lower/ctx.rs:229: pub fn register_or_lookup(&mut self, name: Symbol, inferred_ty: HirType) -> (VarId, HirType, bool)
src/hir/lower/ctx.rs:238: pub fn update_var_type(&mut self, var_id: VarId, new_ty: HirType)
src/hir/lower/ctx.rs:253: pub fn is_enum_type(&self, type_name: &Symbol) -> bool
src/hir/lower/ctx_mono.rs:6: fn rewrite_variant_name(ty: &HirType, base: Symbol, suffix: &str) -> HirType
src/hir/lower/ctx_mono.rs:18: impl crate::hir::lower::Ctx
src/hir/lower/ctx_mono.rs:20: pub fn instantiate_type(&mut self, ty: &HirType) -> Result<()>
src/hir/lower/ctx_mono.rs:35: pub fn instantiate_named(&mut self, name: Symbol) -> Result<()>
src/hir/lower/ctx_mono.rs:80: pub fn adapt_enum_args(
src/hir/lower/ctx_mono.rs:98: pub fn instantiate_enum_value(&mut self, node: HirNodeBox, expected: &HirType) -> Result<HirNodeBox>
src/hir/lower/helpers/convert.rs:3: pub(crate) fn hir_type_to_ast_type(ty: &HirType) -> Type
src/hir/lower/helpers/convert.rs:28: pub(crate) fn infer_generic_from_param<'a>(param_ty: &'a Type, arg_ty: &'a HirType) -> Option<(Symbol, HirType)>
src/hir/lower/helpers/convert.rs:73: pub(crate) fn substitute_hir_type(ty: &HirType, subst: &HashMap<Symbol, HirType>) -> HirType
src/hir/lower/helpers/mod.rs:18: mod convert;
src/hir/lower/helpers/mod.rs:19: mod substitute;
src/hir/lower/helpers/mod.rs:20: mod types;
src/hir/lower/helpers/mod.rs:21: mod wrap;
src/hir/lower/helpers/substitute.rs:3: pub(crate) fn substitute_type_in_type(ty: &Type, subst: &HashMap<Symbol, Type>) -> Type
src/hir/lower/helpers/substitute.rs:29: pub(crate) fn substitute_type_in_expr(expr: &Expr, subst: &HashMap<Symbol, Type>) -> Expr
src/hir/lower/helpers/substitute.rs:124: pub(crate) fn substitute_type_in_block(block: &Block, subst: &HashMap<Symbol, Type>) -> Block
src/hir/lower/helpers/substitute.rs:132: pub(crate) fn substitute_type_in_stmt(stmt: &Stmt, subst: &HashMap<Symbol, Type>) -> Stmt
src/hir/lower/helpers/types.rs:3: pub(crate) fn type_to_string_generic(ty: &Type, interfaces: &HashMap<Symbol, InterfaceReg>) -> String
src/hir/lower/helpers/types.rs:26: pub(crate) fn sig_str_to_hir(s: &str) -> HirType
src/hir/lower/helpers/types.rs:49: fn is_iface_type(inner_hir: &HirType, interfaces: &HashMap<Symbol, super::InterfaceReg>) -> bool
src/hir/lower/helpers/types.rs:60: pub(crate) fn ast_type_to_hir(ty: &Type, interfaces: &HashMap<Symbol, InterfaceReg>) -> HirType
src/hir/lower/helpers/types.rs:120: pub(crate) fn extract_named(ty: &HirType) -> Option<&Symbol>
src/hir/lower/helpers/types.rs:129: pub(crate) fn hir_type_display(ty: &HirType) -> String
src/hir/lower/helpers/types.rs:152: pub(crate) fn needs_deep_copy(ty: &HirType) -> bool
src/hir/lower/helpers/types.rs:157: pub(crate) fn strip_ownership(ty: HirType) -> HirType
src/hir/lower/helpers/types.rs:165: pub(crate) fn strip_ownership_ref(ty: &HirType) -> &HirType
src/hir/lower/helpers/types.rs:173: pub(crate) fn expr_type(expr: &HirNodeBox) -> HirType
src/hir/lower/helpers/types.rs:178: pub(crate) fn is_null_literal(expr: &HirNodeBox) -> bool
src/hir/lower/helpers/types.rs:183: pub(crate) fn is_pointer_type_for_cmp(ty: &HirType) -> bool
src/hir/lower/helpers/wrap.rs:3: pub(crate) fn implicit_move(expr: HirNodeBox) -> HirNodeBox
src/hir/lower/helpers/wrap.rs:14: pub(crate) fn wrap_arg_for_param(arg: HirNodeBox, param_ty: &HirType) -> HirNodeBox
src/hir/lower/helpers/wrap.rs:60: pub(crate) fn wrap_for_unique_param(expr: HirNodeBox, param_ty: &HirType) -> HirNodeBox
src/hir/lower/helpers/wrap.rs:71: pub(crate) fn binary_op_to_fn_name(op: &BinaryOp) -> Option<&'static str>
src/hir/lower/helpers/wrap.rs:90: pub(crate) fn unary_op_to_fn_name(op: &UnaryOp) -> Option<&'static str>
src/hir/lower/mod.rs:1: pub mod body;
src/hir/lower/mod.rs:2: pub mod helpers;
src/hir/lower/mod.rs:3: pub mod to_mir;
src/hir/lower/mod.rs:15: pub(super) fn strip_generic_name(name: &Symbol) -> Symbol
src/hir/lower/mod.rs:31: pub(crate) struct FnSig
src/hir/lower/mod.rs:46: pub(crate) struct InterfaceReg
src/hir/lower/mod.rs:53: mod ctx;
src/hir/lower/mod.rs:54: mod ctx_mono;
src/hir/lower/mod.rs:70: pub fn lower_program(program: &Program) -> Result<HirProgram>
src/hir/lower/to_mir/access.rs:3: impl HirNode for SField
src/hir/lower/to_mir/access.rs:4: fn clone_node(&self) -> Box<dyn HirNode> { Box::new(self.clone()) }
src/hir/lower/to_mir/access.rs:5: fn lower_to_mir(&self, moved: &HashSet<VarId>) -> MirNodeBox
src/hir/lower/to_mir/access.rs:13: fn display(&self, level: usize, w: &mut dyn std::fmt::Write) -> std::fmt::Result
src/hir/lower/to_mir/access.rs:18: fn expr_type(&self) -> HirType { self.ty.clone() }
src/hir/lower/to_mir/access.rs:19: fn for_each_child(&self, f: &mut dyn FnMut(&dyn HirNode))
src/hir/lower/to_mir/access.rs:24: impl HirNode for SStruct
src/hir/lower/to_mir/access.rs:25: fn clone_node(&self) -> Box<dyn HirNode> { Box::new(self.clone()) }
src/hir/lower/to_mir/access.rs:26: fn as_struct_cloned(&self) -> Option<SStruct> { Some(self.clone()) }
src/hir/lower/to_mir/access.rs:27: fn lower_to_mir(&self, moved: &HashSet<VarId>) -> MirNodeBox
src/hir/lower/to_mir/access.rs:34: fn display(&self, level: usize, w: &mut dyn std::fmt::Write) -> std::fmt::Result
src/hir/lower/to_mir/access.rs:42: fn expr_type(&self) -> HirType { self.ty.clone() }
src/hir/lower/to_mir/access.rs:43: fn for_each_child(&self, f: &mut dyn FnMut(&dyn HirNode))
src/hir/lower/to_mir/access.rs:48: impl HirNode for SArrLit
src/hir/lower/to_mir/access.rs:49: fn is_alloc(&self) -> bool { true }
src/hir/lower/to_mir/access.rs:50: fn clone_node(&self) -> Box<dyn HirNode> { Box::new(self.clone()) }
src/hir/lower/to_mir/access.rs:51: fn lower_to_mir(&self, moved: &HashSet<VarId>) -> MirNodeBox
src/hir/lower/to_mir/access.rs:57: fn display(&self, level: usize, w: &mut dyn std::fmt::Write) -> std::fmt::Result
src/hir/lower/to_mir/access.rs:62: fn expr_type(&self) -> HirType { self.ty.clone() }
src/hir/lower/to_mir/access.rs:63: fn for_each_child(&self, f: &mut dyn FnMut(&dyn HirNode))
src/hir/lower/to_mir/access.rs:68: impl HirNode for SArrSz
src/hir/lower/to_mir/access.rs:69: fn is_alloc(&self) -> bool { true }
src/hir/lower/to_mir/access.rs:70: fn clone_node(&self) -> Box<dyn HirNode> { Box::new(self.clone()) }
src/hir/lower/to_mir/access.rs:71: fn lower_to_mir(&self, moved: &HashSet<VarId>) -> MirNodeBox
src/hir/lower/to_mir/access.rs:78: fn display(&self, level: usize, w: &mut dyn std::fmt::Write) -> std::fmt::Result
src/hir/lower/to_mir/access.rs:84: fn expr_type(&self) -> HirType { self.ty.clone() }
src/hir/lower/to_mir/access.rs:85: fn for_each_child(&self, f: &mut dyn FnMut(&dyn HirNode))
src/hir/lower/to_mir/access.rs:90: impl HirNode for SRef
src/hir/lower/to_mir/access.rs:91: fn clone_node(&self) -> Box<dyn HirNode> { Box::new(self.clone()) }
src/hir/lower/to_mir/access.rs:92: fn lower_to_mir(&self, moved: &HashSet<VarId>) -> MirNodeBox
src/hir/lower/to_mir/access.rs:95: fn display(&self, level: usize, w: &mut dyn std::fmt::Write) -> std::fmt::Result
src/hir/lower/to_mir/access.rs:101: fn expr_type(&self) -> HirType { self.ty.clone() }
src/hir/lower/to_mir/access.rs:102: fn for_each_child(&self, f: &mut dyn FnMut(&dyn HirNode))
src/hir/lower/to_mir/access.rs:107: impl HirNode for SIdx
src/hir/lower/to_mir/access.rs:108: fn clone_node(&self) -> Box<dyn HirNode> { Box::new(self.clone()) }
src/hir/lower/to_mir/access.rs:109: fn lower_to_mir(&self, moved: &HashSet<VarId>) -> MirNodeBox
src/hir/lower/to_mir/access.rs:116: fn display(&self, level: usize, w: &mut dyn std::fmt::Write) -> std::fmt::Result
src/hir/lower/to_mir/access.rs:124: fn expr_type(&self) -> HirType { self.ty.clone() }
src/hir/lower/to_mir/access.rs:125: fn for_each_child(&self, f: &mut dyn FnMut(&dyn HirNode))
src/hir/lower/to_mir/basic.rs:3: impl HirNode for SVar
src/hir/lower/to_mir/basic.rs:4: fn clone_node(&self) -> Box<dyn HirNode> { Box::new(self.clone()) }
src/hir/lower/to_mir/basic.rs:5: fn lower_to_mir(&self, moved: &HashSet<VarId>) -> MirNodeBox
src/hir/lower/to_mir/basic.rs:8: fn display(&self, level: usize, w: &mut dyn std::fmt::Write) -> std::fmt::Result
src/hir/lower/to_mir/basic.rs:11: fn expr_type(&self) -> HirType { self.ty.clone() }
src/hir/lower/to_mir/basic.rs:12: fn as_local(&self) -> Option<VarId> { Some(self.var) }
src/hir/lower/to_mir/basic.rs:13: fn collect_var_ids(&self, vars: &mut HashSet<VarId>) { vars.insert(self.var); }
src/hir/lower/to_mir/basic.rs:14: fn for_each_child(&self, f: &mut dyn FnMut(&dyn HirNode))
src/hir/lower/to_mir/basic.rs:19: impl HirNode for SConst
src/hir/lower/to_mir/basic.rs:20: fn is_alloc(&self) -> bool { matches!(self.val, HirLiteral::String(_)) }
src/hir/lower/to_mir/basic.rs:21: fn with_type(&self, ty: HirType) -> Option<HirNodeBox>
src/hir/lower/to_mir/basic.rs:24: fn clone_node(&self) -> Box<dyn HirNode> { Box::new(self.clone()) }
src/hir/lower/to_mir/basic.rs:25: fn lower_to_mir(&self, _moved: &HashSet<VarId>) -> MirNodeBox
src/hir/lower/to_mir/basic.rs:28: fn display(&self, level: usize, w: &mut dyn std::fmt::Write) -> std::fmt::Result
src/hir/lower/to_mir/basic.rs:38: fn expr_type(&self) -> HirType { self.ty.clone() }
src/hir/lower/to_mir/basic.rs:39: fn as_const(&self) -> Option<&HirLiteral> { Some(&self.val) }
src/hir/lower/to_mir/basic.rs:40: fn for_each_child(&self, f: &mut dyn FnMut(&dyn HirNode))
src/hir/lower/to_mir/basic.rs:45: impl HirNode for SBin
src/hir/lower/to_mir/basic.rs:46: fn clone_node(&self) -> Box<dyn HirNode> { Box::new(self.clone()) }
src/hir/lower/to_mir/basic.rs:47: fn lower_to_mir(&self, moved: &HashSet<VarId>) -> MirNodeBox
src/hir/lower/to_mir/basic.rs:55: fn display(&self, level: usize, w: &mut dyn std::fmt::Write) -> std::fmt::Result
src/hir/lower/to_mir/basic.rs:63: fn expr_type(&self) -> HirType { self.ty.clone() }
src/hir/lower/to_mir/basic.rs:64: fn is_comparison(&self) -> bool
src/hir/lower/to_mir/basic.rs:68: fn for_each_child(&self, f: &mut dyn FnMut(&dyn HirNode))
src/hir/lower/to_mir/basic.rs:74: impl HirNode for SUn
src/hir/lower/to_mir/basic.rs:75: fn clone_node(&self) -> Box<dyn HirNode> { Box::new(self.clone()) }
src/hir/lower/to_mir/basic.rs:76: fn lower_to_mir(&self, moved: &HashSet<VarId>) -> MirNodeBox
src/hir/lower/to_mir/basic.rs:79: fn display(&self, level: usize, w: &mut dyn std::fmt::Write) -> std::fmt::Result
src/hir/lower/to_mir/basic.rs:84: fn expr_type(&self) -> HirType { self.ty.clone() }
src/hir/lower/to_mir/basic.rs:85: fn for_each_child(&self, f: &mut dyn FnMut(&dyn HirNode))
src/hir/lower/to_mir/basic.rs:90: impl HirNode for SCall
src/hir/lower/to_mir/basic.rs:91: fn clone_node(&self) -> Box<dyn HirNode> { Box::new(self.clone()) }
src/hir/lower/to_mir/basic.rs:92: fn as_call(&self) -> Option<FnId> { Some(self.fn_id) }
src/hir/lower/to_mir/basic.rs:93: fn lower_to_mir(&self, moved: &HashSet<VarId>) -> MirNodeBox
src/hir/lower/to_mir/basic.rs:100: fn display(&self, level: usize, w: &mut dyn std::fmt::Write) -> std::fmt::Result
src/hir/lower/to_mir/basic.rs:105: fn expr_type(&self) -> HirType { self.ty.clone() }
src/hir/lower/to_mir/basic.rs:106: fn for_each_child(&self, f: &mut dyn FnMut(&dyn HirNode))
src/hir/lower/to_mir/basic.rs:111: impl HirNode for SMove
src/hir/lower/to_mir/basic.rs:112: fn clone_node(&self) -> Box<dyn HirNode> { Box::new(self.clone()) }
src/hir/lower/to_mir/basic.rs:113: fn lower_to_mir(&self, moved: &HashSet<VarId>) -> MirNodeBox
src/hir/lower/to_mir/basic.rs:116: fn display(&self, level: usize, w: &mut dyn std::fmt::Write) -> std::fmt::Result
src/hir/lower/to_mir/basic.rs:121: fn expr_type(&self) -> HirType { self.ty.clone() }
src/hir/lower/to_mir/basic.rs:122: fn is_move_or_clone(&self) -> bool { true }
src/hir/lower/to_mir/basic.rs:123: fn as_move(&self) -> Option<&HirNodeBox> { Some(&self.expr) }
src/hir/lower/to_mir/basic.rs:124: fn record_moves(&self, moved: &mut HashSet<VarId>)
src/hir/lower/to_mir/basic.rs:131: fn for_each_child(&self, f: &mut dyn FnMut(&dyn HirNode))
src/hir/lower/to_mir/basic.rs:136: impl HirNode for SClone
src/hir/lower/to_mir/basic.rs:137: fn clone_node(&self) -> Box<dyn HirNode> { Box::new(self.clone()) }
src/hir/lower/to_mir/basic.rs:138: fn lower_to_mir(&self, moved: &HashSet<VarId>) -> MirNodeBox
src/hir/lower/to_mir/basic.rs:141: fn display(&self, level: usize, w: &mut dyn std::fmt::Write) -> std::fmt::Result
src/hir/lower/to_mir/basic.rs:146: fn expr_type(&self) -> HirType { self.ty.clone() }
src/hir/lower/to_mir/basic.rs:147: fn is_move_or_clone(&self) -> bool { true }
src/hir/lower/to_mir/basic.rs:148: fn as_clone(&self) -> Option<&HirNodeBox> { Some(&self.expr) }
src/hir/lower/to_mir/basic.rs:149: fn for_each_child(&self, f: &mut dyn FnMut(&dyn HirNode))
src/hir/lower/to_mir/basic.rs:154: impl HirNode for SToUnique
src/hir/lower/to_mir/basic.rs:155: fn is_alloc(&self) -> bool { true }
src/hir/lower/to_mir/basic.rs:156: fn clone_node(&self) -> Box<dyn HirNode> { Box::new(self.clone()) }
src/hir/lower/to_mir/basic.rs:157: fn lower_to_mir(&self, moved: &HashSet<VarId>) -> MirNodeBox
src/hir/lower/to_mir/basic.rs:160: fn display(&self, level: usize, w: &mut dyn std::fmt::Write) -> std::fmt::Result
src/hir/lower/to_mir/basic.rs:165: fn expr_type(&self) -> HirType { self.ty.clone() }
src/hir/lower/to_mir/basic.rs:166: fn for_each_child(&self, f: &mut dyn FnMut(&dyn HirNode))
src/hir/lower/to_mir/basic.rs:169: fn record_moves(&self, moved: &mut HashSet<VarId>)
src/hir/lower/to_mir/call.rs:3: impl HirNode for SAsm
src/hir/lower/to_mir/call.rs:4: fn clone_node(&self) -> Box<dyn HirNode> { Box::new(self.clone()) }
src/hir/lower/to_mir/call.rs:5: fn lower_to_mir(&self, moved: &HashSet<VarId>) -> MirNodeBox
src/hir/lower/to_mir/call.rs:13: fn display(&self, level: usize, w: &mut dyn std::fmt::Write) -> std::fmt::Result
src/hir/lower/to_mir/call.rs:25: fn expr_type(&self) -> HirType { self.ty.clone() }
src/hir/lower/to_mir/call.rs:26: fn for_each_child(&self, f: &mut dyn FnMut(&dyn HirNode))
src/hir/lower/to_mir/call.rs:32: impl HirNode for SVCall
src/hir/lower/to_mir/call.rs:33: fn clone_node(&self) -> Box<dyn HirNode> { Box::new(self.clone()) }
src/hir/lower/to_mir/call.rs:34: fn lower_to_mir(&self, moved: &HashSet<VarId>) -> MirNodeBox
src/hir/lower/to_mir/call.rs:43: fn display(&self, level: usize, w: &mut dyn std::fmt::Write) -> std::fmt::Result
src/hir/lower/to_mir/call.rs:50: fn expr_type(&self) -> HirType { self.ty.clone() }
src/hir/lower/to_mir/call.rs:51: fn for_each_child(&self, f: &mut dyn FnMut(&dyn HirNode))
src/hir/lower/to_mir/call.rs:57: impl HirNode for SMFP
src/hir/lower/to_mir/call.rs:58: fn clone_node(&self) -> Box<dyn HirNode> { Box::new(self.clone()) }
src/hir/lower/to_mir/call.rs:59: fn lower_to_mir(&self, moved: &HashSet<VarId>) -> MirNodeBox
src/hir/lower/to_mir/call.rs:67: fn display(&self, level: usize, w: &mut dyn std::fmt::Write) -> std::fmt::Result
src/hir/lower/to_mir/call.rs:72: fn expr_type(&self) -> HirType { self.ty.clone() }
src/hir/lower/to_mir/call.rs:73: fn for_each_child(&self, f: &mut dyn FnMut(&dyn HirNode))
src/hir/lower/to_mir/call.rs:78: impl HirNode for SFnPtr
src/hir/lower/to_mir/call.rs:79: fn clone_node(&self) -> Box<dyn HirNode> { Box::new(self.clone()) }
src/hir/lower/to_mir/call.rs:80: fn lower_to_mir(&self, _moved: &HashSet<VarId>) -> MirNodeBox
src/hir/lower/to_mir/call.rs:83: fn display(&self, level: usize, w: &mut dyn std::fmt::Write) -> std::fmt::Result
src/hir/lower/to_mir/call.rs:86: fn expr_type(&self) -> HirType { self.ty.clone() }
src/hir/lower/to_mir/call.rs:87: fn for_each_child(&self, f: &mut dyn FnMut(&dyn HirNode))
src/hir/lower/to_mir/call.rs:92: impl HirNode for SCallP
src/hir/lower/to_mir/call.rs:93: fn clone_node(&self) -> Box<dyn HirNode> { Box::new(self.clone()) }
src/hir/lower/to_mir/call.rs:94: fn lower_to_mir(&self, moved: &HashSet<VarId>) -> MirNodeBox
src/hir/lower/to_mir/call.rs:101: fn display(&self, level: usize, w: &mut dyn std::fmt::Write) -> std::fmt::Result
src/hir/lower/to_mir/call.rs:104: fn expr_type(&self) -> HirType { self.ty.clone() }
src/hir/lower/to_mir/call.rs:105: fn for_each_child(&self, f: &mut dyn FnMut(&dyn HirNode))
src/hir/lower/to_mir/call.rs:111: impl HirNode for SEnumC
src/hir/lower/to_mir/call.rs:112: fn enum_variant(&self) -> Option<Symbol> { Some(self.variant_name) }
src/hir/lower/to_mir/call.rs:113: fn clone_node(&self) -> Box<dyn HirNode> { Box::new(self.clone()) }
src/hir/lower/to_mir/call.rs:114: fn lower_to_mir(&self, moved: &HashSet<VarId>) -> MirNodeBox
src/hir/lower/to_mir/call.rs:123: fn display(&self, level: usize, w: &mut dyn std::fmt::Write) -> std::fmt::Result
src/hir/lower/to_mir/call.rs:126: fn expr_type(&self) -> HirType { self.ty.clone() }
src/hir/lower/to_mir/call.rs:127: fn for_each_child(&self, f: &mut dyn FnMut(&dyn HirNode))
src/hir/lower/to_mir/call.rs:132: impl HirNode for SEnumM
src/hir/lower/to_mir/call.rs:133: fn clone_node(&self) -> Box<dyn HirNode> { Box::new(self.clone()) }
src/hir/lower/to_mir/call.rs:134: fn lower_to_mir(&self, moved: &HashSet<VarId>) -> MirNodeBox
src/hir/lower/to_mir/call.rs:141: fn display(&self, level: usize, w: &mut dyn std::fmt::Write) -> std::fmt::Result
src/hir/lower/to_mir/call.rs:147: fn expr_type(&self) -> HirType { self.ty.clone() }
src/hir/lower/to_mir/call.rs:148: fn for_each_child(&self, f: &mut dyn FnMut(&dyn HirNode))
src/hir/lower/to_mir/mod.rs:11: mod access;
src/hir/lower/to_mir/mod.rs:12: mod basic;
src/hir/lower/to_mir/mod.rs:13: mod call;
src/hir/mod.rs:1: pub mod attrs;
src/hir/mod.rs:2: pub mod attrs_macro;
src/hir/mod.rs:3: pub mod cfg;
src/hir/mod.rs:4: pub mod contracts;
src/hir/mod.rs:5: pub mod effects;
src/hir/mod.rs:6: pub mod ir;
src/hir/mod.rs:7: pub mod ty;
src/hir/mod.rs:8: pub mod node;
src/hir/mod.rs:9: pub mod stmt;
src/hir/mod.rs:10: pub mod item;
src/hir/mod.rs:11: pub mod lower;
src/hir/mod.rs:12: pub mod display;
src/hir/mod.rs:30: pub struct $name
src/hir/node.rs:9: pub trait HirNode: std::fmt::Debug
src/hir/node.rs:10: fn clone_node(&self) -> Box<dyn HirNode>;
src/hir/node.rs:11: fn lower_to_mir(&self, moved: &HashSet<VarId>) -> MirNodeBox;
src/hir/node.rs:12: fn display(&self, level: usize, w: &mut dyn FmtWrite) -> std::fmt::Result;
src/hir/node.rs:13: fn expr_type(&self) -> HirType;
src/hir/node.rs:16: fn as_local(&self) -> Option<VarId> { None }
src/hir/node.rs:17: fn is_move_or_clone(&self) -> bool { false }
src/hir/node.rs:19: fn for_each_child(&self, _f: &mut dyn FnMut(&dyn HirNode)) {}
src/hir/node.rs:21: fn collect_var_ids(&self, vars: &mut HashSet<VarId>)
src/hir/node.rs:25: fn record_moves(&self, moved: &mut HashSet<VarId>)
src/hir/node.rs:28: fn as_const(&self) -> Option<&HirLiteral> { None }
src/hir/node.rs:29: fn as_move(&self) -> Option<&HirNodeBox> { None }
src/hir/node.rs:31: fn is_comparison(&self) -> bool { false }
src/hir/node.rs:33: fn as_call(&self) -> Option<crate::hir::ty::FnId> { None }
src/hir/node.rs:35: fn is_alloc(&self) -> bool { false }
src/hir/node.rs:37: fn enum_variant(&self) -> Option<Symbol> { None }
src/hir/node.rs:39: fn as_struct_cloned(&self) -> Option<crate::hir::SStruct> { None }
src/hir/node.rs:41: fn with_type(&self, _ty: HirType) -> Option<HirNodeBox> { None }
src/hir/node.rs:42: fn as_clone(&self) -> Option<&HirNodeBox> { None }
src/hir/node.rs:47: pub struct HirNodeBox(pub Box<dyn HirNode>);
src/hir/node.rs:48: impl Clone for HirNodeBox
src/hir/node.rs:49: fn clone(&self) -> Self { HirNodeBox(self.0.clone_node()) }
src/hir/node.rs:51: impl std::ops::Deref for HirNodeBox
src/hir/node.rs:52: type Target = dyn HirNode;
src/hir/node.rs:53: fn deref(&self) -> &Self::Target { &*self.0 }
src/hir/node.rs:55: impl From<HirNodeBox> for Box<dyn HirNode>
src/hir/node.rs:56: fn from(b: HirNodeBox) -> Self { b.0 }
src/hir/node.rs:65: fn from(v: $ty) -> Self { HirNodeBox(Box::new(v) as Box<dyn HirNode>) }
src/hir/stmt.rs:5: pub enum ContractKind
src/hir/stmt.rs:11: impl ContractKind
src/hir/stmt.rs:12: pub fn label(&self) -> &'static str
src/hir/stmt.rs:21: pub fn runtime_fn(&self) -> &'static str
src/hir/stmt.rs:31: pub enum HirStmt
src/hir/stmt.rs:72: pub struct HirBlock
src/hir/stmt.rs:76: impl HirBlock
src/hir/stmt.rs:77: pub fn new(stmts: Vec<HirStmt>) -> Self
src/hir/ty.rs:7: pub struct VarId(pub usize);
src/hir/ty.rs:12: pub struct FnId(pub usize);
src/hir/ty.rs:22: pub enum HirType
src/hir/ty.rs:42: impl HirType
src/hir/ty.rs:44: pub fn is_copy(&self) -> bool
src/hir/ty.rs:59: pub enum HirLiteral
src/intern/interner.rs:3: pub(crate) struct Interner
src/intern/interner.rs:8: impl Interner
src/intern/interner.rs:9: pub(crate) fn new() -> Self
src/intern/interner.rs:19: pub(crate) fn intern(&mut self, name: &str) -> super::symbol::Symbol
src/intern/interner.rs:29: pub(crate) fn resolve(&self, sym: super::symbol::Symbol) -> &str
src/intern/mod.rs:5: pub mod interner;
src/intern/mod.rs:6: pub mod symbol;
src/intern/symbol.rs:6: static INTERNER: RefCell<Interner> = RefCell::new(Interner::new());
src/intern/symbol.rs:10: pub struct Symbol(pub(crate) u32);
src/intern/symbol.rs:12: impl Symbol
src/intern/symbol.rs:13: pub fn intern(name: &str) -> Self
src/intern/symbol.rs:17: pub fn as_str(&self) -> String
src/intern/symbol.rs:22: impl std::fmt::Display for Symbol
src/intern/symbol.rs:23: fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result
src/lexer/delimiter.rs:4: pub enum Delimiter
src/lexer/delimiter.rs:19: impl fmt::Display for Delimiter
src/lexer/delimiter.rs:20: fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result
src/lexer/keyword.rs:4: pub enum Keyword
src/lexer/keyword.rs:45: impl fmt::Display for Keyword
src/lexer/keyword.rs:46: fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result
src/lexer/lexer/mod.rs:6: pub struct Lexer<'a>
src/lexer/lexer/mod.rs:15: impl<'a> Lexer<'a>
src/lexer/lexer/mod.rs:16: pub fn new(src: &'a str) -> Self
src/lexer/lexer/mod.rs:27: fn peek(&self) -> Option<char>
src/lexer/lexer/mod.rs:30: fn peek_next(&self) -> Option<char>
src/lexer/lexer/mod.rs:34: fn bump(&mut self) -> Option<char>
src/lexer/lexer/mod.rs:47: fn skip_whitespace_and_comments(&mut self)
src/lexer/lexer/mod.rs:77: impl<'a> Lexer<'a>
src/lexer/lexer/mod.rs:79: pub fn tokenize_all(&mut self) -> Vec<Token>
src/lexer/lexer/mod.rs:93: mod next;
src/lexer/lexer/mod.rs:94: mod reading;
src/lexer/lexer/mod.rs:97: mod tests
src/lexer/lexer/mod.rs:102: fn smoke()
src/lexer/lexer/next.rs:3: impl<'a> Lexer<'a>
src/lexer/lexer/next.rs:4: pub fn next_token(&mut self) -> Token
src/lexer/lexer/reading.rs:3: impl<'a> Lexer<'a>
src/lexer/lexer/reading.rs:4: pub(super) fn is_ident_start(c: char) -> bool
src/lexer/lexer/reading.rs:7: pub(super) fn is_ident_continue(c: char) -> bool
src/lexer/lexer/reading.rs:11: pub(super) fn read_identifier_or_keyword(&mut self) -> (String, usize, usize, usize)
src/lexer/lexer/reading.rs:31: pub(super) fn read_number(&mut self) -> (String, bool, usize, usize, usize)
src/lexer/lexer/reading.rs:61: pub(super) fn read_char_literal(&mut self) -> (String, usize, usize, usize)
src/lexer/lexer/reading.rs:88: pub(super) fn read_string_literal(&mut self) -> (String, usize, usize, usize)
src/lexer/mod.rs:8: pub mod delimiter;
src/lexer/mod.rs:9: pub mod keyword;
src/lexer/mod.rs:10: pub mod lexer;
src/lexer/mod.rs:11: pub mod token;
src/lexer/mod.rs:12: pub mod token_kind;
src/lexer/mod.rs:13: pub mod test;
src/lexer/test.rs:2: pub mod test
src/lexer/test.rs:7: fn test()
src/lexer/token.rs:7: pub struct Token
src/lexer/token.rs:15: impl Token
src/lexer/token.rs:16: pub fn new(
src/lexer/token.rs:32: pub fn span(&self) -> Span
src/lexer/token.rs:44: impl fmt::Display for Token
src/lexer/token.rs:45: fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result
src/lexer/token_kind.rs:7: pub enum TokenKind
src/lexer/token_kind.rs:19: impl fmt::Display for TokenKind
src/lexer/token_kind.rs:20: fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result
src/lib.rs:17: pub mod generated;
src/lib.rs:18: pub mod compiler;
src/lib.rs:19: pub mod diagnostics;
src/lib.rs:20: pub mod driver;
src/lib.rs:21: pub mod error;
src/lib.rs:22: pub mod formatter;
src/lib.rs:23: pub mod hir;
src/lib.rs:24: pub mod intern;
src/lib.rs:25: pub mod lexer;
src/lib.rs:26: pub mod lir;
src/lib.rs:27: pub mod mir;
src/lib.rs:28: pub mod package;
src/lib.rs:29: pub mod parser;
src/lib.rs:30: pub mod span;
src/lib.rs:33: mod test_parse
src/lib.rs:35: fn test_struct_literal_parse()
src/lir/display.rs:5: pub fn lir_program_to_string(prog: &LirProgram) -> String
src/lir/display.rs:19: fn write_fn(f: &LirFn, w: &mut impl Write) -> std::fmt::Result
src/lir/display.rs:34: fn write_inst(inst: &LirNodeBox, w: &mut impl Write) -> std::fmt::Result
src/lir/emit/functions.rs:5: pub(super) fn llvm_attr_suffix(attrs: &[LirAttr], is_inline: bool, effects: LirEffects) -> String
src/lir/emit/functions.rs:38: pub(super) fn llvm_param_attrs(attrs: &[LirAttr]) -> String
src/lir/emit/functions.rs:50: impl<'a> Emitter<'a>
src/lir/emit/functions.rs:51: pub(super) fn emit_struct_defs(&mut self)
src/lir/emit/functions.rs:67: pub(super) fn struct_llvm_name(&self, name: &Symbol) -> Option<String>
src/lir/emit/functions.rs:83: pub(super) fn emit_string_globals(&mut self)
src/lir/emit/functions.rs:103: pub(super) fn emit_fn(&mut self, f: &LirFn)
src/lir/emit/functions.rs:142: pub(super) fn emit_inst(&mut self, inst: &LirNodeBox)
src/lir/emit/functions.rs:159: pub(super) fn tmp(&mut self) -> u64
src/lir/emit/functions.rs:165: pub(super) fn value_ref(&self, val: &LirValue, expected_ty: &HirType) -> String
src/lir/emit/functions.rs:181: fn escape_llvm_string(s: &str) -> String
src/lir/emit/mod.rs:19: pub fn emit_program(prog: &LirProgram) -> String
src/lir/emit/mod.rs:29: struct Emitter<'a>
src/lir/emit/mod.rs:37: impl<'a> Emitter<'a>
src/lir/emit/mod.rs:38: fn new(prog: &'a LirProgram) -> Self
src/lir/emit/mod.rs:48: fn finish(self) -> String
src/lir/emit/mod.rs:52: fn wln(&mut self, s: &str)
src/lir/emit/mod.rs:60: fn wln_fmt(&mut self, fmt: std::fmt::Arguments<'_>)
src/lir/emit/mod.rs:69: fn emit(&mut self)
src/lir/emit/mod.rs:120: mod functions;
src/lir/emit/mod.rs:121: mod types;
src/lir/emit/mod.rs:122: mod vtable;
src/lir/emit/types.rs:3: impl<'a> Emitter<'a>
src/lir/emit/types.rs:4: pub(super) fn llvm_type(&self, ty: &HirType) -> String
src/lir/emit/vtable.rs:3: impl<'a> Emitter<'a>
src/lir/emit/vtable.rs:4: pub(super) fn emit_vtable_globals(&mut self)
src/lir/emit/vtable.rs:38: pub(super) fn fn_needs_vtable_wrapper(&self, fn_id: FnId) -> bool
src/lir/emit/vtable.rs:50: pub(super) fn fn_ret_type(&self, fn_id: FnId) -> String
src/lir/emit/vtable.rs:57: pub(super) fn fn_first_param_unwrapped(&self, fn_id: FnId) -> String
src/lir/emit/vtable.rs:73: pub(super) fn fn_extra_params(&self, fn_id: FnId) -> (Vec<String>, Vec<String>)
src/lir/emit/vtable.rs:87: pub(super) fn emit_vtable_wrappers(&mut self)
src/lir/ir/helpers.rs:3: pub fn sanitize_name(name: &str) -> String
src/lir/ir/helpers.rs:8: pub(crate) fn lit_to_string(lit: &HirLiteral, expected_ty: &HirType) -> String
src/lir/ir/helpers.rs:23: pub(crate) fn llvm_type_size(ty: &HirType) -> &'static str
src/lir/ir/helpers.rs:39: pub(crate) fn struct_llvm_size(ty: &HirType, struct_defs: &std::collections::HashMap<Symbol, Vec<(Symbol, HirType)>>) -> String
src/lir/ir/helpers.rs:56: pub(super) fn needs_heap_ops(ty: &HirType) -> bool
src/lir/ir/helpers.rs:69: pub(super) fn is_pointer_type(ty: &HirType) -> bool
src/lir/ir/helpers.rs:82: pub(super) fn needs_drop(ty: &HirType, struct_defs: &HashMap<Symbol, Vec<(Symbol, HirType)>>) -> bool
src/lir/ir/helpers.rs:95: pub(super) fn emit_drop_value(
src/lir/ir/mod.rs:12: pub struct IrNode
src/lir/ir/mod.rs:18: pub enum IrValue
src/lir/ir/mod.rs:27: impl IrNode
src/lir/ir/mod.rs:28: pub fn new(kind: &str) -> Self
src/lir/ir/mod.rs:31: pub fn set(&mut self, name: &str, val: IrValue) -> &mut Self
src/lir/ir/mod.rs:35: pub fn get(&self, name: &str) -> Option<&IrValue>
src/lir/ir/mod.rs:41: pub enum ConvKind
src/lir/ir/mod.rs:46: pub enum LirValue
src/lir/ir/mod.rs:58: pub struct LirEmitCtx<'a>
src/lir/ir/mod.rs:64: impl LirEmitCtx<'_>
src/lir/ir/mod.rs:65: pub fn llvm_type(&self, ty: &HirType) -> String
src/lir/ir/mod.rs:100: pub fn value_ref(&self, val: &LirValue, expected_ty: &HirType) -> String
src/lir/ir/mod.rs:109: pub fn tmp(&mut self) -> u64
src/lir/ir/mod.rs:115: pub fn struct_llvm_name(&self, name: &Symbol) -> Option<String>
src/lir/ir/mod.rs:135: pub trait LirNode: std::fmt::Debug
src/lir/ir/mod.rs:136: fn clone_node(&self) -> Box<dyn LirNode>;
src/lir/ir/mod.rs:137: fn kind(&self) -> &'static str;
src/lir/ir/mod.rs:138: fn as_any(&self) -> &dyn std::any::Any;
src/lir/ir/mod.rs:139: fn emit(&self, ctx: &mut LirEmitCtx) -> Vec<String>;
src/lir/ir/mod.rs:140: fn display(&self, f: &mut dyn Write) -> std::fmt::Result;
src/lir/ir/mod.rs:141: fn serialize(&self, buf: &mut Vec<u8>);
src/lir/ir/mod.rs:149: pub struct LirNodeBox(pub Box<dyn LirNode>);
src/lir/ir/mod.rs:151: impl Clone for LirNodeBox
src/lir/ir/mod.rs:152: fn clone(&self) -> Self
src/lir/ir/mod.rs:157: impl std::ops::Deref for LirNodeBox
src/lir/ir/mod.rs:158: type Target = dyn LirNode;
src/lir/ir/mod.rs:159: fn deref(&self) -> &Self::Target
src/lir/ir/mod.rs:164: impl From<SLirCustom> for LirNodeBox
src/lir/ir/mod.rs:165: fn from(v: SLirCustom) -> Self { LirNodeBox(Box::new(v)) }
src/lir/ir/mod.rs:175: pub struct $name { $(pub $field: $ty),* }
src/lir/ir/mod.rs:182: fn from(v: $ty) -> Self { LirNodeBox(Box::new(v) as Box<dyn LirNode>) }
src/lir/ir/mod.rs:187: mod helpers;
src/lir/ir/mod.rs:188: mod nodes;
src/lir/ir/mod.rs:189: mod nodes_a;
src/lir/ir/mod.rs:190: mod nodes_b;
src/lir/ir/mod.rs:191: mod nodes_c;
src/lir/ir/mod.rs:192: mod nodes_d;
src/lir/ir/mod.rs:193: mod nodes_e;
src/lir/ir/nodes_a.rs:3: impl LirNode for SLirAlloca
src/lir/ir/nodes_a.rs:4: fn clone_node(&self) -> Box<dyn LirNode> { Box::new(self.clone()) }
src/lir/ir/nodes_a.rs:5: fn kind(&self) -> &'static str { "Alloca" }
src/lir/ir/nodes_a.rs:6: fn as_any(&self) -> &dyn std::any::Any { self }
src/lir/ir/nodes_a.rs:7: fn emit(&self, ctx: &mut LirEmitCtx) -> Vec<String>
src/lir/ir/nodes_a.rs:11: fn display(&self, f: &mut dyn Write) -> std::fmt::Result
src/lir/ir/nodes_a.rs:14: fn serialize(&self, buf: &mut Vec<u8>)
src/lir/ir/nodes_a.rs:21: impl LirNode for SLirStore
src/lir/ir/nodes_a.rs:22: fn clone_node(&self) -> Box<dyn LirNode> { Box::new(self.clone()) }
src/lir/ir/nodes_a.rs:23: fn kind(&self) -> &'static str { "Store" }
src/lir/ir/nodes_a.rs:24: fn as_any(&self) -> &dyn std::any::Any { self }
src/lir/ir/nodes_a.rs:25: fn emit(&self, ctx: &mut LirEmitCtx) -> Vec<String>
src/lir/ir/nodes_a.rs:41: fn display(&self, f: &mut dyn Write) -> std::fmt::Result
src/lir/ir/nodes_a.rs:44: fn serialize(&self, buf: &mut Vec<u8>)
src/lir/ir/nodes_a.rs:52: impl LirNode for SLirLoad
src/lir/ir/nodes_a.rs:53: fn clone_node(&self) -> Box<dyn LirNode> { Box::new(self.clone()) }
src/lir/ir/nodes_a.rs:54: fn kind(&self) -> &'static str { "Load" }
src/lir/ir/nodes_a.rs:55: fn as_any(&self) -> &dyn std::any::Any { self }
src/lir/ir/nodes_a.rs:56: fn emit(&self, ctx: &mut LirEmitCtx) -> Vec<String>
src/lir/ir/nodes_a.rs:60: fn display(&self, f: &mut dyn Write) -> std::fmt::Result
src/lir/ir/nodes_a.rs:63: fn serialize(&self, buf: &mut Vec<u8>)
src/lir/ir/nodes_a.rs:71: impl LirNode for SLirBinOp
src/lir/ir/nodes_a.rs:72: fn clone_node(&self) -> Box<dyn LirNode> { Box::new(self.clone()) }
src/lir/ir/nodes_a.rs:73: fn kind(&self) -> &'static str { "BinOp" }
src/lir/ir/nodes_a.rs:74: fn as_any(&self) -> &dyn std::any::Any { self }
src/lir/ir/nodes_a.rs:75: fn emit(&self, ctx: &mut LirEmitCtx) -> Vec<String>
src/lir/ir/nodes_a.rs:141: fn display(&self, f: &mut dyn Write) -> std::fmt::Result
src/lir/ir/nodes_a.rs:144: fn serialize(&self, buf: &mut Vec<u8>)
src/lir/ir/nodes_a.rs:155: impl LirNode for SLirUnaryOp
src/lir/ir/nodes_a.rs:156: fn clone_node(&self) -> Box<dyn LirNode> { Box::new(self.clone()) }
src/lir/ir/nodes_a.rs:157: fn kind(&self) -> &'static str { "UnaryOp" }
src/lir/ir/nodes_a.rs:158: fn as_any(&self) -> &dyn std::any::Any { self }
src/lir/ir/nodes_a.rs:159: fn emit(&self, ctx: &mut LirEmitCtx) -> Vec<String>
src/lir/ir/nodes_a.rs:168: fn display(&self, f: &mut dyn Write) -> std::fmt::Result
src/lir/ir/nodes_a.rs:171: fn serialize(&self, buf: &mut Vec<u8>)
src/lir/ir/nodes_a.rs:180: impl LirNode for SLirCall
src/lir/ir/nodes_a.rs:181: fn clone_node(&self) -> Box<dyn LirNode> { Box::new(self.clone()) }
src/lir/ir/nodes_a.rs:182: fn kind(&self) -> &'static str { "Call" }
src/lir/ir/nodes_a.rs:183: fn as_any(&self) -> &dyn std::any::Any { self }
src/lir/ir/nodes_a.rs:184: fn emit(&self, ctx: &mut LirEmitCtx) -> Vec<String>
src/lir/ir/nodes_a.rs:200: fn display(&self, f: &mut dyn Write) -> std::fmt::Result
src/lir/ir/nodes_a.rs:205: fn serialize(&self, buf: &mut Vec<u8>)
src/lir/ir/nodes_a.rs:215: impl LirNode for SLirCallPtr
src/lir/ir/nodes_a.rs:216: fn clone_node(&self) -> Box<dyn LirNode> { Box::new(self.clone()) }
src/lir/ir/nodes_a.rs:217: fn kind(&self) -> &'static str { "CallPtr" }
src/lir/ir/nodes_a.rs:218: fn as_any(&self) -> &dyn std::any::Any { self }
src/lir/ir/nodes_a.rs:219: fn emit(&self, ctx: &mut LirEmitCtx) -> Vec<String>
src/lir/ir/nodes_a.rs:230: fn display(&self, f: &mut dyn Write) -> std::fmt::Result
src/lir/ir/nodes_a.rs:234: fn serialize(&self, buf: &mut Vec<u8>)
src/lir/ir/nodes_a.rs:244: impl LirNode for SLirFnAddr
src/lir/ir/nodes_a.rs:245: fn clone_node(&self) -> Box<dyn LirNode> { Box::new(self.clone()) }
src/lir/ir/nodes_a.rs:246: fn kind(&self) -> &'static str { "FnAddr" }
src/lir/ir/nodes_a.rs:247: fn as_any(&self) -> &dyn std::any::Any { self }
src/lir/ir/nodes_a.rs:248: fn emit(&self, ctx: &mut LirEmitCtx) -> Vec<String>
src/lir/ir/nodes_a.rs:252: fn display(&self, f: &mut dyn Write) -> std::fmt::Result
src/lir/ir/nodes_a.rs:255: fn serialize(&self, buf: &mut Vec<u8>)
src/lir/ir/nodes_a.rs:262: impl LirNode for SLirStrGlobal
src/lir/ir/nodes_a.rs:263: fn clone_node(&self) -> Box<dyn LirNode> { Box::new(self.clone()) }
src/lir/ir/nodes_a.rs:264: fn kind(&self) -> &'static str { "StrGlobal" }
src/lir/ir/nodes_a.rs:265: fn as_any(&self) -> &dyn std::any::Any { self }
src/lir/ir/nodes_a.rs:266: fn emit(&self, ctx: &mut LirEmitCtx) -> Vec<String>
src/lir/ir/nodes_a.rs:279: fn display(&self, f: &mut dyn Write) -> std::fmt::Result
src/lir/ir/nodes_a.rs:282: fn serialize(&self, buf: &mut Vec<u8>)
src/lir/ir/nodes_b.rs:3: impl LirNode for SLirConv
src/lir/ir/nodes_b.rs:4: fn clone_node(&self) -> Box<dyn LirNode> { Box::new(self.clone()) }
src/lir/ir/nodes_b.rs:5: fn kind(&self) -> &'static str { "Conv" }
src/lir/ir/nodes_b.rs:6: fn as_any(&self) -> &dyn std::any::Any { self }
src/lir/ir/nodes_b.rs:7: fn emit(&self, ctx: &mut LirEmitCtx) -> Vec<String>
src/lir/ir/nodes_b.rs:47: fn display(&self, f: &mut dyn Write) -> std::fmt::Result
src/lir/ir/nodes_b.rs:50: fn serialize(&self, buf: &mut Vec<u8>)
src/lir/ir/nodes_b.rs:57: impl LirNode for SLirBr
src/lir/ir/nodes_b.rs:58: fn clone_node(&self) -> Box<dyn LirNode> { Box::new(self.clone()) }
src/lir/ir/nodes_b.rs:59: fn kind(&self) -> &'static str { "Br" }
src/lir/ir/nodes_b.rs:60: fn as_any(&self) -> &dyn std::any::Any { self }
src/lir/ir/nodes_b.rs:61: fn emit(&self, _ctx: &mut LirEmitCtx) -> Vec<String>
src/lir/ir/nodes_b.rs:64: fn display(&self, f: &mut dyn Write) -> std::fmt::Result
src/lir/ir/nodes_b.rs:67: fn serialize(&self, buf: &mut Vec<u8>)
src/lir/ir/nodes_b.rs:73: impl LirNode for SLirBrCond
src/lir/ir/nodes_b.rs:74: fn clone_node(&self) -> Box<dyn LirNode> { Box::new(self.clone()) }
src/lir/ir/nodes_b.rs:75: fn kind(&self) -> &'static str { "BrCond" }
src/lir/ir/nodes_b.rs:76: fn as_any(&self) -> &dyn std::any::Any { self }
src/lir/ir/nodes_b.rs:77: fn emit(&self, ctx: &mut LirEmitCtx) -> Vec<String>
src/lir/ir/nodes_b.rs:81: fn display(&self, f: &mut dyn Write) -> std::fmt::Result
src/lir/ir/nodes_b.rs:84: fn serialize(&self, buf: &mut Vec<u8>)
src/lir/ir/nodes_b.rs:92: impl LirNode for SLirAssume
src/lir/ir/nodes_b.rs:93: fn clone_node(&self) -> Box<dyn LirNode> { Box::new(self.clone()) }
src/lir/ir/nodes_b.rs:94: fn kind(&self) -> &'static str { "Assume" }
src/lir/ir/nodes_b.rs:95: fn as_any(&self) -> &dyn std::any::Any { self }
src/lir/ir/nodes_b.rs:96: fn emit(&self, ctx: &mut LirEmitCtx) -> Vec<String>
src/lir/ir/nodes_b.rs:100: fn display(&self, f: &mut dyn Write) -> std::fmt::Result
src/lir/ir/nodes_b.rs:103: fn serialize(&self, buf: &mut Vec<u8>)
src/lir/ir/nodes_b.rs:109: impl LirNode for SLirContractCheck
src/lir/ir/nodes_b.rs:110: fn clone_node(&self) -> Box<dyn LirNode> { Box::new(self.clone()) }
src/lir/ir/nodes_b.rs:111: fn kind(&self) -> &'static str { "ContractCheck" }
src/lir/ir/nodes_b.rs:112: fn as_any(&self) -> &dyn std::any::Any { self }
src/lir/ir/nodes_b.rs:113: fn emit(&self, ctx: &mut LirEmitCtx) -> Vec<String>
src/lir/ir/nodes_b.rs:124: fn display(&self, f: &mut dyn Write) -> std::fmt::Result
src/lir/ir/nodes_b.rs:127: fn serialize(&self, buf: &mut Vec<u8>)
src/lir/ir/nodes_b.rs:140: impl LirNode for SLirRet
src/lir/ir/nodes_b.rs:141: fn clone_node(&self) -> Box<dyn LirNode> { Box::new(self.clone()) }
src/lir/ir/nodes_b.rs:142: fn kind(&self) -> &'static str { "Ret" }
src/lir/ir/nodes_b.rs:143: fn as_any(&self) -> &dyn std::any::Any { self }
src/lir/ir/nodes_b.rs:144: fn emit(&self, ctx: &mut LirEmitCtx) -> Vec<String>
src/lir/ir/nodes_b.rs:154: fn display(&self, f: &mut dyn Write) -> std::fmt::Result
src/lir/ir/nodes_b.rs:160: fn serialize(&self, buf: &mut Vec<u8>)
src/lir/ir/nodes_b.rs:169: impl LirNode for SLirMakeFatPtr
src/lir/ir/nodes_b.rs:170: fn clone_node(&self) -> Box<dyn LirNode> { Box::new(self.clone()) }
src/lir/ir/nodes_b.rs:171: fn kind(&self) -> &'static str { "MakeFatPtr" }
src/lir/ir/nodes_b.rs:172: fn as_any(&self) -> &dyn std::any::Any { self }
src/lir/ir/nodes_b.rs:173: fn emit(&self, ctx: &mut LirEmitCtx) -> Vec<String>
src/lir/ir/nodes_b.rs:202: fn display(&self, f: &mut dyn Write) -> std::fmt::Result
src/lir/ir/nodes_b.rs:205: fn serialize(&self, buf: &mut Vec<u8>)
src/lir/ir/nodes_b.rs:213: impl LirNode for SLirFieldAccess
src/lir/ir/nodes_b.rs:214: fn clone_node(&self) -> Box<dyn LirNode> { Box::new(self.clone()) }
src/lir/ir/nodes_b.rs:215: fn kind(&self) -> &'static str { "FieldAccess" }
src/lir/ir/nodes_b.rs:216: fn as_any(&self) -> &dyn std::any::Any { self }
src/lir/ir/nodes_b.rs:217: fn emit(&self, ctx: &mut LirEmitCtx) -> Vec<String>
src/lir/ir/nodes_b.rs:238: fn display(&self, f: &mut dyn Write) -> std::fmt::Result
src/lir/ir/nodes_b.rs:241: fn serialize(&self, buf: &mut Vec<u8>)
src/lir/ir/nodes_c.rs:3: impl LirNode for SLirAsm
src/lir/ir/nodes_c.rs:4: fn clone_node(&self) -> Box<dyn LirNode> { Box::new(self.clone()) }
src/lir/ir/nodes_c.rs:5: fn kind(&self) -> &'static str { "Asm" }
src/lir/ir/nodes_c.rs:6: fn as_any(&self) -> &dyn std::any::Any { self }
src/lir/ir/nodes_c.rs:7: fn emit(&self, ctx: &mut LirEmitCtx) -> Vec<String>
src/lir/ir/nodes_c.rs:33: fn display(&self, f: &mut dyn Write) -> std::fmt::Result
src/lir/ir/nodes_c.rs:36: fn serialize(&self, buf: &mut Vec<u8>)
src/lir/ir/nodes_c.rs:55: impl LirNode for SLirArraySized
src/lir/ir/nodes_c.rs:56: fn clone_node(&self) -> Box<dyn LirNode> { Box::new(self.clone()) }
src/lir/ir/nodes_c.rs:57: fn kind(&self) -> &'static str { "ArraySized" }
src/lir/ir/nodes_c.rs:58: fn as_any(&self) -> &dyn std::any::Any { self }
src/lir/ir/nodes_c.rs:59: fn emit(&self, ctx: &mut LirEmitCtx) -> Vec<String>
src/lir/ir/nodes_c.rs:70: fn display(&self, f: &mut dyn Write) -> std::fmt::Result
src/lir/ir/nodes_c.rs:73: fn serialize(&self, buf: &mut Vec<u8>)
src/lir/ir/nodes_c.rs:82: impl LirNode for SLirArrayLit
src/lir/ir/nodes_c.rs:83: fn clone_node(&self) -> Box<dyn LirNode> { Box::new(self.clone()) }
src/lir/ir/nodes_c.rs:84: fn kind(&self) -> &'static str { "ArrayLit" }
src/lir/ir/nodes_c.rs:85: fn as_any(&self) -> &dyn std::any::Any { self }
src/lir/ir/nodes_c.rs:86: fn emit(&self, ctx: &mut LirEmitCtx) -> Vec<String>
src/lir/ir/nodes_c.rs:102: fn display(&self, f: &mut dyn Write) -> std::fmt::Result
src/lir/ir/nodes_c.rs:105: fn serialize(&self, buf: &mut Vec<u8>)
src/lir/ir/nodes_c.rs:116: impl LirNode for SLirIndexAccess
src/lir/ir/nodes_c.rs:117: fn clone_node(&self) -> Box<dyn LirNode> { Box::new(self.clone()) }
src/lir/ir/nodes_c.rs:118: fn kind(&self) -> &'static str { "IndexAccess" }
src/lir/ir/nodes_c.rs:119: fn as_any(&self) -> &dyn std::any::Any { self }
src/lir/ir/nodes_c.rs:120: fn emit(&self, ctx: &mut LirEmitCtx) -> Vec<String>
src/lir/ir/nodes_c.rs:130: fn display(&self, f: &mut dyn Write) -> std::fmt::Result
src/lir/ir/nodes_c.rs:133: fn serialize(&self, buf: &mut Vec<u8>)
src/lir/ir/nodes_c.rs:141: impl LirNode for SLirStructLit
src/lir/ir/nodes_c.rs:142: fn clone_node(&self) -> Box<dyn LirNode> { Box::new(self.clone()) }
src/lir/ir/nodes_c.rs:143: fn kind(&self) -> &'static str { "StructLit" }
src/lir/ir/nodes_c.rs:144: fn as_any(&self) -> &dyn std::any::Any { self }
src/lir/ir/nodes_c.rs:145: fn emit(&self, ctx: &mut LirEmitCtx) -> Vec<String>
src/lir/ir/nodes_c.rs:166: fn display(&self, f: &mut dyn Write) -> std::fmt::Result
src/lir/ir/nodes_c.rs:169: fn serialize(&self, buf: &mut Vec<u8>)
src/lir/ir/nodes_c.rs:180: impl LirNode for SLirVirtualCall
src/lir/ir/nodes_c.rs:181: fn clone_node(&self) -> Box<dyn LirNode> { Box::new(self.clone()) }
src/lir/ir/nodes_c.rs:182: fn kind(&self) -> &'static str { "VirtualCall" }
src/lir/ir/nodes_c.rs:183: fn as_any(&self) -> &dyn std::any::Any { self }
src/lir/ir/nodes_c.rs:184: fn emit(&self, ctx: &mut LirEmitCtx) -> Vec<String>
src/lir/ir/nodes_c.rs:205: fn display(&self, f: &mut dyn Write) -> std::fmt::Result
src/lir/ir/nodes_c.rs:209: fn serialize(&self, buf: &mut Vec<u8>)
src/lir/ir/nodes_c.rs:221: impl LirNode for SLirFieldStore
src/lir/ir/nodes_c.rs:222: fn clone_node(&self) -> Box<dyn LirNode> { Box::new(self.clone()) }
src/lir/ir/nodes_c.rs:223: fn kind(&self) -> &'static str { "FieldStore" }
src/lir/ir/nodes_c.rs:224: fn as_any(&self) -> &dyn std::any::Any { self }
src/lir/ir/nodes_c.rs:225: fn emit(&self, ctx: &mut LirEmitCtx) -> Vec<String>
src/lir/ir/nodes_c.rs:258: fn display(&self, f: &mut dyn Write) -> std::fmt::Result
src/lir/ir/nodes_c.rs:261: fn serialize(&self, buf: &mut Vec<u8>)
src/lir/ir/nodes_d.rs:3: impl LirNode for SLirCustom
src/lir/ir/nodes_d.rs:4: fn clone_node(&self) -> Box<dyn LirNode> { Box::new(self.clone()) }
src/lir/ir/nodes_d.rs:5: fn kind(&self) -> &'static str { "Custom" }
src/lir/ir/nodes_d.rs:6: fn as_any(&self) -> &dyn std::any::Any { self }
src/lir/ir/nodes_d.rs:7: fn emit(&self, _ctx: &mut LirEmitCtx) -> Vec<String>
src/lir/ir/nodes_d.rs:11: fn display(&self, _f: &mut dyn Write) -> std::fmt::Result { Ok(()) }
src/lir/ir/nodes_d.rs:12: fn serialize(&self, _buf: &mut Vec<u8>) {}
src/lir/ir/nodes_d.rs:20: pub struct LirBlock
src/lir/ir/nodes_d.rs:27: pub struct LirAttr
src/lir/ir/nodes_d.rs:34: pub struct ExternDecl
src/lir/ir/nodes_d.rs:47: pub struct LirEffects
src/lir/ir/nodes_d.rs:53: pub struct LirFn
src/lir/ir/nodes_d.rs:73: pub struct VtableDesc
src/lir/ir/nodes_d.rs:79: pub struct LirProgram
src/lir/ir/nodes_d.rs:95: pub(crate) fn put_u32(buf: &mut Vec<u8>, v: u32) { buf.extend_from_slice(&v.to_le_bytes()); }
src/lir/ir/nodes_d.rs:96: pub(crate) fn put_u64(buf: &mut Vec<u8>, v: u64) { buf.extend_from_slice(&v.to_le_bytes()); }
src/lir/ir/nodes_d.rs:97: pub(crate) fn put_str(buf: &mut Vec<u8>, s: &str)
src/lir/ir/nodes_d.rs:103: pub(crate) fn put_type(buf: &mut Vec<u8>, ty: &HirType)
src/lir/ir/nodes_d.rs:128: pub(crate) fn put_value(buf: &mut Vec<u8>, v: &LirValue)
src/lir/ir/nodes_d.rs:141: pub(crate) fn put_literal(buf: &mut Vec<u8>, lit: &HirLiteral)
src/lir/ir/nodes_d.rs:156: pub trait LirLowerCtx
src/lir/ir/nodes_d.rs:157: fn next_tmp(&mut self) -> u64;
src/lir/ir/nodes_d.rs:158: fn emit(&mut self, inst: LirNodeBox);
src/lir/ir/nodes_d.rs:159: fn str_map(&self) -> &HashMap<String, u64>;
src/lir/ir/nodes_d.rs:160: fn loop_stack(&self) -> &Vec<(String, String)>;
src/lir/ir/nodes_d.rs:161: fn loop_stack_mut(&mut self) -> &mut Vec<(String, String)>;
src/lir/ir/nodes_d.rs:162: fn next_block_label(&mut self, prefix: &str) -> String;
src/lir/ir/nodes_d.rs:163: fn set_current_block(&mut self, label: String);
src/lir/ir/nodes_e.rs:3: impl LirNode for SLirDropValue
src/lir/ir/nodes_e.rs:4: fn clone_node(&self) -> Box<dyn LirNode> { Box::new(self.clone()) }
src/lir/ir/nodes_e.rs:5: fn kind(&self) -> &'static str { "DropValue" }
src/lir/ir/nodes_e.rs:6: fn as_any(&self) -> &dyn std::any::Any { self }
src/lir/ir/nodes_e.rs:7: fn emit(&self, ctx: &mut LirEmitCtx) -> Vec<String>
src/lir/ir/nodes_e.rs:12: fn display(&self, f: &mut dyn Write) -> std::fmt::Result
src/lir/ir/nodes_e.rs:15: fn serialize(&self, buf: &mut Vec<u8>)
src/lir/ir/nodes_e.rs:24: impl LirNode for SLirRefInst
src/lir/ir/nodes_e.rs:25: fn clone_node(&self) -> Box<dyn LirNode> { Box::new(self.clone()) }
src/lir/ir/nodes_e.rs:26: fn kind(&self) -> &'static str { "RefInst" }
src/lir/ir/nodes_e.rs:27: fn as_any(&self) -> &dyn std::any::Any { self }
src/lir/ir/nodes_e.rs:28: fn emit(&self, _ctx: &mut LirEmitCtx) -> Vec<String>
src/lir/ir/nodes_e.rs:31: fn display(&self, f: &mut dyn Write) -> std::fmt::Result
src/lir/ir/nodes_e.rs:35: fn serialize(&self, buf: &mut Vec<u8>)
src/lir/ir/nodes_e.rs:44: impl LirNode for SLirRefTmp
src/lir/ir/nodes_e.rs:45: fn clone_node(&self) -> Box<dyn LirNode> { Box::new(self.clone()) }
src/lir/ir/nodes_e.rs:46: fn kind(&self) -> &'static str { "RefTmp" }
src/lir/ir/nodes_e.rs:47: fn as_any(&self) -> &dyn std::any::Any { self }
src/lir/ir/nodes_e.rs:48: fn emit(&self, ctx: &mut LirEmitCtx) -> Vec<String>
src/lir/ir/nodes_e.rs:58: fn display(&self, f: &mut dyn Write) -> std::fmt::Result
src/lir/ir/nodes_e.rs:62: fn serialize(&self, buf: &mut Vec<u8>)
src/lir/ir/nodes_e.rs:72: impl LirNode for SLirIndexStore
src/lir/ir/nodes_e.rs:73: fn clone_node(&self) -> Box<dyn LirNode> { Box::new(self.clone()) }
src/lir/ir/nodes_e.rs:74: fn kind(&self) -> &'static str { "IndexStore" }
src/lir/ir/nodes_e.rs:75: fn as_any(&self) -> &dyn std::any::Any { self }
src/lir/ir/nodes_e.rs:76: fn emit(&self, ctx: &mut LirEmitCtx) -> Vec<String>
src/lir/ir/nodes_e.rs:99: fn display(&self, f: &mut dyn Write) -> std::fmt::Result
src/lir/ir/nodes_e.rs:102: fn serialize(&self, buf: &mut Vec<u8>)
src/lir/lower/ctx.rs:3: pub(super) struct LowerCtx<'a>
src/lir/lower/ctx.rs:13: impl<'a> LowerCtx<'a>
src/lir/lower/ctx.rs:14: pub(super) fn new(str_map: &'a HashMap<String, u64>) -> Self
src/lir/lower/ctx.rs:26: pub(super) fn next_block_label(&mut self, prefix: &str) -> String
src/lir/lower/ctx.rs:32: pub(super) fn set_current_block(&mut self, label: String)
src/lir/lower/ctx.rs:47: pub(super) fn finish(&mut self) -> Vec<LirBlock>
src/lir/lower/ctx.rs:57: impl LirLowerCtx for LowerCtx<'_>
src/lir/lower/ctx.rs:58: fn next_tmp(&mut self) -> u64
src/lir/lower/ctx.rs:63: fn emit(&mut self, inst: LirNodeBox)
src/lir/lower/ctx.rs:66: fn str_map(&self) -> &HashMap<String, u64>
src/lir/lower/ctx.rs:69: fn loop_stack(&self) -> &Vec<(String, String)>
src/lir/lower/ctx.rs:72: fn loop_stack_mut(&mut self) -> &mut Vec<(String, String)>
src/lir/lower/ctx.rs:75: fn next_block_label(&mut self, prefix: &str) -> String
src/lir/lower/ctx.rs:78: fn set_current_block(&mut self, label: String)
src/lir/lower/fn_lower.rs:4: fn lir_attrs(f: &MirFn) -> Vec<LirAttr>
src/lir/lower/fn_lower.rs:9: fn lir_param_attrs(f: &MirFn) -> Vec<Vec<LirAttr>>
src/lir/lower/fn_lower.rs:13: pub(super) fn lower_items(item: &MirItem, str_map: &HashMap<String, u64>) -> Vec<LirFn>
src/lir/lower/fn_lower.rs:23: pub(super) fn lower_fn(f: &MirFn, str_map: &HashMap<String, u64>) -> LirFn
src/lir/lower/fn_lower.rs:85: pub(super) fn lower_stmts(ctx: &mut LowerCtx, stmts: &[MirStmtBox])
src/lir/lower/fn_lower.rs:111: pub(super) fn lower_expr(ctx: &mut dyn LirLowerCtx, expr: &MirNodeBox) -> LirValue
src/lir/lower/fn_lower.rs:115: pub(super) fn default_ret_value(ty: &HirType) -> Option<(LirValue, HirType)>
src/lir/lower/fn_lower.rs:128: pub(super) fn strip_ownership(ty: HirType) -> HirType
src/lir/lower/fn_lower.rs:135: pub(super) fn type_size(ty: &HirType) -> u64
src/lir/lower/mir_contract.rs:3: impl MirStmtNode for SMirAssumeStmt
src/lir/lower/mir_contract.rs:4: fn clone_stmt(&self) -> Box<dyn MirStmtNode> { Box::new(self.clone()) }
src/lir/lower/mir_contract.rs:5: fn lower_to_lir_stmt(&self, ctx: &mut dyn LirLowerCtx)
src/lir/lower/mir_contract.rs:9: fn display_stmt(&self, level: usize, w: &mut dyn std::fmt::Write) -> std::fmt::Result
src/lir/lower/mir_contract.rs:13: fn for_each_child_expr(&self, f: &mut dyn FnMut(&dyn MirNode))
src/lir/lower/mir_contract.rs:18: impl MirStmtNode for SMirContractStmt
src/lir/lower/mir_contract.rs:19: fn clone_stmt(&self) -> Box<dyn MirStmtNode> { Box::new(self.clone()) }
src/lir/lower/mir_contract.rs:20: fn lower_to_lir_stmt(&self, ctx: &mut dyn LirLowerCtx)
src/lir/lower/mir_contract.rs:26: fn display_stmt(&self, level: usize, w: &mut dyn std::fmt::Write) -> std::fmt::Result
src/lir/lower/mir_contract.rs:30: fn for_each_child_expr(&self, f: &mut dyn FnMut(&dyn MirNode))
src/lir/lower/mir_expr.rs:4: impl MirNode for SMirLocal
src/lir/lower/mir_expr.rs:5: fn clone_node(&self) -> Box<dyn MirNode> { Box::new(self.clone()) }
src/lir/lower/mir_expr.rs:6: fn lower_to_lir(&self, ctx: &mut dyn LirLowerCtx) -> LirValue
src/lir/lower/mir_expr.rs:11: fn display(&self, level: usize, w: &mut dyn std::fmt::Write) -> std::fmt::Result
src/lir/lower/mir_expr.rs:15: fn expr_type(&self) -> HirType { self.ty.clone() }
src/lir/lower/mir_expr.rs:16: fn as_local(&self) -> Option<VarId> { Some(self.var) }
src/lir/lower/mir_expr.rs:17: fn collect_var_ids(&self, vars: &mut std::collections::HashSet<VarId>) { vars.insert(self.var); }
src/lir/lower/mir_expr.rs:20: impl MirNode for SMirLiteral
src/lir/lower/mir_expr.rs:21: fn clone_node(&self) -> Box<dyn MirNode> { Box::new(self.clone()) }
src/lir/lower/mir_expr.rs:22: fn lower_to_lir(&self, ctx: &mut dyn LirLowerCtx) -> LirValue
src/lir/lower/mir_expr.rs:65: fn display(&self, level: usize, w: &mut dyn std::fmt::Write) -> std::fmt::Result
src/lir/lower/mir_expr.rs:75: fn expr_type(&self) -> HirType { self.ty.clone() }
src/lir/lower/mir_expr.rs:76: fn as_string_literal(&self) -> Option<&str>
src/lir/lower/mir_expr.rs:81: impl MirNode for SMirBinary
src/lir/lower/mir_expr.rs:82: fn clone_node(&self) -> Box<dyn MirNode> { Box::new(self.clone()) }
src/lir/lower/mir_expr.rs:83: fn lower_to_lir(&self, ctx: &mut dyn LirLowerCtx) -> LirValue
src/lir/lower/mir_expr.rs:94: fn display(&self, level: usize, w: &mut dyn std::fmt::Write) -> std::fmt::Result
src/lir/lower/mir_expr.rs:102: fn expr_type(&self) -> HirType { self.ty.clone() }
src/lir/lower/mir_expr.rs:103: fn for_each_child(&self, f: &mut dyn FnMut(&dyn MirNode)) { f(&*self.lhs); f(&*self.rhs); }
src/lir/lower/mir_expr.rs:106: impl MirNode for SMirUnary
src/lir/lower/mir_expr.rs:107: fn clone_node(&self) -> Box<dyn MirNode> { Box::new(self.clone()) }
src/lir/lower/mir_expr.rs:108: fn lower_to_lir(&self, ctx: &mut dyn LirLowerCtx) -> LirValue
src/lir/lower/mir_expr.rs:114: fn display(&self, level: usize, w: &mut dyn std::fmt::Write) -> std::fmt::Result
src/lir/lower/mir_expr.rs:119: fn expr_type(&self) -> HirType { self.ty.clone() }
src/lir/lower/mir_expr.rs:120: fn for_each_child(&self, f: &mut dyn FnMut(&dyn MirNode)) { f(&*self.arg); }
src/lir/lower/mir_expr.rs:123: impl MirNode for SMirCall
src/lir/lower/mir_expr.rs:124: fn clone_node(&self) -> Box<dyn MirNode> { Box::new(self.clone()) }
src/lir/lower/mir_expr.rs:125: fn call_fn_id(&self) -> Option<crate::hir::ty::FnId> { Some(self.fn_id) }
src/lir/lower/mir_expr.rs:126: fn lower_to_lir(&self, ctx: &mut dyn LirLowerCtx) -> LirValue
src/lir/lower/mir_expr.rs:138: fn display(&self, level: usize, w: &mut dyn std::fmt::Write) -> std::fmt::Result
src/lir/lower/mir_expr.rs:143: fn expr_type(&self) -> HirType { self.ty.clone() }
src/lir/lower/mir_expr.rs:144: fn for_each_child(&self, f: &mut dyn FnMut(&dyn MirNode)) { for a in &self.args { f(&**a); } }    fn is_call(&self) -> bool { true }
src/lir/lower/mir_expr.rs:148: impl MirNode for SMirMove
src/lir/lower/mir_expr.rs:149: fn move_expr(&self) -> Option<&MirNodeBox> { Some(&self.expr) }
src/lir/lower/mir_expr.rs:150: fn clone_node(&self) -> Box<dyn MirNode> { Box::new(self.clone()) }
src/lir/lower/mir_expr.rs:151: fn lower_to_lir(&self, ctx: &mut dyn LirLowerCtx) -> LirValue { self.expr.lower_to_lir(ctx) }
src/lir/lower/mir_expr.rs:152: fn display(&self, level: usize, w: &mut dyn std::fmt::Write) -> std::fmt::Result
src/lir/lower/mir_expr.rs:157: fn expr_type(&self) -> HirType { self.ty.clone() }
src/lir/lower/mir_expr.rs:158: fn for_each_child(&self, f: &mut dyn FnMut(&dyn MirNode)) { f(&*self.expr); }
src/lir/lower/mir_expr.rs:161: impl MirNode for SMirClone
src/lir/lower/mir_expr.rs:162: fn clone_node(&self) -> Box<dyn MirNode> { Box::new(self.clone()) }
src/lir/lower/mir_expr.rs:163: fn lower_to_lir(&self, ctx: &mut dyn LirLowerCtx) -> LirValue { self.expr.lower_to_lir(ctx) }
src/lir/lower/mir_expr.rs:164: fn display(&self, level: usize, w: &mut dyn std::fmt::Write) -> std::fmt::Result
src/lir/lower/mir_expr.rs:169: fn expr_type(&self) -> HirType { self.ty.clone() }
src/lir/lower/mir_expr.rs:170: fn for_each_child(&self, f: &mut dyn FnMut(&dyn MirNode)) { f(&*self.expr); }
src/lir/lower/mir_expr.rs:173: impl MirNode for SMirToUnique
src/lir/lower/mir_expr.rs:174: fn clone_node(&self) -> Box<dyn MirNode> { Box::new(self.clone()) }
src/lir/lower/mir_expr.rs:175: fn lower_to_lir(&self, ctx: &mut dyn LirLowerCtx) -> LirValue
src/lir/lower/mir_expr.rs:188: fn display(&self, level: usize, w: &mut dyn std::fmt::Write) -> std::fmt::Result
src/lir/lower/mir_expr.rs:193: fn expr_type(&self) -> HirType { self.ty.clone() }
src/lir/lower/mir_expr.rs:194: fn for_each_child(&self, f: &mut dyn FnMut(&dyn MirNode)) { f(&*self.expr); }
src/lir/lower/mir_expr.rs:199: impl MirNode for SMirVirtualCall
src/lir/lower/mir_expr.rs:200: fn clone_node(&self) -> Box<dyn MirNode> { Box::new(self.clone()) }
src/lir/lower/mir_expr.rs:201: fn lower_to_lir(&self, ctx: &mut dyn LirLowerCtx) -> LirValue
src/lir/lower/mir_expr.rs:218: fn display(&self, level: usize, w: &mut dyn std::fmt::Write) -> std::fmt::Result
src/lir/lower/mir_expr.rs:225: fn expr_type(&self) -> HirType { self.ty.clone() }
src/lir/lower/mir_expr.rs:226: fn for_each_child(&self, f: &mut dyn FnMut(&dyn MirNode)) { f(&*self.receiver); for a in &self.args { f(&**a); } }    fn is_call(&self) -> bool { true }
src/lir/lower/mir_expr.rs:230: impl MirNode for SMirMakeFatPtr
src/lir/lower/mir_expr.rs:231: fn clone_node(&self) -> Box<dyn MirNode> { Box::new(self.clone()) }
src/lir/lower/mir_expr.rs:232: fn lower_to_lir(&self, ctx: &mut dyn LirLowerCtx) -> LirValue
src/lir/lower/mir_expr.rs:246: fn display(&self, level: usize, w: &mut dyn std::fmt::Write) -> std::fmt::Result
src/lir/lower/mir_expr.rs:251: fn expr_type(&self) -> HirType { self.ty.clone() }
src/lir/lower/mir_expr.rs:252: fn for_each_child(&self, f: &mut dyn FnMut(&dyn MirNode)) { f(&*self.value); }
src/lir/lower/mir_expr2.rs:5: impl MirNode for SMirEnumConstruct
src/lir/lower/mir_expr2.rs:6: fn clone_node(&self) -> Box<dyn MirNode> { Box::new(self.clone()) }
src/lir/lower/mir_expr2.rs:7: fn lower_to_lir(&self, _ctx: &mut dyn LirLowerCtx) -> LirValue
src/lir/lower/mir_expr2.rs:10: fn display(&self, level: usize, w: &mut dyn std::fmt::Write) -> std::fmt::Result
src/lir/lower/mir_expr2.rs:13: fn expr_type(&self) -> HirType { self.ty.clone() }
src/lir/lower/mir_expr2.rs:14: fn for_each_child(&self, f: &mut dyn FnMut(&dyn MirNode)) { for a in &self.args { f(&**a); } }
src/lir/lower/mir_expr2.rs:17: impl MirNode for SMirFnPtr
src/lir/lower/mir_expr2.rs:18: fn clone_node(&self) -> Box<dyn MirNode> { Box::new(self.clone()) }
src/lir/lower/mir_expr2.rs:19: fn lower_to_lir(&self, ctx: &mut dyn LirLowerCtx) -> LirValue
src/lir/lower/mir_expr2.rs:24: fn display(&self, level: usize, w: &mut dyn std::fmt::Write) -> std::fmt::Result
src/lir/lower/mir_expr2.rs:27: fn expr_type(&self) -> HirType { self.ty.clone() }
src/lir/lower/mir_expr2.rs:30: impl MirNode for SMirCallPtr
src/lir/lower/mir_expr2.rs:31: fn clone_node(&self) -> Box<dyn MirNode> { Box::new(self.clone()) }
src/lir/lower/mir_expr2.rs:32: fn lower_to_lir(&self, ctx: &mut dyn LirLowerCtx) -> LirValue
src/lir/lower/mir_expr2.rs:45: fn display(&self, level: usize, w: &mut dyn std::fmt::Write) -> std::fmt::Result
src/lir/lower/mir_expr2.rs:48: fn expr_type(&self) -> HirType { self.ty.clone() }
src/lir/lower/mir_expr2.rs:49: fn for_each_child(&self, f: &mut dyn FnMut(&dyn MirNode)) { f(&*self.fn_ptr); for a in &self.args { f(&**a); } }    fn is_call(&self) -> bool { true }
src/lir/lower/mir_expr2.rs:53: impl MirNode for SMirEnumMatch
src/lir/lower/mir_expr2.rs:54: fn clone_node(&self) -> Box<dyn MirNode> { Box::new(self.clone()) }
src/lir/lower/mir_expr2.rs:55: fn lower_to_lir(&self, ctx: &mut dyn LirLowerCtx) -> LirValue
src/lir/lower/mir_expr2.rs:88: fn display(&self, level: usize, w: &mut dyn std::fmt::Write) -> std::fmt::Result
src/lir/lower/mir_expr2.rs:94: fn expr_type(&self) -> HirType { self.ty.clone() }
src/lir/lower/mir_expr2.rs:95: fn for_each_child(&self, f: &mut dyn FnMut(&dyn MirNode))
src/lir/lower/mir_expr2.rs:101: impl MirNode for SMirFieldAccess
src/lir/lower/mir_expr2.rs:102: fn clone_node(&self) -> Box<dyn MirNode> { Box::new(self.clone()) }
src/lir/lower/mir_expr2.rs:103: fn lower_to_lir(&self, ctx: &mut dyn LirLowerCtx) -> LirValue
src/lir/lower/mir_expr2.rs:113: fn display(&self, level: usize, w: &mut dyn std::fmt::Write) -> std::fmt::Result
src/lir/lower/mir_expr2.rs:118: fn expr_type(&self) -> HirType { self.ty.clone() }
src/lir/lower/mir_expr2.rs:119: fn for_each_child(&self, f: &mut dyn FnMut(&dyn MirNode)) { f(&*self.object); }
src/lir/lower/mir_expr2.rs:122: impl MirNode for SMirStructLiteral
src/lir/lower/mir_expr2.rs:123: fn clone_node(&self) -> Box<dyn MirNode> { Box::new(self.clone()) }
src/lir/lower/mir_expr2.rs:124: fn struct_literal_fields(&self) -> Option<&[(Symbol, MirNodeBox)]> { Some(&self.fields) }
src/lir/lower/mir_expr2.rs:125: fn lower_to_lir(&self, ctx: &mut dyn LirLowerCtx) -> LirValue
src/lir/lower/mir_expr2.rs:134: fn display(&self, level: usize, w: &mut dyn std::fmt::Write) -> std::fmt::Result
src/lir/lower/mir_expr2.rs:142: fn expr_type(&self) -> HirType { self.ty.clone() }
src/lir/lower/mir_expr2.rs:143: fn for_each_child(&self, f: &mut dyn FnMut(&dyn MirNode)) { for (_, e) in &self.fields { f(&**e); } }
src/lir/lower/mir_expr2.rs:146: impl MirNode for SMirArrayLiteral
src/lir/lower/mir_expr2.rs:147: fn clone_node(&self) -> Box<dyn MirNode> { Box::new(self.clone()) }
src/lir/lower/mir_expr2.rs:148: fn lower_to_lir(&self, ctx: &mut dyn LirLowerCtx) -> LirValue
src/lir/lower/mir_expr2.rs:158: fn display(&self, level: usize, w: &mut dyn std::fmt::Write) -> std::fmt::Result
src/lir/lower/mir_expr2.rs:163: fn expr_type(&self) -> HirType { self.ty.clone() }
src/lir/lower/mir_expr2.rs:164: fn for_each_child(&self, f: &mut dyn FnMut(&dyn MirNode)) { for e in &self.elems { f(&**e); } }
src/lir/lower/mir_expr2.rs:167: impl MirNode for SMirArraySized
src/lir/lower/mir_expr2.rs:168: fn clone_node(&self) -> Box<dyn MirNode> { Box::new(self.clone()) }
src/lir/lower/mir_expr2.rs:169: fn lower_to_lir(&self, ctx: &mut dyn LirLowerCtx) -> LirValue
src/lir/lower/mir_expr2.rs:177: fn display(&self, level: usize, w: &mut dyn std::fmt::Write) -> std::fmt::Result
src/lir/lower/mir_expr2.rs:183: fn expr_type(&self) -> HirType { self.ty.clone() }
src/lir/lower/mir_expr2.rs:184: fn for_each_child(&self, f: &mut dyn FnMut(&dyn MirNode)) { f(&*self.count); }
src/lir/lower/mir_expr2.rs:187: impl MirNode for SMirRef
src/lir/lower/mir_expr2.rs:188: fn clone_node(&self) -> Box<dyn MirNode> { Box::new(self.clone()) }
src/lir/lower/mir_expr2.rs:189: fn lower_to_lir(&self, ctx: &mut dyn LirLowerCtx) -> LirValue
src/lir/lower/mir_expr2.rs:202: fn display(&self, level: usize, w: &mut dyn std::fmt::Write) -> std::fmt::Result
src/lir/lower/mir_expr2.rs:208: fn expr_type(&self) -> HirType { self.ty.clone() }
src/lir/lower/mir_expr2.rs:209: fn for_each_child(&self, f: &mut dyn FnMut(&dyn MirNode)) { f(&*self.expr); }
src/lir/lower/mir_expr2.rs:210: fn as_ref(&self) -> Option<(VarId, bool)>
src/lir/lower/mir_expr2.rs:215: impl MirNode for SMirIndex
src/lir/lower/mir_expr2.rs:216: fn clone_node(&self) -> Box<dyn MirNode> { Box::new(self.clone()) }
src/lir/lower/mir_expr2.rs:217: fn lower_to_lir(&self, ctx: &mut dyn LirLowerCtx) -> LirValue
src/lir/lower/mir_expr2.rs:230: fn display(&self, level: usize, w: &mut dyn std::fmt::Write) -> std::fmt::Result
src/lir/lower/mir_expr2.rs:238: fn expr_type(&self) -> HirType { self.ty.clone() }
src/lir/lower/mir_expr2.rs:239: fn for_each_child(&self, f: &mut dyn FnMut(&dyn MirNode)) { f(&*self.object); f(&*self.index); }
src/lir/lower/mir_expr2.rs:242: impl MirNode for SMirAsm
src/lir/lower/mir_expr2.rs:243: fn clone_node(&self) -> Box<dyn MirNode> { Box::new(self.clone()) }
src/lir/lower/mir_expr2.rs:244: fn lower_to_lir(&self, ctx: &mut dyn LirLowerCtx) -> LirValue
src/lir/lower/mir_expr2.rs:258: fn display(&self, level: usize, w: &mut dyn std::fmt::Write) -> std::fmt::Result
src/lir/lower/mir_expr2.rs:270: fn expr_type(&self) -> HirType { self.ty.clone() }
src/lir/lower/mir_expr2.rs:271: fn for_each_child(&self, f: &mut dyn FnMut(&dyn MirNode))
src/lir/lower/mir_stmts.rs:5: pub(super) fn block_ends_with_ret(stmts: &[MirStmtBox]) -> bool
src/lir/lower/mir_stmts.rs:9: impl MirStmtNode for SMirAssignStmt
src/lir/lower/mir_stmts.rs:10: fn clone_stmt(&self) -> Box<dyn MirStmtNode> { Box::new(self.clone()) }
src/lir/lower/mir_stmts.rs:11: fn lower_to_lir_stmt(&self, ctx: &mut dyn LirLowerCtx)
src/lir/lower/mir_stmts.rs:17: fn display_stmt(&self, level: usize, w: &mut dyn std::fmt::Write) -> std::fmt::Result
src/lir/lower/mir_stmts.rs:25: fn for_each_child_expr(&self, f: &mut dyn FnMut(&dyn MirNode)) { f(&*self.target); f(&*self.value); }    fn assign_parts(&self) -> Option<(&MirNodeBox, &MirNodeBox)> { Some((&self.target, &self.value)) }
src/lir/lower/mir_stmts.rs:29: impl MirStmtNode for SMirFieldAssignStmt
src/lir/lower/mir_stmts.rs:30: fn clone_stmt(&self) -> Box<dyn MirStmtNode> { Box::new(self.clone()) }
src/lir/lower/mir_stmts.rs:31: fn lower_to_lir_stmt(&self, ctx: &mut dyn LirLowerCtx)
src/lir/lower/mir_stmts.rs:49: fn display_stmt(&self, level: usize, w: &mut dyn std::fmt::Write) -> std::fmt::Result
src/lir/lower/mir_stmts.rs:57: fn for_each_child_expr(&self, f: &mut dyn FnMut(&dyn MirNode)) { f(&*self.object); f(&*self.value); }    fn field_assign_parts(&self) -> Option<(&MirNodeBox, &MirNodeBox)> { Some((&self.object, &self.value)) }
src/lir/lower/mir_stmts.rs:61: impl MirStmtNode for SMirIndexAssignStmt
src/lir/lower/mir_stmts.rs:62: fn clone_stmt(&self) -> Box<dyn MirStmtNode> { Box::new(self.clone()) }
src/lir/lower/mir_stmts.rs:63: fn lower_to_lir_stmt(&self, ctx: &mut dyn LirLowerCtx)
src/lir/lower/mir_stmts.rs:76: fn display_stmt(&self, level: usize, w: &mut dyn std::fmt::Write) -> std::fmt::Result
src/lir/lower/mir_stmts.rs:86: fn for_each_child_expr(&self, f: &mut dyn FnMut(&dyn MirNode)) { f(&*self.object); f(&*self.index); f(&*self.value); }    fn index_assign_parts(&self) -> Option<(&MirNodeBox, &MirNodeBox, &MirNodeBox)> { Some((&self.object, &self.index, &self.value)) }
src/lir/lower/mir_stmts.rs:91: impl MirStmtNode for SMirReturnStmt
src/lir/lower/mir_stmts.rs:92: fn clone_stmt(&self) -> Box<dyn MirStmtNode> { Box::new(self.clone()) }
src/lir/lower/mir_stmts.rs:93: fn lower_to_lir_stmt(&self, ctx: &mut dyn LirLowerCtx)
src/lir/lower/mir_stmts.rs:101: fn display_stmt(&self, level: usize, w: &mut dyn std::fmt::Write) -> std::fmt::Result
src/lir/lower/mir_stmts.rs:107: fn for_each_child_expr(&self, f: &mut dyn FnMut(&dyn MirNode))
src/lir/lower/mir_stmts.rs:110: fn is_return(&self) -> bool { true }
src/lir/lower/mir_stmts.rs:111: fn return_value(&self) -> Option<&MirNodeBox> { self.value.as_ref() }
src/lir/lower/mir_stmts.rs:114: impl MirStmtNode for SMirIfStmt
src/lir/lower/mir_stmts.rs:115: fn clone_stmt(&self) -> Box<dyn MirStmtNode> { Box::new(self.clone()) }
src/lir/lower/mir_stmts.rs:116: fn lower_to_lir_stmt(&self, ctx: &mut dyn LirLowerCtx)
src/lir/lower/mir_stmts.rs:133: fn display_stmt(&self, level: usize, w: &mut dyn std::fmt::Write) -> std::fmt::Result
src/lir/lower/mir_stmts.rs:151: fn for_each_child_expr(&self, f: &mut dyn FnMut(&dyn MirNode)) { f(&*self.cond); }
src/lir/lower/mir_stmts.rs:152: fn for_each_child_stmt(&self, f: &mut dyn FnMut(&dyn MirStmtNode))
src/lir/lower/mir_stmts.rs:160: pub(super) fn lower_elifs(ctx: &mut dyn LirLowerCtx, elifs: &[(MirNodeBox, Vec<MirStmtBox>)], else_block: &Option<Vec<MirStmtBox>>, merge_lbl: &str)
src/lir/lower/mir_stmts.rs:181: impl MirStmtNode for SMirWhileStmt
src/lir/lower/mir_stmts.rs:182: fn clone_stmt(&self) -> Box<dyn MirStmtNode> { Box::new(self.clone()) }
src/lir/lower/mir_stmts.rs:183: fn lower_to_lir_stmt(&self, ctx: &mut dyn LirLowerCtx)
src/lir/lower/mir_stmts.rs:203: fn display_stmt(&self, level: usize, w: &mut dyn std::fmt::Write) -> std::fmt::Result
src/lir/lower/mir_stmts.rs:211: fn for_each_child_expr(&self, f: &mut dyn FnMut(&dyn MirNode)) { f(&*self.cond); }
src/lir/lower/mir_stmts.rs:212: fn for_each_child_stmt(&self, f: &mut dyn FnMut(&dyn MirStmtNode)) { for s in &self.body { f(&**s); } }    fn as_while(&self) -> Option<WhileParts<'_>> { Some((&self.cond, &self.body)) }
src/lir/lower/mir_stmts.rs:216: impl MirStmtNode for SMirBreakStmt
src/lir/lower/mir_stmts.rs:217: fn clone_stmt(&self) -> Box<dyn MirStmtNode> { Box::new(self.clone()) }
src/lir/lower/mir_stmts.rs:218: fn lower_to_lir_stmt(&self, ctx: &mut dyn LirLowerCtx)
src/lir/lower/mir_stmts.rs:223: fn display_stmt(&self, level: usize, w: &mut dyn std::fmt::Write) -> std::fmt::Result
src/lir/lower/mir_stmts.rs:229: impl MirStmtNode for SMirContinueStmt
src/lir/lower/mir_stmts.rs:230: fn clone_stmt(&self) -> Box<dyn MirStmtNode> { Box::new(self.clone()) }
src/lir/lower/mir_stmts.rs:231: fn lower_to_lir_stmt(&self, ctx: &mut dyn LirLowerCtx)
src/lir/lower/mir_stmts.rs:236: fn display_stmt(&self, level: usize, w: &mut dyn std::fmt::Write) -> std::fmt::Result
src/lir/lower/mir_stmts.rs:242: impl MirStmtNode for SMirExprStmt
src/lir/lower/mir_stmts.rs:243: fn clone_stmt(&self) -> Box<dyn MirStmtNode> { Box::new(self.clone()) }
src/lir/lower/mir_stmts.rs:244: fn lower_to_lir_stmt(&self, ctx: &mut dyn LirLowerCtx)
src/lir/lower/mir_stmts.rs:247: fn display_stmt(&self, level: usize, w: &mut dyn std::fmt::Write) -> std::fmt::Result
src/lir/lower/mir_stmts.rs:252: fn for_each_child_expr(&self, f: &mut dyn FnMut(&dyn MirNode)) { f(&*self.expr); }    fn expr_part(&self) -> Option<&MirNodeBox> { Some(&self.expr) }
src/lir/lower/mir_stmts.rs:256: impl MirStmtNode for SMirBlockStmt
src/lir/lower/mir_stmts.rs:257: fn clone_stmt(&self) -> Box<dyn MirStmtNode> { Box::new(self.clone()) }
src/lir/lower/mir_stmts.rs:258: fn lower_to_lir_stmt(&self, ctx: &mut dyn LirLowerCtx)
src/lir/lower/mir_stmts.rs:261: fn display_stmt(&self, level: usize, w: &mut dyn std::fmt::Write) -> std::fmt::Result
src/lir/lower/mir_stmts.rs:267: fn for_each_child_stmt(&self, f: &mut dyn FnMut(&dyn MirStmtNode)) { for s in &self.stmts { f(&**s); } }    fn as_block(&self) -> Option<&[MirStmtBox]> { Some(&self.stmts) }
src/lir/lower/mir_stmts.rs:271: impl MirStmtNode for SMirDropStmt
src/lir/lower/mir_stmts.rs:272: fn clone_stmt(&self) -> Box<dyn MirStmtNode> { Box::new(self.clone()) }
src/lir/lower/mir_stmts.rs:273: fn lower_to_lir_stmt(&self, ctx: &mut dyn LirLowerCtx)
src/lir/lower/mir_stmts.rs:276: fn display_stmt(&self, level: usize, w: &mut dyn std::fmt::Write) -> std::fmt::Result
src/lir/lower/mir_stmts.rs:279: fn as_drop(&self) -> Option<(VarId, &HirType)> { Some((self.var, &self.ty)) }
src/lir/lower/mod.rs:10: mod ctx;
src/lir/lower/mod.rs:11: mod fn_lower;
src/lir/lower/mod.rs:12: mod mir_expr;
src/lir/lower/mod.rs:13: mod mir_expr2;
src/lir/lower/mod.rs:14: mod mir_contract;
src/lir/lower/mod.rs:15: mod mir_stmts;
src/lir/lower/mod.rs:16: mod names;
src/lir/lower/mod.rs:17: mod strings;
src/lir/lower/mod.rs:18: mod util;
src/lir/lower/mod.rs:21: fn collect_effect_summaries(items: &[MirItem], out: &mut HashMap<String, crate::hir::effects::EffectSummary>)
src/lir/lower/mod.rs:41: pub fn lower_program(mir: &MirProgram) -> LirProgram
src/lir/lower/names.rs:3: pub(super) fn collect_fn_names(mir: &MirProgram) -> HashMap<FnId, String>
src/lir/lower/names.rs:9: pub(super) fn collect_fn_names_items(items: &[MirItem], _prefix: &str, map: &mut HashMap<FnId, String>)
src/lir/lower/names.rs:31: pub(super) fn mangle(prefix: &str, name: &str, params: &[(crate::intern::Symbol, HirType)]) -> String
src/lir/lower/names.rs:49: pub(super) fn type_to_mangle(ty: &HirType) -> String
src/lir/lower/strings.rs:3: pub(super) fn collect_strings(mir: &MirProgram) -> Vec<String>
src/lir/lower/strings.rs:11: pub(super) fn collect_strings_items(items: &[MirItem], out: &mut Vec<String>)
src/lir/lower/strings.rs:21: pub(super) fn collect_strings_stmts(stmts: &[MirStmtBox], out: &mut Vec<String>)
src/lir/lower/strings.rs:29: pub(super) fn collect_strings_dyn(node: &dyn MirNode, out: &mut Vec<String>)
src/lir/lower/util.rs:3: pub(super) fn write_stmt_block(stmts: &[MirStmtBox], level: usize, w: &mut dyn std::fmt::Write) -> std::fmt::Result
src/lir/lower/util.rs:11: pub(super) fn attr_arg_to_string(arg: &crate::parser::ast::AttrArg) -> String
src/lir/lower/util.rs:21: pub(super) fn lir_effects(
src/lir/lower/util.rs:35: pub(super) fn attrs_to_lir(attrs: &[crate::parser::ast::Attr]) -> Vec<LirAttr>
src/lir/lower/util.rs:43: pub(super) fn display_hir_type(ty: &HirType) -> String
src/lir/lower/util.rs:47: pub(super) fn extract_var(val: &LirValue) -> VarId
src/lir/mod.rs:12: pub mod ir;
src/lir/mod.rs:13: pub mod lower;
src/lir/mod.rs:14: pub mod display;
src/lir/mod.rs:15: pub mod emit;
src/lir/mod.rs:16: pub mod serialize;
src/lir/serialize/decode.rs:3: impl<'a> Reader<'a>
src/lir/serialize/decode.rs:4: pub(super) fn literal(&mut self) -> Result<HirLiteral>
src/lir/serialize/decode.rs:15: pub(super) fn value(&mut self) -> Result<LirValue>
src/lir/serialize/decode.rs:25: pub(super) fn inst(&mut self) -> Result<LirNodeBox>
src/lir/serialize/decode.rs:204: pub(super) fn read_fn(&mut self) -> Result<LirFn>
src/lir/serialize/mod.rs:11: pub fn program_to_bytes(p: &LirProgram) -> Vec<u8>
src/lir/serialize/mod.rs:94: pub fn program_from_bytes(data: &[u8]) -> Result<LirProgram>
src/lir/serialize/mod.rs:206: struct Reader<'a>
src/lir/serialize/mod.rs:211: mod decode;
src/lir/serialize/mod.rs:212: mod reader;
src/lir/serialize/mod.rs:213: mod write;
src/lir/serialize/reader.rs:3: impl<'a> Reader<'a>
src/lir/serialize/reader.rs:4: pub(super) fn read(&mut self, n: usize) -> Result<&'a [u8]>
src/lir/serialize/reader.rs:12: pub(super) fn u32(&mut self) -> Result<u32>
src/lir/serialize/reader.rs:16: pub(super) fn u64(&mut self) -> Result<u64>
src/lir/serialize/reader.rs:20: pub(super) fn str(&mut self) -> Result<String>
src/lir/serialize/reader.rs:25: pub(super) fn ty(&mut self) -> Result<HirType>
src/lir/serialize/write.rs:7: pub(super) fn put_inst(buf: &mut Vec<u8>, inst: &LirNodeBox)
src/lir/serialize/write.rs:11: pub(super) fn put_fn(buf: &mut Vec<u8>, f: &LirFn)
src/main.rs:19: fn main()
src/main.rs:56: pub(crate) fn find_project(dir: &Path) -> Option<(PathBuf, String)>
src/main.rs:71: pub(crate) fn project_entry(project_dir: &Path) -> Option<PathBuf>
src/main.rs:80: pub(crate) fn resolve_path(arg: Option<&str>) -> PathBuf
src/main.rs:108: mod cli;
src/main.rs:112: pub(crate) fn load_config() -> Option<(PathBuf, ayanami::package::config::ProjectConfig)>
src/mir/borrow/cfg.rs:5: pub enum Payload<'a>
src/mir/borrow/cfg.rs:14: pub struct Node<'a>
src/mir/borrow/cfg.rs:25: pub struct Cfg<'a>
src/mir/borrow/cfg.rs:30: pub fn build(body: &[MirStmtBox]) -> Cfg<'_>
src/mir/borrow/cfg.rs:37: struct Builder<'a>
src/mir/borrow/cfg.rs:41: impl<'a> Builder<'a>
src/mir/borrow/cfg.rs:42: fn push(&mut self, payload: Payload<'a>, succ: Vec<usize>) -> usize
src/mir/borrow/cfg.rs:56: fn seq(&mut self, stmts: &'a [MirStmtBox], next: usize, loops: &[(usize, usize)]) -> usize
src/mir/borrow/cfg.rs:64: fn stmt(&mut self, s: &'a dyn MirStmtNode, next: usize, loops: &[(usize, usize)]) -> usize
src/mir/borrow/cfg.rs:105: fn stmt_io(s: &dyn MirStmtNode, uses: &mut HashSet<VarId>) -> (HashSet<VarId>, HashSet<VarId>)
src/mir/borrow/collect.rs:11: pub(super) fn collect(
src/mir/borrow/follow.rs:12: pub fn match_source(
src/mir/borrow/follow.rs:53: pub struct FollowInfo
src/mir/borrow/follow.rs:58: pub fn build_table(mir: &MirProgram) -> HashMap<FnId, FollowInfo>
src/mir/borrow/follow.rs:64: fn collect_items(items: &[MirItem], out: &mut HashMap<FnId, FollowInfo>)
src/mir/borrow/follow.rs:89: pub fn source_names(mir_fn: &MirFn) -> HashMap<VarId, Symbol>
src/mir/borrow/liveness.rs:6: pub fn live_in(cfg: &cfg::Cfg) -> Vec<HashSet<VarId>>
src/mir/borrow/loans.rs:8: pub(super) struct Loan
src/mir/borrow/loans.rs:20: pub fn check_fn(mir_fn: &MirFn, ref_params: &[(VarId, bool)], table: &super::FollowTable) -> Result<()>
src/mir/borrow/mod.rs:14: mod cfg;
src/mir/borrow/mod.rs:15: mod collect;
src/mir/borrow/mod.rs:16: mod follow;
src/mir/borrow/mod.rs:17: mod liveness;
src/mir/borrow/mod.rs:18: mod loans;
src/mir/borrow/mod.rs:19: mod walk;
src/mir/borrow/mod.rs:28: pub type FollowTable = HashMap<crate::hir::ty::FnId, follow::FollowInfo>;
src/mir/borrow/mod.rs:31: pub fn build_follow_table(mir: &MirProgram) -> FollowTable
src/mir/borrow/mod.rs:35: pub fn check_borrows(mir_fn: &MirFn, table: &FollowTable) -> Result<()>
src/mir/borrow/walk.rs:6: pub(super) fn var_name(mir_fn: &MirFn, v: VarId) -> String
src/mir/borrow/walk.rs:15: pub(super) fn walk_stmt(
src/mir/borrow/walk.rs:46: fn walk_expr(
src/mir/display.rs:7: pub fn display_mir_program(program: &MirProgram)
src/mir/display.rs:11: pub fn mir_program_to_string(program: &MirProgram) -> String
src/mir/display.rs:20: fn pad(n: usize) -> String
src/mir/display.rs:24: fn write_item(item: &MirItem, level: usize, w: &mut impl Write) -> std::fmt::Result
src/mir/display.rs:60: fn write_stmts(stmts: &[MirStmtBox], level: usize, w: &mut impl Write) -> std::fmt::Result
src/mir/ir.rs:10: pub trait MirNode: std::fmt::Debug
src/mir/ir.rs:11: fn clone_node(&self) -> Box<dyn MirNode>;
src/mir/ir.rs:12: fn lower_to_lir(&self, ctx: &mut dyn LirLowerCtx) -> LirValue;
src/mir/ir.rs:13: fn display(&self, level: usize, w: &mut dyn FmtWrite) -> std::fmt::Result;
src/mir/ir.rs:14: fn expr_type(&self) -> HirType;
src/mir/ir.rs:15: fn as_local(&self) -> Option<VarId> { None }
src/mir/ir.rs:17: fn collect_var_ids(&self, vars: &mut HashSet<VarId>)
src/mir/ir.rs:20: fn record_moves(&self, moved: &mut HashSet<VarId>)
src/mir/ir.rs:23: fn for_each_child(&self, f: &mut dyn FnMut(&dyn MirNode)) {}
src/mir/ir.rs:24: fn as_string_literal(&self) -> Option<&str> { None }
src/mir/ir.rs:25: fn as_ref(&self) -> Option<(VarId, bool)> { None }
src/mir/ir.rs:27: fn is_call(&self) -> bool { false }
src/mir/ir.rs:29: fn struct_literal_fields(&self) -> Option<&[(Symbol, MirNodeBox)]> { None }
src/mir/ir.rs:31: fn move_expr(&self) -> Option<&MirNodeBox> { None }
src/mir/ir.rs:33: fn call_fn_id(&self) -> Option<crate::hir::ty::FnId> { None }
src/mir/ir.rs:37: pub struct MirNodeBox(pub Box<dyn MirNode>);
src/mir/ir.rs:38: impl Clone for MirNodeBox
src/mir/ir.rs:39: fn clone(&self) -> Self { MirNodeBox(self.0.clone_node()) }
src/mir/ir.rs:41: impl std::ops::Deref for MirNodeBox
src/mir/ir.rs:42: type Target = dyn MirNode;
src/mir/ir.rs:43: fn deref(&self) -> &Self::Target { &*self.0 }
src/mir/ir.rs:46: pub trait MirStmtNode: std::fmt::Debug
src/mir/ir.rs:47: fn clone_stmt(&self) -> Box<dyn MirStmtNode>;
src/mir/ir.rs:48: fn lower_to_lir_stmt(&self, ctx: &mut dyn LirLowerCtx);
src/mir/ir.rs:49: fn display_stmt(&self, level: usize, w: &mut dyn FmtWrite) -> std::fmt::Result;
src/mir/ir.rs:50: fn for_each_child_expr(&self, f: &mut dyn FnMut(&dyn MirNode)) {}
src/mir/ir.rs:51: fn for_each_child_stmt(&self, f: &mut dyn FnMut(&dyn MirStmtNode)) {}
src/mir/ir.rs:52: fn is_return(&self) -> bool { false }
src/mir/ir.rs:53: fn return_value(&self) -> Option<&MirNodeBox> { None }
src/mir/ir.rs:54: fn as_drop(&self) -> Option<(VarId, &HirType)> { None }
src/mir/ir.rs:56: fn as_if(&self) -> Option<IfParts<'_>> { None }
src/mir/ir.rs:57: fn as_while(&self) -> Option<WhileParts<'_>> { None }
src/mir/ir.rs:58: fn as_block(&self) -> Option<&[MirStmtBox]> { None }
src/mir/ir.rs:59: fn is_break(&self) -> bool { false }
src/mir/ir.rs:60: fn is_continue(&self) -> bool { false }
src/mir/ir.rs:61: fn assign_parts(&self) -> Option<(&MirNodeBox, &MirNodeBox)> { None }
src/mir/ir.rs:62: fn field_assign_parts(&self) -> Option<(&MirNodeBox, &MirNodeBox)> { None }
src/mir/ir.rs:63: fn index_assign_parts(&self) -> Option<(&MirNodeBox, &MirNodeBox, &MirNodeBox)> { None }
src/mir/ir.rs:64: fn expr_part(&self) -> Option<&MirNodeBox> { None }
src/mir/ir.rs:67: pub type IfParts<'a> = (&'a MirNodeBox, &'a [MirStmtBox], &'a [(MirNodeBox, Vec<MirStmtBox>)], &'a Option<Vec<MirStmtBox>>);
src/mir/ir.rs:68: pub type WhileParts<'a> = (&'a MirNodeBox, &'a [MirStmtBox]);
src/mir/ir.rs:71: pub struct MirStmtBox(pub Box<dyn MirStmtNode>);
src/mir/ir.rs:72: impl Clone for MirStmtBox
src/mir/ir.rs:73: fn clone(&self) -> Self { MirStmtBox(self.0.clone_stmt()) }
src/mir/ir.rs:75: impl std::ops::Deref for MirStmtBox
src/mir/ir.rs:76: type Target = dyn MirStmtNode;
src/mir/ir.rs:77: fn deref(&self) -> &Self::Target { &*self.0 }
src/mir/ir.rs:83: pub struct $name { $(pub $field: $ty),* }
src/mir/ir.rs:91: pub struct $name { $(pub $field: $ty),* }
src/mir/ir.rs:137: fn from(v: $ty) -> Self { MirNodeBox(Box::new(v) as Box<dyn MirNode>) }
src/mir/ir.rs:146: fn from(v: $ty) -> Self { MirStmtBox(Box::new(v) as Box<dyn MirStmtNode>) }
src/mir/ir.rs:153: pub struct MirLocal
src/mir/ir.rs:159: impl MirLocal
src/mir/ir.rs:160: pub fn new(name: Symbol, ty: HirType, mutable: bool) -> Self
src/mir/ir.rs:166: pub struct MirFn
src/mir/ir.rs:190: pub enum MirItem
src/mir/ir.rs:203: pub struct MirProgram
src/mir/lower/checks.rs:5: fn collect_stmt_var_ids(stmt: &HirStmt, vars: &mut HashSet<VarId>)
src/mir/lower/checks.rs:57: impl Ctx
src/mir/lower/checks.rs:60: pub(super) fn check_use_after_move(&mut self, stmt: &HirStmt)
src/mir/lower/ctx.rs:5: impl Ctx
src/mir/lower/ctx.rs:6: pub(super) fn new(f: &HirFn, struct_defs: &HashMap<Symbol, Vec<(Symbol, HirType)>>) -> Self
src/mir/lower/ctx.rs:31: fn emit_assign_cleanup(&self, var: &VarId) -> Vec<MirStmtBox>
src/mir/lower/ctx.rs:43: pub(super) fn lower_stmt(&mut self, stmt: &HirStmt) -> Vec<MirStmtBox>
src/mir/lower/ctx.rs:90: fn track_stmt_moves(&mut self, stmt: &HirStmt)
src/mir/lower/ctx.rs:115: fn lower_assign(&mut self, target: &HirNodeBox, value: &HirNodeBox) -> Vec<MirStmtBox>
src/mir/lower/ctx.rs:159: fn lower_return(&mut self, value: &Option<HirNodeBox>) -> Vec<MirStmtBox>
src/mir/lower/ctx.rs:195: fn new_temp(&mut self, ty: HirType) -> VarId
src/mir/lower/ctx.rs:202: fn lower_if(
src/mir/lower/ctx.rs:226: fn lower_while(&mut self, cond: &HirNodeBox, body: &HirBlock) -> Vec<MirStmtBox>
src/mir/lower/ctx.rs:233: fn lower_block(&mut self, stmts: &[HirStmt]) -> Vec<MirStmtBox>
src/mir/lower/ctx.rs:262: fn mark_alive(&mut self, var: VarId)
src/mir/lower/functions.rs:5: pub(super) fn lower_item(item: &HirItem, struct_defs: &HashMap<Symbol, Vec<(Symbol, HirType)>>) -> Result<Vec<MirItem>>
src/mir/lower/functions.rs:23: fn lower_fn(f: &HirFn, struct_defs: &HashMap<Symbol, Vec<(Symbol, HirType)>>) -> Result<MirFn>
src/mir/lower/mem.rs:6: fn needs_drop(ty: &HirType, struct_defs: &HashMap<Symbol, Vec<(Symbol, HirType)>>) -> bool
src/mir/lower/mem.rs:18: pub(super) fn strategy_for(ty: &HirType, struct_defs: &HashMap<Symbol, Vec<(Symbol, HirType)>>) -> Box<dyn MemStrategy>
src/mir/lower/mem.rs:25: pub(super) fn action_to_stmt(_var: VarId, ty: &HirType, action: &MemAction) -> MirStmtBox
src/mir/lower/mod.rs:9: mod checks;
src/mir/lower/mod.rs:10: mod ctx;
src/mir/lower/mod.rs:11: mod functions;
src/mir/lower/mod.rs:12: mod mem;
src/mir/lower/mod.rs:16: pub fn lower_program(hir: &HirProgram) -> crate::error::Result<MirProgram>
src/mir/lower/mod.rs:41: struct Ctx
src/mir/mem/drop.rs:6: pub struct DropStrategy;
src/mir/mem/drop.rs:8: impl MemStrategy for DropStrategy
src/mir/mem/drop.rs:9: fn on_scope_end(&self, var: VarId, _ty: &HirType) -> Vec<MemAction>
src/mir/mem/drop.rs:12: fn on_move_out(&self, _var: VarId, _ty: &HirType) -> Vec<MemAction>
src/mir/mem/drop.rs:15: fn on_clone(&self, _var: VarId, _ty: &HirType) -> Vec<MemAction>
src/mir/mem/drop.rs:18: fn on_assign_overwrite(&self, var: VarId, _ty: &HirType) -> Vec<MemAction>
src/mir/mem/mod.rs:4: pub enum MemAction
src/mir/mem/mod.rs:8: pub trait MemStrategy
src/mir/mem/mod.rs:9: fn on_scope_end(&self, var: VarId, ty: &HirType) -> Vec<MemAction>;
src/mir/mem/mod.rs:10: fn on_move_out(&self, var: VarId, ty: &HirType) -> Vec<MemAction>;
src/mir/mem/mod.rs:11: fn on_clone(&self, var: VarId, ty: &HirType) -> Vec<MemAction>;
src/mir/mem/mod.rs:12: fn on_assign_overwrite(&self, var: VarId, ty: &HirType) -> Vec<MemAction>;
src/mir/mem/mod.rs:15: pub mod drop;
src/mir/mem/mod.rs:16: pub mod value;
src/mir/mem/value.rs:4: pub struct ValueStrategy;
src/mir/mem/value.rs:6: impl MemStrategy for ValueStrategy
src/mir/mem/value.rs:7: fn on_scope_end(&self, _var: VarId, _ty: &HirType) -> Vec<MemAction>
src/mir/mem/value.rs:10: fn on_move_out(&self, _var: VarId, _ty: &HirType) -> Vec<MemAction>
src/mir/mem/value.rs:13: fn on_clone(&self, _var: VarId, _ty: &HirType) -> Vec<MemAction>
src/mir/mem/value.rs:16: fn on_assign_overwrite(&self, _var: VarId, _ty: &HirType) -> Vec<MemAction>
src/mir/mod.rs:7: pub mod ir;
src/mir/mod.rs:8: pub mod lower;
src/mir/mod.rs:9: pub mod display;
src/mir/mod.rs:10: pub mod mem;
src/mir/mod.rs:11: pub mod borrow;
src/package/bytes.rs:3: impl Package
src/package/bytes.rs:6: pub fn to_bytes(&self) -> Vec<u8>
src/package/bytes.rs:69: pub fn write_to_file(&self, path: &str) -> Result<()>
src/package/config.rs:5: pub struct ProjectConfig
src/package/config.rs:13: impl ProjectConfig
src/package/config.rs:14: pub fn load(toml_content: &str) -> Self
src/package/config.rs:60: pub fn resolve_import<'a>(&'a self, import_path: &str, base_dir: &Path) -> Option<String>
src/package/config.rs:77: pub fn resolve_target(&self, file_path: &Path) -> &str
src/package/load.rs:3: pub fn load_package(path: &str) -> Result<(Vec<ImportedSymbol>, Vec<String>, Vec<u8>, Vec<TargetType>)>
src/package/load.rs:92: fn parse_ini_value(s: &str) -> String
src/package/load.rs:101: pub(super) fn type_to_string(ty: &Type) -> String
src/package/mod.rs:1: pub mod config;
src/package/mod.rs:6: mod bytes;
src/package/mod.rs:7: mod load;
src/package/mod.rs:8: mod symbols;
src/package/mod.rs:9: mod target;
src/package/mod.rs:10: mod types;
src/package/symbols.rs:4: impl Package
src/package/symbols.rs:5: pub fn new(name: String, version: String) -> Self
src/package/symbols.rs:20: pub fn set_effect_summaries(
src/package/symbols.rs:27: pub fn collect_symbols(&mut self, stmts: &[Stmt])
src/package/symbols.rs:32: pub fn collect_all_symbols(&mut self, stmts: &[Stmt])
src/package/symbols.rs:36: fn collect_symbols_with_prefix(&mut self, stmts: &[Stmt], all: bool, ns_prefix: &str)
src/package/symbols.rs:43: fn collect_stmt_symbols(&mut self, stmt: &Stmt, all: bool, ns_prefix: &str)
src/package/target.rs:6: pub enum TargetType
src/package/target.rs:12: impl TargetType
src/package/target.rs:13: pub fn as_str(&self) -> &'static str
src/package/target.rs:21: pub fn from_str(s: &str) -> Option<Self>
src/package/types.rs:5: pub struct Package
src/package/types.rs:17: pub enum PackageSymbol
src/package/types.rs:41: pub enum ImportedSymbol
src/parser/ast/binary_op.rs:2: pub enum BinaryOp
src/parser/ast/block.rs:5: pub struct Block
src/parser/ast/block.rs:10: impl Block
src/parser/ast/block.rs:11: pub fn new(stmts: Vec<Stmt>, span: Span) -> Self
src/parser/ast/expr.rs:9: pub enum Expr
src/parser/ast/expr.rs:93: impl Expr
src/parser/ast/expr.rs:94: pub fn span(&self) -> Span
src/parser/ast/literal.rs:4: pub enum Literal
src/parser/ast/literal.rs:12: impl Literal
src/parser/ast/literal.rs:13: pub fn span(&self) -> Span
src/parser/ast/mod.rs:1: pub mod binary_op;
src/parser/ast/mod.rs:2: pub mod block;
src/parser/ast/mod.rs:3: pub mod expr;
src/parser/ast/mod.rs:4: pub mod literal;
src/parser/ast/mod.rs:5: pub mod program;
src/parser/ast/mod.rs:6: pub mod stmt;
src/parser/ast/mod.rs:7: pub mod ty;
src/parser/ast/mod.rs:8: pub mod unary_op;
src/parser/ast/mod.rs:9: pub mod vis;
src/parser/ast/program.rs:4: pub struct Program
src/parser/ast/program.rs:8: impl Program
src/parser/ast/program.rs:9: pub fn new(stmts: Vec<Stmt>) -> Self
src/parser/ast/stmt.rs:10: pub struct Attr
src/parser/ast/stmt.rs:18: impl Attr
src/parser/ast/stmt.rs:20: pub fn is_builtin(&self) -> bool
src/parser/ast/stmt.rs:26: pub fn path_str(&self) -> String
src/parser/ast/stmt.rs:36: pub enum AttrArg
src/parser/ast/stmt.rs:42: pub struct InterfaceMethod
src/parser/ast/stmt.rs:51: pub enum EnumFields
src/parser/ast/stmt.rs:58: pub struct EnumVariant
src/parser/ast/stmt.rs:64: pub struct MatchArm
src/parser/ast/stmt.rs:71: pub enum Stmt
src/parser/ast/stmt.rs:195: impl Stmt
src/parser/ast/stmt.rs:196: pub fn span(&self) -> Span
src/parser/ast/ty.rs:5: pub enum Type
src/parser/ast/ty.rs:22: impl Type
src/parser/ast/ty.rs:23: pub fn span(&self) -> Span
src/parser/ast/ty.rs:41: pub fn inner(&self) -> Option<&Type>
src/parser/ast/unary_op.rs:2: pub enum UnaryOp
src/parser/ast/vis.rs:2: pub enum Visibility
src/parser/ast/vis.rs:8: impl Visibility
src/parser/ast/vis.rs:9: pub fn is_public(&self) -> bool
src/parser/gen_bridge/expr.rs:3: pub fn node_to_expr(node: &Node) -> Result<Expr>
src/parser/gen_bridge/mod.rs:10: pub fn node_to_program(node: &Node) -> Result<crate::parser::ast::block::Block>
src/parser/gen_bridge/mod.rs:20: mod expr;
src/parser/gen_bridge/mod.rs:21: mod stmt;
src/parser/gen_bridge/mod.rs:22: mod ty;
src/parser/gen_bridge/mod.rs:27: fn get_str(node: &Node, field: &str) -> Result<String>
src/parser/gen_bridge/mod.rs:32: fn str_to_binop(s: &str) -> Result<BinaryOp>
src/parser/gen_bridge/mod.rs:44: fn params_from_node(node: &Node) -> Result<Vec<(Symbol, Type)>>
src/parser/gen_bridge/mod.rs:54: fn block_from_node(node: &Node) -> Result<crate::parser::ast::block::Block>
src/parser/gen_bridge/mod.rs:59: fn default_span() -> Span
src/parser/gen_bridge/stmt.rs:3: pub fn node_to_stmt(node: &Node) -> Result<Stmt>
src/parser/gen_bridge/ty.rs:3: pub fn type_from_node(node: &Node) -> Result<Type>
src/parser/mod.rs:7: pub mod parser;
src/parser/mod.rs:8: pub mod ast;
src/parser/mod.rs:9: pub mod gen_bridge;
src/parser/mod.rs:17: pub fn parse_source(source: &str) -> Result<crate::parser::ast::program::Program>
src/parser/mod.rs:34: fn fallback_parse(source: &str) -> Result<crate::parser::ast::program::Program>
src/parser/parser/atom.rs:3: impl Parser
src/parser/parser/atom.rs:4: pub(super) fn parse_atom(&mut self) -> Result<Expr>
src/parser/parser/core.rs:3: impl Parser
src/parser/parser/core.rs:4: pub fn new(tokens: Vec<Token>) -> Self
src/parser/parser/core.rs:8: pub(super) fn peek(&self) -> Option<&Token>
src/parser/parser/core.rs:12: pub(super) fn advance(&mut self) -> Option<Token>
src/parser/parser/core.rs:18: pub(super) fn error(&self, msg: &str) -> Error
src/parser/parser/core.rs:26: pub(super) fn expect_keyword(&mut self, kw: Keyword) -> Result<()>
src/parser/parser/core.rs:37: pub(super) fn expect_delimiter(&mut self, d: Delimiter) -> Result<()>
src/parser/parser/core.rs:48: pub(super) fn expect_operator(&mut self, op: &str) -> Result<()>
src/parser/parser/core.rs:60: pub(super) fn is_stmt_only_keyword(kind: &TokenKind) -> bool
src/parser/parser/core.rs:71: pub(super) fn is_stmt_start(kind: &TokenKind) -> bool
src/parser/parser/core.rs:84: pub(super) fn try_semicolon(&mut self) -> Result<()>
src/parser/parser/core.rs:95: pub(super) fn parse_visibility(&mut self) -> Visibility
src/parser/parser/core.rs:116: pub(super) fn expect_identifier(&mut self) -> Result<String>
src/parser/parser/core.rs:134: pub fn parse_program(&mut self) -> Result<Program>
src/parser/parser/core.rs:142: pub(super) fn parse_attr_list(&mut self) -> Result<Vec<crate::parser::ast::Attr>>
src/parser/parser/core.rs:194: fn parse_attr_arg(&mut self) -> Result<crate::parser::ast::AttrArg>
src/parser/parser/decl.rs:3: impl Parser
src/parser/parser/decl.rs:5: pub(super) fn parse_fn_decl(&mut self, vis: Visibility, is_inline: bool, extern_c: bool, attrs: Vec<crate::parser::ast::Attr>) -> Result<Stmt>
src/parser/parser/decl.rs:80: pub(super) fn parse_return(&mut self) -> Result<Stmt>
src/parser/parser/decl.rs:96: pub(super) fn parse_if(&mut self) -> Result<Stmt>
src/parser/parser/decl.rs:128: pub(super) fn parse_for(&mut self) -> Result<Stmt>
src/parser/parser/decl.rs:155: pub(super) fn parse_while(&mut self) -> Result<Stmt>
src/parser/parser/decl.rs:165: pub(super) fn parse_match_stmt(&mut self) -> Result<Stmt>
src/parser/parser/decl.rs:196: pub(super) fn parse_namespace(&mut self, vis: Visibility) -> Result<Stmt>
src/parser/parser/decl.rs:219: pub(super) fn parse_struct_def(&mut self, vis: Visibility, attrs: Vec<crate::parser::ast::Attr>) -> Result<Stmt>
src/parser/parser/enum_iface.rs:3: impl Parser
src/parser/parser/enum_iface.rs:6: pub(super) fn parse_enum_def(&mut self, vis: Visibility, attrs: Vec<crate::parser::ast::Attr>) -> Result<Stmt>
src/parser/parser/enum_iface.rs:74: pub(super) fn parse_interface_def(&mut self, attrs: Vec<crate::parser::ast::Attr>) -> Result<Stmt>
src/parser/parser/enum_iface.rs:118: pub(super) fn parse_interface_method(&mut self) -> Result<InterfaceMethod>
src/parser/parser/expr.rs:3: impl Parser
src/parser/parser/expr.rs:6: pub(super) fn parse_expr(&mut self) -> Result<Expr>
src/parser/parser/expr.rs:10: pub(super) fn parse_or(&mut self) -> Result<Expr>
src/parser/parser/expr.rs:26: pub(super) fn parse_and(&mut self) -> Result<Expr>
src/parser/parser/expr.rs:42: pub(super) fn parse_compare(&mut self) -> Result<Expr>
src/parser/parser/expr.rs:75: pub(super) fn parse_sum(&mut self) -> Result<Expr>
src/parser/parser/expr.rs:104: pub(super) fn parse_product(&mut self) -> Result<Expr>
src/parser/parser/impls.rs:3: impl Parser
src/parser/parser/impls.rs:6: pub(super) fn parse_import(&mut self) -> Result<Stmt>
src/parser/parser/impls.rs:35: pub(super) fn parse_impl_block(&mut self, attrs: Vec<crate::parser::ast::Attr>) -> Result<Stmt>
src/parser/parser/impls.rs:61: fn extract_type_name(ty: &Type) -> Symbol
src/parser/parser/impls.rs:98: pub(super) fn parse_impl_method(&mut self, impl_type: &Symbol, impl_generic_params: &[(Symbol, Option<Symbol>)]) -> Result<Stmt>
src/parser/parser/impls.rs:244: pub(super) fn parse_block(&mut self) -> Result<Block>
src/parser/parser/mod.rs:8: pub struct Parser
src/parser/parser/mod.rs:13: mod atom;
src/parser/parser/mod.rs:14: mod core;
src/parser/parser/mod.rs:15: mod decl;
src/parser/parser/mod.rs:16: mod enum_iface;
src/parser/parser/mod.rs:17: mod expr;
src/parser/parser/mod.rs:18: mod impls;
src/parser/parser/mod.rs:19: mod stmt;
src/parser/parser/mod.rs:20: mod types;
src/parser/parser/mod.rs:21: mod unary;
src/parser/parser/stmt.rs:3: impl Parser
src/parser/parser/stmt.rs:6: pub(super) fn parse_stmt(&mut self) -> Result<Stmt>
src/parser/parser/stmt.rs:85: pub(super) fn parse_any_assign_or_expr(&mut self) -> Result<Stmt>
src/parser/parser/stmt.rs:111: pub(super) fn is_type_start(&self, pos: usize) -> bool
src/parser/parser/stmt.rs:119: pub(super) fn parse_lambda(&mut self) -> Result<Expr>
src/parser/parser/types.rs:3: impl Parser
src/parser/parser/types.rs:6: pub(super) fn parse_type(&mut self) -> Result<Type>
src/parser/parser/types.rs:26: pub(super) fn parse_base_type(&mut self) -> Result<Type>
src/parser/parser/types.rs:100: pub(super) fn handle_path_sep(&mut self, name_str: &mut String, name_sym: &mut Symbol) -> Result<()>
src/parser/parser/unary.rs:3: impl Parser
src/parser/parser/unary.rs:4: pub(super) fn parse_unary(&mut self) -> Result<Expr>
src/parser/parser/unary.rs:53: pub(super) fn parse_postfix(&mut self) -> Result<Expr>
src/span.rs:5: pub struct Span
src/span.rs:14: impl Span
src/span.rs:15: pub fn new(
src/span.rs:33: pub fn from_bytes(start_byte: usize, end_byte: usize) -> Self
src/span.rs:44: pub fn merge(self, other: Span) -> Self
src/span.rs:56: impl Default for Span
src/span.rs:57: fn default() -> Self
src/span.rs:62: impl std::fmt::Display for Span
src/span.rs:63: fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result

## Ayanami (.aya)
std/io.aya:12: pub fn getchar() -> int
std/io.aya:15: pub fn putchar(int c)
std/io.aya:18: pub fn print(ref String n)
std/io.aya:21: pub fn println()
std/io.aya:24: pub fn println(ref String s)
std/main.aya:4: fn main()->int
std/math.aya:5: pub fn abs(int x) -> int
std/math.aya:11: pub fn min(int a, int b) -> int
std/math.aya:17: pub fn max(int a, int b) -> int
std/math.aya:23: pub fn clamp(int x, int lo, int hi) -> int
std/math.aya:30: pub fn pow(int base, int exp) -> int
std/src/arraylist.aya:4: pub struct ArrayList[T:ToString]
std/src/arraylist.aya:10: impl[T:ToString] ArrayList[T]
std/src/arraylist.aya:11: fn expand(ref mut self)
std/src/arraylist.aya:23: pub fn push(ref mut self, T val)
std/src/arraylist.aya:29: pub fn index(ref self, int index)->T{ return self.data[index] }
std/src/arraylist.aya:30: pub fn to_string(ref self)->String
std/src/arraylist.aya:39: pub fn len(ref self)->int{ return self.len }
std/src/arraylist.aya:41: pub fn iter(ref self, fn(T) f)
std/src/io.aya:12: pub fn getchar() -> int
std/src/io.aya:15: pub fn putchar(int c)
std/src/io.aya:18: pub fn print(ref String n)
std/src/io.aya:21: pub fn println()
std/src/io.aya:24: pub fn println(ref String s)
std/src/linkedlist.aya:6: pub struct LinkedList[T:ToString]
std/src/linkedlist.aya:13: pub fn new[T:ToString]()->LinkedList[T]
std/src/linkedlist.aya:18: impl[T:ToString] LinkedList[T]
std/src/linkedlist.aya:19: fn expand(ref mut self)
std/src/linkedlist.aya:31: pub fn push(ref mut self, T val)
std/src/linkedlist.aya:37: pub fn index(ref self, int index)->T
std/src/linkedlist.aya:41: pub fn len(ref self)->int{ return self.len }
std/src/linkedlist.aya:43: pub fn iter(ref self, fn(T) f)
std/src/linkedlist.aya:49: pub fn to_string(ref self) -> String
std/src/list.aya:1: pub interface List[T:ToString]
std/src/list.aya:2: fn push(ref mut self, T val);
std/src/list.aya:3: fn index(ref self, int index)->T;
std/src/list.aya:4: fn to_string(ref self)->String;
std/src/list.aya:5: fn len(ref self)->int;
std/src/list.aya:6: fn iter(ref self, fn(T) f);
std/src/math.aya:5: pub fn abs(int x) -> int
std/src/math.aya:11: pub fn min(int a, int b) -> int
std/src/math.aya:17: pub fn max(int a, int b) -> int
std/src/math.aya:23: pub fn clamp(int x, int lo, int hi) -> int
std/src/math.aya:30: pub fn pow(int base, int exp) -> int
std/src/std.aya:9: pub interface Error
std/src/std.aya:10: fn what(ref self) -> String;
std/src/std.aya:13: pub enum Result[T, E]
std/src/std.aya:18: impl[T, E] Result[T, E]
std/src/std.aya:19: pub fn try_unwrap(self) -> T
std/src/string.aya:1: pub struct String
std/src/string.aya:6: pub interface ToString
std/src/string.aya:7: fn to_string(self)->String;
std/src/string.aya:10: pub interface Error
std/src/string.aya:11: fn what(ref self) -> String;
std/src/string.aya:14: pub enum Result[T, E]
std/src/string.aya:19: impl[T, E] Result[T, E]
std/src/string.aya:20: pub fn try_unwrap(self) -> T
std/src/string.aya:28: impl int
std/src/string.aya:29: fn to_string(self) -> String
std/src/string.aya:65: impl float
std/src/string.aya:66: fn to_string(self) -> String
std/src/string.aya:73: impl char
std/src/string.aya:74: fn to_string(self) -> String
std/src/string.aya:79: impl bool
std/src/string.aya:80: fn to_string(self) -> String
std/src/string.aya:93: impl int
std/src/string.aya:94: pub fn add(self, int other) -> int { return self + other }
std/src/string.aya:95: pub fn sub(self, int other) -> int { return self - other }
std/src/string.aya:96: pub fn mul(self, int other) -> int { return self * other }
std/src/string.aya:97: pub fn div(self, int other) -> int { return self / other }
std/src/string.aya:98: pub fn rem(self, int other) -> int { return self % other }
std/src/string.aya:100: pub fn eq(self, int other) -> bool { return self == other }
std/src/string.aya:101: pub fn ne(self, int other) -> bool { return self != other }
std/src/string.aya:102: pub fn lt(self, int other) -> bool { return self < other }
std/src/string.aya:103: pub fn gt(self, int other) -> bool { return self > other }
std/src/string.aya:104: pub fn le(self, int other) -> bool { return self <= other }
std/src/string.aya:105: pub fn ge(self, int other) -> bool { return self >= other }
std/src/string.aya:107: pub fn neg(self) -> int { return -self }
std/src/string.aya:114: impl float
std/src/string.aya:115: pub fn add(self, float other) -> float { return self + other }
std/src/string.aya:116: pub fn sub(self, float other) -> float { return self - other }
std/src/string.aya:117: pub fn mul(self, float other) -> float { return self * other }
std/src/string.aya:118: pub fn div(self, float other) -> float { return self / other }
std/src/string.aya:120: pub fn eq(self, float other) -> bool { return self == other }
std/src/string.aya:121: pub fn ne(self, float other) -> bool { return self != other }
std/src/string.aya:122: pub fn lt(self, float other) -> bool { return self < other }
std/src/string.aya:123: pub fn gt(self, float other) -> bool { return self > other }
std/src/string.aya:124: pub fn le(self, float other) -> bool { return self <= other }
std/src/string.aya:125: pub fn ge(self, float other) -> bool { return self >= other }
std/src/string.aya:127: pub fn neg(self) -> float { return -self }
std/src/string.aya:134: impl char
std/src/string.aya:135: pub fn eq(self, char other) -> bool { return self == other }
std/src/string.aya:136: pub fn ne(self, char other) -> bool { return self != other }
std/src/string.aya:137: pub fn lt(self, char other) -> bool { return self < other }
std/src/string.aya:138: pub fn gt(self, char other) -> bool { return self > other }
std/src/string.aya:139: pub fn le(self, char other) -> bool { return self <= other }
std/src/string.aya:140: pub fn ge(self, char other) -> bool { return self >= other }
std/src/string.aya:147: impl bool
std/src/string.aya:148: pub fn eq(self, bool other) -> bool { return self == other }
std/src/string.aya:149: pub fn ne(self, bool other) -> bool { return self != other }
std/src/string.aya:156: impl String
std/src/string.aya:158: pub fn to_string(self) -> String
std/src/string.aya:162: pub fn index(ref self, int i) -> char
std/src/string.aya:166: pub fn len(ref self) -> int
std/src/string.aya:170: pub fn add(ref self, ref String other) -> String
std/src/string.aya:184: pub fn add[T:ToString](ref self, T a) -> String
std/src/string.aya:189: pub fn eq(ref self, ref String other) -> bool
std/src/string.aya:201: pub fn ne(ref self, ref String other) -> bool
std/src/string.aya:205: pub fn copy(ref self) -> String
std/std.aya:9: pub interface Error
std/std.aya:10: fn what(ref self) -> String;
std/std.aya:13: pub enum Result[T, E]
std/std.aya:18: impl[T, E] Result[T, E]
std/std.aya:19: pub fn try_unwrap(self) -> T
std/string.aya:1: pub struct String
std/string.aya:6: pub interface ToString
std/string.aya:7: fn to_string(self)->String;
std/string.aya:10: pub interface Error
std/string.aya:11: fn what(ref self) -> String;
std/string.aya:14: pub enum Result[T, E]
std/string.aya:19: impl[T, E] Result[T, E]
std/string.aya:20: pub fn try_unwrap(self) -> T
std/string.aya:28: impl int
std/string.aya:29: fn to_string(self) -> String
std/string.aya:65: impl float
std/string.aya:66: fn to_string(self) -> String
std/string.aya:73: impl char
std/string.aya:74: fn to_string(self) -> String
std/string.aya:79: impl bool
std/string.aya:80: fn to_string(self) -> String
std/string.aya:93: impl int
std/string.aya:94: pub fn add(self, int other) -> int { return self + other }
std/string.aya:95: pub fn sub(self, int other) -> int { return self - other }
std/string.aya:96: pub fn mul(self, int other) -> int { return self * other }
std/string.aya:97: pub fn div(self, int other) -> int { return self / other }
std/string.aya:98: pub fn rem(self, int other) -> int { return self % other }
std/string.aya:100: pub fn eq(self, int other) -> bool { return self == other }
std/string.aya:101: pub fn ne(self, int other) -> bool { return self != other }
std/string.aya:102: pub fn lt(self, int other) -> bool { return self < other }
std/string.aya:103: pub fn gt(self, int other) -> bool { return self > other }
std/string.aya:104: pub fn le(self, int other) -> bool { return self <= other }
std/string.aya:105: pub fn ge(self, int other) -> bool { return self >= other }
std/string.aya:107: pub fn neg(self) -> int { return -self }
std/string.aya:114: impl float
std/string.aya:115: pub fn add(self, float other) -> float { return self + other }
std/string.aya:116: pub fn sub(self, float other) -> float { return self - other }
std/string.aya:117: pub fn mul(self, float other) -> float { return self * other }
std/string.aya:118: pub fn div(self, float other) -> float { return self / other }
std/string.aya:120: pub fn eq(self, float other) -> bool { return self == other }
std/string.aya:121: pub fn ne(self, float other) -> bool { return self != other }
std/string.aya:122: pub fn lt(self, float other) -> bool { return self < other }
std/string.aya:123: pub fn gt(self, float other) -> bool { return self > other }
std/string.aya:124: pub fn le(self, float other) -> bool { return self <= other }
std/string.aya:125: pub fn ge(self, float other) -> bool { return self >= other }
std/string.aya:127: pub fn neg(self) -> float { return -self }
std/string.aya:134: impl char
std/string.aya:135: pub fn eq(self, char other) -> bool { return self == other }
std/string.aya:136: pub fn ne(self, char other) -> bool { return self != other }
std/string.aya:137: pub fn lt(self, char other) -> bool { return self < other }
std/string.aya:138: pub fn gt(self, char other) -> bool { return self > other }
std/string.aya:139: pub fn le(self, char other) -> bool { return self <= other }
std/string.aya:140: pub fn ge(self, char other) -> bool { return self >= other }
std/string.aya:147: impl bool
std/string.aya:148: pub fn eq(self, bool other) -> bool { return self == other }
std/string.aya:149: pub fn ne(self, bool other) -> bool { return self != other }
std/string.aya:156: impl String
std/string.aya:158: pub fn to_string(self) -> String
std/string.aya:162: pub fn index(ref self, int i) -> char
std/string.aya:166: pub fn len(ref self) -> int
std/string.aya:170: pub fn add(ref self, ref String other) -> String
std/string.aya:184: pub fn add[T:ToString](ref self, T a) -> String
std/string.aya:189: pub fn eq(ref self, ref String other) -> bool
std/string.aya:201: pub fn ne(ref self, ref String other) -> bool
std/string.aya:205: pub fn copy(ref self) -> String
example/macro_lib.aya:3: pub fn answer() -> String { return "fn answer() -> int { return 42 }" }
example/math_lib.aya:1: pub fn add(int a, int b) -> int
example/test.aya:1: fn main()->int
example/test.aya:14: fn add(int a, int b)->int
example/test.aya:17: fn sub(int a, int b)->int
example/test_array.aya:1: fn main() -> int
example/test_array_param.aya:1: fn sum(unique [int] arr) -> int
example/test_array_param.aya:5: fn main() -> int
example/test_asm.aya:1: fn main() -> int
example/test_assume.aya:3: fn dec(int n) -> int { return n - 1 }
example/test_assume.aya:7: fn id(int x) -> int { return x }
example/test_assume.aya:9: fn main() -> int
example/test_attrs.aya:3: fn add(int a, int b) -> int { return a + b }
example/test_attrs.aya:9: fn main() -> int
example/test_bool.aya:1: fn main() -> int
example/test_bool2.aya:1: fn main() -> int
example/test_cfg.aya:3: fn on_linux() -> int { return 0 }
example/test_cfg.aya:6: fn on_windows() -> int { return 1 }
example/test_cfg.aya:9: fn on_unix() -> int { return 2 }
example/test_cfg.aya:12: fn not_windows() -> int { return 3 }
example/test_cfg.aya:14: fn main() -> int
example/test_comments.aya:3: fn main() -> int
example/test_eff_import.aya:4: fn main() -> int
example/test_effects.aya:6: fn parse(int x) -> int { return x }
example/test_effects.aya:10: fn pure_fn() -> int { return 0 }
example/test_effects.aya:13: fn id[T](T x) -> T { return x }
example/test_effects.aya:17: fn declared_io() -> int { putchar(65); return 0 }
example/test_effects.aya:23: fn main() -> int
example/test_ensures.aya:3: fn abs2(int x) -> int
example/test_ensures.aya:9: fn inc(int n) -> int { return n + 1 }
example/test_ensures.aya:12: fn fallthrough() -> int { }
example/test_ensures.aya:14: fn main() -> int
example/test_ffi_attrs.aya:18: fn rarely() -> int { return 1 }
example/test_ffi_attrs.aya:21: fn fast() -> int { return 2 }
example/test_ffi_attrs.aya:24: fn always_returns() -> int { return 3 }
example/test_ffi_attrs.aya:26: fn touch(#[nonnull] ref int x) -> int { return 0 }
example/test_ffi_attrs.aya:28: fn main() -> int
example/test_follow_with.aya:2: struct S
example/test_follow_with.aya:8: fn pick(ref S s) -> ref S { return s }
example/test_follow_with.aya:12: fn first(ref S a, ref S b) -> ref S { return a }
example/test_follow_with.aya:14: struct Holder
example/test_follow_with.aya:21: fn read_item(ref S s) -> int
example/test_follow_with.aya:28: fn pick_second(ref S a, ref S b) -> ref S { return b }
example/test_follow_with.aya:31: fn caller(ref S x, ref S y) -> ref S
example/test_follow_with.aya:38: fn pick_ab(ref S a, ref S b) -> ref S { return b }
example/test_follow_with.aya:41: fn joint(ref S x, ref S y) -> ref S
example/test_follow_with.aya:46: fn main() -> int
example/test_import.aya:3: fn main() -> int
example/test_invariant.aya:3: fn win_only() -> int { return 1 }
example/test_invariant.aya:5: fn main() -> int
example/test_macro.aya:5: fn placeholder() -> int { return 0 }
example/test_macro.aya:7: fn main() -> int { return answer() - 42 }
example/test_macro_import.aya:4: fn main() -> int
example/test_memory.aya:5: struct Point
example/test_memory.aya:10: fn make() -> int
example/test_memory.aya:18: fn main() -> int
example/test_memory_enum.aya:4: struct Point
example/test_memory_enum.aya:9: enum Maybe
example/test_memory_enum.aya:14: fn make() -> int
example/test_memory_enum.aya:21: fn main() -> int
example/test_memory_loop.aya:5: struct Point
example/test_memory_loop.aya:10: fn loop_allocs() -> int
example/test_memory_loop.aya:20: fn main() -> int
example/test_nll.aya:4: struct S
example/test_nll.aya:8: fn f(ref S s) -> int { return s.v }
example/test_nll.aya:10: fn g(ref mut S s) -> int
example/test_nll.aya:15: fn main() -> int
example/test_ns.aya:2: fn add(int a, int b) -> int
example/test_ns.aya:7: fn main() -> int
example/test_ns_main.aya:2: fn add(int a, int b) -> int
example/test_ns_main.aya:7: fn main() -> int
example/test_oop.aya:1: interface Drawable
example/test_oop.aya:2: fn draw(ref self) -> void;
example/test_oop.aya:3: fn get_id(ref self) -> int;
example/test_oop.aya:6: interface Resizable
example/test_oop.aya:7: fn resize(ref self, int factor) -> int;
example/test_oop.aya:10: impl int
example/test_oop.aya:11: fn draw(self) -> void {}
example/test_oop.aya:12: fn get_id(self) -> int { return self; }
example/test_oop.aya:13: fn resize(self, int factor) -> int { return self + factor; }
example/test_oop.aya:14: fn triple(self) -> int { return self * 3; }
example/test_oop.aya:17: impl float
example/test_oop.aya:18: fn draw(self) -> void {}
example/test_oop.aya:19: fn get_id(self) -> int { return 0; }
example/test_oop.aya:20: fn resize(self, int factor) -> int { return 0; }
example/test_oop.aya:23: fn render_drawable(ref Drawable d) -> void
example/test_oop.aya:27: fn get_any_id(ref Drawable d) -> int
example/test_oop.aya:31: fn scale(ref Resizable r, int f) -> int
example/test_oop.aya:35: fn main() -> int
example/test_op_overload.aya:1: struct Point
example/test_op_overload.aya:6: impl Point
example/test_op_overload.aya:7: fn add(ref self, ref Point other) -> Point
example/test_op_overload.aya:14: fn main() -> int
example/test_overload.aya:1: fn main()->int
example/test_overload.aya:5: fn add(int a, int b)->int
example/test_overload.aya:9: fn add(float a, float b)->float
example/test_poly.aya:1: interface Shape
example/test_poly.aya:2: fn area(ref self) -> int;
example/test_poly.aya:3: fn describe(ref self) -> void;
example/test_poly.aya:6: impl int
example/test_poly.aya:7: fn area(self) -> int
example/test_poly.aya:10: fn describe(self) -> void {}
example/test_poly.aya:13: impl float
example/test_poly.aya:14: fn area(self) -> int
example/test_poly.aya:17: fn describe(self) -> void {}
example/test_poly.aya:20: fn double_area(ref Shape s) -> int
example/test_poly.aya:25: fn main() -> int
example/test_ref_param.aya:2: struct Box2
example/test_ref_param.aya:6: fn get(ref Box2 b) -> int
example/test_ref_param.aya:10: fn main() -> int
example/test_ref_return.aya:3: struct S
example/test_ref_return.aya:7: fn pick(ref S s) -> ref S
example/test_ref_return.aya:11: fn main() -> int
example/test_ref_self.aya:2: struct Counter
example/test_ref_self.aya:6: impl Counter
example/test_ref_self.aya:7: fn get(ref self) -> int
example/test_ref_self.aya:10: fn inc(ref mut self)
example/test_ref_self.aya:13: fn into(self) -> int
example/test_ref_self.aya:18: fn main() -> int
example/test_requires.aya:3: fn dec(int n) -> int { return n - 1 }
example/test_requires.aya:7: fn clamp100(int x) -> int { return x }
example/test_requires.aya:9: fn main() -> int
example/test_self.aya:1: fn main() -> int
example/test_std.aya:3: fn main() -> int
example/test_std_io.aya:3: fn main() -> int
example/test_struct.aya:1: struct Point
example/test_struct.aya:6: fn main() -> int
example/test_struct2.aya:1: struct Point
example/test_struct2.aya:6: fn main() -> int
example/test_struct_fn.aya:1: struct Point
example/test_struct_fn.aya:6: fn add_points(Point a, Point b) -> Point
example/test_struct_fn.aya:11: fn main() -> int
example/test_struct_impl.aya:1: struct Point
example/test_struct_impl.aya:6: impl Point
example/test_struct_impl.aya:7: fn get_x(unique self) -> int
example/test_struct_impl.aya:12: fn main() -> int
example/test_try.aya:4: fn inner(int x) -> Result[int, int]
example/test_try.aya:9: fn outer(int x) -> Result[int, int]
example/test_try.aya:15: fn take(Result[int, int] r) -> int { return r._tag }
example/test_try.aya:17: fn main() -> int
example/test_two_phase.aya:1: struct S { int v }
example/test_two_phase.aya:2: impl S
example/test_two_phase.aya:3: fn add(ref mut self, int x) { self.v = self.v + x }
example/test_two_phase.aya:5: fn main() -> int
example/test_unique_struct.aya:1: struct Point
example/test_unique_struct.aya:6: fn main() -> int
example/test_vis.aya:1: pub fn main() -> int
example/test_vis2.aya:6: pub fn main() -> int
