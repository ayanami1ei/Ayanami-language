# SYMBOLS.md — Ayanami 符号地图

> 本文件由 scripts/gen_symbols.sh 自动生成，请勿手改。重新生成：./scripts/gen_symbols.sh
> 查询：rg "关键词" SYMBOLS.md
> 标准库（std/）符号由 [Ayanami-std](https://github.com/ayanami1ei/Ayanami-std) 子仓自行维护，见其 SYMBOLS.md。

## Rust
src/cli/build.rs:3: pub(crate) fn cmd_build(args: &[String])
src/cli/build.rs:28: pub(crate) fn cmd_run(args: &[String])
src/cli/check.rs:3: fn do_check(path: &Path) -> ayanami::error::Result<()>
src/cli/check.rs:14: pub(crate) fn cmd_check(args: &[String])
src/cli/check.rs:36: fn watch_file(path: &Path)
src/cli/clean.rs:3: pub(crate) fn cmd_clean()
src/cli/defs.rs:3: pub(crate) fn cmd_defs(args: &[String])
src/cli/defs.rs:43: fn resolve_import_defs(stmts: &[ayanami::parser::ast::Stmt], base_path: &str, visited: &mut std::collections::HashSet<std::path::PathBuf>, defs: &mut Vec<ayanami::compiler::SymDef>)
src/cli/dump.rs:4: pub(crate) fn cmd_dump(args: &[String])
src/cli/fmt.rs:3: pub(crate) fn cmd_fmt(args: &[String])
src/cli/mod.rs:7: pub(crate) mod build;
src/cli/mod.rs:8: pub(crate) mod check;
src/cli/mod.rs:9: pub(crate) mod clean;
src/cli/mod.rs:10: pub(crate) mod defs;
src/cli/mod.rs:11: pub(crate) mod dump;
src/cli/mod.rs:12: pub(crate) mod fmt;
src/cli/mod.rs:13: pub(crate) mod new;
src/cli/mod.rs:14: pub(crate) mod types;
src/cli/mod.rs:15: pub(crate) mod package_install;
src/cli/mod.rs:28: pub(crate) struct CliFlags
src/cli/mod.rs:36: pub(crate) fn split_flags(args: &[String]) -> (Vec<String>, CliFlags)
src/cli/mod.rs:51: pub(crate) fn apply_mode(flags: &CliFlags)
src/cli/new.rs:3: pub(crate) fn cmd_new(args: &[String])
src/cli/package_install.rs:3: pub(crate) fn cmd_package(args: &[String])
src/cli/package_install.rs:22: pub(crate) fn cmd_install(args: &[String])
src/cli/types.rs:9: pub(crate) fn cmd_types(args: &[String])
src/cli/types.rs:62: fn collect_items(items: &[HirItem], out: &mut Vec<(String, String, usize, usize)>)
src/cli/types.rs:72: fn collect_fn(f: &HirFn, out: &mut Vec<(String, String, usize, usize)>)
src/cli/types.rs:78: fn walk_stmts(
src/cli/types.rs:124: fn fmt_type(ty: &HirType) -> String
src/cli/types.rs:150: fn name_at(code: &str, line: usize, col: usize, name: &str) -> bool
src/cli/types.rs:167: fn esc(s: &str) -> String
src/compiler/build/compile.rs:6: pub fn compile_file(
src/compiler/build/compile.rs:144: pub(super) fn parse_and_check(code: &str, src_path: &Path) -> Result<Program>
src/compiler/build/deps.rs:3: pub(super) fn resolve_dependencies(
src/compiler/build/deps.rs:127: pub(super) fn merge_dep_struct_defs(
src/compiler/build/deps.rs:144: pub(super) fn merge_symbols(
src/compiler/build/lir.rs:3: pub(super) fn lower_to_lir(program: &Program, src_path: &Path) -> Result<(
src/compiler/build/lir.rs:34: pub(super) fn build_target_artifact(
src/compiler/build/mod.rs:14: mod compile;
src/compiler/build/mod.rs:15: mod deps;
src/compiler/build/mod.rs:16: mod lir;
src/compiler/build/mod.rs:17: mod package;
src/compiler/build/mod.rs:18: mod passes;
src/compiler/build/mod.rs:19: mod target;
src/compiler/build/mod.rs:26: pub fn build_source(src_path: &str, code: &str) -> Result<()>
src/compiler/build/mod.rs:31: pub fn build_source_to(src_path: &str, _code: &str, out_dir: &str) -> Result<()>
src/compiler/build/package.rs:4: pub(super) fn emit_lcl_package(
src/compiler/build/package.rs:26: pub fn package_source(src_path: &str, _code: &str) -> Result<()>
src/compiler/build/package.rs:79: fn dep_stems(paths: &[PathBuf]) -> Vec<String>
src/compiler/build/package.rs:89: pub fn install_package(lcl_path: &str, target_type: Option<&str>) -> Result<()>
src/compiler/build/passes/apply.rs:18: fn compute_sizes(child_counts: &[i64]) -> Vec<usize>
src/compiler/build/passes/apply.rs:35: fn collect_expr(e: &dyn MirNode, out: &mut Vec<Option<MirNodeBox>>, idx: &mut usize)
src/compiler/build/passes/apply.rs:42: fn collect_stmt(s: &dyn MirStmtNode, out: &mut Vec<Option<MirNodeBox>>, idx: &mut usize)
src/compiler/build/passes/apply.rs:48: fn collect_clones(f: &MirFn, n: usize) -> Vec<Option<MirNodeBox>>
src/compiler/build/passes/apply.rs:57: fn apply_expr(
src/compiler/build/passes/apply.rs:140: fn apply_stmt(
src/compiler/build/passes/apply.rs:179: pub(super) fn apply_edits(f: &mut MirFn, view: &FlatView, edits: &EditView, a: &Attr) -> Result<()>
src/compiler/build/passes/flat.rs:11: pub(super) const SCHEMA_VERSION: i64 = 3;
src/compiler/build/passes/flat.rs:13: pub(super) const HEADER_WORDS: usize = 4;
src/compiler/build/passes/flat.rs:15: pub(super) const ARRAY_COUNT: usize = 15;
src/compiler/build/passes/flat.rs:17: pub(super) const READONLY_ARRAYS: usize = 13;
src/compiler/build/passes/flat.rs:19: pub(super) const KIND_FN: i64 = 0;
src/compiler/build/passes/flat.rs:20: pub(super) const KIND_STMT: i64 = 1;
src/compiler/build/passes/flat.rs:21: pub(super) const KIND_EXPR: i64 = 2;
src/compiler/build/passes/flat.rs:23: pub(super) struct FlatView
src/compiler/build/passes/flat.rs:28: impl FlatView
src/compiler/build/passes/flat.rs:29: pub(super) fn new(purity: HashMap<FnId, bool>) -> Self
src/compiler/build/passes/flat.rs:33: pub(super) fn len(&self) -> usize
src/compiler/build/passes/flat.rs:40: fn push(&mut self, kind: i64, call: bool, callee_pure: bool, alloc: bool, asm: bool, store: bool, op: i64, lit_kind: i64, lit_i64: i64, lit_f64: i64, stmt_kind: i64, var_id: i64) -> usize
src/compiler/build/passes/flat.rs:49: fn set_child_count(&mut self, idx: usize, n: i64)
src/compiler/build/passes/flat.rs:54: fn op_code(op: BinaryOp) -> i64
src/compiler/build/passes/flat.rs:65: fn unary_code(op: UnaryOp) -> i64
src/compiler/build/passes/flat.rs:71: fn lit_kind(v: &HirLiteral) -> i64
src/compiler/build/passes/flat.rs:82: fn lit_i64(v: &HirLiteral) -> i64
src/compiler/build/passes/flat.rs:92: fn lit_f64(v: &HirLiteral) -> i64
src/compiler/build/passes/flat.rs:99: fn walk_expr(e: &dyn MirNode, v: &mut FlatView) -> usize
src/compiler/build/passes/flat.rs:123: fn stmt_kind(s: &dyn MirStmtNode) -> i64
src/compiler/build/passes/flat.rs:139: fn walk_stmt(s: &dyn MirStmtNode, v: &mut FlatView) -> usize
src/compiler/build/passes/flat.rs:149: fn push_i64(buf: &mut Vec<u8>, v: i64)
src/compiler/build/passes/flat.rs:154: pub(super) fn serialize_fn(f: &MirFn, purity: &HashMap<FnId, bool>) -> Vec<u8>
src/compiler/build/passes/flat.rs:177: pub(super) fn read_i64(buf: &[u8], off: usize) -> i64
src/compiler/build/passes/flat.rs:183: pub(super) struct EditView
src/compiler/build/passes/flat.rs:188: pub(super) struct PassOutput
src/compiler/build/passes/flat.rs:195: pub(super) fn parse_output(input: &[u8], out: &[u8], a: &Attr) -> Result<PassOutput>
src/compiler/build/passes/flat.rs:223: pub(super) fn deserialize_view(blob: &[u8]) -> FlatView
src/compiler/build/passes/mod.rs:15: mod apply;
src/compiler/build/passes/mod.rs:16: mod flat;
src/compiler/build/passes/mod.rs:22: fn purity_map(hir: &HirProgram) -> HashMap<FnId, bool>
src/compiler/build/passes/mod.rs:39: fn is_builtin_attr(a: &Attr) -> bool
src/compiler/build/passes/mod.rs:46: pub(super) fn apply_passes(
src/compiler/build/target.rs:4: pub fn build_source_with_target(
src/compiler/build/target.rs:128: pub fn run_executable(exe_name: &str) -> Result<i32>
src/compiler/check.rs:5: pub fn check_hir_returns(hir: &HirProgram, _src_path: &Path) -> Result<()>
src/compiler/check.rs:33: fn has_return_in_item(item: &HirItem) -> bool
src/compiler/check.rs:72: fn has_return_in_stmt(s: &HirStmt) -> bool
src/compiler/debug/expr.rs:5: pub(super) fn write_expr(expr: &Expr, level: usize, w: &mut impl Write)
src/compiler/debug/format.rs:3: pub(super) fn pad(n: usize) -> String
src/compiler/debug/format.rs:7: pub(super) fn format_type(ty: &Type) -> String
src/compiler/debug/format.rs:40: pub(super) fn format_op(op: &BinaryOp) -> &str
src/compiler/debug/format.rs:63: pub(super) fn format_unary(op: &UnaryOp) -> &str
src/compiler/debug/format.rs:71: pub(super) fn format_literal(lit: &Literal) -> String
src/compiler/debug/mod.rs:5: mod expr;
src/compiler/debug/mod.rs:6: mod format;
src/compiler/debug/mod.rs:7: mod stmt;
src/compiler/debug/mod.rs:13: pub fn format_program(program: &Program) -> String
src/compiler/debug/stmt.rs:5: fn write_block(block: &Block, level: usize, w: &mut impl Write)
src/compiler/debug/stmt.rs:13: pub(super) fn write_stmt(stmt: &Stmt, level: usize, w: &mut impl Write)
src/compiler/import.rs:2: pub fn find_std_dir() -> Option<std::path::PathBuf>
src/compiler/import.rs:20: pub fn resolve_import_path(
src/compiler/import.rs:84: pub fn load_config_for_file(file_path: &std::path::Path, _out_dir: &std::path::Path) -> Option<String>
src/compiler/macro_expand/annotations.rs:14: pub(super) struct PkgAnnotations
src/compiler/macro_expand/annotations.rs:23: pub(crate) enum AnnKind
src/compiler/macro_expand/annotations.rs:31: pub(crate) struct AnnotationTables
src/compiler/macro_expand/annotations.rs:35: impl AnnotationTables
src/compiler/macro_expand/annotations.rs:36: pub fn is_empty(&self) -> bool
src/compiler/macro_expand/annotations.rs:40: pub fn collect(stmts: &[Stmt]) -> Result<Self>
src/compiler/macro_expand/annotations.rs:46: fn collect_stmts(&mut self, stmts: &[Stmt]) -> Result<()>
src/compiler/macro_expand/annotations.rs:82: pub fn resolve(&self, a: &Attr) -> Result<Option<(String, String, AnnKind)>>
src/compiler/macro_expand/annotations.rs:126: pub fn lcl_path(&self, pkg: &str) -> Option<String>
src/compiler/macro_expand/check_plugin.rs:13: type RunFn = unsafe extern "C" fn(AyaBuf, AyaBufList) -> AyaBuf;
src/compiler/macro_expand/check_plugin.rs:14: type TakeFn = unsafe extern "C" fn() -> AyaBuf;
src/compiler/macro_expand/check_plugin.rs:15: type FreeFn = unsafe extern "C" fn(AyaBuf);
src/compiler/macro_expand/check_plugin.rs:17: pub(super) fn invoke_check(lcl_path: &str, check_name: &str, blob: &[u8]) -> Result<(Vec<u8>, Vec<(i64, String)>)>
src/compiler/macro_expand/check_plugin.rs:22: fn read_i64(buf: &[u8], off: usize) -> i64
src/compiler/macro_expand/check_plugin.rs:28: fn parse_diags(buf: &[u8]) -> Vec<(i64, String)>
src/compiler/macro_expand/check_plugin.rs:43: fn call_check(so_path: &Path, symbol: &str, blob: &[u8]) -> Result<(Vec<u8>, Vec<(i64, String)>)>
src/compiler/macro_expand/mod.rs:18: const MAX_DEPTH: usize = 32;
src/compiler/macro_expand/mod.rs:20: mod annotations;
src/compiler/macro_expand/mod.rs:21: mod check_plugin;
src/compiler/macro_expand/mod.rs:22: mod pass_plugin;
src/compiler/macro_expand/mod.rs:23: mod plugin;
src/compiler/macro_expand/mod.rs:24: mod plugin_abi;
src/compiler/macro_expand/mod.rs:27: pub fn expand(program: &Program, src_path: &Path) -> Result<Program>
src/compiler/macro_expand/mod.rs:37: pub(crate) fn annotation_tables(stmts: &[Stmt]) -> Result<AnnotationTables>
src/compiler/macro_expand/mod.rs:42: pub(crate) fn invoke_pass(lcl_path: &str, name: &str, blob: &[u8]) -> Result<Vec<u8>>
src/compiler/macro_expand/mod.rs:47: pub(crate) fn invoke_check(lcl_path: &str, name: &str, blob: &[u8]) -> Result<(Vec<u8>, Vec<(i64, String)>)>
src/compiler/macro_expand/mod.rs:51: struct MacroCtx
src/compiler/macro_expand/mod.rs:56: impl MacroCtx
src/compiler/macro_expand/mod.rs:57: fn collect(program: &Program, src_path: &Path) -> Result<Self>
src/compiler/macro_expand/mod.rs:65: fn expand_block(block: Block, ctx: &MacroCtx, depth: usize) -> Result<Block>
src/compiler/macro_expand/mod.rs:71: fn expand_stmts(stmts: &[Stmt], ctx: &MacroCtx, depth: usize) -> Result<Vec<Stmt>>
src/compiler/macro_expand/mod.rs:88: fn expand_nested(stmt: Stmt, out: &mut Vec<Stmt>, ctx: &MacroCtx, depth: usize) -> Result<()>
src/compiler/macro_expand/mod.rs:181: fn resolve_macro_call(ctx: &MacroCtx, attr: &Attr) -> Result<(String, String, Vec<String>)>
src/compiler/macro_expand/mod.rs:202: fn expand_one(stmt: &Stmt, ctx: &MacroCtx, depth: usize) -> Result<Vec<Stmt>>
src/compiler/macro_expand/mod.rs:229: fn parse_source(code: &str) -> Result<Program>
src/compiler/macro_expand/mod.rs:239: fn is_compiler_attr(a: &Attr) -> bool
src/compiler/macro_expand/mod.rs:250: fn attrs_of(stmt: &Stmt) -> Option<&Vec<Attr>>
src/compiler/macro_expand/mod.rs:261: fn attrs_of_mut(stmt: &mut Stmt) -> Option<&mut Vec<Attr>>
src/compiler/macro_expand/pass_plugin.rs:17: const SCHEMA_VERSION: i64 = 3;
src/compiler/macro_expand/pass_plugin.rs:21: pub(super) fn build_for(lcl_path: &str, ann_name: &str, want_mut: bool) -> Result<(PathBuf, String)>
src/compiler/macro_expand/pass_plugin.rs:47: pub(super) fn invoke_pass(lcl_path: &str, pass_name: &str, blob: &[u8]) -> Result<Vec<u8>>
src/compiler/macro_expand/pass_plugin.rs:52: fn build_pass_plugin(lir: &crate::lir::ir::LirProgram, symbol: &str, dep_lcls: &[String]) -> Result<PathBuf>
src/compiler/macro_expand/pass_plugin.rs:118: fn pass_shim(symbol: &str) -> String
src/compiler/macro_expand/pass_plugin.rs:125: const long long *kinds, *is_call, *callee_pure, *is_alloc, *is_asm, *is_store, *child_counts,\n\
src/compiler/macro_expand/pass_plugin.rs:130: static long long rd64(const char* p) {{\n\
src/compiler/macro_expand/pass_plugin.rs:135: static long long* rd_arr(const char* p, long* pos, long long n) {{\n\
src/compiler/macro_expand/pass_plugin.rs:141: static void wr64(char* p, long long v) {{\n\
src/compiler/macro_expand/pass_plugin.rs:145: static char* g_diag = 0;\n\
src/compiler/macro_expand/pass_plugin.rs:146: static long g_diag_len = 0;\n\
src/compiler/macro_expand/pass_plugin.rs:160: static MirFunction mir_decode(const char* p, long len) {{\n\
src/compiler/macro_expand/pass_plugin.rs:184: static AyaBuf mir_encode(MirFunction f) {{\n\
src/compiler/macro_expand/pass_plugin.rs:221: type RunFn = unsafe extern "C" fn(AyaBuf, AyaBufList) -> AyaBuf;
src/compiler/macro_expand/pass_plugin.rs:222: type FreeFn = unsafe extern "C" fn(AyaBuf);
src/compiler/macro_expand/pass_plugin.rs:224: fn call_pass(so_path: &Path, symbol: &str, blob: &[u8]) -> Result<Vec<u8>>
src/compiler/macro_expand/plugin.rs:17: pub(super) fn invoke_plugin(lcl_path: &str, macro_name: &str, input: &str, args: &[String]) -> Result<String>
src/compiler/macro_expand/plugin.rs:30: pub(crate) fn invoke_macro_expr(
src/compiler/macro_expand/plugin.rs:48: fn build_plugin(lir: &crate::lir::ir::LirProgram, symbol: &str, mapping: &[ParamKind], dep_lcls: &[String]) -> Result<PathBuf>
src/compiler/macro_expand/plugin.rs:129: static AyaBuf dup_buf(AyaBuf b) {{\n\
src/compiler/macro_expand/plugin.rs:167: type ExpandFn = unsafe extern "C" fn(AyaBuf, AyaBufList, i64, i64, AyaBuf) -> AyaBuf;
src/compiler/macro_expand/plugin.rs:168: type FreeFn = unsafe extern "C" fn(AyaBuf);
src/compiler/macro_expand/plugin.rs:170: fn call_plugin(so_path: &Path, symbol: &str, input: &str, args: &[String], line: i64, col: i64, file: &str) -> Result<String>
src/compiler/macro_expand/plugin_abi.rs:11: pub(crate) const ABI_VERSION: u32 = 5;
src/compiler/macro_expand/plugin_abi.rs:15: pub(crate) enum ParamKind
src/compiler/macro_expand/plugin_abi.rs:26: pub(crate) mod dl
src/compiler/macro_expand/plugin_abi.rs:30: pub fn dlopen(filename: *const c_char, flag: c_int) -> *mut c_void;
src/compiler/macro_expand/plugin_abi.rs:31: pub fn dlsym(handle: *mut c_void, symbol: *const c_char) -> *mut c_void;
src/compiler/macro_expand/plugin_abi.rs:32: pub fn dlclose(handle: *mut c_void) -> c_int;
src/compiler/macro_expand/plugin_abi.rs:33: pub fn dlerror() -> *mut c_char;
src/compiler/macro_expand/plugin_abi.rs:36: pub(crate) const RTLD_NOW: c_int = 2;
src/compiler/macro_expand/plugin_abi.rs:40: pub(crate) struct AyaBuf
src/compiler/macro_expand/plugin_abi.rs:47: pub(crate) struct AyaBufList
src/compiler/macro_expand/plugin_abi.rs:53: pub(crate) fn load_macro_fn(lcl_path: &str, macro_name: &str) -> Result<(crate::lir::ir::LirFn, crate::lir::ir::LirProgram)>
src/compiler/macro_expand/plugin_abi.rs:65: pub(crate) fn plan_params(f: &crate::lir::ir::LirFn, args_len: usize, allow_reserved: bool) -> Result<Vec<ParamKind>>
src/compiler/macro_expand/plugin_abi.rs:111: pub(crate) fn compile_pic(ll_path: &Path, obj_path: &Path) -> Result<()>
src/compiler/macro_expand/plugin_abi.rs:127: pub(crate) fn resolve_dep_lcls(lcl_path: &str) -> Result<Vec<String>>
src/compiler/macro_expand/plugin_abi.rs:149: fn resolve_dep(stem: &str, dir: &Path) -> Option<PathBuf>
src/compiler/mod.rs:1: pub mod build;
src/compiler/mod.rs:2: pub mod check;
src/compiler/mod.rs:3: pub mod debug;
src/compiler/mod.rs:4: pub mod import;
src/compiler/mod.rs:5: pub mod macro_expand;
src/compiler/mod.rs:6: pub mod symdef;
src/compiler/mod.rs:24: pub struct CompiledFile
src/compiler/mod.rs:40: pub struct CompileResult
src/compiler/mod.rs:68: pub struct CompilerPipeline
src/compiler/mod.rs:85: impl CompilerPipeline
src/compiler/mod.rs:87: pub fn new(code: &str) -> Self
src/compiler/mod.rs:100: pub fn lex(&mut self) -> Result<&mut Self>
src/compiler/mod.rs:113: pub fn parse(&mut self) -> Result<&mut Self>
src/compiler/mod.rs:120: pub fn lower_hir(&mut self) -> Result<&mut Self>
src/compiler/mod.rs:131: pub fn check_returns(&mut self) -> Result<&mut Self>
src/compiler/mod.rs:142: pub fn lower_mir(&mut self) -> Result<&mut Self>
src/compiler/mod.rs:151: pub fn check_borrows(&mut self) -> Result<&mut Self>
src/compiler/mod.rs:167: pub fn lower_lir(&mut self) -> &mut Self
src/compiler/mod.rs:176: pub fn emit(&mut self) -> &mut Self
src/compiler/mod.rs:184: pub fn compile(&mut self) -> Result<&mut Self>
src/compiler/mod.rs:195: pub fn result(&self) -> CompileResult
src/compiler/mod.rs:204: pub fn hir_program(&self) -> Option<&crate::hir::ir::HirProgram>
src/compiler/mod.rs:209: pub fn mir_program(&self) -> Option<&crate::mir::ir::MirProgram>
src/compiler/mod.rs:214: pub fn ast_program(&self) -> Option<&Program>
src/compiler/mod.rs:220: pub fn compile_source(code: &str) -> Result<CompileResult>
src/compiler/mod.rs:228: pub fn check_source(code: &str, _out_dir: &str) -> Result<()>
src/compiler/symdef.rs:5: pub struct SymDef
src/compiler/symdef.rs:18: fn stmt_effect_tokens(stmt: &Stmt) -> Vec<String>
src/compiler/symdef.rs:29: fn stmt_attr_names(stmt: &Stmt) -> Vec<String>
src/compiler/symdef.rs:42: pub fn collect_defs_from_stmts(
src/compiler/symdef.rs:53: fn collect_defs_from_stmt(stmt: &Stmt, file: &str, prefix: &str, defs: &mut Vec<SymDef>)
src/compiler/symdef.rs:177: pub fn defs_to_json(defs: &[SymDef]) -> String
src/diagnostics.rs:11: pub fn supports_color() -> bool
src/diagnostics.rs:15: pub fn error(msg: &str)
src/diagnostics.rs:23: pub fn warning(msg: &str)
src/diagnostics.rs:32: pub fn status(word: &str, msg: &str)
src/diagnostics.rs:42: pub fn success(msg: &str)
src/diagnostics.rs:50: enum Sev
src/diagnostics.rs:55: fn at_location(sev: Sev, file: &Path, line: usize, col: usize, msg: &str)
src/diagnostics.rs:91: pub fn error_at(file: &Path, line: usize, col: usize, msg: &str)
src/diagnostics.rs:96: pub fn warning_at(file: &Path, line: usize, col: usize, msg: &str)
src/diagnostics.rs:101: fn parse_error(raw: &str) -> (Option<String>, usize, usize, String)
src/diagnostics.rs:153: fn parse_lc(s: &str) -> Option<(usize, usize)>
src/diagnostics.rs:158: fn clean_error_message(msg: &str) -> String
src/diagnostics.rs:190: pub fn report_error(default_file: &Path, raw: &str)
src/driver/lto.rs:6: static LTO: std::sync::atomic::AtomicBool = std::sync::atomic::AtomicBool::new(false);
src/driver/lto.rs:7: pub fn set_lto(v: bool) { LTO.store(v, std::sync::atomic::Ordering::Relaxed); }
src/driver/lto.rs:8: pub fn is_lto() -> bool { LTO.load(std::sync::atomic::Ordering::Relaxed) }
src/driver/lto.rs:11: fn find_tool(name: &str, local_only: bool) -> Option<(PathBuf, PathBuf)>
src/driver/lto.rs:29: pub fn link_modules_lto(modules: &[PathBuf], out_obj: &Path) -> Result<()>
src/driver/mod.rs:10: mod lto;
src/driver/mod.rs:11: mod runtime;
src/driver/mod.rs:16: pub(crate) fn find_llc() -> Result<(PathBuf, PathBuf)>
src/driver/mod.rs:36: pub(super) fn find_opt(local_only: bool) -> Option<(PathBuf, PathBuf)>
src/driver/mod.rs:56: pub fn ir_to_object(llvm_ir: &str, obj_path: impl AsRef<Path>) -> Result<()>
src/driver/mod.rs:116: pub fn objects_to_exe(obj_paths: &[PathBuf], exe_path: impl AsRef<Path>) -> Result<()>
src/driver/mod.rs:121: pub fn objects_to_exe_with_flags(obj_paths: &[PathBuf], extra_flags: &[String], exe_path: impl AsRef<Path>) -> Result<()>
src/driver/mod.rs:129: pub fn objects_to_exe_with_runtime(
src/driver/mod.rs:156: pub fn object_to_exe(obj_path: impl AsRef<Path>, exe_path: impl AsRef<Path>) -> Result<()>
src/driver/mod.rs:162: pub(crate) fn find_runtime_c() -> Result<String>
src/driver/mod.rs:205: pub fn object_to_static_lib(obj_path: impl AsRef<Path>, lib_path: impl AsRef<Path>) -> Result<()>
src/driver/mod.rs:220: pub fn objects_to_shared_lib(obj_paths: &[PathBuf], lib_path: impl AsRef<Path>) -> Result<()>
src/driver/mod.rs:231: pub fn object_to_shared_lib(obj_path: impl AsRef<Path>, lib_path: impl AsRef<Path>) -> Result<()>
src/driver/mod.rs:248: pub fn ir_to_library(llvm_ir: &str, lib_path: impl AsRef<Path>, lib_type: &str) -> Result<()>
src/driver/mod.rs:268: pub fn ir_to_object_keep(llvm_ir: &str, obj_path: impl AsRef<Path>) -> Result<()>
src/driver/mod.rs:273: pub fn ir_to_executable(llvm_ir: &str, exe_path: impl AsRef<Path>) -> Result<()>
src/driver/runtime.rs:12: pub(crate) fn resolve_runtime(src_path: &Path) -> Result<Option<PathBuf>>
src/error.rs:10: pub enum Error
src/error.rs:61: pub type Result<T> = std::result::Result<T, Error>;
src/formatter/expr.rs:4: pub(super) fn write_type(ty: &Type) -> String
src/formatter/expr.rs:25: pub(super) fn write_expr(expr: &Expr) -> String
src/formatter/expr.rs:31: fn binop_prec(op: &BinaryOp) -> u8
src/formatter/expr.rs:46: fn expr_prec(e: &Expr) -> u8
src/formatter/expr.rs:55: fn maybe_paren(cond: bool, e: &Expr, level: usize) -> String
src/formatter/expr.rs:60: pub(super) fn write_expr_at(expr: &Expr, level: usize) -> String
src/formatter/expr.rs:214: fn asm_constraint(c: &str) -> &str
src/formatter/expr.rs:218: pub(super) fn write_literal(lit: &Literal) -> String
src/formatter/expr.rs:262: pub(super) fn write_bin_op(op: &BinaryOp) -> &str
src/formatter/expr.rs:285: pub(super) fn vis_str(vis: &Visibility) -> &str
src/formatter/helpers.rs:4: pub(super) fn write_stmt_separator(out: &mut String, stmt: &Stmt)
src/formatter/helpers.rs:14: pub(super) fn indent(level: usize) -> String
src/formatter/helpers.rs:18: pub(super) fn write_block_same_line(out: &mut String, block: &Block, level: usize)
src/formatter/helpers.rs:35: pub(super) fn write_attrs(out: &mut String, attrs: &[Attr], level: usize)
src/formatter/helpers.rs:47: pub(super) fn write_attr_arg(arg: &crate::parser::ast::AttrArg) -> String
src/formatter/helpers.rs:55: pub(super) fn write_generic_params(out: &mut String, params: &[(Symbol, Option<Symbol>)])
src/formatter/helpers.rs:68: pub(super) fn write_params(out: &mut String, params: &[(Symbol, Type)], param_attrs: &[Vec<crate::parser::ast::Attr>])
src/formatter/helpers.rs:96: pub(super) fn write_return_type(out: &mut String, ty: &Type)
src/formatter/helpers.rs:106: pub(super) fn write_fn_type(kw: &str, params: &[Type], ret: &Type) -> String
src/formatter/helpers.rs:116: pub(super) fn write_match_body(body: &crate::parser::ast::stmt::MatchBody, level: usize) -> String
src/formatter/mod.rs:7: const INDENT: &str = "    ";
src/formatter/mod.rs:9: pub fn format_program(program: &Program) -> String
src/formatter/mod.rs:21: mod expr;
src/formatter/mod.rs:22: mod helpers;
src/formatter/mod.rs:23: mod stmt;
src/formatter/mod.rs:26: pub(crate) fn format_expr(expr: &Expr) -> String
src/formatter/mod.rs:31: pub(crate) fn format_attr_arg(arg: &AttrArg) -> String
src/formatter/mod.rs:38: pub fn format_file(code: &str) -> crate::error::Result<String>
src/formatter/stmt.rs:5: pub(super) fn write_stmt(out: &mut String, stmt: &Stmt, level: usize)
src/hir/attrs.rs:8: pub const ALLOWED: &[&str] = &[
src/hir/attrs.rs:36: pub const PARAM_ALLOWED: &[&str] = &["noalias", "nonnull"];
src/hir/attrs.rs:40: pub struct Imports
src/hir/attrs.rs:47: impl Imports
src/hir/attrs.rs:48: pub fn collect(program: &Program) -> Self
src/hir/attrs.rs:56: fn collect_stmt(&mut self, stmt: &Stmt)
src/hir/attrs.rs:79: pub fn pkg_stem(path: &str) -> String
src/hir/attrs.rs:89: pub fn has(attrs: &[Attr], name: &str) -> bool
src/hir/attrs.rs:94: pub fn validate(attrs: &[Attr], imports: &Imports) -> Result<()>
src/hir/attrs.rs:102: fn resolve(a: &Attr, imports: &Imports) -> Result<()>
src/hir/attrs.rs:137: fn validate_follow_with(a: &Attr) -> Result<()>
src/hir/attrs.rs:157: fn validate_follow_with_attrs(attrs: &[Attr]) -> Result<()>
src/hir/attrs.rs:164: fn reject_follow_with(attrs: &[Attr], place: &str) -> Result<()>
src/hir/attrs.rs:176: pub fn validate_program(program: &Program) -> Result<()>
src/hir/attrs.rs:184: fn validate_param(attrs: &[Attr], ty: &crate::parser::ast::Type) -> Result<()>
src/hir/attrs.rs:207: fn validate_stmt(stmt: &Stmt, imports: &Imports) -> Result<()>
src/hir/attrs_export.rs:5: fn find_export(attrs: &[Attr]) -> Option<&Attr>
src/hir/attrs_export.rs:10: pub(super) fn validate_fn_export(attrs: &[Attr], has_generics: bool) -> Result<()>
src/hir/attrs_export.rs:23: pub(super) fn validate_compile_time(attrs: &[Attr], has_generics: bool) -> Result<()>
src/hir/attrs_export.rs:36: pub(super) fn reject_compile_time(attrs: &[Attr], place: &str) -> Result<()>
src/hir/attrs_export.rs:47: pub(super) fn reject_export(attrs: &[Attr], place: &str) -> Result<()>
src/hir/attrs_macro.rs:16: fn is_compiler_attr(a: &Attr) -> bool
src/hir/attrs_macro.rs:28: pub fn validate_macros(
src/hir/attrs_macro.rs:43: enum AnnKind { Macro, Pass, Check }
src/hir/attrs_macro.rs:47: fn lookup_annotation(
src/hir/attrs_macro.rs:102: fn validate_macros_stmt(
src/hir/attrs_macro.rs:177: fn validate_block(
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
src/hir/display.rs:48: fn write_item(item: &HirItem, level: usize, w: &mut impl Write) -> std::fmt::Result
src/hir/display.rs:93: fn write_block(block: &HirBlock, level: usize, w: &mut impl Write) -> std::fmt::Result
src/hir/display.rs:103: pub(crate) fn write_expr(expr: &HirNodeBox, level: usize, w: &mut impl Write) -> std::fmt::Result
src/hir/display.rs:107: fn write_stmt(stmt: &HirStmt, level: usize, w: &mut impl Write) -> std::fmt::Result
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
src/hir/effects/infer.rs:181: fn declared_to_set(d: &EffectDecl) -> EffectSet
src/hir/effects/infer.rs:189: pub(super) fn collect_fns<'a>(items: &'a [HirItem], out: &mut Vec<&'a HirFn>)
src/hir/effects/infer.rs:199: fn for_each_fn_mut(items: &mut [HirItem], f: &mut impl FnMut(&mut HirFn))
src/hir/effects/infer.rs:210: struct CallInfo
src/hir/effects/infer.rs:217: fn walk_stmts(stmts: &[HirStmt], calls: &mut Vec<CallInfo>)
src/hir/effects/infer.rs:270: fn state_marker() -> CallInfo
src/hir/effects/infer.rs:274: fn is_observable_target(object: &crate::hir::HirNodeBox) -> bool
src/hir/effects/infer.rs:278: fn walk_expr(e: &dyn HirNode, calls: &mut Vec<CallInfo>)
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
src/hir/effects/mod.rs:127: pub fn from_tokens(tokens: &[String]) -> Self
src/hir/effects/mod.rs:162: pub fn parse(attrs: &[Attr]) -> Result<EffectDecl>
src/hir/effects/mod.rs:183: fn parse_throws(a: &Attr) -> Result<ThrowsDecl>
src/hir/effects/mod.rs:204: fn merge_throws(decl: &mut EffectDecl, new: ThrowsDecl)
src/hir/effects/mod.rs:221: fn bad_throws(a: &Attr) -> Error
src/hir/effects/scan.rs:8: pub(super) enum Obs
src/hir/effects/scan.rs:15: impl Obs
src/hir/effects/scan.rs:17: pub(super) fn site_of(&self, kind: &str) -> Option<(usize, usize)>
src/hir/effects/scan.rs:28: pub(super) fn collect_ast_observations(program: &crate::parser::ast::Program) -> HashMap<String, Vec<Obs>>
src/hir/effects/scan.rs:36: fn qualify(prefix: &str, name: &str) -> String
src/hir/effects/scan.rs:40: fn scan_decl(stmt: &crate::parser::ast::Stmt, prefix: &str, map: &mut HashMap<String, Vec<Obs>>)
src/hir/effects/scan.rs:62: fn scan_body(stmt: &crate::parser::ast::Stmt, obs: &mut Vec<Obs>)
src/hir/effects/scan.rs:119: fn scan_expr(e: &crate::parser::ast::Expr, obs: &mut Vec<Obs>)
src/hir/item.rs:7: pub struct HirStructField
src/hir/item.rs:13: pub struct HirStructDef
src/hir/item.rs:19: pub struct HirLocal
src/hir/item.rs:25: impl HirLocal
src/hir/item.rs:26: pub fn new(name: Symbol, ty: HirType, mutable: bool) -> Self
src/hir/item.rs:34: pub struct VtableEntry
src/hir/item.rs:42: pub struct HirInterfaceMethod
src/hir/item.rs:50: pub struct HirFn
src/hir/item.rs:79: pub enum HirItem
src/hir/item.rs:95: pub struct ImportedFnSig
src/hir/item.rs:110: pub struct HirStatic
src/hir/item.rs:122: pub struct HirProgram
src/hir/lower/body/block_lower.rs:3: impl crate::hir::lower::Ctx
src/hir/lower/body/block_lower.rs:8: pub(crate) fn lower_block(&mut self, block: &Block) -> Result<HirBlock>
src/hir/lower/body/block_lower.rs:14: pub(crate) fn lower_block_impl(&mut self, block: &Block, tail_as_value: bool) -> Result<(HirBlock, Option<HirNodeBox>)>
src/hir/lower/body/collect_enum.rs:5: impl crate::hir::lower::Ctx
src/hir/lower/body/collect_enum.rs:6: pub(crate) fn collect_enum_def(
src/hir/lower/body/collect_fns.rs:3: impl crate::hir::lower::Ctx
src/hir/lower/body/collect_fns.rs:8: pub(crate) fn collect_fns(&mut self, stmts: &[Stmt]) -> Result<()>
src/hir/lower/body/collect_fns.rs:15: fn collect_const_fns(&mut self, stmts: &[Stmt], prefix: &str) -> Result<()>
src/hir/lower/body/collect_import.rs:3: impl crate::hir::lower::Ctx
src/hir/lower/body/collect_import.rs:4: pub(crate) fn collect_import(&mut self, path: &String, _ns_prefix: &str, stmt_span: crate::span::Span) -> Result<()>
src/hir/lower/body/collect_ns.rs:4: pub(super) fn check_unique_generic_params(gp: &[(Symbol, Option<Symbol>)], span: &Span) -> Result<()>
src/hir/lower/body/collect_ns.rs:17: impl crate::hir::lower::Ctx
src/hir/lower/body/collect_ns.rs:18: pub(super) fn collect_fns_with_ns(&mut self, stmts: &[Stmt], ns_prefix: &str) -> Result<()>
src/hir/lower/body/const_array.rs:3: impl crate::hir::lower::Ctx
src/hir/lower/body/const_array.rs:5: pub(crate) fn eval_const_array(
src/hir/lower/body/const_coerce.rs:4: pub(super) fn coerce_literal(lit_ty: &mut HirType, lit: &mut HirLiteral, want: &HirType, span: &Span) -> Result<()>
src/hir/lower/body/const_coerce.rs:83: pub(super) fn check_intn_range(v: i64, bits: u8, signed: bool, want: &HirType, span: &Span) -> Result<()>
src/hir/lower/body/const_coerce.rs:102: fn lit_scalar_type(l: &HirLiteral) -> Option<HirType>
src/hir/lower/body/const_eval.rs:3: impl crate::hir::lower::Ctx
src/hir/lower/body/const_eval.rs:5: pub(crate) fn collect_const_decl(
src/hir/lower/body/const_eval.rs:25: pub(crate) fn collect_static_decl(
src/hir/lower/body/const_eval.rs:44: pub(crate) fn eval_const_expr(
src/hir/lower/body/const_fn.rs:4: const MAX_CONST_DEPTH: usize = 64;
src/hir/lower/body/const_fn.rs:5: const MAX_CONST_ITERS: u64 = 1_000_000;
src/hir/lower/body/const_fn.rs:7: pub(crate) enum Flow { Normal, Break, Continue, Return(HirType, HirLiteral) }
src/hir/lower/body/const_fn.rs:9: impl crate::hir::lower::Ctx
src/hir/lower/body/const_fn.rs:10: pub(crate) fn eval_const_fn(&self, name: Symbol, args: Vec<(HirType, HirLiteral)>, depth: usize) -> Result<(HirType, HirLiteral)>
src/hir/lower/body/const_fn.rs:51: pub(crate) fn eval_const_block_value(&self, block: &Block, env: &HashMap<Symbol, (HirType, HirLiteral)>, depth: usize) -> Result<(HirType, HirLiteral)>
src/hir/lower/body/const_fn.rs:66: fn exec_const_stmts(&self, stmts: &[Stmt], env: &mut HashMap<Symbol, (HirType, HirLiteral)>, depth: usize) -> Result<Flow>
src/hir/lower/body/const_fn.rs:76: fn exec_const_stmt(&self, stmt: &Stmt, env: &mut HashMap<Symbol, (HirType, HirLiteral)>, depth: usize) -> Result<Flow>
src/hir/lower/body/const_struct.rs:3: impl crate::hir::lower::Ctx
src/hir/lower/body/const_struct.rs:5: pub(crate) fn eval_const_struct(
src/hir/lower/body/ensure.rs:8: impl crate::hir::lower::Ctx
src/hir/lower/body/ensure.rs:10: pub(crate) fn inject_ensures(
src/hir/lower/body/ensure.rs:60: impl crate::hir::lower::Ctx
src/hir/lower/body/ensure.rs:62: pub(crate) fn loop_check_stmts(
src/hir/lower/body/ensure.rs:89: fn result_node(var: VarId, ty: &HirType) -> HirNodeBox
src/hir/lower/body/ensure.rs:94: fn rewrite_returns(stmts: &mut Vec<HirStmt>, var: VarId, ty: &HirType, checks: &[HirStmt])
src/hir/lower/body/ensure.rs:126: fn always_returns(stmts: &[HirStmt]) -> bool
src/hir/lower/body/ensure.rs:139: fn default_literal(ty: &HirType) -> Option<HirLiteral>
src/hir/lower/body/ensure.rs:150: fn append_default_return(
src/hir/lower/body/expr_access.rs:3: impl crate::hir::lower::Ctx
src/hir/lower/body/expr_access.rs:4: pub(crate) fn lower_field_access(&mut self, object: &Box<Expr>, field: &Symbol, expr_span: &Span) -> Result<HirNodeBox>
src/hir/lower/body/expr_access.rs:17: pub(crate) fn lower_struct_literal(&mut self, type_name: &Symbol, generic_args: &Vec<Type>, fields: &Vec<(Symbol, Expr)>, span: &Span) -> Result<HirNodeBox>
src/hir/lower/body/expr_access.rs:132: pub(crate) fn lower_array_literal(&mut self, elems: &Vec<Expr>, span: &Span) -> Result<HirNodeBox>
src/hir/lower/body/expr_access.rs:152: pub(crate) fn lower_index(&mut self, object: &Box<Expr>, index: &Box<Expr>, span: &Span) -> Result<HirNodeBox>
src/hir/lower/body/expr_call.rs:3: impl crate::hir::lower::Ctx
src/hir/lower/body/expr_call.rs:5: pub(crate) fn lower_fn_call(&mut self, name: &Symbol, args: &Vec<Expr>, explicit: Option<&Vec<Type>>, span: &Span) -> Result<HirNodeBox>
src/hir/lower/body/expr_call_extra.rs:3: impl crate::hir::lower::Ctx
src/hir/lower/body/expr_call_extra.rs:4: pub(crate) fn lower_call_expr(&mut self, target: &Box<Expr>, args: &Vec<Expr>, span: &Span) -> Result<HirNodeBox>
src/hir/lower/body/expr_call_extra.rs:67: pub(crate) fn make_fatptr_arg(&self, arg: HirNodeBox, param_ty: &HirType, ct: Symbol, iface: Symbol) -> HirNodeBox
src/hir/lower/body/expr_cast.rs:3: impl crate::hir::lower::Ctx
src/hir/lower/body/expr_cast.rs:5: pub(crate) fn lower_cast(&mut self, inner: &Box<Expr>, ty: &Type, span: &Span) -> Result<HirNodeBox>
src/hir/lower/body/expr_enum.rs:3: impl crate::hir::lower::Ctx
src/hir/lower/body/expr_enum.rs:4: pub(crate) fn lower_enum_construct(&mut self, enum_name: &Symbol, variant_name: &Symbol, tuple_args: &Vec<Expr>, named_args: &Vec<(Symbol, Expr)>, span: &Span) -> Result<HirNodeBox>
src/hir/lower/body/expr_enum.rs:68: fn monomorphize_enum_construct(
src/hir/lower/body/expr_logic.rs:3: impl crate::hir::lower::Ctx
src/hir/lower/body/expr_logic.rs:5: pub(super) fn lower_short_circuit(&mut self, op: BinaryOp, lhs: &Box<Expr>, rhs: &Box<Expr>, span: &Span) -> Result<HirNodeBox>
src/hir/lower/body/expr_lower.rs:3: impl crate::hir::lower::Ctx
src/hir/lower/body/expr_lower.rs:4: pub(crate) fn lower_expr(&mut self, expr: &Expr) -> Result<HirNodeBox>
src/hir/lower/body/expr_method.rs:3: impl crate::hir::lower::Ctx
src/hir/lower/body/expr_method.rs:4: pub(crate) fn lower_method_call(&mut self, object: &Box<Expr>, method: &Symbol, args: &Vec<Expr>, span: &Span) -> Result<HirNodeBox>
src/hir/lower/body/expr_misc.rs:3: impl crate::hir::lower::Ctx
src/hir/lower/body/expr_misc.rs:4: pub(crate) fn lower_asm(&mut self, template: &String, outputs: &Vec<(String, Box<Expr>)>, inputs: &Vec<(String, Box<Expr>)>) -> Result<HirNodeBox>
src/hir/lower/body/expr_misc.rs:24: pub(crate) fn lower_lambda(&mut self, params: &Vec<(Symbol, Type)>, return_type: &Type, body: &Block) -> Result<HirNodeBox>
src/hir/lower/body/expr_ops1.rs:3: impl crate::hir::lower::Ctx
src/hir/lower/body/expr_ops1.rs:4: pub(crate) fn lower_binary(&mut self, op: &BinaryOp, lhs: &Box<Expr>, rhs: &Box<Expr>, span: &Span) -> Result<HirNodeBox>
src/hir/lower/body/expr_ops1.rs:160: pub(crate) fn lower_unary(&mut self, op: &UnaryOp, arg: &Box<Expr>) -> Result<HirNodeBox>
src/hir/lower/body/expr_ops1.rs:205: pub(crate) fn lower_try_op(&mut self, inner: &Box<Expr>, span: &Span) -> Result<HirNodeBox>
src/hir/lower/body/generic_check.rs:13: struct CheckCtx
src/hir/lower/body/generic_check.rs:24: impl crate::hir::lower::Ctx
src/hir/lower/body/generic_check.rs:26: pub(crate) fn check_generic_bodies(&self) -> Result<()>
src/hir/lower/body/generic_check.rs:47: fn check_block_names(&self, block: &Block, cx: &mut CheckCtx) -> Result<()>
src/hir/lower/body/generic_check.rs:57: fn check_stmt_names(&self, stmt: &Stmt, cx: &mut CheckCtx) -> Result<()>
src/hir/lower/body/generic_check.rs:132: fn check_expr_names(&self, expr: &Expr, cx: &mut CheckCtx) -> Result<()>
src/hir/lower/body/generic_locals.rs:5: pub(super) fn collect_local_names_block(block: &Block, out: &mut Vec<Symbol>)
src/hir/lower/body/generic_locals.rs:14: pub(super) fn collect_local_names_stmt(stmt: &Stmt, out: &mut Vec<Symbol>)
src/hir/lower/body/generic_locals.rs:69: pub(super) fn collect_local_names_expr(expr: &Expr, out: &mut Vec<Symbol>)
src/hir/lower/body/generic_specialize.rs:3: impl crate::hir::lower::Ctx
src/hir/lower/body/generic_specialize.rs:6: pub(crate) fn specialize_generic_call(&mut self, name: &Symbol, arg_types: &[HirType], span: &crate::span::Span) -> Result<FnId>
src/hir/lower/body/generic_specialize.rs:12: pub(crate) fn has_generic_method_candidate(&self, method: &Symbol, receiver_ty: &HirType, argc: usize) -> bool
src/hir/lower/body/generic_specialize.rs:34: pub(crate) fn specialize_generic_call_with(&mut self, name: &Symbol, arg_types: &[HirType], explicit: Option<&Vec<Type>>, span: &crate::span::Span) -> Result<FnId>
src/hir/lower/body/generic_types.rs:10: pub(crate) enum GType
src/hir/lower/body/generic_types.rs:19: pub(super) fn ast_is_concrete_primitive(ty: &Type) -> bool
src/hir/lower/body/generic_types.rs:28: pub(super) fn hir_is_concrete_primitive(ty: &HirType) -> bool
src/hir/lower/body/generic_types.rs:35: impl crate::hir::lower::Ctx
src/hir/lower/body/generic_types.rs:37: pub(crate) fn ast_gtype(&self, ty: &Type, gp: &[(Symbol, Option<Symbol>)]) -> GType
src/hir/lower/body/generic_types.rs:57: pub(crate) fn infer_gtype(
src/hir/lower/body/generic_types.rs:101: pub(crate) fn check_param_operator(
src/hir/lower/body/generic_types.rs:140: pub(crate) fn check_param_method(
src/hir/lower/body/if_expr.rs:3: impl crate::hir::lower::Ctx
src/hir/lower/body/if_expr.rs:6: pub(crate) fn lower_if_expr(
src/hir/lower/body/iface_match.rs:3: impl crate::hir::lower::Ctx
src/hir/lower/body/iface_match.rs:4: pub(crate) fn infer_iface_generic(
src/hir/lower/body/iface_match.rs:17: pub(crate) fn substitute_iface_type(ty: &HirType, subst: &HashMap<Symbol, HirType>, gp_names: &[Symbol]) -> HirType
src/hir/lower/body/iface_match.rs:30: pub(crate) fn type_matches(a: &HirType, b: &HirType) -> bool
src/hir/lower/body/iface_match.rs:43: pub(crate) fn is_self_type(ty: &HirType) -> bool
src/hir/lower/body/iface_match.rs:48: pub(crate) fn contains_self_type(ty: &HirType) -> bool
src/hir/lower/body/iface_match.rs:59: pub(crate) fn substitute_self_type(ty: &HirType, impl_self: &HirType) -> HirType
src/hir/lower/body/iface_match.rs:77: pub(crate) fn is_impl_self_type(actual: &HirType, impl_self_name: &Symbol) -> bool
src/hir/lower/body/iface_match.rs:90: pub(crate) fn type_matches_self(expected: &HirType, actual: &HirType, impl_self_name: &Symbol) -> bool
src/hir/lower/body/iface_match.rs:111: pub(crate) fn find_fn_by_sig(&self, name: Symbol, param_types: &[HirType]) -> Option<FnId>
src/hir/lower/body/iface_match.rs:124: pub(crate) fn extract_concrete_type_name(ty: &HirType) -> Option<Symbol>
src/hir/lower/body/iface_match.rs:139: pub(crate) fn is_fatptr_compatible(&self, param_ty: &HirType, arg_ty: &HirType) -> bool
src/hir/lower/body/iface_match.rs:177: pub(crate) fn check_generic_fns_for_iface(&self, type_name: &Symbol, iface_name: &Symbol) -> bool
src/hir/lower/body/iface_match.rs:212: pub(crate) fn ensure_specialized_interface(&mut self, specialized_name: &Symbol) -> Result<()>
src/hir/lower/body/iface_match.rs:248: pub(crate) fn register_generic_vtable(
src/hir/lower/body/literal.rs:3: impl crate::hir::lower::Ctx
src/hir/lower/body/literal.rs:5: pub(crate) fn lower_suffixed(&mut self, lit: &Literal, suffix: &Symbol, span: &Span) -> Result<HirNodeBox>
src/hir/lower/body/literal.rs:34: pub(crate) fn lower_literal(&mut self, lit: &Literal) -> Result<HirNodeBox>
src/hir/lower/body/literal.rs:46: fn check_literal_range(n: i64, ty: &HirType, span: &Span) -> Result<()>
src/hir/lower/body/lower_items.rs:3: impl crate::hir::lower::Ctx
src/hir/lower/body/lower_items.rs:4: pub(crate) fn lower_items(&mut self, stmts: &[Stmt]) -> Result<Vec<HirItem>>
src/hir/lower/body/lower_items.rs:8: pub(crate) fn lower_items_with_ns(&mut self, stmts: &[Stmt], ns_prefix: &str) -> Result<Vec<HirItem>>
src/hir/lower/body/lower_items.rs:117: pub(crate) fn collect_gp_from_type(ty: &HirType, out: &mut Vec<Symbol>)
src/hir/lower/body/lower_items.rs:145: pub(crate) fn lower_fn(
src/hir/lower/body/macro_call.rs:5: impl crate::hir::lower::Ctx
src/hir/lower/body/macro_call.rs:6: pub(crate) fn lower_macro_call(&mut self, name: &Symbol, args: &Vec<Expr>, span: &Span) -> Result<HirNodeBox>
src/hir/lower/body/match_coverage.rs:8: pub(super) fn enum_exhaustive(total: usize, pats: &[&Pattern]) -> bool
src/hir/lower/body/match_coverage.rs:15: pub(super) fn bool_exhaustive(pats: &[&Pattern]) -> bool
src/hir/lower/body/match_coverage.rs:23: fn scalar_domain(ty: &HirType) -> Option<(i128, i128)>
src/hir/lower/body/match_coverage.rs:41: pub(super) fn scalar_exhaustive(base: &HirType, pats: &[&Pattern]) -> bool
src/hir/lower/body/match_coverage.rs:48: fn collect_intervals(p: &Pattern, out: &mut Vec<(i128, i128)>)
src/hir/lower/body/match_coverage.rs:66: fn covers_domain(intervals: &mut Vec<(i128, i128)>, min: i128, max: i128) -> bool
src/hir/lower/body/match_coverage.rs:83: fn collect_variants(p: &Pattern, out: &mut std::collections::HashSet<Symbol>)
src/hir/lower/body/match_coverage.rs:91: fn collect_bools(p: &Pattern, t: &mut bool, f: &mut bool)
src/hir/lower/body/match_coverage.rs:100: pub(super) fn check_or_bindings(
src/hir/lower/body/match_lower.rs:7: struct LoweredArm
src/hir/lower/body/match_lower.rs:14: impl crate::hir::lower::Ctx
src/hir/lower/body/match_lower.rs:16: fn setup_match_value(&mut self, value: &Expr, span: &Span) -> Result<(HirNodeBox, HirType)>
src/hir/lower/body/match_lower.rs:41: fn normalize_pattern(&self, p: &Pattern, val_ty: &HirType) -> Pattern
src/hir/lower/body/match_lower.rs:56: fn lower_arm_body(&mut self, arm: &MatchArm) -> Result<(Vec<HirStmt>, HirNodeBox)>
src/hir/lower/body/match_lower.rs:87: fn lower_match_arms(
src/hir/lower/body/match_lower.rs:112: fn is_exhaustive(&self, val_ty: &HirType, arms: &[MatchArm]) -> bool
src/hir/lower/body/match_lower.rs:133: fn enum_variant_count(&self, name: &Symbol) -> usize
src/hir/lower/body/match_lower.rs:140: fn build_match_stmts(
src/hir/lower/body/match_lower.rs:204: pub(crate) fn lower_match_stmt(&mut self, value: &Expr, arms: &[MatchArm], span: &Span) -> Result<HirStmt>
src/hir/lower/body/match_lower.rs:221: pub(crate) fn lower_match_expr(&mut self, value: &Expr, arms: &[MatchArm], span: &Span) -> Result<HirNodeBox>
src/hir/lower/body/match_pattern.rs:6: impl crate::hir::lower::Ctx
src/hir/lower/body/match_pattern.rs:8: pub(super) fn variant_tag(&self, enum_name: &Symbol, variant: &Symbol) -> Option<i64>
src/hir/lower/body/match_pattern.rs:15: pub(super) fn tag_cond(val_node: &HirNodeBox, tag: usize) -> HirNodeBox
src/hir/lower/body/match_pattern.rs:30: pub(super) fn pattern_match(
src/hir/lower/body/match_pattern.rs:41: fn pattern_match_in(
src/hir/lower/body/match_pattern.rs:239: fn cmp_const(
src/hir/lower/body/match_pattern.rs:255: fn and_cond(a: HirNodeBox, b: HirNodeBox) -> HirNodeBox
src/hir/lower/body/match_pattern.rs:259: fn or_cond(a: HirNodeBox, b: HirNodeBox) -> HirNodeBox
src/hir/lower/body/match_util.rs:5: pub(super) fn check_scalar_pattern(base: &HirType, lit: &crate::parser::ast::Literal, range: bool, span: &Span) -> Result<()>
src/hir/lower/body/match_util.rs:25: pub(super) fn hir_literal(l: &crate::parser::ast::Literal) -> HirLiteral
src/hir/lower/body/match_util.rs:37: pub(super) fn match_result_type(a: &HirType, b: &HirType) -> HirType
src/hir/lower/body/match_util.rs:53: pub(super) fn block_diverges(b: &HirBlock) -> bool
src/hir/lower/body/mod.rs:10: mod collect_fns;
src/hir/lower/body/mod.rs:11: mod collect_ns;
src/hir/lower/body/mod.rs:12: mod vtables;
src/hir/lower/body/mod.rs:13: mod iface_match;
src/hir/lower/body/mod.rs:14: mod overload_resolve;
src/hir/lower/body/mod.rs:15: mod generic_check;
src/hir/lower/body/mod.rs:16: mod generic_locals;
src/hir/lower/body/mod.rs:17: mod generic_specialize;
src/hir/lower/body/mod.rs:18: mod generic_types;
src/hir/lower/body/mod.rs:19: mod lower_items;
src/hir/lower/body/mod.rs:20: mod stmt_lower;
src/hir/lower/body/mod.rs:21: mod stmt_loops;
src/hir/lower/body/mod.rs:22: mod ref_assign;
src/hir/lower/body/mod.rs:23: mod expr_method;
src/hir/lower/body/mod.rs:24: mod match_coverage;
src/hir/lower/body/mod.rs:25: mod match_lower;
src/hir/lower/body/mod.rs:26: mod match_pattern;
src/hir/lower/body/mod.rs:27: mod match_util;
src/hir/lower/body/mod.rs:28: mod usage_infer;
src/hir/lower/body/mod.rs:29: mod macro_call;
src/hir/lower/body/mod.rs:30: mod expr_logic;
src/hir/lower/body/mod.rs:31: mod expr_lower;
src/hir/lower/body/mod.rs:32: mod if_expr;
src/hir/lower/body/mod.rs:33: mod ensure;
src/hir/lower/body/mod.rs:34: mod literal;
src/hir/lower/body/mod.rs:35: mod collect_enum;
src/hir/lower/body/mod.rs:36: mod const_coerce;
src/hir/lower/body/mod.rs:37: mod const_array;
src/hir/lower/body/mod.rs:38: mod const_struct;
src/hir/lower/body/mod.rs:39: mod const_eval;
src/hir/lower/body/mod.rs:40: mod const_fn;
src/hir/lower/body/mod.rs:41: mod collect_import;
src/hir/lower/body/mod.rs:42: mod expr_access;
src/hir/lower/body/mod.rs:43: mod expr_call;
src/hir/lower/body/mod.rs:44: mod expr_call_extra;
src/hir/lower/body/mod.rs:45: mod block_lower;
src/hir/lower/body/mod.rs:46: mod expr_cast;
src/hir/lower/body/mod.rs:47: mod expr_enum;
src/hir/lower/body/mod.rs:48: mod expr_misc;
src/hir/lower/body/mod.rs:49: mod expr_ops1;
src/hir/lower/body/overload_resolve.rs:4: fn same_base_name(a: &HirType, b: &HirType) -> bool
src/hir/lower/body/overload_resolve.rs:14: impl crate::hir::lower::Ctx
src/hir/lower/body/overload_resolve.rs:15: pub(crate) fn param_compatible(&self, param_ty: &HirType, arg_ty: &HirType) -> bool
src/hir/lower/body/overload_resolve.rs:60: pub(crate) fn resolve_fn_call(&self, name: &Symbol, arg_types: &[HirType]) -> Option<FnId>
src/hir/lower/body/overload_resolve.rs:128: pub(crate) fn receiver_matches_param(receiver: &HirType, param: &HirType) -> bool
src/hir/lower/body/overload_resolve.rs:168: pub(crate) fn resolve_fn_call_literals(&self, name: &Symbol, arg_types: &[HirType], lit_mask: &[bool]) -> Option<FnId>
src/hir/lower/body/overload_resolve.rs:188: pub(crate) fn resolve_method_literals(&self, receiver_type: &HirType, method_name: &Symbol, arg_types: &[HirType], lit_mask: &[bool]) -> Option<FnId>
src/hir/lower/body/overload_resolve.rs:211: pub(crate) fn resolve_method(&self, receiver_type: &HirType, method_name: &Symbol, arg_types: &[HirType]) -> Option<FnId>
src/hir/lower/body/ref_assign.rs:4: impl crate::hir::lower::Ctx
src/hir/lower/body/ref_assign.rs:6: pub(crate) fn try_lower_ref_assign(
src/hir/lower/body/stmt_loops.rs:3: impl crate::hir::lower::Ctx
src/hir/lower/body/stmt_loops.rs:5: pub(crate) fn lower_while(
src/hir/lower/body/stmt_loops.rs:47: pub(crate) fn lower_for(
src/hir/lower/body/stmt_loops.rs:126: fn for_bound_type(start: &HirNodeBox, end: &HirNodeBox, step: &HirNodeBox, span: &Span) -> Result<HirType>
src/hir/lower/body/stmt_loops.rs:155: fn coerce_for_bound(expr: HirNodeBox, target: &HirType, span: &Span) -> Result<HirNodeBox>
src/hir/lower/body/stmt_lower.rs:3: impl crate::hir::lower::Ctx
src/hir/lower/body/stmt_lower.rs:4: pub(crate) fn lower_stmt(&mut self, stmt: &Stmt) -> Result<HirStmt>
src/hir/lower/body/usage_infer.rs:8: impl crate::hir::lower::Ctx
src/hir/lower/body/usage_infer.rs:10: pub(crate) fn collect_usage_hints(&self, stmts: &[Stmt]) -> HashMap<Symbol, Vec<Type>>
src/hir/lower/body/usage_infer.rs:26: fn find_elem_usage_stmts(&self, var: &Symbol, stmts: &[Stmt]) -> Option<Type>
src/hir/lower/body/usage_infer.rs:51: fn find_elem_usage(&self, var: &Symbol, stmt: &Stmt) -> Option<Type>
src/hir/lower/body/usage_infer.rs:69: fn infer_expr_type_ast(&self, e: &Expr) -> Option<Type>
src/hir/lower/body/vtables.rs:3: impl crate::hir::lower::Ctx
src/hir/lower/body/vtables.rs:6: pub(crate) fn build_vtables(&mut self) -> Result<()>
src/hir/lower/body/vtables.rs:76: pub(crate) fn try_match_interface(
src/hir/lower/body/vtables.rs:138: pub(crate) fn try_match_generic_interface(
src/hir/lower/ctx.rs:21: pub(crate) struct Ctx
src/hir/lower/ctx.rs:87: impl Ctx
src/hir/lower/ctx.rs:89: pub fn new() -> Self
src/hir/lower/ctx.rs:140: pub fn push_scope(&mut self) { self.scopes.push(HashMap::new()); }
src/hir/lower/ctx.rs:143: pub fn pop_scope(&mut self) { self.scopes.pop(); }
src/hir/lower/ctx.rs:146: pub fn bind_var(&mut self, name: Symbol, id: VarId, ty: HirType, mutable: bool)
src/hir/lower/ctx.rs:151: pub fn lookup_var(&self, name: &Symbol) -> Option<(VarId, HirType, bool)>
src/hir/lower/ctx.rs:159: pub fn find_field_index(&self, struct_ty: &HirType, field: &Symbol, span: &Span) -> Result<usize>
src/hir/lower/ctx.rs:172: fn find_field_index_by_name(&self, type_name: &Symbol, field: &Symbol, span: &Span) -> Result<usize>
src/hir/lower/ctx.rs:189: pub fn find_field_type(&self, struct_ty: &HirType, field: &Symbol, span: &Span) -> Result<HirType>
src/hir/lower/ctx.rs:207: fn find_field_type_by_name(&self, type_name: &Symbol, field: &Symbol, span: &Span) -> Result<HirType>
src/hir/lower/ctx.rs:226: pub fn variant_payload_type(&self, enum_ty: &HirType, data_field: &Symbol, span: &Span) -> Result<HirType>
src/hir/lower/ctx.rs:248: pub(crate) fn build_generic_subst(&self, type_name: &Symbol, base: &Symbol) -> HashMap<Symbol, HirType>
src/hir/lower/ctx.rs:266: pub fn collected_generic_params(&self, type_name: &Symbol) -> Vec<(Symbol, Option<Symbol>)>
src/hir/lower/ctx.rs:271: pub fn register_or_lookup(&mut self, name: Symbol, inferred_ty: HirType) -> (VarId, HirType, bool)
src/hir/lower/ctx.rs:281: pub(crate) fn receiver_type_known(&self, ty: &HirType) -> bool
src/hir/lower/ctx.rs:294: pub fn is_enum_type(&self, type_name: &Symbol) -> bool
src/hir/lower/ctx_mono.rs:6: fn rewrite_variant_name(ty: &HirType, base: Symbol, suffix: &str) -> HirType
src/hir/lower/ctx_mono.rs:18: impl crate::hir::lower::Ctx
src/hir/lower/ctx_mono.rs:20: pub fn instantiate_type(&mut self, ty: &HirType) -> Result<()>
src/hir/lower/ctx_mono.rs:35: pub fn instantiate_named(&mut self, name: Symbol) -> Result<()>
src/hir/lower/ctx_mono.rs:89: pub fn adapt_enum_args(
src/hir/lower/ctx_mono.rs:109: pub fn instantiate_enum_value(&mut self, node: HirNodeBox, expected: &HirType) -> Result<HirNodeBox>
src/hir/lower/ctx_mono.rs:141: pub fn refine_enum_result_type(&self, ty: HirType) -> HirType
src/hir/lower/ctx_mono.rs:156: pub fn update_var_type(&mut self, var_id: VarId, new_ty: HirType)
src/hir/lower/helpers/caller.rs:7: pub(crate) fn is_hidden_param(name: &Symbol) -> bool
src/hir/lower/helpers/caller.rs:12: pub(crate) fn count_hidden_params(params: &[(Symbol, HirType)]) -> usize
src/hir/lower/helpers/caller.rs:17: pub(crate) fn count_hidden_names<T>(params: &[(Symbol, T)]) -> usize
src/hir/lower/helpers/caller.rs:21: impl crate::hir::lower::Ctx
src/hir/lower/helpers/caller.rs:23: pub(crate) fn caller_hidden_args(&self, span: &crate::span::Span, count: usize) -> Vec<crate::hir::HirNodeBox>
src/hir/lower/helpers/caller.rs:43: impl crate::hir::lower::Ctx
src/hir/lower/helpers/caller.rs:45: pub(crate) fn append_caller_args(
src/hir/lower/helpers/closure.rs:13: pub(crate) fn collect_captures(
src/hir/lower/helpers/closure.rs:25: fn enclosing_var_type(ctx: &crate::hir::lower::Ctx, name: &Symbol) -> Option<HirType>
src/hir/lower/helpers/closure.rs:35: fn use_name(
src/hir/lower/helpers/closure.rs:53: fn assign_name(
src/hir/lower/helpers/closure.rs:72: fn ensure_capturable(name: &Symbol, ty: &HirType, span: &Span) -> Result<()>
src/hir/lower/helpers/closure.rs:83: fn walk_block(
src/hir/lower/helpers/closure.rs:100: fn walk_stmt(
src/hir/lower/helpers/closure.rs:155: fn walk_match(
src/hir/lower/helpers/closure.rs:179: fn walk_expr(
src/hir/lower/helpers/closure_checks.rs:11: pub(crate) fn find_capture_move_out(stmts: &[HirStmt], env_var: VarId) -> Option<Symbol>
src/hir/lower/helpers/closure_checks.rs:15: fn stmt_move_out(stmt: &HirStmt, env_var: VarId) -> Option<Symbol>
src/hir/lower/helpers/closure_checks.rs:47: fn node_move_out(node: &dyn HirNode, env_var: VarId) -> Option<Symbol>
src/hir/lower/helpers/closure_lower.rs:17: pub struct LambdaEnv
src/hir/lower/helpers/closure_lower.rs:26: impl LambdaEnv
src/hir/lower/helpers/closure_lower.rs:27: pub fn lookup(&self, name: &Symbol) -> Option<(usize, HirType)>
src/hir/lower/helpers/closure_lower.rs:33: pub(crate) fn closure_iface_name(params: &[HirType], ret: &HirType) -> Symbol
src/hir/lower/helpers/closure_lower.rs:45: pub(crate) fn mangle_for_symbol(ty: &HirType) -> String
src/hir/lower/helpers/closure_lower.rs:65: pub(crate) fn find_return_type(stmts: &[crate::hir::HirStmt]) -> Option<HirType>
src/hir/lower/helpers/closure_lower.rs:90: impl crate::hir::lower::Ctx
src/hir/lower/helpers/closure_lower.rs:92: pub(crate) fn lower_closure_lambda(
src/hir/lower/helpers/closure_lower.rs:202: fn lower_closure_drop_fn(&mut self, fn_id: FnId, name: Symbol, params: &Vec<(Symbol, Type)>) -> Result<()>
src/hir/lower/helpers/closure_lower.rs:218: fn capture_source_expr(&mut self, name: &Symbol) -> Result<HirNodeBox>
src/hir/lower/helpers/closure_static.rs:15: impl crate::hir::lower::Ctx
src/hir/lower/helpers/closure_static.rs:17: pub(crate) fn make_static_closure(
src/hir/lower/helpers/closure_static.rs:33: fn ensure_closure_tramp(&mut self, ps: &[HirType], ret: &HirType) -> (Symbol, Symbol)
src/hir/lower/helpers/closure_static.rs:114: fn ensure_noop_drop(&mut self) -> FnId
src/hir/lower/helpers/closure_static.rs:157: fn fn_value_type(sig: &FnSig) -> HirType
src/hir/lower/helpers/closure_static.rs:162: impl crate::hir::lower::Ctx
src/hir/lower/helpers/closure_static.rs:164: pub(crate) fn lower_extern_fn_arg(&mut self, ast: &Expr, span: &Span) -> Result<HirNodeBox>
src/hir/lower/helpers/coerce.rs:6: pub(crate) fn implicit_cast_ok(from: &HirType, to: &HirType) -> bool
src/hir/lower/helpers/coerce.rs:12: pub(crate) fn auto_deref(expr: HirNodeBox) -> HirNodeBox
src/hir/lower/helpers/coerce.rs:21: pub(crate) fn deref_type(ty: &HirType) -> HirType
src/hir/lower/helpers/coerce.rs:29: pub(crate) fn as_int_literal(e: &HirNodeBox) -> Option<i64>
src/hir/lower/helpers/coerce.rs:37: pub(crate) fn as_float_literal(e: &HirNodeBox) -> Option<f64>
src/hir/lower/helpers/coerce.rs:45: pub(crate) fn is_numeric_literal(e: &HirNodeBox) -> bool
src/hir/lower/helpers/coerce.rs:50: pub(crate) fn coerce_index(expr: HirNodeBox, target: &HirType, span: &Span) -> Result<HirNodeBox>
src/hir/lower/helpers/coerce.rs:66: pub(crate) fn is_float_type(ty: &HirType) -> bool
src/hir/lower/helpers/coerce.rs:71: pub(crate) fn retype_float_literal(expr: HirNodeBox, target: &HirType) -> HirNodeBox
src/hir/lower/helpers/coerce.rs:83: pub(crate) fn is_int_type(ty: &HirType) -> bool
src/hir/lower/helpers/coerce.rs:88: pub(crate) fn retype_int_literal(expr: HirNodeBox, target: &HirType) -> HirNodeBox
src/hir/lower/helpers/coerce.rs:95: fn is_primitive(ty: &HirType) -> bool
src/hir/lower/helpers/coerce.rs:103: pub(crate) fn coerce_expr(expr: HirNodeBox, target: &HirType, span: &Span) -> Result<HirNodeBox>
src/hir/lower/helpers/convert.rs:3: pub(crate) fn hir_type_to_ast_type(ty: &HirType) -> Type
src/hir/lower/helpers/convert.rs:34: pub(crate) fn infer_generic_from_param<'a>(param_ty: &'a Type, arg_ty: &'a HirType) -> Vec<(Symbol, HirType)>
src/hir/lower/helpers/convert.rs:113: pub(crate) fn generic_inner(s: &str) -> Option<&str>
src/hir/lower/helpers/convert.rs:132: pub(crate) fn split_generic_args(s: &str) -> Vec<&str>
src/hir/lower/helpers/convert.rs:152: pub(crate) fn substitute_hir_type(ty: &HirType, subst: &HashMap<Symbol, HirType>) -> HirType
src/hir/lower/helpers/fn_type.rs:9: pub(crate) fn ast_type_to_hir_extern(ty: &Type, interfaces: &HashMap<Symbol, InterfaceReg>) -> HirType
src/hir/lower/helpers/fn_type.rs:20: pub(crate) fn ast_type_to_hir_param(
src/hir/lower/helpers/fn_type.rs:34: pub(crate) fn unify_legacy_fn_type(ty: &HirType) -> HirType
src/hir/lower/helpers/mod.rs:18: mod caller;
src/hir/lower/helpers/mod.rs:19: mod closure;
src/hir/lower/helpers/mod.rs:20: mod closure_checks;
src/hir/lower/helpers/mod.rs:21: mod closure_lower;
src/hir/lower/helpers/mod.rs:22: mod closure_static;
src/hir/lower/helpers/mod.rs:23: mod coerce;
src/hir/lower/helpers/mod.rs:24: mod overflow;
src/hir/lower/helpers/mod.rs:25: mod convert;
src/hir/lower/helpers/mod.rs:26: mod fn_type;
src/hir/lower/helpers/mod.rs:27: mod substitute;
src/hir/lower/helpers/mod.rs:28: mod types;
src/hir/lower/helpers/mod.rs:29: mod wrap;
src/hir/lower/helpers/overflow.rs:10: pub(crate) fn ovf_op_name(op: &BinaryOp) -> Option<&'static str>
src/hir/lower/helpers/overflow.rs:19: impl crate::hir::lower::Ctx
src/hir/lower/helpers/overflow.rs:21: pub(crate) fn lower_ovf_call(&mut self, op_name: &str, ty: &HirType, lhs: HirNodeBox, rhs: HirNodeBox, span: &Span) -> HirNodeBox
src/hir/lower/helpers/overflow.rs:29: pub(crate) fn ensure_ovf_helper(&mut self, op: &str, ty: &HirType) -> FnId
src/hir/lower/helpers/substitute.rs:4: pub(crate) fn substitute_type_in_type(ty: &Type, subst: &HashMap<Symbol, Type>) -> Type
src/hir/lower/helpers/substitute.rs:33: pub(crate) fn substitute_type_in_expr(expr: &Expr, subst: &HashMap<Symbol, Type>) -> Expr
src/hir/lower/helpers/substitute.rs:151: pub(crate) fn substitute_type_in_block(block: &Block, subst: &HashMap<Symbol, Type>) -> Block
src/hir/lower/helpers/substitute.rs:160: pub(crate) fn substitute_type_in_stmt(stmt: &Stmt, subst: &HashMap<Symbol, Type>) -> Stmt
src/hir/lower/helpers/substitute.rs:287: fn substitute_match_arm(a: &MatchArm, subst: &HashMap<Symbol, Type>) -> MatchArm
src/hir/lower/helpers/types.rs:3: pub(crate) fn type_to_string_generic(ty: &Type, interfaces: &HashMap<Symbol, InterfaceReg>) -> String
src/hir/lower/helpers/types.rs:34: pub(crate) fn fixed_width_type(name: &str) -> Option<HirType>
src/hir/lower/helpers/types.rs:53: pub(crate) fn intn_name(bits: u8, signed: bool) -> String
src/hir/lower/helpers/types.rs:57: pub(crate) fn sig_str_to_hir(s: &str) -> HirType
src/hir/lower/helpers/types.rs:120: pub(crate) fn normalize_generic_brackets(s: &str) -> String
src/hir/lower/helpers/types.rs:146: fn is_iface_type(inner_hir: &HirType, interfaces: &HashMap<Symbol, super::InterfaceReg>) -> bool
src/hir/lower/helpers/types.rs:157: pub(crate) fn ast_type_to_hir(ty: &Type, interfaces: &HashMap<Symbol, InterfaceReg>) -> HirType
src/hir/lower/helpers/types.rs:228: pub(crate) fn extract_named(ty: &HirType) -> Option<&Symbol>
src/hir/lower/helpers/types.rs:237: pub(crate) fn hir_type_display(ty: &HirType) -> String
src/hir/lower/helpers/types.rs:269: pub(crate) fn strip_ownership(ty: HirType) -> HirType
src/hir/lower/helpers/types.rs:277: pub(crate) fn strip_ownership_ref(ty: &HirType) -> &HirType
src/hir/lower/helpers/types.rs:285: pub(crate) fn expr_type(expr: &HirNodeBox) -> HirType
src/hir/lower/helpers/types.rs:290: pub(crate) fn is_null_literal(expr: &HirNodeBox) -> bool
src/hir/lower/helpers/types.rs:295: pub(crate) fn is_pointer_type_for_cmp(ty: &HirType) -> bool
src/hir/lower/helpers/wrap.rs:3: pub(crate) fn implicit_move(expr: HirNodeBox) -> HirNodeBox
src/hir/lower/helpers/wrap.rs:14: fn wrap_arg_conversions(arg: HirNodeBox, param_ty: &HirType) -> HirNodeBox
src/hir/lower/helpers/wrap.rs:93: pub(crate) fn wrap_arg_for_param(arg: HirNodeBox, param_ty: &HirType) -> HirNodeBox
src/hir/lower/helpers/wrap.rs:102: pub(crate) fn wrap_arg_borrowed(arg: HirNodeBox, param_ty: &HirType) -> HirNodeBox
src/hir/lower/helpers/wrap.rs:112: pub(crate) fn wrap_for_unique_param(expr: HirNodeBox, param_ty: &HirType) -> HirNodeBox
src/hir/lower/helpers/wrap.rs:123: pub(crate) fn binary_op_to_fn_name(op: &BinaryOp) -> Option<&'static str>
src/hir/lower/helpers/wrap.rs:147: pub(crate) fn unary_op_to_fn_name(op: &UnaryOp) -> Option<&'static str>
src/hir/lower/mod.rs:1: pub mod body;
src/hir/lower/mod.rs:2: pub mod helpers;
src/hir/lower/mod.rs:3: pub mod to_mir;
src/hir/lower/mod.rs:16: pub(super) fn variant_struct_name(enum_name: &Symbol, variant: &Symbol) -> Symbol
src/hir/lower/mod.rs:26: pub(super) fn strip_generic_name(name: &Symbol) -> Symbol
src/hir/lower/mod.rs:42: pub(crate) struct FnSig
src/hir/lower/mod.rs:63: pub(crate) struct InterfaceReg
src/hir/lower/mod.rs:70: mod ctx;
src/hir/lower/mod.rs:71: mod ctx_mono;
src/hir/lower/mod.rs:88: static SOURCE_PATH: std::sync::OnceLock<String> = std::sync::OnceLock::new();
src/hir/lower/mod.rs:90: pub fn set_source_path(path: &std::path::Path)
src/hir/lower/mod.rs:94: pub(crate) fn source_path() -> String
src/hir/lower/mod.rs:98: pub fn lower_program(program: &Program) -> Result<HirProgram>
src/hir/lower/to_mir/access.rs:3: impl HirNode for SField
src/hir/lower/to_mir/access.rs:4: fn clone_node(&self) -> Box<dyn HirNode> { Box::new(self.clone()) }
src/hir/lower/to_mir/access.rs:5: fn as_field_access(&self) -> Option<(&HirNodeBox, Symbol, usize)>
src/hir/lower/to_mir/access.rs:8: fn lower_to_mir(&self, moved: &HashSet<VarId>) -> MirNodeBox
src/hir/lower/to_mir/access.rs:16: fn display(&self, level: usize, w: &mut dyn std::fmt::Write) -> std::fmt::Result
src/hir/lower/to_mir/access.rs:21: fn expr_type(&self) -> HirType { self.ty.clone() }
src/hir/lower/to_mir/access.rs:22: fn for_each_child(&self, f: &mut dyn FnMut(&dyn HirNode))
src/hir/lower/to_mir/access.rs:27: impl HirNode for SStruct
src/hir/lower/to_mir/access.rs:28: fn clone_node(&self) -> Box<dyn HirNode> { Box::new(self.clone()) }
src/hir/lower/to_mir/access.rs:29: fn as_struct_cloned(&self) -> Option<SStruct> { Some(self.clone()) }
src/hir/lower/to_mir/access.rs:30: fn lower_to_mir(&self, moved: &HashSet<VarId>) -> MirNodeBox
src/hir/lower/to_mir/access.rs:37: fn display(&self, level: usize, w: &mut dyn std::fmt::Write) -> std::fmt::Result
src/hir/lower/to_mir/access.rs:45: fn expr_type(&self) -> HirType { self.ty.clone() }
src/hir/lower/to_mir/access.rs:46: fn for_each_child(&self, f: &mut dyn FnMut(&dyn HirNode))
src/hir/lower/to_mir/access.rs:51: impl HirNode for SArrLit
src/hir/lower/to_mir/access.rs:52: fn is_alloc(&self) -> bool { true }
src/hir/lower/to_mir/access.rs:53: fn clone_node(&self) -> Box<dyn HirNode> { Box::new(self.clone()) }
src/hir/lower/to_mir/access.rs:54: fn lower_to_mir(&self, moved: &HashSet<VarId>) -> MirNodeBox
src/hir/lower/to_mir/access.rs:60: fn display(&self, level: usize, w: &mut dyn std::fmt::Write) -> std::fmt::Result
src/hir/lower/to_mir/access.rs:65: fn expr_type(&self) -> HirType { self.ty.clone() }
src/hir/lower/to_mir/access.rs:66: fn for_each_child(&self, f: &mut dyn FnMut(&dyn HirNode))
src/hir/lower/to_mir/access.rs:71: impl HirNode for SArrSz
src/hir/lower/to_mir/access.rs:72: fn is_alloc(&self) -> bool { true }
src/hir/lower/to_mir/access.rs:73: fn clone_node(&self) -> Box<dyn HirNode> { Box::new(self.clone()) }
src/hir/lower/to_mir/access.rs:74: fn lower_to_mir(&self, moved: &HashSet<VarId>) -> MirNodeBox
src/hir/lower/to_mir/access.rs:81: fn display(&self, level: usize, w: &mut dyn std::fmt::Write) -> std::fmt::Result
src/hir/lower/to_mir/access.rs:87: fn expr_type(&self) -> HirType { self.ty.clone() }
src/hir/lower/to_mir/access.rs:88: fn for_each_child(&self, f: &mut dyn FnMut(&dyn HirNode))
src/hir/lower/to_mir/access.rs:93: impl HirNode for SRef
src/hir/lower/to_mir/access.rs:94: fn clone_node(&self) -> Box<dyn HirNode> { Box::new(self.clone()) }
src/hir/lower/to_mir/access.rs:95: fn lower_to_mir(&self, moved: &HashSet<VarId>) -> MirNodeBox
src/hir/lower/to_mir/access.rs:98: fn display(&self, level: usize, w: &mut dyn std::fmt::Write) -> std::fmt::Result
src/hir/lower/to_mir/access.rs:104: fn expr_type(&self) -> HirType { self.ty.clone() }
src/hir/lower/to_mir/access.rs:105: fn for_each_child(&self, f: &mut dyn FnMut(&dyn HirNode))
src/hir/lower/to_mir/access.rs:110: impl HirNode for SDeref
src/hir/lower/to_mir/access.rs:111: fn clone_node(&self) -> Box<dyn HirNode> { Box::new(self.clone()) }
src/hir/lower/to_mir/access.rs:112: fn lower_to_mir(&self, moved: &HashSet<VarId>) -> MirNodeBox
src/hir/lower/to_mir/access.rs:115: fn display(&self, level: usize, w: &mut dyn std::fmt::Write) -> std::fmt::Result
src/hir/lower/to_mir/access.rs:120: fn expr_type(&self) -> HirType { self.ty.clone() }
src/hir/lower/to_mir/access.rs:121: fn for_each_child(&self, f: &mut dyn FnMut(&dyn HirNode))
src/hir/lower/to_mir/access.rs:126: impl HirNode for SIdx
src/hir/lower/to_mir/access.rs:127: fn clone_node(&self) -> Box<dyn HirNode> { Box::new(self.clone()) }
src/hir/lower/to_mir/access.rs:128: fn lower_to_mir(&self, moved: &HashSet<VarId>) -> MirNodeBox
src/hir/lower/to_mir/access.rs:135: fn display(&self, level: usize, w: &mut dyn std::fmt::Write) -> std::fmt::Result
src/hir/lower/to_mir/access.rs:143: fn expr_type(&self) -> HirType { self.ty.clone() }
src/hir/lower/to_mir/access.rs:144: fn for_each_child(&self, f: &mut dyn FnMut(&dyn HirNode))
src/hir/lower/to_mir/basic.rs:3: impl HirNode for SVar
src/hir/lower/to_mir/basic.rs:4: fn clone_node(&self) -> Box<dyn HirNode> { Box::new(self.clone()) }
src/hir/lower/to_mir/basic.rs:5: fn lower_to_mir(&self, moved: &HashSet<VarId>) -> MirNodeBox
src/hir/lower/to_mir/basic.rs:8: fn display(&self, level: usize, w: &mut dyn std::fmt::Write) -> std::fmt::Result
src/hir/lower/to_mir/basic.rs:11: fn expr_type(&self) -> HirType { self.ty.clone() }
src/hir/lower/to_mir/basic.rs:12: fn as_local(&self) -> Option<VarId> { Some(self.var) }
src/hir/lower/to_mir/basic.rs:13: fn collect_var_ids(&self, vars: &mut HashSet<VarId>) { vars.insert(self.var); }
src/hir/lower/to_mir/basic.rs:14: fn for_each_child(&self, _f: &mut dyn FnMut(&dyn HirNode))
src/hir/lower/to_mir/basic.rs:18: impl HirNode for SConst
src/hir/lower/to_mir/basic.rs:19: fn is_alloc(&self) -> bool { matches!(self.val, HirLiteral::String(_)) }
src/hir/lower/to_mir/basic.rs:20: fn with_type(&self, ty: HirType) -> Option<HirNodeBox>
src/hir/lower/to_mir/basic.rs:23: fn clone_node(&self) -> Box<dyn HirNode> { Box::new(self.clone()) }
src/hir/lower/to_mir/basic.rs:24: fn lower_to_mir(&self, _moved: &HashSet<VarId>) -> MirNodeBox
src/hir/lower/to_mir/basic.rs:27: fn display(&self, level: usize, w: &mut dyn std::fmt::Write) -> std::fmt::Result
src/hir/lower/to_mir/basic.rs:39: fn expr_type(&self) -> HirType { self.ty.clone() }
src/hir/lower/to_mir/basic.rs:40: fn as_const(&self) -> Option<&HirLiteral> { Some(&self.val) }
src/hir/lower/to_mir/basic.rs:41: fn for_each_child(&self, _f: &mut dyn FnMut(&dyn HirNode))
src/hir/lower/to_mir/basic.rs:46: impl HirNode for SGlobal
src/hir/lower/to_mir/basic.rs:47: fn clone_node(&self) -> Box<dyn HirNode> { Box::new(self.clone()) }
src/hir/lower/to_mir/basic.rs:48: fn lower_to_mir(&self, _moved: &HashSet<VarId>) -> MirNodeBox
src/hir/lower/to_mir/basic.rs:51: fn display(&self, level: usize, w: &mut dyn std::fmt::Write) -> std::fmt::Result
src/hir/lower/to_mir/basic.rs:54: fn expr_type(&self) -> HirType
src/hir/lower/to_mir/basic.rs:57: fn for_each_child(&self, _f: &mut dyn FnMut(&dyn HirNode))
src/hir/lower/to_mir/basic.rs:63: impl HirNode for SFileArg
src/hir/lower/to_mir/basic.rs:64: fn clone_node(&self) -> Box<dyn HirNode> { Box::new(self.clone()) }
src/hir/lower/to_mir/basic.rs:65: fn lower_to_mir(&self, _moved: &HashSet<VarId>) -> MirNodeBox
src/hir/lower/to_mir/basic.rs:68: fn display(&self, level: usize, w: &mut dyn std::fmt::Write) -> std::fmt::Result
src/hir/lower/to_mir/basic.rs:71: fn expr_type(&self) -> HirType { self.ty.clone() }
src/hir/lower/to_mir/basic.rs:72: fn for_each_child(&self, _f: &mut dyn FnMut(&dyn HirNode))
src/hir/lower/to_mir/basic.rs:76: impl HirNode for SBin
src/hir/lower/to_mir/basic.rs:77: fn clone_node(&self) -> Box<dyn HirNode> { Box::new(self.clone()) }
src/hir/lower/to_mir/basic.rs:78: fn lower_to_mir(&self, moved: &HashSet<VarId>) -> MirNodeBox
src/hir/lower/to_mir/basic.rs:86: fn display(&self, level: usize, w: &mut dyn std::fmt::Write) -> std::fmt::Result
src/hir/lower/to_mir/basic.rs:94: fn expr_type(&self) -> HirType { self.ty.clone() }
src/hir/lower/to_mir/basic.rs:95: fn is_comparison(&self) -> bool
src/hir/lower/to_mir/basic.rs:99: fn for_each_child(&self, f: &mut dyn FnMut(&dyn HirNode))
src/hir/lower/to_mir/basic.rs:105: impl HirNode for SUn
src/hir/lower/to_mir/basic.rs:106: fn clone_node(&self) -> Box<dyn HirNode> { Box::new(self.clone()) }
src/hir/lower/to_mir/basic.rs:107: fn lower_to_mir(&self, moved: &HashSet<VarId>) -> MirNodeBox
src/hir/lower/to_mir/basic.rs:110: fn display(&self, level: usize, w: &mut dyn std::fmt::Write) -> std::fmt::Result
src/hir/lower/to_mir/basic.rs:115: fn expr_type(&self) -> HirType { self.ty.clone() }
src/hir/lower/to_mir/basic.rs:116: fn as_neg_int_literal(&self) -> Option<i64>
src/hir/lower/to_mir/basic.rs:124: fn as_neg_float_literal(&self) -> Option<f64>
src/hir/lower/to_mir/basic.rs:132: fn for_each_child(&self, f: &mut dyn FnMut(&dyn HirNode))
src/hir/lower/to_mir/basic.rs:137: impl HirNode for SCall
src/hir/lower/to_mir/basic.rs:138: fn clone_node(&self) -> Box<dyn HirNode> { Box::new(self.clone()) }
src/hir/lower/to_mir/basic.rs:139: fn as_call(&self) -> Option<FnId> { Some(self.fn_id) }
src/hir/lower/to_mir/basic.rs:140: fn lower_to_mir(&self, moved: &HashSet<VarId>) -> MirNodeBox
src/hir/lower/to_mir/basic.rs:147: fn display(&self, level: usize, w: &mut dyn std::fmt::Write) -> std::fmt::Result
src/hir/lower/to_mir/basic.rs:152: fn expr_type(&self) -> HirType { self.ty.clone() }
src/hir/lower/to_mir/basic.rs:153: fn for_each_child(&self, f: &mut dyn FnMut(&dyn HirNode))
src/hir/lower/to_mir/basic.rs:158: impl HirNode for SMove
src/hir/lower/to_mir/basic.rs:159: fn clone_node(&self) -> Box<dyn HirNode> { Box::new(self.clone()) }
src/hir/lower/to_mir/basic.rs:160: fn lower_to_mir(&self, moved: &HashSet<VarId>) -> MirNodeBox
src/hir/lower/to_mir/basic.rs:163: fn display(&self, level: usize, w: &mut dyn std::fmt::Write) -> std::fmt::Result
src/hir/lower/to_mir/basic.rs:168: fn expr_type(&self) -> HirType { self.ty.clone() }
src/hir/lower/to_mir/basic.rs:169: fn is_move_or_clone(&self) -> bool { true }
src/hir/lower/to_mir/basic.rs:170: fn as_move(&self) -> Option<&HirNodeBox> { Some(&self.expr) }
src/hir/lower/to_mir/basic.rs:171: fn record_moves(&self, moved: &mut HashSet<VarId>)
src/hir/lower/to_mir/basic.rs:178: fn for_each_child(&self, f: &mut dyn FnMut(&dyn HirNode))
src/hir/lower/to_mir/basic.rs:183: impl HirNode for SClone
src/hir/lower/to_mir/basic.rs:184: fn clone_node(&self) -> Box<dyn HirNode> { Box::new(self.clone()) }
src/hir/lower/to_mir/basic.rs:185: fn lower_to_mir(&self, moved: &HashSet<VarId>) -> MirNodeBox
src/hir/lower/to_mir/basic.rs:188: fn display(&self, level: usize, w: &mut dyn std::fmt::Write) -> std::fmt::Result
src/hir/lower/to_mir/basic.rs:193: fn expr_type(&self) -> HirType { self.ty.clone() }
src/hir/lower/to_mir/basic.rs:194: fn is_move_or_clone(&self) -> bool { true }
src/hir/lower/to_mir/basic.rs:195: fn as_clone(&self) -> Option<&HirNodeBox> { Some(&self.expr) }
src/hir/lower/to_mir/basic.rs:196: fn for_each_child(&self, f: &mut dyn FnMut(&dyn HirNode))
src/hir/lower/to_mir/basic.rs:201: impl HirNode for SToUnique
src/hir/lower/to_mir/basic.rs:202: fn is_alloc(&self) -> bool { true }
src/hir/lower/to_mir/basic.rs:203: fn clone_node(&self) -> Box<dyn HirNode> { Box::new(self.clone()) }
src/hir/lower/to_mir/basic.rs:204: fn lower_to_mir(&self, moved: &HashSet<VarId>) -> MirNodeBox
src/hir/lower/to_mir/basic.rs:207: fn display(&self, level: usize, w: &mut dyn std::fmt::Write) -> std::fmt::Result
src/hir/lower/to_mir/basic.rs:212: fn expr_type(&self) -> HirType { self.ty.clone() }
src/hir/lower/to_mir/basic.rs:213: fn as_to_unique(&self) -> Option<&HirNodeBox> { Some(&self.expr) }
src/hir/lower/to_mir/basic.rs:214: fn for_each_child(&self, f: &mut dyn FnMut(&dyn HirNode))
src/hir/lower/to_mir/basic.rs:217: fn record_moves(&self, moved: &mut HashSet<VarId>)
src/hir/lower/to_mir/basic.rs:226: impl HirNode for SCast
src/hir/lower/to_mir/basic.rs:227: fn clone_node(&self) -> Box<dyn HirNode> { Box::new(self.clone()) }
src/hir/lower/to_mir/basic.rs:228: fn lower_to_mir(&self, moved: &HashSet<VarId>) -> MirNodeBox
src/hir/lower/to_mir/basic.rs:231: fn display(&self, level: usize, w: &mut dyn std::fmt::Write) -> std::fmt::Result
src/hir/lower/to_mir/basic.rs:236: fn expr_type(&self) -> HirType { self.ty.clone() }
src/hir/lower/to_mir/basic.rs:237: fn for_each_child(&self, f: &mut dyn FnMut(&dyn HirNode))
src/hir/lower/to_mir/basic.rs:240: fn record_moves(&self, moved: &mut HashSet<VarId>)
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
src/hir/lower/to_mir/call.rs:87: fn for_each_child(&self, _f: &mut dyn FnMut(&dyn HirNode))
src/hir/lower/to_mir/call.rs:91: impl HirNode for SCallP
src/hir/lower/to_mir/call.rs:92: fn clone_node(&self) -> Box<dyn HirNode> { Box::new(self.clone()) }
src/hir/lower/to_mir/call.rs:93: fn lower_to_mir(&self, moved: &HashSet<VarId>) -> MirNodeBox
src/hir/lower/to_mir/call.rs:100: fn display(&self, level: usize, w: &mut dyn std::fmt::Write) -> std::fmt::Result
src/hir/lower/to_mir/call.rs:103: fn expr_type(&self) -> HirType { self.ty.clone() }
src/hir/lower/to_mir/call.rs:104: fn for_each_child(&self, f: &mut dyn FnMut(&dyn HirNode))
src/hir/lower/to_mir/call.rs:110: impl HirNode for SEnumC
src/hir/lower/to_mir/call.rs:111: fn enum_variant(&self) -> Option<Symbol> { Some(self.variant_name) }
src/hir/lower/to_mir/call.rs:112: fn clone_node(&self) -> Box<dyn HirNode> { Box::new(self.clone()) }
src/hir/lower/to_mir/call.rs:113: fn lower_to_mir(&self, moved: &HashSet<VarId>) -> MirNodeBox
src/hir/lower/to_mir/call.rs:122: fn display(&self, level: usize, w: &mut dyn std::fmt::Write) -> std::fmt::Result
src/hir/lower/to_mir/call.rs:125: fn expr_type(&self) -> HirType { self.ty.clone() }
src/hir/lower/to_mir/call.rs:126: fn for_each_child(&self, f: &mut dyn FnMut(&dyn HirNode))
src/hir/lower/to_mir/call.rs:131: impl HirNode for SEnumM
src/hir/lower/to_mir/call.rs:132: fn clone_node(&self) -> Box<dyn HirNode> { Box::new(self.clone()) }
src/hir/lower/to_mir/call.rs:133: fn lower_to_mir(&self, moved: &HashSet<VarId>) -> MirNodeBox
src/hir/lower/to_mir/call.rs:140: fn display(&self, level: usize, w: &mut dyn std::fmt::Write) -> std::fmt::Result
src/hir/lower/to_mir/call.rs:146: fn expr_type(&self) -> HirType { self.ty.clone() }
src/hir/lower/to_mir/call.rs:147: fn for_each_child(&self, f: &mut dyn FnMut(&dyn HirNode))
src/hir/lower/to_mir/mod.rs:11: mod access;
src/hir/lower/to_mir/mod.rs:12: mod basic;
src/hir/lower/to_mir/mod.rs:13: mod call;
src/hir/mod.rs:1: pub mod attrs;
src/hir/mod.rs:2: pub mod attrs_export;
src/hir/mod.rs:3: pub mod attrs_macro;
src/hir/mod.rs:4: pub mod cfg;
src/hir/mod.rs:5: pub mod contracts;
src/hir/mod.rs:6: pub mod effects;
src/hir/mod.rs:7: pub mod ir;
src/hir/mod.rs:8: pub mod ty;
src/hir/mod.rs:9: pub mod node;
src/hir/mod.rs:10: pub mod stmt;
src/hir/mod.rs:11: pub mod item;
src/hir/mod.rs:12: pub mod lower;
src/hir/mod.rs:13: pub mod display;
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
src/hir/node.rs:30: fn as_neg_int_literal(&self) -> Option<i64> { None }
src/hir/node.rs:32: fn as_neg_float_literal(&self) -> Option<f64> { None }
src/hir/node.rs:33: fn as_move(&self) -> Option<&HirNodeBox> { None }
src/hir/node.rs:35: fn as_to_unique(&self) -> Option<&HirNodeBox> { None }
src/hir/node.rs:37: fn as_field_access(&self) -> Option<(&HirNodeBox, Symbol, usize)> { None }
src/hir/node.rs:39: fn is_comparison(&self) -> bool { false }
src/hir/node.rs:41: fn as_call(&self) -> Option<crate::hir::ty::FnId> { None }
src/hir/node.rs:43: fn is_alloc(&self) -> bool { false }
src/hir/node.rs:45: fn enum_variant(&self) -> Option<Symbol> { None }
src/hir/node.rs:47: fn as_struct_cloned(&self) -> Option<crate::hir::SStruct> { None }
src/hir/node.rs:49: fn with_type(&self, _ty: HirType) -> Option<HirNodeBox> { None }
src/hir/node.rs:50: fn as_clone(&self) -> Option<&HirNodeBox> { None }
src/hir/node.rs:55: pub struct HirNodeBox(pub Box<dyn HirNode>);
src/hir/node.rs:56: impl Clone for HirNodeBox
src/hir/node.rs:57: fn clone(&self) -> Self { HirNodeBox(self.0.clone_node()) }
src/hir/node.rs:59: impl std::ops::Deref for HirNodeBox
src/hir/node.rs:60: type Target = dyn HirNode;
src/hir/node.rs:61: fn deref(&self) -> &Self::Target { &*self.0 }
src/hir/node.rs:63: impl From<HirNodeBox> for Box<dyn HirNode>
src/hir/node.rs:64: fn from(b: HirNodeBox) -> Self { b.0 }
src/hir/node.rs:73: fn from(v: $ty) -> Self { HirNodeBox(Box::new(v) as Box<dyn HirNode>) }
src/hir/stmt.rs:6: pub enum ContractKind
src/hir/stmt.rs:12: impl ContractKind
src/hir/stmt.rs:13: pub fn label(&self) -> &'static str
src/hir/stmt.rs:22: pub fn runtime_fn(&self) -> &'static str
src/hir/stmt.rs:32: pub enum HirStmt
src/hir/stmt.rs:84: impl HirStmt
src/hir/stmt.rs:86: pub fn span(&self) -> Span
src/hir/stmt.rs:113: pub struct HirBlock
src/hir/stmt.rs:117: impl HirBlock
src/hir/stmt.rs:118: pub fn new(stmts: Vec<HirStmt>) -> Self
src/hir/ty.rs:4: pub struct VarId(pub usize);
src/hir/ty.rs:9: pub struct FnId(pub usize);
src/hir/ty.rs:19: pub enum HirType
src/hir/ty.rs:50: impl HirType
src/hir/ty.rs:52: pub fn is_copy(&self) -> bool
src/hir/ty.rs:71: pub enum HirLiteral
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
src/lexer/keyword.rs:48: impl fmt::Display for Keyword
src/lexer/keyword.rs:49: fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result
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
src/lexer/lexer/reading.rs:91: fn read_escape(&mut self) -> Option<char>
src/lexer/lexer/reading.rs:111: pub(super) fn read_char_literal(&mut self) -> (String, usize, usize, usize)
src/lexer/lexer/reading.rs:138: pub(super) fn read_string_literal(&mut self) -> (String, usize, usize, usize)
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
src/lib.rs:17: pub mod compiler;
src/lib.rs:18: pub mod diagnostics;
src/lib.rs:19: pub mod driver;
src/lib.rs:20: pub mod error;
src/lib.rs:21: pub mod formatter;
src/lib.rs:22: pub mod hir;
src/lib.rs:23: pub mod intern;
src/lib.rs:24: pub mod lexer;
src/lib.rs:25: pub mod lir;
src/lib.rs:26: pub mod mir;
src/lib.rs:27: pub mod package;
src/lib.rs:28: pub mod parser;
src/lib.rs:29: pub mod span;
src/lib.rs:32: mod test_parse
src/lib.rs:34: fn test_struct_literal_parse()
src/lir/display.rs:5: pub fn lir_program_to_string(prog: &LirProgram) -> String
src/lir/display.rs:19: fn write_fn(f: &LirFn, w: &mut impl Write) -> std::fmt::Result
src/lir/display.rs:34: fn write_inst(inst: &LirNodeBox, w: &mut impl Write) -> std::fmt::Result
src/lir/emit/consts.rs:5: fn strip_unique(ty: &HirType) -> &HirType
src/lir/emit/consts.rs:12: fn path_key(path: &[usize]) -> String
src/lir/emit/consts.rs:16: impl<'a> Emitter<'a>
src/lir/emit/consts.rs:21: pub(super) fn emit_global_defs(&mut self)
src/lir/emit/consts.rs:58: fn collect_array_fields(
src/lir/emit/consts.rs:90: fn const_value_at(
src/lir/emit/functions.rs:4: pub(super) fn is_ptr_like(t: &HirType) -> bool
src/lir/emit/functions.rs:14: pub(super) fn llvm_attr_suffix(
src/lir/emit/functions.rs:56: pub(super) fn infer_param_attrs(t: &HirType) -> &'static str
src/lir/emit/functions.rs:66: pub(super) fn infer_ret_attr(t: &HirType) -> &'static str
src/lir/emit/functions.rs:75: pub(super) fn llvm_param_attrs(attrs: &[LirAttr]) -> String
src/lir/emit/functions.rs:87: impl<'a> Emitter<'a>
src/lir/emit/functions.rs:88: pub(super) fn emit_struct_defs(&mut self)
src/lir/emit/functions.rs:108: pub(super) fn emit_string_globals(&mut self)
src/lir/emit/functions.rs:128: pub(super) fn emit_fn(&mut self, f: &LirFn)
src/lir/emit/functions.rs:226: pub(super) fn emit_inst(&mut self, inst: &LirNodeBox)
src/lir/emit/functions.rs:243: pub(super) fn tmp(&mut self) -> u64
src/lir/emit/functions.rs:257: fn escape_llvm_string(s: &str) -> String
src/lir/emit/mod.rs:18: pub fn emit_program(prog: &LirProgram) -> String
src/lir/emit/mod.rs:28: struct Emitter<'a>
src/lir/emit/mod.rs:36: impl<'a> Emitter<'a>
src/lir/emit/mod.rs:37: fn new(prog: &'a LirProgram) -> Self
src/lir/emit/mod.rs:47: fn finish(self) -> String
src/lir/emit/mod.rs:51: fn wln(&mut self, s: &str)
src/lir/emit/mod.rs:67: fn wln_fmt(&mut self, fmt: std::fmt::Arguments<'_>)
src/lir/emit/mod.rs:76: fn emit(&mut self)
src/lir/emit/mod.rs:154: mod consts;
src/lir/emit/mod.rs:155: mod functions;
src/lir/emit/mod.rs:156: mod types;
src/lir/emit/mod.rs:157: mod vtable;
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
src/lir/ir/helpers.rs:12: pub(crate) fn lit_to_string(lit: &HirLiteral, expected_ty: &HirType) -> String
src/lir/ir/helpers.rs:47: pub(crate) fn float_binop(op: BinaryOp, ty: &HirType, dest: u64, l: &str, r: &str) -> String
src/lir/ir/helpers.rs:57: pub(crate) fn float_cmp(op: BinaryOp, ty: &HirType, dest: u64, l: &str, r: &str) -> String
src/lir/ir/helpers.rs:67: pub(crate) fn int_info(ty: &HirType) -> Option<(u32, bool)>
src/lir/ir/helpers.rs:77: pub(crate) fn llvm_type_size(ty: &HirType) -> &'static str
src/lir/ir/helpers.rs:97: pub(crate) fn elem_layout_size(
src/lir/ir/helpers.rs:101: fn layout(ty: &HirType, defs: &std::collections::HashMap<Symbol, Vec<(Symbol, HirType)>>) -> (u64, u64)
src/lir/ir/helpers.rs:140: pub(crate) fn struct_llvm_size(ty: &HirType, struct_defs: &std::collections::HashMap<Symbol, Vec<(Symbol, HirType)>>) -> String
src/lir/ir/helpers.rs:158: pub(super) fn is_pointer_type(ty: &HirType) -> bool
src/lir/ir/helpers.rs:171: pub(super) fn needs_drop(ty: &HirType, struct_defs: &HashMap<Symbol, Vec<(Symbol, HirType)>>) -> bool
src/lir/ir/helpers.rs:185: pub(super) fn emit_drop_value(
src/lir/ir/mod.rs:12: pub struct IrNode
src/lir/ir/mod.rs:18: pub enum IrValue
src/lir/ir/mod.rs:27: impl IrNode
src/lir/ir/mod.rs:28: pub fn new(kind: &str) -> Self
src/lir/ir/mod.rs:31: pub fn set(&mut self, name: &str, val: IrValue) -> &mut Self
src/lir/ir/mod.rs:35: pub fn get(&self, name: &str) -> Option<&IrValue>
src/lir/ir/mod.rs:41: pub enum ConvKind
src/lir/ir/mod.rs:47: pub enum LirValue
src/lir/ir/mod.rs:59: pub struct LirEmitCtx<'a>
src/lir/ir/mod.rs:65: impl LirEmitCtx<'_>
src/lir/ir/mod.rs:66: pub fn llvm_type(&self, ty: &HirType) -> String
src/lir/ir/mod.rs:104: pub fn value_ref(&self, val: &LirValue, expected_ty: &HirType) -> String
src/lir/ir/mod.rs:113: pub fn tmp(&mut self) -> u64
src/lir/ir/mod.rs:119: pub fn struct_llvm_name(&self, name: &Symbol) -> Option<String>
src/lir/ir/mod.rs:139: pub trait LirNode: std::fmt::Debug
src/lir/ir/mod.rs:140: fn clone_node(&self) -> Box<dyn LirNode>;
src/lir/ir/mod.rs:141: fn kind(&self) -> &'static str;
src/lir/ir/mod.rs:142: fn as_any(&self) -> &dyn std::any::Any;
src/lir/ir/mod.rs:143: fn emit(&self, ctx: &mut LirEmitCtx) -> Vec<String>;
src/lir/ir/mod.rs:146: fn alloca_lines(&self, _ctx: &LirEmitCtx) -> Vec<String> { Vec::new() }
src/lir/ir/mod.rs:147: fn display(&self, f: &mut dyn Write) -> std::fmt::Result;
src/lir/ir/mod.rs:148: fn serialize(&self, buf: &mut Vec<u8>);
src/lir/ir/mod.rs:156: pub struct LirNodeBox(pub Box<dyn LirNode>);
src/lir/ir/mod.rs:158: impl Clone for LirNodeBox
src/lir/ir/mod.rs:159: fn clone(&self) -> Self
src/lir/ir/mod.rs:164: impl std::ops::Deref for LirNodeBox
src/lir/ir/mod.rs:165: type Target = dyn LirNode;
src/lir/ir/mod.rs:166: fn deref(&self) -> &Self::Target
src/lir/ir/mod.rs:171: impl From<SLirCustom> for LirNodeBox
src/lir/ir/mod.rs:172: fn from(v: SLirCustom) -> Self { LirNodeBox(Box::new(v)) }
src/lir/ir/mod.rs:182: pub struct $name { $(pub $field: $ty),* }
src/lir/ir/mod.rs:189: fn from(v: $ty) -> Self { LirNodeBox(Box::new(v) as Box<dyn LirNode>) }
src/lir/ir/mod.rs:194: mod helpers;
src/lir/ir/mod.rs:195: mod nodes;
src/lir/ir/mod.rs:196: mod nodes_a;
src/lir/ir/mod.rs:197: mod nodes_b;
src/lir/ir/mod.rs:198: mod nodes_c;
src/lir/ir/mod.rs:199: mod nodes_d;
src/lir/ir/mod.rs:200: mod nodes_e;
src/lir/ir/mod.rs:201: mod nodes_f;
src/lir/ir/nodes_a.rs:3: impl LirNode for SLirAlloca
src/lir/ir/nodes_a.rs:4: fn clone_node(&self) -> Box<dyn LirNode> { Box::new(self.clone()) }
src/lir/ir/nodes_a.rs:5: fn kind(&self) -> &'static str { "Alloca" }
src/lir/ir/nodes_a.rs:6: fn as_any(&self) -> &dyn std::any::Any { self }
src/lir/ir/nodes_a.rs:7: fn emit(&self, _ctx: &mut LirEmitCtx) -> Vec<String> { Vec::new() }
src/lir/ir/nodes_a.rs:8: fn alloca_lines(&self, ctx: &LirEmitCtx) -> Vec<String>
src/lir/ir/nodes_a.rs:12: fn display(&self, f: &mut dyn Write) -> std::fmt::Result
src/lir/ir/nodes_a.rs:15: fn serialize(&self, buf: &mut Vec<u8>)
src/lir/ir/nodes_a.rs:22: impl LirNode for SLirStore
src/lir/ir/nodes_a.rs:23: fn clone_node(&self) -> Box<dyn LirNode> { Box::new(self.clone()) }
src/lir/ir/nodes_a.rs:24: fn kind(&self) -> &'static str { "Store" }
src/lir/ir/nodes_a.rs:25: fn as_any(&self) -> &dyn std::any::Any { self }
src/lir/ir/nodes_a.rs:26: fn emit(&self, ctx: &mut LirEmitCtx) -> Vec<String>
src/lir/ir/nodes_a.rs:42: fn display(&self, f: &mut dyn Write) -> std::fmt::Result
src/lir/ir/nodes_a.rs:45: fn serialize(&self, buf: &mut Vec<u8>)
src/lir/ir/nodes_a.rs:53: impl LirNode for SLirLoad
src/lir/ir/nodes_a.rs:54: fn clone_node(&self) -> Box<dyn LirNode> { Box::new(self.clone()) }
src/lir/ir/nodes_a.rs:55: fn kind(&self) -> &'static str { "Load" }
src/lir/ir/nodes_a.rs:56: fn as_any(&self) -> &dyn std::any::Any { self }
src/lir/ir/nodes_a.rs:57: fn emit(&self, ctx: &mut LirEmitCtx) -> Vec<String>
src/lir/ir/nodes_a.rs:61: fn display(&self, f: &mut dyn Write) -> std::fmt::Result
src/lir/ir/nodes_a.rs:64: fn serialize(&self, buf: &mut Vec<u8>)
src/lir/ir/nodes_a.rs:72: impl LirNode for SLirBinOp
src/lir/ir/nodes_a.rs:73: fn clone_node(&self) -> Box<dyn LirNode> { Box::new(self.clone()) }
src/lir/ir/nodes_a.rs:74: fn kind(&self) -> &'static str { "BinOp" }
src/lir/ir/nodes_a.rs:75: fn as_any(&self) -> &dyn std::any::Any { self }
src/lir/ir/nodes_a.rs:76: fn emit(&self, ctx: &mut LirEmitCtx) -> Vec<String>
src/lir/ir/nodes_a.rs:166: fn display(&self, f: &mut dyn Write) -> std::fmt::Result
src/lir/ir/nodes_a.rs:169: fn serialize(&self, buf: &mut Vec<u8>)
src/lir/ir/nodes_a.rs:180: impl LirNode for SLirUnaryOp
src/lir/ir/nodes_a.rs:181: fn clone_node(&self) -> Box<dyn LirNode> { Box::new(self.clone()) }
src/lir/ir/nodes_a.rs:182: fn kind(&self) -> &'static str { "UnaryOp" }
src/lir/ir/nodes_a.rs:183: fn as_any(&self) -> &dyn std::any::Any { self }
src/lir/ir/nodes_a.rs:184: fn emit(&self, ctx: &mut LirEmitCtx) -> Vec<String>
src/lir/ir/nodes_a.rs:200: fn display(&self, f: &mut dyn Write) -> std::fmt::Result
src/lir/ir/nodes_a.rs:203: fn serialize(&self, buf: &mut Vec<u8>)
src/lir/ir/nodes_a.rs:212: impl LirNode for SLirCall
src/lir/ir/nodes_a.rs:213: fn clone_node(&self) -> Box<dyn LirNode> { Box::new(self.clone()) }
src/lir/ir/nodes_a.rs:214: fn kind(&self) -> &'static str { "Call" }
src/lir/ir/nodes_a.rs:215: fn as_any(&self) -> &dyn std::any::Any { self }
src/lir/ir/nodes_a.rs:216: fn emit(&self, ctx: &mut LirEmitCtx) -> Vec<String>
src/lir/ir/nodes_a.rs:233: fn display(&self, f: &mut dyn Write) -> std::fmt::Result
src/lir/ir/nodes_a.rs:238: fn serialize(&self, buf: &mut Vec<u8>)
src/lir/ir/nodes_a.rs:248: impl LirNode for SLirCallPtr
src/lir/ir/nodes_a.rs:249: fn clone_node(&self) -> Box<dyn LirNode> { Box::new(self.clone()) }
src/lir/ir/nodes_a.rs:250: fn kind(&self) -> &'static str { "CallPtr" }
src/lir/ir/nodes_a.rs:251: fn as_any(&self) -> &dyn std::any::Any { self }
src/lir/ir/nodes_a.rs:252: fn emit(&self, ctx: &mut LirEmitCtx) -> Vec<String>
src/lir/ir/nodes_a.rs:264: fn display(&self, f: &mut dyn Write) -> std::fmt::Result
src/lir/ir/nodes_a.rs:268: fn serialize(&self, buf: &mut Vec<u8>)
src/lir/ir/nodes_a.rs:278: impl LirNode for SLirFnAddr
src/lir/ir/nodes_a.rs:279: fn clone_node(&self) -> Box<dyn LirNode> { Box::new(self.clone()) }
src/lir/ir/nodes_a.rs:280: fn kind(&self) -> &'static str { "FnAddr" }
src/lir/ir/nodes_a.rs:281: fn as_any(&self) -> &dyn std::any::Any { self }
src/lir/ir/nodes_a.rs:282: fn emit(&self, ctx: &mut LirEmitCtx) -> Vec<String>
src/lir/ir/nodes_a.rs:286: fn display(&self, f: &mut dyn Write) -> std::fmt::Result
src/lir/ir/nodes_a.rs:289: fn serialize(&self, buf: &mut Vec<u8>)
src/lir/ir/nodes_b.rs:3: impl LirNode for SLirConv
src/lir/ir/nodes_b.rs:4: fn alloca_lines(&self, ctx: &LirEmitCtx) -> Vec<String>
src/lir/ir/nodes_b.rs:16: fn clone_node(&self) -> Box<dyn LirNode> { Box::new(self.clone()) }
src/lir/ir/nodes_b.rs:17: fn kind(&self) -> &'static str { "Conv" }
src/lir/ir/nodes_b.rs:18: fn as_any(&self) -> &dyn std::any::Any { self }
src/lir/ir/nodes_b.rs:19: fn emit(&self, ctx: &mut LirEmitCtx) -> Vec<String>
src/lir/ir/nodes_b.rs:94: fn display(&self, f: &mut dyn Write) -> std::fmt::Result
src/lir/ir/nodes_b.rs:97: fn serialize(&self, buf: &mut Vec<u8>)
src/lir/ir/nodes_b.rs:104: impl LirNode for SLirBr
src/lir/ir/nodes_b.rs:105: fn clone_node(&self) -> Box<dyn LirNode> { Box::new(self.clone()) }
src/lir/ir/nodes_b.rs:106: fn kind(&self) -> &'static str { "Br" }
src/lir/ir/nodes_b.rs:107: fn as_any(&self) -> &dyn std::any::Any { self }
src/lir/ir/nodes_b.rs:108: fn emit(&self, _ctx: &mut LirEmitCtx) -> Vec<String>
src/lir/ir/nodes_b.rs:111: fn display(&self, f: &mut dyn Write) -> std::fmt::Result
src/lir/ir/nodes_b.rs:114: fn serialize(&self, buf: &mut Vec<u8>)
src/lir/ir/nodes_b.rs:120: impl LirNode for SLirBrCond
src/lir/ir/nodes_b.rs:121: fn clone_node(&self) -> Box<dyn LirNode> { Box::new(self.clone()) }
src/lir/ir/nodes_b.rs:122: fn kind(&self) -> &'static str { "BrCond" }
src/lir/ir/nodes_b.rs:123: fn as_any(&self) -> &dyn std::any::Any { self }
src/lir/ir/nodes_b.rs:124: fn emit(&self, ctx: &mut LirEmitCtx) -> Vec<String>
src/lir/ir/nodes_b.rs:128: fn display(&self, f: &mut dyn Write) -> std::fmt::Result
src/lir/ir/nodes_b.rs:131: fn serialize(&self, buf: &mut Vec<u8>)
src/lir/ir/nodes_b.rs:139: impl LirNode for SLirAssume
src/lir/ir/nodes_b.rs:140: fn clone_node(&self) -> Box<dyn LirNode> { Box::new(self.clone()) }
src/lir/ir/nodes_b.rs:141: fn kind(&self) -> &'static str { "Assume" }
src/lir/ir/nodes_b.rs:142: fn as_any(&self) -> &dyn std::any::Any { self }
src/lir/ir/nodes_b.rs:143: fn emit(&self, ctx: &mut LirEmitCtx) -> Vec<String>
src/lir/ir/nodes_b.rs:147: fn display(&self, f: &mut dyn Write) -> std::fmt::Result
src/lir/ir/nodes_b.rs:150: fn serialize(&self, buf: &mut Vec<u8>)
src/lir/ir/nodes_b.rs:156: impl LirNode for SLirContractCheck
src/lir/ir/nodes_b.rs:157: fn clone_node(&self) -> Box<dyn LirNode> { Box::new(self.clone()) }
src/lir/ir/nodes_b.rs:158: fn kind(&self) -> &'static str { "ContractCheck" }
src/lir/ir/nodes_b.rs:159: fn as_any(&self) -> &dyn std::any::Any { self }
src/lir/ir/nodes_b.rs:160: fn emit(&self, ctx: &mut LirEmitCtx) -> Vec<String>
src/lir/ir/nodes_b.rs:171: fn display(&self, f: &mut dyn Write) -> std::fmt::Result
src/lir/ir/nodes_b.rs:174: fn serialize(&self, buf: &mut Vec<u8>)
src/lir/ir/nodes_b.rs:187: impl LirNode for SLirRet
src/lir/ir/nodes_b.rs:188: fn clone_node(&self) -> Box<dyn LirNode> { Box::new(self.clone()) }
src/lir/ir/nodes_b.rs:189: fn kind(&self) -> &'static str { "Ret" }
src/lir/ir/nodes_b.rs:190: fn as_any(&self) -> &dyn std::any::Any { self }
src/lir/ir/nodes_b.rs:191: fn emit(&self, ctx: &mut LirEmitCtx) -> Vec<String>
src/lir/ir/nodes_b.rs:205: fn display(&self, f: &mut dyn Write) -> std::fmt::Result
src/lir/ir/nodes_b.rs:211: fn serialize(&self, buf: &mut Vec<u8>)
src/lir/ir/nodes_b.rs:220: impl LirNode for SLirMakeFatPtr
src/lir/ir/nodes_b.rs:221: fn clone_node(&self) -> Box<dyn LirNode> { Box::new(self.clone()) }
src/lir/ir/nodes_b.rs:222: fn kind(&self) -> &'static str { "MakeFatPtr" }
src/lir/ir/nodes_b.rs:223: fn as_any(&self) -> &dyn std::any::Any { self }
src/lir/ir/nodes_b.rs:224: fn emit(&self, ctx: &mut LirEmitCtx) -> Vec<String>
src/lir/ir/nodes_b.rs:253: fn display(&self, f: &mut dyn Write) -> std::fmt::Result
src/lir/ir/nodes_b.rs:256: fn serialize(&self, buf: &mut Vec<u8>)
src/lir/ir/nodes_b.rs:264: impl LirNode for SLirFieldAccess
src/lir/ir/nodes_b.rs:265: fn clone_node(&self) -> Box<dyn LirNode> { Box::new(self.clone()) }
src/lir/ir/nodes_b.rs:266: fn kind(&self) -> &'static str { "FieldAccess" }
src/lir/ir/nodes_b.rs:267: fn as_any(&self) -> &dyn std::any::Any { self }
src/lir/ir/nodes_b.rs:268: fn emit(&self, ctx: &mut LirEmitCtx) -> Vec<String>
src/lir/ir/nodes_b.rs:289: fn display(&self, f: &mut dyn Write) -> std::fmt::Result
src/lir/ir/nodes_b.rs:292: fn serialize(&self, buf: &mut Vec<u8>)
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
src/lir/ir/nodes_c.rs:72: fn display(&self, f: &mut dyn Write) -> std::fmt::Result
src/lir/ir/nodes_c.rs:75: fn serialize(&self, buf: &mut Vec<u8>)
src/lir/ir/nodes_c.rs:84: impl LirNode for SLirArrayLit
src/lir/ir/nodes_c.rs:85: fn clone_node(&self) -> Box<dyn LirNode> { Box::new(self.clone()) }
src/lir/ir/nodes_c.rs:86: fn kind(&self) -> &'static str { "ArrayLit" }
src/lir/ir/nodes_c.rs:87: fn as_any(&self) -> &dyn std::any::Any { self }
src/lir/ir/nodes_c.rs:88: fn emit(&self, ctx: &mut LirEmitCtx) -> Vec<String>
src/lir/ir/nodes_c.rs:104: fn display(&self, f: &mut dyn Write) -> std::fmt::Result
src/lir/ir/nodes_c.rs:107: fn serialize(&self, buf: &mut Vec<u8>)
src/lir/ir/nodes_c.rs:118: impl LirNode for SLirIndexAccess
src/lir/ir/nodes_c.rs:119: fn clone_node(&self) -> Box<dyn LirNode> { Box::new(self.clone()) }
src/lir/ir/nodes_c.rs:120: fn kind(&self) -> &'static str { "IndexAccess" }
src/lir/ir/nodes_c.rs:121: fn as_any(&self) -> &dyn std::any::Any { self }
src/lir/ir/nodes_c.rs:122: fn emit(&self, ctx: &mut LirEmitCtx) -> Vec<String>
src/lir/ir/nodes_c.rs:141: fn display(&self, f: &mut dyn Write) -> std::fmt::Result
src/lir/ir/nodes_c.rs:144: fn serialize(&self, buf: &mut Vec<u8>)
src/lir/ir/nodes_c.rs:152: impl LirNode for SLirStructLit
src/lir/ir/nodes_c.rs:153: fn clone_node(&self) -> Box<dyn LirNode> { Box::new(self.clone()) }
src/lir/ir/nodes_c.rs:154: fn kind(&self) -> &'static str { "StructLit" }
src/lir/ir/nodes_c.rs:155: fn as_any(&self) -> &dyn std::any::Any { self }
src/lir/ir/nodes_c.rs:156: fn alloca_lines(&self, _ctx: &LirEmitCtx) -> Vec<String>
src/lir/ir/nodes_c.rs:160: fn emit(&self, ctx: &mut LirEmitCtx) -> Vec<String>
src/lir/ir/nodes_c.rs:180: fn display(&self, f: &mut dyn Write) -> std::fmt::Result
src/lir/ir/nodes_c.rs:183: fn serialize(&self, buf: &mut Vec<u8>)
src/lir/ir/nodes_c.rs:194: impl LirNode for SLirVirtualCall
src/lir/ir/nodes_c.rs:195: fn clone_node(&self) -> Box<dyn LirNode> { Box::new(self.clone()) }
src/lir/ir/nodes_c.rs:196: fn kind(&self) -> &'static str { "VirtualCall" }
src/lir/ir/nodes_c.rs:197: fn as_any(&self) -> &dyn std::any::Any { self }
src/lir/ir/nodes_c.rs:198: fn emit(&self, ctx: &mut LirEmitCtx) -> Vec<String>
src/lir/ir/nodes_c.rs:219: fn display(&self, f: &mut dyn Write) -> std::fmt::Result
src/lir/ir/nodes_c.rs:223: fn serialize(&self, buf: &mut Vec<u8>)
src/lir/ir/nodes_c.rs:235: impl LirNode for SLirFieldStore
src/lir/ir/nodes_c.rs:236: fn clone_node(&self) -> Box<dyn LirNode> { Box::new(self.clone()) }
src/lir/ir/nodes_c.rs:237: fn kind(&self) -> &'static str { "FieldStore" }
src/lir/ir/nodes_c.rs:238: fn as_any(&self) -> &dyn std::any::Any { self }
src/lir/ir/nodes_c.rs:239: fn emit(&self, ctx: &mut LirEmitCtx) -> Vec<String>
src/lir/ir/nodes_c.rs:272: fn display(&self, f: &mut dyn Write) -> std::fmt::Result
src/lir/ir/nodes_c.rs:275: fn serialize(&self, buf: &mut Vec<u8>)
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
src/lir/ir/nodes_d.rs:75: pub struct VtableDesc
src/lir/ir/nodes_d.rs:82: pub struct LirGlobal
src/lir/ir/nodes_d.rs:91: pub struct LirProgram
src/lir/ir/nodes_d.rs:113: pub(crate) fn put_u32(buf: &mut Vec<u8>, v: u32) { buf.extend_from_slice(&v.to_le_bytes()); }
src/lir/ir/nodes_d.rs:114: pub(crate) fn put_u64(buf: &mut Vec<u8>, v: u64) { buf.extend_from_slice(&v.to_le_bytes()); }
src/lir/ir/nodes_d.rs:115: pub(crate) fn put_str(buf: &mut Vec<u8>, s: &str)
src/lir/ir/nodes_d.rs:121: pub(crate) fn put_type(buf: &mut Vec<u8>, ty: &HirType)
src/lir/ir/nodes_d.rs:157: pub(crate) fn put_value(buf: &mut Vec<u8>, v: &LirValue)
src/lir/ir/nodes_d.rs:170: pub(crate) fn put_literal(buf: &mut Vec<u8>, lit: &HirLiteral)
src/lir/ir/nodes_d.rs:195: pub trait LirLowerCtx
src/lir/ir/nodes_d.rs:196: fn next_tmp(&mut self) -> u64;
src/lir/ir/nodes_d.rs:197: fn emit(&mut self, inst: LirNodeBox);
src/lir/ir/nodes_d.rs:198: fn str_map(&self) -> &HashMap<String, u64>;
src/lir/ir/nodes_d.rs:199: fn loop_stack(&self) -> &Vec<(String, String)>;
src/lir/ir/nodes_d.rs:200: fn loop_stack_mut(&mut self) -> &mut Vec<(String, String)>;
src/lir/ir/nodes_d.rs:201: fn next_block_label(&mut self, prefix: &str) -> String;
src/lir/ir/nodes_d.rs:202: fn set_current_block(&mut self, label: String);
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
src/lir/ir/nodes_e.rs:48: fn alloca_lines(&self, ctx: &LirEmitCtx) -> Vec<String>
src/lir/ir/nodes_e.rs:52: fn emit(&self, ctx: &mut LirEmitCtx) -> Vec<String>
src/lir/ir/nodes_e.rs:61: fn display(&self, f: &mut dyn Write) -> std::fmt::Result
src/lir/ir/nodes_e.rs:65: fn serialize(&self, buf: &mut Vec<u8>)
src/lir/ir/nodes_e.rs:75: impl LirNode for SLirLoadPtr
src/lir/ir/nodes_e.rs:76: fn clone_node(&self) -> Box<dyn LirNode> { Box::new(self.clone()) }
src/lir/ir/nodes_e.rs:77: fn kind(&self) -> &'static str { "LoadPtr" }
src/lir/ir/nodes_e.rs:78: fn as_any(&self) -> &dyn std::any::Any { self }
src/lir/ir/nodes_e.rs:79: fn emit(&self, ctx: &mut LirEmitCtx) -> Vec<String>
src/lir/ir/nodes_e.rs:84: fn display(&self, f: &mut dyn Write) -> std::fmt::Result
src/lir/ir/nodes_e.rs:87: fn serialize(&self, buf: &mut Vec<u8>)
src/lir/ir/nodes_e.rs:95: impl LirNode for SLirIndexStore
src/lir/ir/nodes_e.rs:96: fn clone_node(&self) -> Box<dyn LirNode> { Box::new(self.clone()) }
src/lir/ir/nodes_e.rs:97: fn kind(&self) -> &'static str { "IndexStore" }
src/lir/ir/nodes_e.rs:98: fn as_any(&self) -> &dyn std::any::Any { self }
src/lir/ir/nodes_e.rs:99: fn emit(&self, ctx: &mut LirEmitCtx) -> Vec<String>
src/lir/ir/nodes_e.rs:122: fn display(&self, f: &mut dyn Write) -> std::fmt::Result
src/lir/ir/nodes_e.rs:125: fn serialize(&self, buf: &mut Vec<u8>)
src/lir/ir/nodes_e.rs:133: impl LirNode for SLirIndexAddr
src/lir/ir/nodes_e.rs:134: fn clone_node(&self) -> Box<dyn LirNode> { Box::new(self.clone()) }
src/lir/ir/nodes_e.rs:135: fn kind(&self) -> &'static str { "IndexAddr" }
src/lir/ir/nodes_e.rs:136: fn as_any(&self) -> &dyn std::any::Any { self }
src/lir/ir/nodes_e.rs:137: fn emit(&self, ctx: &mut LirEmitCtx) -> Vec<String>
src/lir/ir/nodes_e.rs:151: fn display(&self, f: &mut dyn Write) -> std::fmt::Result
src/lir/ir/nodes_e.rs:154: fn serialize(&self, buf: &mut Vec<u8>)
src/lir/ir/nodes_e.rs:161: impl LirNode for SLirStrGlobal
src/lir/ir/nodes_e.rs:162: fn clone_node(&self) -> Box<dyn LirNode> { Box::new(self.clone()) }
src/lir/ir/nodes_e.rs:163: fn kind(&self) -> &'static str { "StrGlobal" }
src/lir/ir/nodes_e.rs:164: fn as_any(&self) -> &dyn std::any::Any { self }
src/lir/ir/nodes_e.rs:165: fn emit(&self, ctx: &mut LirEmitCtx) -> Vec<String>
src/lir/ir/nodes_e.rs:178: fn display(&self, f: &mut dyn Write) -> std::fmt::Result
src/lir/ir/nodes_e.rs:181: fn serialize(&self, buf: &mut Vec<u8>)
src/lir/ir/nodes_e.rs:188: impl LirNode for SLirFieldStorePtr
src/lir/ir/nodes_e.rs:189: fn clone_node(&self) -> Box<dyn LirNode> { Box::new(self.clone()) }
src/lir/ir/nodes_e.rs:190: fn kind(&self) -> &'static str { "FieldStorePtr" }
src/lir/ir/nodes_e.rs:191: fn as_any(&self) -> &dyn std::any::Any { self }
src/lir/ir/nodes_e.rs:192: fn emit(&self, ctx: &mut LirEmitCtx) -> Vec<String>
src/lir/ir/nodes_e.rs:218: fn display(&self, f: &mut dyn Write) -> std::fmt::Result
src/lir/ir/nodes_e.rs:221: fn serialize(&self, buf: &mut Vec<u8>)
src/lir/ir/nodes_e.rs:232: impl LirNode for SLirGlobalAddr
src/lir/ir/nodes_e.rs:233: fn clone_node(&self) -> Box<dyn LirNode> { Box::new(self.clone()) }
src/lir/ir/nodes_e.rs:234: fn kind(&self) -> &'static str { "GlobalAddr" }
src/lir/ir/nodes_e.rs:235: fn as_any(&self) -> &dyn std::any::Any { self }
src/lir/ir/nodes_e.rs:236: fn emit(&self, _ctx: &mut LirEmitCtx) -> Vec<String>
src/lir/ir/nodes_e.rs:239: fn display(&self, f: &mut dyn Write) -> std::fmt::Result
src/lir/ir/nodes_e.rs:242: fn serialize(&self, buf: &mut Vec<u8>)
src/lir/ir/nodes_e.rs:249: impl LirNode for SLirFieldAddr
src/lir/ir/nodes_e.rs:250: fn clone_node(&self) -> Box<dyn LirNode> { Box::new(self.clone()) }
src/lir/ir/nodes_e.rs:251: fn kind(&self) -> &'static str { "FieldAddr" }
src/lir/ir/nodes_e.rs:252: fn as_any(&self) -> &dyn std::any::Any { self }
src/lir/ir/nodes_e.rs:253: fn emit(&self, ctx: &mut LirEmitCtx) -> Vec<String>
src/lir/ir/nodes_e.rs:267: fn display(&self, f: &mut dyn Write) -> std::fmt::Result
src/lir/ir/nodes_e.rs:270: fn serialize(&self, buf: &mut Vec<u8>)
src/lir/ir/nodes_f.rs:4: impl LirNode for SLirFieldTake
src/lir/ir/nodes_f.rs:5: fn clone_node(&self) -> Box<dyn LirNode> { Box::new(self.clone()) }
src/lir/ir/nodes_f.rs:6: fn kind(&self) -> &'static str { "FieldTake" }
src/lir/ir/nodes_f.rs:7: fn as_any(&self) -> &dyn std::any::Any { self }
src/lir/ir/nodes_f.rs:8: fn emit(&self, ctx: &mut LirEmitCtx) -> Vec<String>
src/lir/ir/nodes_f.rs:27: fn display(&self, f: &mut dyn Write) -> std::fmt::Result
src/lir/ir/nodes_f.rs:30: fn serialize(&self, buf: &mut Vec<u8>)
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
src/lir/lower/fn_lower.rs:87: pub(super) fn lower_stmts(ctx: &mut LowerCtx, stmts: &[MirStmtBox])
src/lir/lower/fn_lower.rs:113: pub(super) fn lower_expr(ctx: &mut dyn LirLowerCtx, expr: &MirNodeBox) -> LirValue
src/lir/lower/fn_lower.rs:117: pub(super) fn default_ret_value(ty: &HirType) -> Option<(LirValue, HirType)>
src/lir/lower/fn_lower.rs:132: pub(super) fn strip_ownership(ty: HirType) -> HirType
src/lir/lower/fn_lower.rs:139: pub(super) fn type_size(ty: &HirType) -> u64
src/lir/lower/mir_contract.rs:3: impl MirStmtNode for SMirAssumeStmt
src/lir/lower/mir_contract.rs:4: fn span(&self) -> crate::span::Span { self.span }
src/lir/lower/mir_contract.rs:5: fn for_each_child_expr_mut(&mut self, f: &mut dyn FnMut(&mut MirNodeBox)) { f(&mut self.cond); }
src/lir/lower/mir_contract.rs:6: fn clone_stmt(&self) -> Box<dyn MirStmtNode> { Box::new(self.clone()) }
src/lir/lower/mir_contract.rs:7: fn lower_to_lir_stmt(&self, ctx: &mut dyn LirLowerCtx)
src/lir/lower/mir_contract.rs:11: fn display_stmt(&self, level: usize, w: &mut dyn std::fmt::Write) -> std::fmt::Result
src/lir/lower/mir_contract.rs:15: fn for_each_child_expr(&self, f: &mut dyn FnMut(&dyn MirNode))
src/lir/lower/mir_contract.rs:20: impl MirStmtNode for SMirContractStmt
src/lir/lower/mir_contract.rs:21: fn span(&self) -> crate::span::Span { self.span }
src/lir/lower/mir_contract.rs:22: fn for_each_child_expr_mut(&mut self, f: &mut dyn FnMut(&mut MirNodeBox)) { f(&mut self.cond); }
src/lir/lower/mir_contract.rs:23: fn clone_stmt(&self) -> Box<dyn MirStmtNode> { Box::new(self.clone()) }
src/lir/lower/mir_contract.rs:24: fn lower_to_lir_stmt(&self, ctx: &mut dyn LirLowerCtx)
src/lir/lower/mir_contract.rs:30: fn display_stmt(&self, level: usize, w: &mut dyn std::fmt::Write) -> std::fmt::Result
src/lir/lower/mir_contract.rs:34: fn for_each_child_expr(&self, f: &mut dyn FnMut(&dyn MirNode))
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
src/lir/lower/mir_expr.rs:63: fn display(&self, level: usize, w: &mut dyn std::fmt::Write) -> std::fmt::Result
src/lir/lower/mir_expr.rs:75: fn expr_type(&self) -> HirType { self.ty.clone() }
src/lir/lower/mir_expr.rs:76: fn as_string_literal(&self) -> Option<&str>
src/lir/lower/mir_expr.rs:79: fn is_alloc(&self) -> bool { matches!(self.val, HirLiteral::String(_)) }
src/lir/lower/mir_expr.rs:80: fn literal_value(&self) -> Option<(&HirLiteral, &HirType)> { Some((&self.val, &self.ty)) }
src/lir/lower/mir_expr.rs:83: impl MirNode for SMirBinary
src/lir/lower/mir_expr.rs:84: fn for_each_child_mut(&mut self, f: &mut dyn FnMut(&mut MirNodeBox)) { f(&mut self.lhs); f(&mut self.rhs); }
src/lir/lower/mir_expr.rs:85: fn clone_node(&self) -> Box<dyn MirNode> { Box::new(self.clone()) }
src/lir/lower/mir_expr.rs:86: fn lower_to_lir(&self, ctx: &mut dyn LirLowerCtx) -> LirValue
src/lir/lower/mir_expr.rs:97: fn display(&self, level: usize, w: &mut dyn std::fmt::Write) -> std::fmt::Result
src/lir/lower/mir_expr.rs:105: fn expr_type(&self) -> HirType { self.ty.clone() }
src/lir/lower/mir_expr.rs:106: fn for_each_child(&self, f: &mut dyn FnMut(&dyn MirNode)) { f(&*self.lhs); f(&*self.rhs); }
src/lir/lower/mir_expr.rs:107: fn binary_op(&self) -> Option<BinaryOp> { Some(self.op) }
src/lir/lower/mir_expr.rs:110: impl MirNode for SMirUnary
src/lir/lower/mir_expr.rs:111: fn for_each_child_mut(&mut self, f: &mut dyn FnMut(&mut MirNodeBox)) { f(&mut self.arg); }
src/lir/lower/mir_expr.rs:112: fn clone_node(&self) -> Box<dyn MirNode> { Box::new(self.clone()) }
src/lir/lower/mir_expr.rs:113: fn lower_to_lir(&self, ctx: &mut dyn LirLowerCtx) -> LirValue
src/lir/lower/mir_expr.rs:119: fn display(&self, level: usize, w: &mut dyn std::fmt::Write) -> std::fmt::Result
src/lir/lower/mir_expr.rs:124: fn expr_type(&self) -> HirType { self.ty.clone() }
src/lir/lower/mir_expr.rs:125: fn for_each_child(&self, f: &mut dyn FnMut(&dyn MirNode)) { f(&*self.arg); }
src/lir/lower/mir_expr.rs:126: fn unary_op(&self) -> Option<UnaryOp> { Some(self.op) }
src/lir/lower/mir_expr.rs:129: impl MirNode for SMirCall
src/lir/lower/mir_expr.rs:130: fn for_each_child_mut(&mut self, f: &mut dyn FnMut(&mut MirNodeBox)) { for a in &mut self.args { f(a); } }
src/lir/lower/mir_expr.rs:131: fn clone_node(&self) -> Box<dyn MirNode> { Box::new(self.clone()) }
src/lir/lower/mir_expr.rs:132: fn call_fn_id(&self) -> Option<crate::hir::ty::FnId> { Some(self.fn_id) }
src/lir/lower/mir_expr.rs:133: fn lower_to_lir(&self, ctx: &mut dyn LirLowerCtx) -> LirValue
src/lir/lower/mir_expr.rs:145: fn display(&self, level: usize, w: &mut dyn std::fmt::Write) -> std::fmt::Result
src/lir/lower/mir_expr.rs:150: fn expr_type(&self) -> HirType { self.ty.clone() }
src/lir/lower/mir_expr.rs:151: fn for_each_child(&self, f: &mut dyn FnMut(&dyn MirNode)) { for a in &self.args { f(&**a); } }    fn is_call(&self) -> bool { true }
src/lir/lower/mir_expr.rs:155: impl MirNode for SMirClone
src/lir/lower/mir_expr.rs:156: fn for_each_child_mut(&mut self, f: &mut dyn FnMut(&mut MirNodeBox)) { f(&mut self.expr); }
src/lir/lower/mir_expr.rs:157: fn clone_node(&self) -> Box<dyn MirNode> { Box::new(self.clone()) }
src/lir/lower/mir_expr.rs:158: fn lower_to_lir(&self, ctx: &mut dyn LirLowerCtx) -> LirValue { self.expr.lower_to_lir(ctx) }
src/lir/lower/mir_expr.rs:159: fn display(&self, level: usize, w: &mut dyn std::fmt::Write) -> std::fmt::Result
src/lir/lower/mir_expr.rs:164: fn expr_type(&self) -> HirType { self.ty.clone() }
src/lir/lower/mir_expr.rs:165: fn for_each_child(&self, f: &mut dyn FnMut(&dyn MirNode)) { f(&*self.expr); }
src/lir/lower/mir_expr.rs:168: impl MirNode for SMirToUnique
src/lir/lower/mir_expr.rs:169: fn for_each_child_mut(&mut self, f: &mut dyn FnMut(&mut MirNodeBox)) { f(&mut self.expr); }
src/lir/lower/mir_expr.rs:170: fn is_alloc(&self) -> bool { true }
src/lir/lower/mir_expr.rs:171: fn clone_node(&self) -> Box<dyn MirNode> { Box::new(self.clone()) }
src/lir/lower/mir_expr.rs:172: fn lower_to_lir(&self, ctx: &mut dyn LirLowerCtx) -> LirValue
src/lir/lower/mir_expr.rs:185: fn display(&self, level: usize, w: &mut dyn std::fmt::Write) -> std::fmt::Result
src/lir/lower/mir_expr.rs:190: fn expr_type(&self) -> HirType { self.ty.clone() }
src/lir/lower/mir_expr.rs:191: fn for_each_child(&self, f: &mut dyn FnMut(&dyn MirNode)) { f(&*self.expr); }
src/lir/lower/mir_expr.rs:194: impl MirNode for SMirCast
src/lir/lower/mir_expr.rs:195: fn for_each_child_mut(&mut self, f: &mut dyn FnMut(&mut MirNodeBox)) { f(&mut self.expr); }
src/lir/lower/mir_expr.rs:196: fn clone_node(&self) -> Box<dyn MirNode> { Box::new(self.clone()) }
src/lir/lower/mir_expr.rs:197: fn lower_to_lir(&self, ctx: &mut dyn LirLowerCtx) -> LirValue
src/lir/lower/mir_expr.rs:211: fn display(&self, level: usize, w: &mut dyn std::fmt::Write) -> std::fmt::Result
src/lir/lower/mir_expr.rs:216: fn expr_type(&self) -> HirType { self.ty.clone() }
src/lir/lower/mir_expr.rs:217: fn for_each_child(&self, f: &mut dyn FnMut(&dyn MirNode)) { f(&*self.expr); }
src/lir/lower/mir_expr.rs:222: impl MirNode for SMirVirtualCall
src/lir/lower/mir_expr.rs:223: fn for_each_child_mut(&mut self, f: &mut dyn FnMut(&mut MirNodeBox)) { f(&mut self.receiver); for a in &mut self.args { f(a); } }
src/lir/lower/mir_expr.rs:224: fn clone_node(&self) -> Box<dyn MirNode> { Box::new(self.clone()) }
src/lir/lower/mir_expr.rs:225: fn lower_to_lir(&self, ctx: &mut dyn LirLowerCtx) -> LirValue
src/lir/lower/mir_expr.rs:242: fn display(&self, level: usize, w: &mut dyn std::fmt::Write) -> std::fmt::Result
src/lir/lower/mir_expr.rs:249: fn expr_type(&self) -> HirType { self.ty.clone() }
src/lir/lower/mir_expr.rs:250: fn for_each_child(&self, f: &mut dyn FnMut(&dyn MirNode)) { f(&*self.receiver); for a in &self.args { f(&**a); } }    fn is_call(&self) -> bool { true }
src/lir/lower/mir_expr.rs:254: impl MirNode for SMirMakeFatPtr
src/lir/lower/mir_expr.rs:255: fn for_each_child_mut(&mut self, f: &mut dyn FnMut(&mut MirNodeBox)) { f(&mut self.value); }
src/lir/lower/mir_expr.rs:256: fn clone_node(&self) -> Box<dyn MirNode> { Box::new(self.clone()) }
src/lir/lower/mir_expr.rs:257: fn lower_to_lir(&self, ctx: &mut dyn LirLowerCtx) -> LirValue
src/lir/lower/mir_expr.rs:271: fn display(&self, level: usize, w: &mut dyn std::fmt::Write) -> std::fmt::Result
src/lir/lower/mir_expr.rs:276: fn expr_type(&self) -> HirType { self.ty.clone() }
src/lir/lower/mir_expr.rs:277: fn for_each_child(&self, f: &mut dyn FnMut(&dyn MirNode)) { f(&*self.value); }
src/lir/lower/mir_expr2.rs:5: impl MirNode for SMirEnumConstruct
src/lir/lower/mir_expr2.rs:6: fn for_each_child_mut(&mut self, f: &mut dyn FnMut(&mut MirNodeBox)) { for a in &mut self.args { f(a); } }
src/lir/lower/mir_expr2.rs:7: fn clone_node(&self) -> Box<dyn MirNode> { Box::new(self.clone()) }
src/lir/lower/mir_expr2.rs:8: fn lower_to_lir(&self, _ctx: &mut dyn LirLowerCtx) -> LirValue
src/lir/lower/mir_expr2.rs:11: fn display(&self, level: usize, w: &mut dyn std::fmt::Write) -> std::fmt::Result
src/lir/lower/mir_expr2.rs:14: fn expr_type(&self) -> HirType { self.ty.clone() }
src/lir/lower/mir_expr2.rs:15: fn for_each_child(&self, f: &mut dyn FnMut(&dyn MirNode)) { for a in &self.args { f(&**a); } }
src/lir/lower/mir_expr2.rs:18: impl MirNode for SMirFnPtr
src/lir/lower/mir_expr2.rs:19: fn clone_node(&self) -> Box<dyn MirNode> { Box::new(self.clone()) }
src/lir/lower/mir_expr2.rs:20: fn lower_to_lir(&self, ctx: &mut dyn LirLowerCtx) -> LirValue
src/lir/lower/mir_expr2.rs:25: fn display(&self, level: usize, w: &mut dyn std::fmt::Write) -> std::fmt::Result
src/lir/lower/mir_expr2.rs:28: fn expr_type(&self) -> HirType { self.ty.clone() }
src/lir/lower/mir_expr2.rs:31: impl MirNode for SMirCallPtr
src/lir/lower/mir_expr2.rs:32: fn for_each_child_mut(&mut self, f: &mut dyn FnMut(&mut MirNodeBox)) { f(&mut self.fn_ptr); for a in &mut self.args { f(a); } }
src/lir/lower/mir_expr2.rs:33: fn clone_node(&self) -> Box<dyn MirNode> { Box::new(self.clone()) }
src/lir/lower/mir_expr2.rs:34: fn lower_to_lir(&self, ctx: &mut dyn LirLowerCtx) -> LirValue
src/lir/lower/mir_expr2.rs:47: fn display(&self, level: usize, w: &mut dyn std::fmt::Write) -> std::fmt::Result
src/lir/lower/mir_expr2.rs:50: fn expr_type(&self) -> HirType { self.ty.clone() }
src/lir/lower/mir_expr2.rs:51: fn for_each_child(&self, f: &mut dyn FnMut(&dyn MirNode)) { f(&*self.fn_ptr); for a in &self.args { f(&**a); } }    fn is_call(&self) -> bool { true }
src/lir/lower/mir_expr2.rs:55: impl MirNode for SMirEnumMatch
src/lir/lower/mir_expr2.rs:56: fn for_each_child_mut(&mut self, f: &mut dyn FnMut(&mut MirNodeBox)) { f(&mut self.value); for (_, e) in &mut self.arms { f(e); } }
src/lir/lower/mir_expr2.rs:57: fn clone_node(&self) -> Box<dyn MirNode> { Box::new(self.clone()) }
src/lir/lower/mir_expr2.rs:58: fn lower_to_lir(&self, ctx: &mut dyn LirLowerCtx) -> LirValue
src/lir/lower/mir_expr2.rs:91: fn display(&self, level: usize, w: &mut dyn std::fmt::Write) -> std::fmt::Result
src/lir/lower/mir_expr2.rs:97: fn expr_type(&self) -> HirType { self.ty.clone() }
src/lir/lower/mir_expr2.rs:98: fn for_each_child(&self, f: &mut dyn FnMut(&dyn MirNode))
src/lir/lower/mir_expr2.rs:104: impl MirNode for SMirFieldAccess
src/lir/lower/mir_expr2.rs:105: fn for_each_child_mut(&mut self, f: &mut dyn FnMut(&mut MirNodeBox)) { f(&mut self.object); }
src/lir/lower/mir_expr2.rs:106: fn clone_node(&self) -> Box<dyn MirNode> { Box::new(self.clone()) }
src/lir/lower/mir_expr2.rs:107: fn lower_to_lir(&self, ctx: &mut dyn LirLowerCtx) -> LirValue
src/lir/lower/mir_expr2.rs:117: fn display(&self, level: usize, w: &mut dyn std::fmt::Write) -> std::fmt::Result
src/lir/lower/mir_expr2.rs:122: fn expr_type(&self) -> HirType { self.ty.clone() }
src/lir/lower/mir_expr2.rs:123: fn for_each_child(&self, f: &mut dyn FnMut(&dyn MirNode)) { f(&*self.object); }
src/lir/lower/mir_expr2.rs:124: fn as_field_access(&self) -> Option<(&MirNodeBox, usize)> { Some((&self.object, self.field_index)) }
src/lir/lower/mir_expr2.rs:127: impl MirNode for SMirStructLiteral
src/lir/lower/mir_expr2.rs:128: fn for_each_child_mut(&mut self, f: &mut dyn FnMut(&mut MirNodeBox)) { for (_, e) in &mut self.fields { f(e); } }
src/lir/lower/mir_expr2.rs:129: fn clone_node(&self) -> Box<dyn MirNode> { Box::new(self.clone()) }
src/lir/lower/mir_expr2.rs:130: fn struct_literal_fields(&self) -> Option<&[(Symbol, MirNodeBox)]> { Some(&self.fields) }
src/lir/lower/mir_expr2.rs:131: fn lower_to_lir(&self, ctx: &mut dyn LirLowerCtx) -> LirValue
src/lir/lower/mir_expr2.rs:140: fn display(&self, level: usize, w: &mut dyn std::fmt::Write) -> std::fmt::Result
src/lir/lower/mir_expr2.rs:148: fn expr_type(&self) -> HirType { self.ty.clone() }
src/lir/lower/mir_expr2.rs:149: fn for_each_child(&self, f: &mut dyn FnMut(&dyn MirNode)) { for (_, e) in &self.fields { f(&**e); } }
src/lir/lower/mir_expr2.rs:152: impl MirNode for SMirArrayLiteral
src/lir/lower/mir_expr2.rs:153: fn for_each_child_mut(&mut self, f: &mut dyn FnMut(&mut MirNodeBox)) { for a in &mut self.elems { f(a); } }
src/lir/lower/mir_expr2.rs:154: fn is_alloc(&self) -> bool { true }
src/lir/lower/mir_expr2.rs:155: fn clone_node(&self) -> Box<dyn MirNode> { Box::new(self.clone()) }
src/lir/lower/mir_expr2.rs:156: fn lower_to_lir(&self, ctx: &mut dyn LirLowerCtx) -> LirValue
src/lir/lower/mir_expr2.rs:167: fn display(&self, level: usize, w: &mut dyn std::fmt::Write) -> std::fmt::Result
src/lir/lower/mir_expr2.rs:172: fn expr_type(&self) -> HirType { self.ty.clone() }
src/lir/lower/mir_expr2.rs:173: fn for_each_child(&self, f: &mut dyn FnMut(&dyn MirNode)) { for e in &self.elems { f(&**e); } }
src/lir/lower/mir_expr2.rs:176: impl MirNode for SMirArraySized
src/lir/lower/mir_expr2.rs:177: fn for_each_child_mut(&mut self, f: &mut dyn FnMut(&mut MirNodeBox)) { f(&mut self.count); }
src/lir/lower/mir_expr2.rs:178: fn is_alloc(&self) -> bool { true }
src/lir/lower/mir_expr2.rs:179: fn clone_node(&self) -> Box<dyn MirNode> { Box::new(self.clone()) }
src/lir/lower/mir_expr2.rs:180: fn lower_to_lir(&self, ctx: &mut dyn LirLowerCtx) -> LirValue
src/lir/lower/mir_expr2.rs:188: fn display(&self, level: usize, w: &mut dyn std::fmt::Write) -> std::fmt::Result
src/lir/lower/mir_expr2.rs:194: fn expr_type(&self) -> HirType { self.ty.clone() }
src/lir/lower/mir_expr2.rs:195: fn for_each_child(&self, f: &mut dyn FnMut(&dyn MirNode)) { f(&*self.count); }
src/lir/lower/mir_expr2.rs:198: impl MirNode for SMirIndex
src/lir/lower/mir_expr2.rs:199: fn as_index(&self) -> Option<(&MirNodeBox, &MirNodeBox)> { Some((&self.object, &self.index)) }
src/lir/lower/mir_expr2.rs:200: fn for_each_child_mut(&mut self, f: &mut dyn FnMut(&mut MirNodeBox)) { f(&mut self.object); f(&mut self.index); }
src/lir/lower/mir_expr2.rs:201: fn clone_node(&self) -> Box<dyn MirNode> { Box::new(self.clone()) }
src/lir/lower/mir_expr2.rs:202: fn lower_to_lir(&self, ctx: &mut dyn LirLowerCtx) -> LirValue
src/lir/lower/mir_expr2.rs:217: fn display(&self, level: usize, w: &mut dyn std::fmt::Write) -> std::fmt::Result
src/lir/lower/mir_expr2.rs:225: fn expr_type(&self) -> HirType { self.ty.clone() }
src/lir/lower/mir_expr2.rs:226: fn for_each_child(&self, f: &mut dyn FnMut(&dyn MirNode)) { f(&*self.object); f(&*self.index); }
src/lir/lower/mir_expr2.rs:229: impl MirNode for SMirAsm
src/lir/lower/mir_expr2.rs:230: fn for_each_child_mut(&mut self, f: &mut dyn FnMut(&mut MirNodeBox)) { for (_, e) in &mut self.outputs { f(e); } for (_, e) in &mut self.inputs { f(e); } }
src/lir/lower/mir_expr2.rs:231: fn is_asm(&self) -> bool { true }
src/lir/lower/mir_expr2.rs:232: fn clone_node(&self) -> Box<dyn MirNode> { Box::new(self.clone()) }
src/lir/lower/mir_expr2.rs:233: fn lower_to_lir(&self, ctx: &mut dyn LirLowerCtx) -> LirValue
src/lir/lower/mir_expr2.rs:247: fn display(&self, level: usize, w: &mut dyn std::fmt::Write) -> std::fmt::Result
src/lir/lower/mir_expr2.rs:259: fn expr_type(&self) -> HirType { self.ty.clone() }
src/lir/lower/mir_expr2.rs:260: fn for_each_child(&self, f: &mut dyn FnMut(&dyn MirNode))
src/lir/lower/mir_ref.rs:7: pub(super) fn place_ptr(expr: &MirNodeBox, ctx: &mut dyn LirLowerCtx) -> Option<LirValue>
src/lir/lower/mir_ref.rs:65: pub(super) fn array_elem_ty(ty: &HirType) -> Option<HirType>
src/lir/lower/mir_ref.rs:78: pub(super) fn array_base_through_ref(ctx: &mut dyn LirLowerCtx, base_tmp: u64, ty: &HirType) -> u64
src/lir/lower/mir_ref.rs:89: impl MirNode for SMirRef
src/lir/lower/mir_ref.rs:90: fn refs_global(&self) -> Option<Symbol>
src/lir/lower/mir_ref.rs:94: fn for_each_child_mut(&mut self, f: &mut dyn FnMut(&mut MirNodeBox)) { f(&mut self.expr); }
src/lir/lower/mir_ref.rs:95: fn clone_node(&self) -> Box<dyn MirNode> { Box::new(self.clone()) }
src/lir/lower/mir_ref.rs:96: fn lower_to_lir(&self, ctx: &mut dyn LirLowerCtx) -> LirValue
src/lir/lower/mir_ref.rs:112: fn display(&self, level: usize, w: &mut dyn std::fmt::Write) -> std::fmt::Result
src/lir/lower/mir_ref.rs:118: fn expr_type(&self) -> HirType { self.ty.clone() }
src/lir/lower/mir_ref.rs:119: fn for_each_child(&self, f: &mut dyn FnMut(&dyn MirNode)) { f(&*self.expr); }
src/lir/lower/mir_ref.rs:120: fn as_ref(&self) -> Option<(VarId, bool)>
src/lir/lower/mir_ref.rs:125: impl MirNode for SMirGlobal
src/lir/lower/mir_ref.rs:126: fn as_global(&self) -> Option<Symbol> { Some(self.name) }
src/lir/lower/mir_ref.rs:127: fn clone_node(&self) -> Box<dyn MirNode> { Box::new(self.clone()) }
src/lir/lower/mir_ref.rs:128: fn lower_to_lir(&self, ctx: &mut dyn LirLowerCtx) -> LirValue
src/lir/lower/mir_ref.rs:133: fn display(&self, level: usize, w: &mut dyn std::fmt::Write) -> std::fmt::Result
src/lir/lower/mir_ref.rs:136: fn expr_type(&self) -> HirType { HirType::Ref(Box::new(self.ty.clone()), self.mutable) }
src/lir/lower/mir_ref.rs:137: fn for_each_child(&self, _f: &mut dyn FnMut(&dyn MirNode)) {}
src/lir/lower/mir_ref.rs:140: impl MirNode for SMirDeref
src/lir/lower/mir_ref.rs:141: fn as_deref(&self) -> Option<&MirNodeBox> { Some(&self.expr) }
src/lir/lower/mir_ref.rs:142: fn for_each_child_mut(&mut self, f: &mut dyn FnMut(&mut MirNodeBox)) { f(&mut self.expr); }
src/lir/lower/mir_ref.rs:143: fn clone_node(&self) -> Box<dyn MirNode> { Box::new(self.clone()) }
src/lir/lower/mir_ref.rs:144: fn lower_to_lir(&self, ctx: &mut dyn LirLowerCtx) -> LirValue
src/lir/lower/mir_ref.rs:161: fn display(&self, level: usize, w: &mut dyn std::fmt::Write) -> std::fmt::Result
src/lir/lower/mir_ref.rs:166: fn expr_type(&self) -> HirType { self.ty.clone() }
src/lir/lower/mir_ref.rs:167: fn for_each_child(&self, f: &mut dyn FnMut(&dyn MirNode)) { f(&*self.expr); }
src/lir/lower/mir_ref.rs:170: impl MirStmtNode for SMirDerefAssignStmt { fn span(&self) -> crate::span::Span { self.span }
src/lir/lower/mir_ref.rs:171: fn for_each_child_expr_mut(&mut self, f: &mut dyn FnMut(&mut MirNodeBox)) { f(&mut self.target); f(&mut self.value); }
src/lir/lower/mir_ref.rs:172: fn clone_stmt(&self) -> Box<dyn MirStmtNode> { Box::new(self.clone()) }
src/lir/lower/mir_ref.rs:173: fn lower_to_lir_stmt(&self, ctx: &mut dyn LirLowerCtx)
src/lir/lower/mir_ref.rs:192: fn display_stmt(&self, level: usize, w: &mut dyn std::fmt::Write) -> std::fmt::Result
src/lir/lower/mir_ref.rs:200: fn for_each_child_expr(&self, f: &mut dyn FnMut(&dyn MirNode)) { f(&*self.target); f(&*self.value); }
src/lir/lower/mir_ref.rs:201: fn deref_assign_parts(&self) -> Option<(&MirNodeBox, &MirNodeBox)> { Some((&self.target, &self.value)) }
src/lir/lower/mir_ref.rs:205: pub(super) fn lower_field_addr_of(ctx: &mut dyn LirLowerCtx, node: &dyn MirNode) -> Option<LirValue>
src/lir/lower/mir_ref.rs:228: impl MirNode for SMirMove
src/lir/lower/mir_ref.rs:229: fn for_each_child_mut(&mut self, f: &mut dyn FnMut(&mut MirNodeBox)) { f(&mut self.expr); }
src/lir/lower/mir_ref.rs:230: fn move_expr(&self) -> Option<&MirNodeBox> { Some(&self.expr) }
src/lir/lower/mir_ref.rs:231: fn clone_node(&self) -> Box<dyn MirNode> { Box::new(self.clone()) }
src/lir/lower/mir_ref.rs:232: fn lower_to_lir(&self, ctx: &mut dyn LirLowerCtx) -> LirValue
src/lir/lower/mir_ref.rs:246: fn display(&self, level: usize, w: &mut dyn std::fmt::Write) -> std::fmt::Result
src/lir/lower/mir_ref.rs:251: fn expr_type(&self) -> HirType { self.ty.clone() }
src/lir/lower/mir_ref.rs:252: fn for_each_child(&self, f: &mut dyn FnMut(&dyn MirNode)) { f(&*self.expr); }
src/lir/lower/mir_stmts.rs:6: impl MirStmtNode for SMirAssignStmt { fn span(&self) -> crate::span::Span { self.span }
src/lir/lower/mir_stmts.rs:7: fn for_each_child_expr_mut(&mut self, f: &mut dyn FnMut(&mut MirNodeBox)) { f(&mut self.target); f(&mut self.value); }
src/lir/lower/mir_stmts.rs:8: fn clone_stmt(&self) -> Box<dyn MirStmtNode> { Box::new(self.clone()) }
src/lir/lower/mir_stmts.rs:9: fn lower_to_lir_stmt(&self, ctx: &mut dyn LirLowerCtx)
src/lir/lower/mir_stmts.rs:17: fn display_stmt(&self, level: usize, w: &mut dyn std::fmt::Write) -> std::fmt::Result
src/lir/lower/mir_stmts.rs:25: fn for_each_child_expr(&self, f: &mut dyn FnMut(&dyn MirNode)) { f(&*self.target); f(&*self.value); }    fn assign_parts(&self) -> Option<(&MirNodeBox, &MirNodeBox)> { Some((&self.target, &self.value)) }
src/lir/lower/mir_stmts.rs:29: impl MirStmtNode for SMirFieldAssignStmt { fn span(&self) -> crate::span::Span { self.span }
src/lir/lower/mir_stmts.rs:30: fn for_each_child_expr_mut(&mut self, f: &mut dyn FnMut(&mut MirNodeBox)) { f(&mut self.object); f(&mut self.value); }
src/lir/lower/mir_stmts.rs:31: fn clone_stmt(&self) -> Box<dyn MirStmtNode> { Box::new(self.clone()) }
src/lir/lower/mir_stmts.rs:32: fn lower_to_lir_stmt(&self, ctx: &mut dyn LirLowerCtx)
src/lir/lower/mir_stmts.rs:59: fn display_stmt(&self, level: usize, w: &mut dyn std::fmt::Write) -> std::fmt::Result
src/lir/lower/mir_stmts.rs:67: fn for_each_child_expr(&self, f: &mut dyn FnMut(&dyn MirNode)) { f(&*self.object); f(&*self.value); }    fn field_assign_parts(&self) -> Option<(&MirNodeBox, &MirNodeBox)> { Some((&self.object, &self.value)) }
src/lir/lower/mir_stmts.rs:71: impl MirStmtNode for SMirIndexAssignStmt { fn span(&self) -> crate::span::Span { self.span }
src/lir/lower/mir_stmts.rs:72: fn for_each_child_expr_mut(&mut self, f: &mut dyn FnMut(&mut MirNodeBox)) { f(&mut self.object); f(&mut self.index); f(&mut self.value); }
src/lir/lower/mir_stmts.rs:73: fn clone_stmt(&self) -> Box<dyn MirStmtNode> { Box::new(self.clone()) }
src/lir/lower/mir_stmts.rs:74: fn lower_to_lir_stmt(&self, ctx: &mut dyn LirLowerCtx)
src/lir/lower/mir_stmts.rs:87: fn display_stmt(&self, level: usize, w: &mut dyn std::fmt::Write) -> std::fmt::Result
src/lir/lower/mir_stmts.rs:97: fn for_each_child_expr(&self, f: &mut dyn FnMut(&dyn MirNode)) { f(&*self.object); f(&*self.index); f(&*self.value); }    fn index_assign_parts(&self) -> Option<(&MirNodeBox, &MirNodeBox, &MirNodeBox)> { Some((&self.object, &self.index, &self.value)) }
src/lir/lower/mir_stmts.rs:102: impl MirStmtNode for SMirReturnStmt { fn span(&self) -> crate::span::Span { self.span }
src/lir/lower/mir_stmts.rs:103: fn for_each_child_expr_mut(&mut self, f: &mut dyn FnMut(&mut MirNodeBox)) { if let Some(v) = &mut self.value { f(v); } }
src/lir/lower/mir_stmts.rs:104: fn clone_stmt(&self) -> Box<dyn MirStmtNode> { Box::new(self.clone()) }
src/lir/lower/mir_stmts.rs:105: fn lower_to_lir_stmt(&self, ctx: &mut dyn LirLowerCtx)
src/lir/lower/mir_stmts.rs:113: fn display_stmt(&self, level: usize, w: &mut dyn std::fmt::Write) -> std::fmt::Result
src/lir/lower/mir_stmts.rs:119: fn for_each_child_expr(&self, f: &mut dyn FnMut(&dyn MirNode))
src/lir/lower/mir_stmts.rs:122: fn is_return(&self) -> bool { true }
src/lir/lower/mir_stmts.rs:123: fn return_value(&self) -> Option<&MirNodeBox> { self.value.as_ref() }
src/lir/lower/mir_stmts.rs:126: impl MirStmtNode for SMirIfStmt { fn span(&self) -> crate::span::Span { self.span }
src/lir/lower/mir_stmts.rs:127: fn for_each_child_expr_mut(&mut self, f: &mut dyn FnMut(&mut MirNodeBox)) { f(&mut self.cond); for (c, _) in &mut self.elifs { f(c); } }
src/lir/lower/mir_stmts.rs:128: fn for_each_child_stmt_mut(&mut self, f: &mut dyn FnMut(&mut MirStmtBox)) { for s in &mut self.then_block { f(s); } for (_, b) in &mut self.elifs { for s in b { f(s); } } if let Some(b) = &mut self.else_block { for s in b { f(s); } } }
src/lir/lower/mir_stmts.rs:129: fn clone_stmt(&self) -> Box<dyn MirStmtNode> { Box::new(self.clone()) }
src/lir/lower/mir_stmts.rs:130: fn lower_to_lir_stmt(&self, ctx: &mut dyn LirLowerCtx)
src/lir/lower/mir_stmts.rs:147: fn display_stmt(&self, level: usize, w: &mut dyn std::fmt::Write) -> std::fmt::Result
src/lir/lower/mir_stmts.rs:165: fn for_each_child_expr(&self, f: &mut dyn FnMut(&dyn MirNode)) { f(&*self.cond); for (c, _) in &self.elifs { f(&**c); } }
src/lir/lower/mir_stmts.rs:166: fn for_each_child_stmt(&self, f: &mut dyn FnMut(&dyn MirStmtNode))
src/lir/lower/mir_stmts.rs:174: pub(super) fn lower_elifs(ctx: &mut dyn LirLowerCtx, elifs: &[(MirNodeBox, Vec<MirStmtBox>)], else_block: &Option<Vec<MirStmtBox>>, merge_lbl: &str)
src/lir/lower/mir_stmts.rs:195: impl MirStmtNode for SMirWhileStmt { fn span(&self) -> crate::span::Span { self.span }
src/lir/lower/mir_stmts.rs:196: fn for_each_child_expr_mut(&mut self, f: &mut dyn FnMut(&mut MirNodeBox)) { f(&mut self.cond); }
src/lir/lower/mir_stmts.rs:197: fn for_each_child_stmt_mut(&mut self, f: &mut dyn FnMut(&mut MirStmtBox)) { for s in &mut self.body { f(s); } }
src/lir/lower/mir_stmts.rs:198: fn clone_stmt(&self) -> Box<dyn MirStmtNode> { Box::new(self.clone()) }
src/lir/lower/mir_stmts.rs:199: fn lower_to_lir_stmt(&self, ctx: &mut dyn LirLowerCtx)
src/lir/lower/mir_stmts.rs:219: fn display_stmt(&self, level: usize, w: &mut dyn std::fmt::Write) -> std::fmt::Result
src/lir/lower/mir_stmts.rs:227: fn for_each_child_expr(&self, f: &mut dyn FnMut(&dyn MirNode)) { f(&*self.cond); }
src/lir/lower/mir_stmts.rs:228: fn for_each_child_stmt(&self, f: &mut dyn FnMut(&dyn MirStmtNode)) { for s in &self.body { f(&**s); } }    fn as_while(&self) -> Option<WhileParts<'_>> { Some((&self.cond, &self.body)) }
src/lir/lower/mir_stmts.rs:232: impl MirStmtNode for SMirBreakStmt { fn span(&self) -> crate::span::Span { self.span }
src/lir/lower/mir_stmts.rs:233: fn clone_stmt(&self) -> Box<dyn MirStmtNode> { Box::new(self.clone()) }
src/lir/lower/mir_stmts.rs:234: fn lower_to_lir_stmt(&self, ctx: &mut dyn LirLowerCtx)
src/lir/lower/mir_stmts.rs:239: fn display_stmt(&self, level: usize, w: &mut dyn std::fmt::Write) -> std::fmt::Result
src/lir/lower/mir_stmts.rs:245: impl MirStmtNode for SMirContinueStmt { fn span(&self) -> crate::span::Span { self.span }
src/lir/lower/mir_stmts.rs:246: fn clone_stmt(&self) -> Box<dyn MirStmtNode> { Box::new(self.clone()) }
src/lir/lower/mir_stmts.rs:247: fn lower_to_lir_stmt(&self, ctx: &mut dyn LirLowerCtx)
src/lir/lower/mir_stmts.rs:252: fn display_stmt(&self, level: usize, w: &mut dyn std::fmt::Write) -> std::fmt::Result
src/lir/lower/mir_stmts.rs:258: impl MirStmtNode for SMirExprStmt { fn span(&self) -> crate::span::Span { self.span }
src/lir/lower/mir_stmts.rs:259: fn for_each_child_expr_mut(&mut self, f: &mut dyn FnMut(&mut MirNodeBox)) { f(&mut self.expr); }
src/lir/lower/mir_stmts.rs:260: fn clone_stmt(&self) -> Box<dyn MirStmtNode> { Box::new(self.clone()) }
src/lir/lower/mir_stmts.rs:261: fn lower_to_lir_stmt(&self, ctx: &mut dyn LirLowerCtx)
src/lir/lower/mir_stmts.rs:264: fn display_stmt(&self, level: usize, w: &mut dyn std::fmt::Write) -> std::fmt::Result
src/lir/lower/mir_stmts.rs:269: fn for_each_child_expr(&self, f: &mut dyn FnMut(&dyn MirNode)) { f(&*self.expr); }    fn expr_part(&self) -> Option<&MirNodeBox> { Some(&self.expr) }
src/lir/lower/mir_stmts.rs:273: impl MirStmtNode for SMirBlockStmt { fn span(&self) -> crate::span::Span { self.span }
src/lir/lower/mir_stmts.rs:274: fn for_each_child_stmt_mut(&mut self, f: &mut dyn FnMut(&mut MirStmtBox)) { for s in &mut self.stmts { f(s); } }
src/lir/lower/mir_stmts.rs:275: fn clone_stmt(&self) -> Box<dyn MirStmtNode> { Box::new(self.clone()) }
src/lir/lower/mir_stmts.rs:276: fn lower_to_lir_stmt(&self, ctx: &mut dyn LirLowerCtx)
src/lir/lower/mir_stmts.rs:279: fn display_stmt(&self, level: usize, w: &mut dyn std::fmt::Write) -> std::fmt::Result
src/lir/lower/mir_stmts.rs:285: fn for_each_child_stmt(&self, f: &mut dyn FnMut(&dyn MirStmtNode)) { for s in &self.stmts { f(&**s); } }    fn as_block(&self) -> Option<&[MirStmtBox]> { Some(&self.stmts) }
src/lir/lower/mir_stmts.rs:289: impl MirStmtNode for SMirDropStmt { fn span(&self) -> crate::span::Span { self.span }
src/lir/lower/mir_stmts.rs:290: fn clone_stmt(&self) -> Box<dyn MirStmtNode> { Box::new(self.clone()) }
src/lir/lower/mir_stmts.rs:291: fn lower_to_lir_stmt(&self, ctx: &mut dyn LirLowerCtx)
src/lir/lower/mir_stmts.rs:294: fn display_stmt(&self, level: usize, w: &mut dyn std::fmt::Write) -> std::fmt::Result
src/lir/lower/mir_stmts.rs:297: fn as_drop(&self) -> Option<(VarId, &HirType)> { Some((self.var, &self.ty)) }
src/lir/lower/mod.rs:10: mod ctx;
src/lir/lower/mod.rs:11: mod fn_lower;
src/lir/lower/mod.rs:12: mod mir_expr;
src/lir/lower/mod.rs:13: mod mir_expr2;
src/lir/lower/mod.rs:14: mod mir_contract;
src/lir/lower/mod.rs:15: mod mir_stmts;
src/lir/lower/mod.rs:16: mod mir_ref;
src/lir/lower/mod.rs:17: pub(crate) mod names;
src/lir/lower/mod.rs:18: mod strings;
src/lir/lower/mod.rs:19: mod util;
src/lir/lower/mod.rs:22: fn collect_effect_summaries(items: &[MirItem], out: &mut HashMap<String, crate::hir::effects::EffectSummary>)
src/lir/lower/mod.rs:38: fn collect_specialized_fns(items: &[MirItem], out: &mut std::collections::HashSet<FnId>)
src/lir/lower/mod.rs:53: pub fn lower_program(mir: &MirProgram) -> LirProgram
src/lir/lower/names.rs:3: pub(super) fn collect_fn_names(mir: &MirProgram) -> HashMap<FnId, String>
src/lir/lower/names.rs:9: pub(super) fn collect_fn_names_items(items: &[MirItem], _prefix: &str, map: &mut HashMap<FnId, String>)
src/lir/lower/names.rs:34: fn unique_fn_name(base: &str, f: &MirFn, map: &HashMap<FnId, String>) -> String
src/lir/lower/names.rs:46: pub(crate) fn mangle(prefix: &str, name: &str, params: &[(crate::intern::Symbol, HirType)]) -> String
src/lir/lower/names.rs:65: pub(super) fn type_to_mangle(ty: &HirType) -> String
src/lir/lower/strings.rs:3: pub(super) fn collect_strings(mir: &MirProgram) -> Vec<String>
src/lir/lower/strings.rs:11: pub(super) fn collect_strings_items(items: &[MirItem], out: &mut Vec<String>)
src/lir/lower/strings.rs:21: pub(super) fn collect_strings_stmts(stmts: &[MirStmtBox], out: &mut Vec<String>)
src/lir/lower/strings.rs:28: pub(super) fn collect_strings_stmt(stmt: &dyn MirStmtNode, out: &mut Vec<String>)
src/lir/lower/strings.rs:37: pub(super) fn collect_strings_dyn(node: &dyn MirNode, out: &mut Vec<String>)
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
src/lir/serialize/decode.rs:27: pub(super) fn value(&mut self) -> Result<LirValue>
src/lir/serialize/decode.rs:37: pub(super) fn inst(&mut self) -> Result<LirNodeBox>
src/lir/serialize/decode.rs:245: pub(super) fn read_fn(&mut self) -> Result<LirFn>
src/lir/serialize/mod.rs:11: pub fn program_to_bytes(p: &LirProgram) -> Vec<u8>
src/lir/serialize/mod.rs:100: pub fn program_from_bytes(data: &[u8]) -> Result<LirProgram>
src/lir/serialize/mod.rs:228: struct Reader<'a>
src/lir/serialize/mod.rs:235: mod decode;
src/lir/serialize/mod.rs:236: mod reader;
src/lir/serialize/mod.rs:237: mod write;
src/lir/serialize/reader.rs:3: impl<'a> Reader<'a>
src/lir/serialize/reader.rs:4: pub(super) fn read(&mut self, n: usize) -> Result<&'a [u8]>
src/lir/serialize/reader.rs:12: pub(super) fn u32(&mut self) -> Result<u32>
src/lir/serialize/reader.rs:16: pub(super) fn u64(&mut self) -> Result<u64>
src/lir/serialize/reader.rs:20: pub(super) fn str(&mut self) -> Result<String>
src/lir/serialize/reader.rs:25: pub(super) fn ty(&mut self) -> Result<HirType>
src/lir/serialize/write.rs:7: pub(super) fn put_inst(buf: &mut Vec<u8>, inst: &LirNodeBox)
src/lir/serialize/write.rs:11: pub(super) fn put_fn(buf: &mut Vec<u8>, f: &LirFn)
src/main.rs:18: fn main()
src/main.rs:59: pub(crate) fn find_project(dir: &Path) -> Option<(PathBuf, String)>
src/main.rs:74: pub(crate) fn project_entry(project_dir: &Path) -> Option<PathBuf>
src/main.rs:83: pub(crate) fn resolve_path(arg: Option<&str>) -> PathBuf
src/main.rs:111: mod cli;
src/main.rs:115: pub(crate) fn load_config() -> Option<(PathBuf, ayanami::package::config::ProjectConfig)>
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
src/mir/borrow/follow.rs:60: pub struct FollowInfo
src/mir/borrow/follow.rs:65: pub fn build_table(mir: &MirProgram) -> HashMap<FnId, FollowInfo>
src/mir/borrow/follow.rs:71: fn collect_items(items: &[MirItem], out: &mut HashMap<FnId, FollowInfo>)
src/mir/borrow/liveness.rs:6: pub fn live_in(cfg: &cfg::Cfg) -> Vec<HashSet<VarId>>
src/mir/borrow/loans.rs:6: pub(super) struct Loan
src/mir/borrow/loans.rs:18: pub fn check_fn(mir_fn: &MirFn, ref_params: &[(VarId, bool)], table: &super::FollowTable) -> Result<()>
src/mir/borrow/mod.rs:14: mod cfg;
src/mir/borrow/mod.rs:15: mod collect;
src/mir/borrow/mod.rs:16: mod follow;
src/mir/borrow/mod.rs:17: mod liveness;
src/mir/borrow/mod.rs:18: mod loans;
src/mir/borrow/mod.rs:19: mod walk;
src/mir/borrow/mod.rs:28: pub type FollowTable = HashMap<crate::hir::ty::FnId, follow::FollowInfo>;
src/mir/borrow/mod.rs:31: pub fn build_follow_table(mir: &MirProgram) -> FollowTable
src/mir/borrow/mod.rs:35: pub fn check_borrows(mir_fn: &MirFn, table: &FollowTable) -> Result<()>
src/mir/borrow/walk.rs:5: pub(super) fn var_name(mir_fn: &MirFn, v: VarId) -> String
src/mir/borrow/walk.rs:14: pub(super) fn walk_stmt(
src/mir/borrow/walk.rs:55: fn walk_expr(
src/mir/display.rs:7: pub fn display_mir_program(program: &MirProgram)
src/mir/display.rs:11: pub fn mir_program_to_string(program: &MirProgram) -> String
src/mir/display.rs:20: fn pad(n: usize) -> String
src/mir/display.rs:24: fn write_item(item: &MirItem, level: usize, w: &mut impl Write) -> std::fmt::Result
src/mir/display.rs:60: fn write_stmts(stmts: &[MirStmtBox], level: usize, w: &mut impl Write) -> std::fmt::Result
src/mir/ir.rs:11: pub trait MirNode: std::fmt::Debug
src/mir/ir.rs:12: fn clone_node(&self) -> Box<dyn MirNode>;
src/mir/ir.rs:13: fn lower_to_lir(&self, ctx: &mut dyn LirLowerCtx) -> LirValue;
src/mir/ir.rs:14: fn display(&self, level: usize, w: &mut dyn FmtWrite) -> std::fmt::Result;
src/mir/ir.rs:15: fn expr_type(&self) -> HirType;
src/mir/ir.rs:16: fn as_local(&self) -> Option<VarId> { None }
src/mir/ir.rs:18: fn as_global(&self) -> Option<Symbol> { None }
src/mir/ir.rs:20: fn as_deref(&self) -> Option<&MirNodeBox> { None }
src/mir/ir.rs:22: fn refs_global(&self) -> Option<Symbol> { None }
src/mir/ir.rs:24: fn as_index(&self) -> Option<(&MirNodeBox, &MirNodeBox)> { None }
src/mir/ir.rs:26: fn collect_var_ids(&self, vars: &mut HashSet<VarId>)
src/mir/ir.rs:29: fn record_moves(&self, moved: &mut HashSet<VarId>)
src/mir/ir.rs:32: fn for_each_child(&self, _f: &mut dyn FnMut(&dyn MirNode)) {}
src/mir/ir.rs:34: fn for_each_child_mut(&mut self, _f: &mut dyn FnMut(&mut MirNodeBox)) {}
src/mir/ir.rs:35: fn as_string_literal(&self) -> Option<&str> { None }
src/mir/ir.rs:36: fn as_ref(&self) -> Option<(VarId, bool)> { None }
src/mir/ir.rs:38: fn as_field_access(&self) -> Option<(&MirNodeBox, usize)> { None }
src/mir/ir.rs:40: fn is_call(&self) -> bool { false }
src/mir/ir.rs:42: fn struct_literal_fields(&self) -> Option<&[(Symbol, MirNodeBox)]> { None }
src/mir/ir.rs:44: fn move_expr(&self) -> Option<&MirNodeBox> { None }
src/mir/ir.rs:46: fn call_fn_id(&self) -> Option<crate::hir::ty::FnId> { None }
src/mir/ir.rs:48: fn is_alloc(&self) -> bool { false }
src/mir/ir.rs:50: fn is_asm(&self) -> bool { false }
src/mir/ir.rs:52: fn binary_op(&self) -> Option<BinaryOp> { None }
src/mir/ir.rs:53: fn unary_op(&self) -> Option<UnaryOp> { None }
src/mir/ir.rs:54: fn literal_value(&self) -> Option<(&HirLiteral, &HirType)> { None }
src/mir/ir.rs:58: pub struct MirNodeBox(pub Box<dyn MirNode>);
src/mir/ir.rs:59: impl Clone for MirNodeBox
src/mir/ir.rs:60: fn clone(&self) -> Self { MirNodeBox(self.0.clone_node()) }
src/mir/ir.rs:62: impl std::ops::Deref for MirNodeBox
src/mir/ir.rs:63: type Target = dyn MirNode;
src/mir/ir.rs:64: fn deref(&self) -> &Self::Target { &*self.0 }
src/mir/ir.rs:66: impl std::ops::DerefMut for MirNodeBox
src/mir/ir.rs:67: fn deref_mut(&mut self) -> &mut Self::Target { &mut *self.0 }
src/mir/ir.rs:70: pub trait MirStmtNode: std::fmt::Debug
src/mir/ir.rs:71: fn clone_stmt(&self) -> Box<dyn MirStmtNode>;
src/mir/ir.rs:72: fn lower_to_lir_stmt(&self, ctx: &mut dyn LirLowerCtx);
src/mir/ir.rs:73: fn display_stmt(&self, level: usize, w: &mut dyn FmtWrite) -> std::fmt::Result;
src/mir/ir.rs:74: fn for_each_child_expr(&self, _f: &mut dyn FnMut(&dyn MirNode)) {}
src/mir/ir.rs:75: fn for_each_child_stmt(&self, _f: &mut dyn FnMut(&dyn MirStmtNode)) {}
src/mir/ir.rs:77: fn for_each_child_expr_mut(&mut self, _f: &mut dyn FnMut(&mut MirNodeBox)) {}
src/mir/ir.rs:78: fn for_each_child_stmt_mut(&mut self, _f: &mut dyn FnMut(&mut MirStmtBox)) {}
src/mir/ir.rs:80: fn span(&self) -> Span { Span::default() }
src/mir/ir.rs:81: fn is_return(&self) -> bool { false }
src/mir/ir.rs:82: fn return_value(&self) -> Option<&MirNodeBox> { None }
src/mir/ir.rs:83: fn as_drop(&self) -> Option<(VarId, &HirType)> { None }
src/mir/ir.rs:85: fn as_if(&self) -> Option<IfParts<'_>> { None }
src/mir/ir.rs:86: fn as_while(&self) -> Option<WhileParts<'_>> { None }
src/mir/ir.rs:87: fn as_block(&self) -> Option<&[MirStmtBox]> { None }
src/mir/ir.rs:88: fn is_break(&self) -> bool { false }
src/mir/ir.rs:89: fn is_continue(&self) -> bool { false }
src/mir/ir.rs:90: fn assign_parts(&self) -> Option<(&MirNodeBox, &MirNodeBox)> { None }
src/mir/ir.rs:91: fn field_assign_parts(&self) -> Option<(&MirNodeBox, &MirNodeBox)> { None }
src/mir/ir.rs:92: fn index_assign_parts(&self) -> Option<(&MirNodeBox, &MirNodeBox, &MirNodeBox)> { None }
src/mir/ir.rs:94: fn deref_assign_parts(&self) -> Option<(&MirNodeBox, &MirNodeBox)> { None }
src/mir/ir.rs:95: fn expr_part(&self) -> Option<&MirNodeBox> { None }
src/mir/ir.rs:98: pub type IfParts<'a> = (&'a MirNodeBox, &'a [MirStmtBox], &'a [(MirNodeBox, Vec<MirStmtBox>)], &'a Option<Vec<MirStmtBox>>);
src/mir/ir.rs:99: pub type WhileParts<'a> = (&'a MirNodeBox, &'a [MirStmtBox]);
src/mir/ir.rs:102: pub struct MirStmtBox(pub Box<dyn MirStmtNode>);
src/mir/ir.rs:103: impl Clone for MirStmtBox
src/mir/ir.rs:104: fn clone(&self) -> Self { MirStmtBox(self.0.clone_stmt()) }
src/mir/ir.rs:106: impl std::ops::Deref for MirStmtBox
src/mir/ir.rs:107: type Target = dyn MirStmtNode;
src/mir/ir.rs:108: fn deref(&self) -> &Self::Target { &*self.0 }
src/mir/ir.rs:110: impl std::ops::DerefMut for MirStmtBox
src/mir/ir.rs:111: fn deref_mut(&mut self) -> &mut Self::Target { &mut *self.0 }
src/mir/ir.rs:117: pub struct $name { $(pub $field: $ty),* }
src/mir/ir.rs:125: pub struct $name { $(pub $field: $ty),* }
src/mir/ir.rs:176: fn from(v: $ty) -> Self { MirNodeBox(Box::new(v) as Box<dyn MirNode>) }
src/mir/ir.rs:185: fn from(v: $ty) -> Self { MirStmtBox(Box::new(v) as Box<dyn MirStmtNode>) }
src/mir/ir.rs:192: pub struct MirLocal
src/mir/ir.rs:198: impl MirLocal
src/mir/ir.rs:199: pub fn new(name: Symbol, ty: HirType, mutable: bool) -> Self
src/mir/ir.rs:205: pub struct MirFn
src/mir/ir.rs:235: pub enum MirItem
src/mir/ir.rs:248: pub struct MirProgram
src/mir/lower/checks.rs:8: pub(super) fn block_always_returns(stmts: &[HirStmt]) -> bool
src/mir/lower/checks.rs:21: impl Ctx
src/mir/lower/checks.rs:25: pub(super) fn check_use_after_move(&mut self, stmt: &HirStmt)
src/mir/lower/checks.rs:53: fn walk_stmt_moves(&mut self, expr: &dyn HirNode, sp: Span)
src/mir/lower/control.rs:4: impl Ctx
src/mir/lower/control.rs:5: pub(super) fn lower_if(
src/mir/lower/control.rs:58: pub(super) fn lower_while(&mut self, cond: &HirNodeBox, body: &HirBlock, span: crate::span::Span) -> Vec<MirStmtBox>
src/mir/lower/control.rs:65: pub(super) fn lower_block(&mut self, stmts: &[HirStmt]) -> Vec<MirStmtBox>
src/mir/lower/control.rs:94: pub(super) fn mark_alive(&mut self, var: VarId)
src/mir/lower/ctx.rs:4: impl Ctx
src/mir/lower/ctx.rs:5: pub(super) fn new(f: &HirFn, struct_defs: &HashMap<Symbol, Vec<(Symbol, HirType)>>) -> Self
src/mir/lower/ctx.rs:30: fn emit_assign_cleanup(&self, var: &VarId) -> Vec<MirStmtBox>
src/mir/lower/ctx.rs:42: pub(super) fn lower_stmt(&mut self, stmt: &HirStmt) -> Vec<MirStmtBox>
src/mir/lower/ctx.rs:99: fn track_stmt_moves(&mut self, stmt: &HirStmt)
src/mir/lower/ctx.rs:125: fn lower_assign(&mut self, target: &HirNodeBox, value: &HirNodeBox, span: crate::span::Span) -> Vec<MirStmtBox>
src/mir/lower/ctx.rs:170: fn lower_return(&mut self, value: &Option<HirNodeBox>, span: crate::span::Span) -> Vec<MirStmtBox>
src/mir/lower/ctx.rs:219: fn new_temp(&mut self, ty: HirType) -> VarId
src/mir/lower/functions.rs:5: pub(super) fn lower_item(item: &HirItem, struct_defs: &HashMap<Symbol, Vec<(Symbol, HirType)>>) -> Result<Vec<MirItem>>
src/mir/lower/functions.rs:23: fn lower_fn(f: &HirFn, struct_defs: &HashMap<Symbol, Vec<(Symbol, HirType)>>) -> Result<MirFn>
src/mir/lower/mem.rs:6: fn needs_drop(ty: &HirType, struct_defs: &HashMap<Symbol, Vec<(Symbol, HirType)>>) -> bool
src/mir/lower/mem.rs:19: pub(super) fn strategy_for(ty: &HirType, struct_defs: &HashMap<Symbol, Vec<(Symbol, HirType)>>) -> Box<dyn MemStrategy>
src/mir/lower/mem.rs:26: pub(super) fn action_to_stmt(_var: VarId, ty: &HirType, action: &MemAction) -> MirStmtBox
src/mir/lower/mod.rs:9: mod checks;
src/mir/lower/mod.rs:10: mod control;
src/mir/lower/mod.rs:11: mod ctx;
src/mir/lower/mod.rs:12: mod functions;
src/mir/lower/mod.rs:13: mod mem;
src/mir/lower/mod.rs:17: pub fn lower_program(hir: &HirProgram) -> crate::error::Result<MirProgram>
src/mir/lower/mod.rs:43: struct Ctx
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
src/package/bytes.rs:93: pub fn write_to_file(&self, path: &str) -> Result<()>
src/package/config.rs:5: pub struct ProjectConfig
src/package/config.rs:15: impl ProjectConfig
src/package/config.rs:16: pub fn load(toml_content: &str) -> Self
src/package/config.rs:67: pub fn resolve_import<'a>(&'a self, import_path: &str, base_dir: &Path) -> Option<String>
src/package/config.rs:84: pub fn resolve_target(&self, file_path: &Path) -> &str
src/package/const_codec.rs:9: pub fn type_str(ty: &HirType) -> Option<String>
src/package/const_codec.rs:30: pub fn type_from_str(s: &str) -> Option<HirType>
src/package/const_codec.rs:64: pub fn lit_str(lit: &HirLiteral) -> Option<String>
src/package/const_codec.rs:75: pub fn lit_from_str(ty: &HirType, s: &str) -> Option<HirLiteral>
src/package/const_codec.rs:86: pub fn encode(ty: &HirType, lit: &HirLiteral) -> Option<(String, String)>
src/package/const_codec.rs:91: pub fn decode(ty: &str, value: &str) -> Option<(HirType, HirLiteral)>
src/package/load.rs:3: pub fn load_package(path: &str) -> Result<(Vec<ImportedSymbol>, Vec<String>, Vec<u8>, Vec<TargetType>)>
src/package/load.rs:123: fn parse_ini_value(s: &str) -> String
src/package/load.rs:134: pub fn resolve_package_deps(path: &str) -> Vec<std::path::PathBuf>
src/package/load.rs:162: pub fn load_package_deps(path: &str) -> Result<Vec<String>>
src/package/load.rs:192: pub(super) fn type_to_string(ty: &Type) -> String
src/package/mod.rs:1: pub mod config;
src/package/mod.rs:6: mod bytes;
src/package/mod.rs:7: pub(crate) mod const_codec;
src/package/mod.rs:8: mod load;
src/package/mod.rs:9: mod symbols;
src/package/mod.rs:10: mod target;
src/package/mod.rs:11: mod types;
src/package/symbols.rs:4: impl Package
src/package/symbols.rs:5: pub fn new(name: String, version: String) -> Self
src/package/symbols.rs:21: pub fn set_effect_summaries(
src/package/symbols.rs:29: pub fn set_consts(&mut self, consts: &[(crate::intern::Symbol, crate::hir::ir::HirType, crate::hir::ir::HirLiteral)])
src/package/symbols.rs:42: pub fn set_statics(&mut self, statics: &[crate::hir::HirStatic], all: bool)
src/package/symbols.rs:55: pub fn collect_symbols(&mut self, stmts: &[Stmt])
src/package/symbols.rs:60: pub fn collect_all_symbols(&mut self, stmts: &[Stmt])
src/package/symbols.rs:64: fn collect_symbols_with_prefix(&mut self, stmts: &[Stmt], all: bool, ns_prefix: &str)
src/package/symbols.rs:71: fn collect_stmt_symbols(&mut self, stmt: &Stmt, all: bool, ns_prefix: &str)
src/package/target.rs:4: pub enum TargetType
src/package/target.rs:10: impl TargetType
src/package/target.rs:11: pub fn as_str(&self) -> &'static str
src/package/target.rs:19: pub fn from_str(s: &str) -> Option<Self>
src/package/types.rs:5: pub struct Package
src/package/types.rs:19: pub enum PackageSymbol
src/package/types.rs:69: pub enum ImportedSymbol
src/parser/ast/binary_op.rs:2: pub enum BinaryOp
src/parser/ast/block.rs:6: pub struct Block
src/parser/ast/block.rs:13: impl Block
src/parser/ast/block.rs:14: pub fn new(stmts: Vec<Stmt>, span: Span) -> Self
src/parser/ast/expr.rs:9: pub enum Expr
src/parser/ast/expr.rs:128: impl Expr
src/parser/ast/expr.rs:129: pub fn span(&self) -> Span
src/parser/ast/literal.rs:4: pub enum Literal
src/parser/ast/literal.rs:12: impl Literal
src/parser/ast/literal.rs:13: pub fn span(&self) -> Span
src/parser/ast/mod.rs:1: pub mod binary_op;
src/parser/ast/mod.rs:2: pub mod block;
src/parser/ast/mod.rs:3: pub mod expr;
src/parser/ast/mod.rs:4: pub mod literal;
src/parser/ast/mod.rs:5: pub mod pattern;
src/parser/ast/mod.rs:6: pub mod program;
src/parser/ast/mod.rs:7: pub mod stmt;
src/parser/ast/mod.rs:8: pub mod ty;
src/parser/ast/mod.rs:9: pub mod unary_op;
src/parser/ast/mod.rs:10: pub mod vis;
src/parser/ast/pattern.rs:6: pub enum Pattern
src/parser/ast/pattern.rs:23: impl Pattern
src/parser/ast/pattern.rs:25: pub fn is_irrefutable(&self) -> bool
src/parser/ast/pattern.rs:35: pub fn bindings(&self) -> Vec<Symbol>
src/parser/ast/pattern.rs:46: pub fn display(&self) -> String
src/parser/ast/pattern.rs:77: pub fn enum_variant(&self) -> Option<&Symbol>
src/parser/ast/pattern.rs:86: fn display_lit(l: &Literal) -> String
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
src/parser/ast/stmt.rs:65: pub enum MatchBody
src/parser/ast/stmt.rs:71: pub struct MatchArm
src/parser/ast/stmt.rs:80: pub enum Stmt
src/parser/ast/stmt.rs:223: impl Stmt
src/parser/ast/stmt.rs:224: pub fn span(&self) -> Span
src/parser/ast/ty.rs:5: pub enum Type
src/parser/ast/ty.rs:28: impl Type
src/parser/ast/ty.rs:29: pub fn span(&self) -> Span
src/parser/ast/ty.rs:50: pub fn inner(&self) -> Option<&Type>
src/parser/ast/unary_op.rs:2: pub enum UnaryOp
src/parser/ast/vis.rs:2: pub enum Visibility
src/parser/ast/vis.rs:8: impl Visibility
src/parser/ast/vis.rs:9: pub fn is_public(&self) -> bool
src/parser/mod.rs:7: pub mod parser;
src/parser/mod.rs:8: pub mod ast;
src/parser/mod.rs:15: pub fn parse_source(source: &str) -> Result<crate::parser::ast::program::Program>
src/parser/mod.rs:25: pub fn parse_expression(source: &str) -> Result<crate::parser::ast::expr::Expr>
src/parser/parser/array_expr.rs:3: impl Parser
src/parser/parser/array_expr.rs:5: pub(super) fn parse_array_expr(&mut self, bracket_span: Span) -> Result<Expr>
src/parser/parser/atom.rs:5: impl Parser
src/parser/parser/atom.rs:6: pub(super) fn parse_atom(&mut self) -> Result<Expr>
src/parser/parser/const_decl.rs:3: impl Parser
src/parser/parser/const_decl.rs:5: pub(super) fn parse_static_decl(
src/parser/parser/const_decl.rs:31: pub(super) fn parse_const_decl(
src/parser/parser/core.rs:3: impl Parser
src/parser/parser/core.rs:4: pub fn new(tokens: Vec<Token>) -> Self
src/parser/parser/core.rs:9: pub(crate) fn allow_legacy_fn_types(&mut self)
src/parser/parser/core.rs:13: pub(super) fn peek(&self) -> Option<&Token>
src/parser/parser/core.rs:17: pub(super) fn advance(&mut self) -> Option<Token>
src/parser/parser/core.rs:23: pub(super) fn error(&self, msg: &str) -> Error
src/parser/parser/core.rs:31: pub(super) fn expect_keyword(&mut self, kw: Keyword) -> Result<()>
src/parser/parser/core.rs:42: pub(super) fn expect_delimiter(&mut self, d: Delimiter) -> Result<()>
src/parser/parser/core.rs:53: pub(super) fn expect_operator(&mut self, op: &str) -> Result<()>
src/parser/parser/core.rs:65: pub(super) fn is_stmt_only_keyword(kind: &TokenKind) -> bool
src/parser/parser/core.rs:76: pub(super) fn is_stmt_start(kind: &TokenKind) -> bool
src/parser/parser/core.rs:91: pub(super) fn try_semicolon(&mut self) -> Result<()>
src/parser/parser/core.rs:102: pub(super) fn parse_visibility(&mut self) -> Visibility
src/parser/parser/core.rs:123: pub(super) fn expect_identifier(&mut self) -> Result<String>
src/parser/parser/core.rs:141: pub fn parse_program(&mut self) -> Result<Program>
src/parser/parser/core.rs:149: pub(super) fn parse_attr_list(&mut self) -> Result<Vec<crate::parser::ast::Attr>>
src/parser/parser/core.rs:206: fn parse_attr_arg(&mut self) -> Result<crate::parser::ast::AttrArg>
src/parser/parser/core.rs:224: impl Parser
src/parser/parser/core.rs:226: pub(super) fn matching_bracket(&self, start: usize) -> Option<usize>
src/parser/parser/decl.rs:3: impl Parser
src/parser/parser/decl.rs:5: pub(super) fn parse_fn_decl(&mut self, vis: Visibility, is_inline: bool, extern_c: bool, attrs: Vec<crate::parser::ast::Attr>) -> Result<Stmt>
src/parser/parser/decl.rs:88: pub(super) fn parse_return(&mut self) -> Result<Stmt>
src/parser/parser/decl.rs:105: pub(super) fn parse_cond_expr(&mut self) -> Result<Expr>
src/parser/parser/decl.rs:112: pub(super) fn parse_if(&mut self) -> Result<Stmt>
src/parser/parser/decl.rs:118: pub(super) fn parse_if_expr(&mut self) -> Result<Expr>
src/parser/parser/decl.rs:123: fn parse_if_parts(&mut self) -> Result<(Expr, Block, Vec<(Expr, Block)>, Option<Block>, Span)>
src/parser/parser/decl.rs:149: pub(super) fn parse_for(&mut self) -> Result<Stmt>
src/parser/parser/decl.rs:176: pub(super) fn parse_while(&mut self) -> Result<Stmt>
src/parser/parser/decl.rs:186: pub(super) fn parse_match_stmt(&mut self) -> Result<Stmt>
src/parser/parser/decl.rs:194: pub(super) fn parse_match_expr(&mut self) -> Result<Expr>
src/parser/parser/decl.rs:224: pub(super) fn parse_namespace(&mut self, vis: Visibility) -> Result<Stmt>
src/parser/parser/decl.rs:247: pub(super) fn parse_struct_def(&mut self, vis: Visibility, attrs: Vec<crate::parser::ast::Attr>) -> Result<Stmt>
src/parser/parser/enum_iface.rs:3: impl Parser
src/parser/parser/enum_iface.rs:6: pub(super) fn parse_enum_def(&mut self, vis: Visibility, attrs: Vec<crate::parser::ast::Attr>) -> Result<Stmt>
src/parser/parser/enum_iface.rs:74: pub(super) fn parse_interface_def(&mut self, attrs: Vec<crate::parser::ast::Attr>) -> Result<Stmt>
src/parser/parser/enum_iface.rs:118: pub(super) fn parse_interface_method(&mut self) -> Result<InterfaceMethod>
src/parser/parser/expr.rs:3: impl Parser
src/parser/parser/expr.rs:7: pub fn parse_expr_entry(&mut self) -> Result<Expr>
src/parser/parser/expr.rs:11: pub(super) fn parse_expr(&mut self) -> Result<Expr>
src/parser/parser/expr.rs:15: pub(super) fn parse_or(&mut self) -> Result<Expr>
src/parser/parser/expr.rs:31: pub(super) fn parse_and(&mut self) -> Result<Expr>
src/parser/parser/expr.rs:47: pub(super) fn parse_compare(&mut self) -> Result<Expr>
src/parser/parser/expr.rs:80: pub(super) fn parse_bitor(&mut self) -> Result<Expr>
src/parser/parser/expr.rs:96: pub(super) fn parse_bitxor(&mut self) -> Result<Expr>
src/parser/parser/expr.rs:112: pub(super) fn parse_bitand(&mut self) -> Result<Expr>
src/parser/parser/expr.rs:128: pub(super) fn parse_shift(&mut self) -> Result<Expr>
src/parser/parser/expr.rs:157: pub(super) fn parse_sum(&mut self) -> Result<Expr>
src/parser/parser/expr.rs:186: pub(super) fn parse_cast(&mut self) -> Result<Expr>
src/parser/parser/expr.rs:197: pub(super) fn parse_product(&mut self) -> Result<Expr>
src/parser/parser/impls.rs:3: impl Parser
src/parser/parser/impls.rs:6: pub(super) fn parse_import(&mut self) -> Result<Stmt>
src/parser/parser/impls.rs:35: pub(super) fn parse_impl_block(&mut self, attrs: Vec<crate::parser::ast::Attr>) -> Result<Stmt>
src/parser/parser/impls.rs:61: fn extract_type_name(ty: &Type) -> Symbol
src/parser/parser/impls.rs:98: pub(super) fn parse_impl_method(&mut self, impl_type: &Symbol, impl_generic_params: &[(Symbol, Option<Symbol>)]) -> Result<Stmt>
src/parser/parser/impls.rs:244: pub(super) fn parse_block(&mut self) -> Result<Block>
src/parser/parser/literal_text.rs:4: pub(super) fn split_literal_suffix(s: &str) -> (&str, Option<&str>)
src/parser/parser/literal_text.rs:25: pub(super) fn parse_int_text(s: &str) -> Option<i64>
src/parser/parser/macro_call.rs:3: impl Parser
src/parser/parser/macro_call.rs:5: pub(super) fn parse_macro_call(&mut self) -> Result<Expr>
src/parser/parser/mod.rs:8: pub struct Parser
src/parser/parser/mod.rs:18: fn ast_type_text(ty: &Type) -> String
src/parser/parser/mod.rs:39: mod array_expr;
src/parser/parser/mod.rs:40: mod atom;
src/parser/parser/mod.rs:41: mod const_decl;
src/parser/parser/mod.rs:42: mod self_type;
src/parser/parser/mod.rs:43: mod core;
src/parser/parser/mod.rs:44: mod decl;
src/parser/parser/mod.rs:45: mod enum_iface;
src/parser/parser/mod.rs:46: mod expr;
src/parser/parser/mod.rs:47: mod impls;
src/parser/parser/mod.rs:48: mod literal_text;
src/parser/parser/mod.rs:49: mod macro_call;
src/parser/parser/mod.rs:50: mod pattern;
src/parser/parser/mod.rs:51: mod stmt;
src/parser/parser/mod.rs:52: mod types;
src/parser/parser/mod.rs:53: mod unary;
src/parser/parser/pattern.rs:6: impl Parser
src/parser/parser/pattern.rs:8: pub(super) fn parse_pattern(&mut self) -> Result<Pattern>
src/parser/parser/pattern.rs:18: fn parse_pattern_literal(&mut self) -> Result<crate::parser::ast::literal::Literal>
src/parser/parser/pattern.rs:41: fn parse_pattern_atom(&mut self) -> Result<Pattern>
src/parser/parser/pattern.rs:121: impl Parser
src/parser/parser/pattern.rs:123: pub(super) fn parse_match_arm_body(&mut self) -> Result<crate::parser::ast::stmt::MatchBody>
src/parser/parser/self_type.rs:4: pub(super) fn subst_self_in_type(ty: &Type, self_ty: &Type) -> Type
src/parser/parser/stmt.rs:3: impl Parser
src/parser/parser/stmt.rs:6: pub(super) fn parse_stmt(&mut self) -> Result<Stmt>
src/parser/parser/stmt.rs:95: pub(super) fn parse_any_assign_or_expr(&mut self) -> Result<Stmt>
src/parser/parser/stmt.rs:121: pub(super) fn is_type_start(&self, pos: usize) -> bool
src/parser/parser/stmt.rs:129: pub(super) fn parse_lambda(&mut self) -> Result<Expr>
src/parser/parser/types.rs:3: impl Parser
src/parser/parser/types.rs:6: pub(super) fn parse_type(&mut self) -> Result<Type>
src/parser/parser/types.rs:24: pub(super) fn parse_base_type(&mut self) -> Result<Type>
src/parser/parser/types.rs:149: pub(super) fn handle_path_sep(&mut self, name_str: &mut String, name_sym: &mut Symbol) -> Result<()>
src/parser/parser/unary.rs:3: impl Parser
src/parser/parser/unary.rs:4: pub(super) fn parse_unary(&mut self) -> Result<Expr>
src/parser/parser/unary.rs:71: pub(super) fn parse_postfix(&mut self) -> Result<Expr>
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
example/check_lib.aya:5: pub fn warn_side_effects(ref MirFunction f)
example/const_prop_lib.aya:5: pub fn const_prop(ref mut MirFunction f)
example/constfold_lib.aya:5: fn op_is_band(int op) -> bool { return op == 14 }
example/constfold_lib.aya:6: fn op_is_bor(int op) -> bool { return op == 15 }
example/constfold_lib.aya:7: fn op_is_bxor(int op) -> bool { return op == 16 }
example/constfold_lib.aya:8: fn op_is_shl(int op) -> bool { return op == 17 }
example/constfold_lib.aya:9: fn op_is_shr(int op) -> bool { return op == 18 }
example/constfold_lib.aya:11: fn lt_int(int a, int b) -> int
example/constfold_lib.aya:16: fn gt_int(int a, int b) -> int
example/constfold_lib.aya:21: fn eq_int(int a, int b) -> int
example/constfold_lib.aya:26: fn neq_int(int a, int b) -> int
example/constfold_lib.aya:32: pub fn const_fold(ref mut MirFunction f)
example/macro_lib.aya:6: pub fn answer() -> String { return "fn answer() -> int { return 42 }" }
example/macro_lib.aya:10: pub fn keep(String input) -> String { return input }
example/macro_lib.aya:15: pub fn emit_const(String input, String name, String value) -> String
example/macro_lib.aya:23: pub fn show(String input, String arg) -> String
example/macro_lib.aya:31: pub fn twice(String input) -> String { return input + input }
example/macro_lib.aya:36: pub fn guarded(String input, String cond) -> String
example/macro_lib.aya:44: pub fn double(String input, String x) -> String { return "(" + x + " + " + x + ")" }
example/macro_lib.aya:48: pub fn my_line(String input, int __line, int __col) -> String { return "" + __line }
example/math_lib.aya:1: pub fn add(int a, int b) -> int
example/pass_lib.aya:5: pub fn auto_pure(ref mut MirFunction f)
example/test.aya:1: fn main()->int
example/test.aya:14: fn add(int a, int b)->int
example/test.aya:17: fn sub(int a, int b)->int
example/test_aggregate_array.aya:5: struct Point
example/test_aggregate_array.aya:11: enum E
example/test_aggregate_array.aya:17: fn main() -> int
example/test_annotation_scope.aya:5: fn placeholder() -> int { return 0 }
example/test_annotation_scope.aya:7: fn main() -> int { return answer() - 42 }
example/test_array.aya:1: fn main() -> int
example/test_array_elem_field.aya:2: struct Inner { int a }
example/test_array_elem_field.aya:3: struct Outer
example/test_array_elem_field.aya:8: fn main() -> int
example/test_array_param.aya:1: fn sum([int] arr) -> int
example/test_array_param.aya:5: fn main() -> int
example/test_arraylist.aya:3: fn main() -> int
example/test_asm.aya:1: fn main() -> int
example/test_assume.aya:3: fn dec(int n) -> int { return n - 1 }
example/test_assume.aya:7: fn id(int x) -> int { return x }
example/test_assume.aya:9: fn main() -> int
example/test_attrs.aya:3: fn add(int a, int b) -> int { return a + b }
example/test_attrs.aya:9: fn main() -> int
example/test_bitwise.aya:2: fn shl_u8(u8 a, u8 b) -> u8 { return a << b }
example/test_bitwise.aya:3: fn shr_u8(u8 a, u8 b) -> u8 { return a >> b }
example/test_bitwise.aya:4: fn shr_i8(i8 a, i8 b) -> i8 { return a >> b }
example/test_bitwise.aya:5: fn bitand32(i32 a, i32 b) -> i32 { return a & b }
example/test_bitwise.aya:6: fn bitor32(i32 a, i32 b) -> i32 { return a | b }
example/test_bitwise.aya:7: fn bitxor32(i32 a, i32 b) -> i32 { return a ^ b }
example/test_bitwise.aya:8: fn bitnot_i32(i32 a) -> i32 { return ~a }
example/test_bitwise.aya:9: fn shl32(i32 a, i32 b) -> i32 { return a << b }
example/test_bitwise.aya:11: impl i32
example/test_bitwise.aya:12: fn bit_and32(self, i32 other) -> i32 { return self & other }
example/test_bitwise.aya:15: fn main() -> int
example/test_bool.aya:1: fn main() -> int
example/test_bool2.aya:1: fn main() -> int
example/test_bool_var.aya:2: fn main() -> int
example/test_bounds.aya:4: fn main() -> int
example/test_c_export.aya:3: fn aya_add(int a, int b) -> int
example/test_c_export.aya:12: fn main() -> int
example/test_cast.aya:2: fn main() -> int
example/test_cfg.aya:3: fn on_linux() -> int { return 0 }
example/test_cfg.aya:6: fn on_windows() -> int { return 1 }
example/test_cfg.aya:9: fn on_unix() -> int { return 2 }
example/test_cfg.aya:12: fn not_windows() -> int { return 3 }
example/test_cfg.aya:14: fn main() -> int
example/test_check.aya:6: fn allocs() -> int
example/test_check.aya:12: fn pure_ok(int a, int b) -> int
example/test_check.aya:16: fn main() -> int
example/test_closure_capture.aya:4: fn run() -> int
example/test_closure_capture.aya:15: fn main() -> int
example/test_closure_collections.aya:4: fn main() -> int
example/test_closure_hof.aya:2: enum Opt[T] { Some(T), None }
example/test_closure_hof.aya:4: impl[T] Opt[T]
example/test_closure_hof.aya:5: pub fn map[U](self, Fn(T) -> U f) -> Opt[U]
example/test_closure_hof.aya:13: struct Holder
example/test_closure_hof.aya:18: fn make_adder(int base) -> Fn(int) -> int
example/test_closure_hof.aya:22: fn main() -> int
example/test_collection_any.aya:5: enum E
example/test_collection_any.aya:10: fn main() -> int
example/test_comments.aya:3: fn main() -> int
example/test_const.aya:10: fn main() -> int
example/test_const_fn.aya:3: fn fib(int n) -> int
example/test_const_fn.aya:10: fn sum_to(int n) -> int
example/test_const_fn.aya:18: fn abs(int x) -> int
example/test_const_fn.aya:25: fn square(int x) -> int { return x * x }
example/test_const_fn.aya:29: fn main() -> int
example/test_const_ns.aya:11: fn main() -> int
example/test_constfold.aya:5: fn folded() -> int
example/test_constfold.aya:10: fn folded_mul() -> int
example/test_constfold.aya:16: fn folded_cmp_true() -> int
example/test_constfold.aya:22: fn folded_cmp_false() -> int
example/test_constfold.aya:29: fn folded_bitand() -> int
example/test_constfold.aya:34: fn folded_shl() -> int
example/test_constfold.aya:38: fn main() -> int
example/test_constprop.aya:7: fn prop_fold() -> int
example/test_constprop.aya:13: fn main() -> int
example/test_constructors.aya:6: fn main() -> int
example/test_conv.aya:2: fn take(int x) -> int { return x }
example/test_conv.aya:3: fn takef(float x) -> float { return x }
example/test_conv.aya:4: fn ret_char() -> int { return 'a' as int }
example/test_conv.aya:5: fn ret_int() -> float { return 3 as float }
example/test_conv.aya:7: struct P
example/test_conv.aya:12: impl P
example/test_conv.aya:13: fn bump(ref mut self, int v) -> int
example/test_conv.aya:19: fn main() -> int
example/test_early_return_move.aya:4: fn build(bool c) -> ArrayList[int]
example/test_early_return_move.aya:14: fn pick(bool c) -> [int]
example/test_early_return_move.aya:23: fn main() -> int
example/test_eff_import.aya:4: fn main() -> int
example/test_effects.aya:6: fn parse(int x) -> int { return x }
example/test_effects.aya:10: fn pure_fn() -> int { return 0 }
example/test_effects.aya:13: fn id[T](T x) -> T { return x }
example/test_effects.aya:17: fn declared_io() -> int { putchar(65); return 0 }
example/test_effects.aya:23: fn main() -> int
example/test_elif_string.aya:4: fn f(ref String s) -> int
example/test_elif_string.aya:11: fn main() -> int
example/test_ensures.aya:3: fn abs2(int x) -> int
example/test_ensures.aya:9: fn inc(int n) -> int { return n + 1 }
example/test_ensures.aya:12: fn fallthrough() -> int { }
example/test_ensures.aya:14: fn main() -> int
example/test_f32.aya:2: fn addf(f32 a, f32 b) -> f32 { return a + b }
example/test_f32.aya:3: fn take32(f32 x) -> f32 { return x }
example/test_f32.aya:4: fn to64(f32 x) -> float { return x as float }
example/test_f32.aya:5: fn to32(float x) -> f32 { return x as f32 }
example/test_f32.aya:6: fn f2i(f32 x) -> int { return x as int }
example/test_f32.aya:7: fn i2f(int x) -> f32 { return x as f32 }
example/test_f32.aya:8: fn pi32() -> f32 { return 3.14 }
example/test_f32.aya:10: struct P
example/test_f32.aya:15: fn main() -> int
example/test_ffi_attrs.aya:22: fn rarely() -> int { return 1 }
example/test_ffi_attrs.aya:25: fn fast() -> int { return 2 }
example/test_ffi_attrs.aya:28: fn always_returns() -> int { return 3 }
example/test_ffi_attrs.aya:30: fn touch(#[nonnull] ref int x) -> int { return 0 }
example/test_ffi_attrs.aya:32: fn main() -> int
example/test_field_move.aya:4: struct S { [int] data }
example/test_field_move.aya:6: fn run() -> int
example/test_field_move.aya:12: fn main() -> int
example/test_fn_once.aya:5: struct Holder
example/test_fn_once.aya:9: fn make(int base) -> FnOnce() -> int
example/test_fn_once.aya:14: fn apply(FnOnce() -> int f) -> int
example/test_fn_once.aya:18: fn run() -> int
example/test_fn_once.aya:32: fn main() -> int
example/test_fn_value.aya:2: fn seven() -> int
example/test_fn_value.aya:6: fn call0(Fn() -> int f) -> int
example/test_fn_value.aya:10: fn main() -> int
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
example/test_for_usize.aya:2: fn sum_upto(usize n) -> int
example/test_for_usize.aya:10: fn main() -> int
example/test_generic_bound_check.aya:2: interface Show
example/test_generic_bound_check.aya:3: fn show(ref self) -> int;
example/test_generic_bound_check.aya:6: interface Eq
example/test_generic_bound_check.aya:7: fn eq(ref self, Self other) -> bool;
example/test_generic_bound_check.aya:10: struct C[T] { T v }
example/test_generic_bound_check.aya:12: impl[T: Show] C[T]
example/test_generic_bound_check.aya:14: fn never_called(ref self) -> int
example/test_generic_bound_check.aya:18: pub fn called(ref self) -> int
example/test_generic_bound_check.aya:23: fn get(ref self) -> T
example/test_generic_bound_check.aya:28: impl int
example/test_generic_bound_check.aya:29: fn show(ref self) -> int { return self }
example/test_generic_bound_check.aya:30: fn eq(ref self, int other) -> bool { return self <= other && self >= other }
example/test_generic_bound_check.aya:33: struct D[T] { T v }
example/test_generic_bound_check.aya:35: impl[T: Eq] D[T]
example/test_generic_bound_check.aya:37: fn same(ref self) -> bool
example/test_generic_bound_check.aya:42: fn main() -> int
example/test_generic_constraint.aya:1: pub interface Into[T]
example/test_generic_constraint.aya:2: fn into(self) -> T;
example/test_generic_constraint.aya:5: struct F { float v }
example/test_generic_constraint.aya:7: impl F
example/test_generic_constraint.aya:8: fn into(self) -> float { return self.v }
example/test_generic_constraint.aya:11: fn use_into[U: Into[float]](U x) -> float
example/test_generic_constraint.aya:15: fn main() -> int
example/test_generic_enum_ctor.aya:4: enum E[T]
example/test_generic_enum_ctor.aya:9: impl[T] E[T]
example/test_generic_enum_ctor.aya:10: pub fn tag(self) -> int
example/test_generic_enum_ctor.aya:18: enum Pair[A, B]
example/test_generic_enum_ctor.aya:23: impl[A, B] Pair[A, B]
example/test_generic_enum_ctor.aya:24: pub fn pair_tag(self) -> int
example/test_generic_enum_ctor.aya:32: fn none_opt() -> Option[int]
example/test_generic_enum_ctor.aya:36: fn main() -> int
example/test_generic_enum_match.aya:2: pub enum O[T]
example/test_generic_enum_match.aya:7: fn mk() -> O[int] { return O::Some(7) }
example/test_generic_enum_match.aya:9: fn get(O[int] v) -> int
example/test_generic_enum_match.aya:16: fn main() -> int
example/test_generic_impl_multi.aya:2: pub enum Pair[A, B]
example/test_generic_impl_multi.aya:7: impl[A, B] Pair[A, B]
example/test_generic_impl_multi.aya:8: pub fn first(self) -> A
example/test_generic_impl_multi.aya:14: pub fn second(self) -> B
example/test_generic_impl_multi.aya:22: pub enum Wrap[T] { W(T) }
example/test_generic_impl_multi.aya:24: impl[T] Wrap[T]
example/test_generic_impl_multi.aya:25: pub fn get(self) -> T
example/test_generic_impl_multi.aya:32: fn mk_p() -> Pair[int, int] { return Pair::P(5) }
example/test_generic_impl_multi.aya:33: fn mk_q() -> Pair[int, int] { return Pair::Q(7) }
example/test_generic_impl_multi.aya:34: fn mk_w() -> Wrap[int] { return Wrap::W(9) }
example/test_generic_impl_multi.aya:36: fn main() -> int
example/test_generic_nested_arg.aya:6: fn main() -> int
example/test_generic_pair_map.aya:3: enum Pair[A, B]
example/test_generic_pair_map.aya:8: impl[A, B] Pair[A, B]
example/test_generic_pair_map.aya:9: pub fn map_a[C](self, Fn(A) -> C f) -> Pair[C, B]
example/test_generic_pair_map.aya:17: fn mk() -> Pair[int, String]
example/test_generic_pair_map.aya:21: fn get_p(Pair[int, String] q) -> int
example/test_generic_pair_map.aya:28: fn main() -> int
example/test_generic_struct_lit.aya:2: struct Pair[A, B]
example/test_generic_struct_lit.aya:7: struct Box[T]
example/test_generic_struct_lit.aya:11: fn main() -> int
example/test_if_expr.aya:2: fn pick(int x) -> int
example/test_if_expr.aya:7: fn pick2(bool c) -> int
example/test_if_expr.aya:15: fn classify(int x) -> int
example/test_if_expr.aya:20: fn main() -> int
example/test_import.aya:3: fn main() -> int
example/test_int_width.aya:2: fn add32(i32 a, i32 b) -> i32 { return a + b }
example/test_int_width.aya:3: fn sub32(i32 a, i32 b) -> i32 { return a - b }
example/test_int_width.aya:4: fn mul32(i32 a, i32 b) -> i32 { return a * b }
example/test_int_width.aya:5: fn add8(i8 a, i8 b) -> i8 { return a + b }
example/test_int_width.aya:6: fn div8(i8 a, i8 b) -> i8 { return a / b }
example/test_int_width.aya:7: fn add_u8(u8 a, u8 b) -> u8 { return a + b }
example/test_int_width.aya:8: fn div_u8(u8 a, u8 b) -> u8 { return a / b }
example/test_int_width.aya:9: fn mod_u8(u8 a, u8 b) -> u8 { return a % b }
example/test_int_width.aya:10: fn less_u8(u8 a, u8 b) -> bool { return a < b }
example/test_int_width.aya:11: fn less_i8(i8 a, i8 b) -> bool { return a < b }
example/test_int_width.aya:12: fn add_i16(i16 a, i16 b) -> i16 { return a + b }
example/test_int_width.aya:13: fn add_u16(u16 a, u16 b) -> u16 { return a + b }
example/test_int_width.aya:14: fn add_u32(u32 a, u32 b) -> u32 { return a + b }
example/test_int_width.aya:15: fn add_i64(i64 a, i64 b) -> i64 { return a + b }
example/test_int_width.aya:16: fn add_u64(u64 a, u64 b) -> u64 { return a + b }
example/test_int_width.aya:17: fn add_isize(isize a, isize b) -> isize { return a + b }
example/test_int_width.aya:18: fn add_usize(usize a, usize b) -> usize { return a + b }
example/test_int_width.aya:19: fn add_i128(i128 a, i128 b) -> i128 { return a + b }
example/test_int_width.aya:20: fn add_u128(u128 a, u128 b) -> u128 { return a + b }
example/test_int_width.aya:22: impl i32
example/test_int_width.aya:23: fn plus32(self, i32 other) -> i32 { return self + other }
example/test_int_width.aya:26: fn main() -> int
example/test_invariant.aya:3: fn win_only() -> int { return 1 }
example/test_invariant.aya:5: fn main() -> int
example/test_lambda_generic_method.aya:2: enum Opt[T] { Some(T), None }
example/test_lambda_generic_method.aya:4: impl[T] Opt[T]
example/test_lambda_generic_method.aya:5: pub fn map[U](self, Fn(T) -> U f) -> Opt[U]
example/test_lambda_generic_method.aya:12: pub fn filter(self, Fn(T) -> bool f) -> Opt[T]
example/test_lambda_generic_method.aya:19: pub fn and_then[U](self, Fn(T) -> Opt[U] f) -> Opt[U]
example/test_lambda_generic_method.aya:27: fn get(Opt[int] o) -> int
example/test_lambda_generic_method.aya:34: fn mk() -> Opt[int] { return Opt::Some(3) }
example/test_lambda_generic_method.aya:36: fn main() -> int
example/test_lcl_owned_arg.aya:5: fn mk() -> String
example/test_lcl_owned_arg.aya:12: fn main() -> int
example/test_literals.aya:2: fn main() -> int
example/test_macro.aya:5: fn placeholder() -> int { return 0 }
example/test_macro.aya:7: fn main() -> int { return answer() - 42 }
example/test_macro_args.aya:5: fn kept() -> int { return 7 }
example/test_macro_args.aya:8: fn placeholder() -> int { return 0 }
example/test_macro_args.aya:11: fn shown_placeholder() -> int { return 0 }
example/test_macro_args.aya:13: fn main() -> int
example/test_macro_fn.aya:3: fn main() -> int
example/test_macro_import.aya:4: fn main() -> int
example/test_macro_stmt.aya:4: fn helper() -> int
example/test_macro_stmt.aya:11: fn main() -> int
example/test_match_arms.aya:2: enum Shape { Circle(int), Square(int), Dot }
example/test_match_arms.aya:4: fn area(Shape s) -> int
example/test_match_arms.aya:11: fn classify(u8 x) -> int
example/test_match_arms.aya:19: fn pick(int x) -> int
example/test_match_arms.aya:27: fn pick_stmt(int x) -> int
example/test_match_arms.aya:34: fn main() -> int
example/test_match_destructure.aya:2: enum Opt[T]
example/test_match_destructure.aya:7: struct Point
example/test_match_destructure.aya:12: fn nested(Opt[Opt[int]] o) -> int
example/test_match_destructure.aya:20: fn destr(Point p) -> int
example/test_match_destructure.aya:26: fn destr_rename(Point p) -> int
example/test_match_destructure.aya:32: fn ranged(int n) -> int
example/test_match_destructure.aya:41: fn nested_struct(Opt[Point] o) -> int
example/test_match_destructure.aya:48: fn main() -> int
example/test_match_expr.aya:5: enum TokenType
example/test_match_expr.aya:11: impl TokenType
example/test_match_expr.aya:12: fn to_string(ref self) -> String
example/test_match_expr.aya:21: fn main() -> int
example/test_match_literals.aya:2: enum Color
example/test_match_literals.aya:8: enum Opt[T]
example/test_match_literals.aya:13: fn name(int x) -> int
example/test_match_literals.aya:22: fn kind(Color c) -> int
example/test_match_literals.aya:29: fn flag(bool b) -> int
example/test_match_literals.aya:36: fn letter(char c) -> int
example/test_match_literals.aya:43: fn opt_kind(Opt[int] o) -> int
example/test_match_literals.aya:51: fn main() -> int
example/test_memory.aya:5: fn make() -> int
example/test_memory.aya:11: fn main() -> int
example/test_memory_enum.aya:4: enum Maybe
example/test_memory_enum.aya:9: fn make() -> int
example/test_memory_enum.aya:15: fn main() -> int
example/test_memory_loop.aya:5: fn loop_allocs() -> int
example/test_memory_loop.aya:14: fn main() -> int
example/test_negative.aya:2: fn take(int x) -> int { return x }
example/test_negative.aya:4: fn main() -> int
example/test_nested_generic_type.aya:2: struct Box[T] { T v }
example/test_nested_generic_type.aya:3: struct Wrap[T] { T inner }
example/test_nested_generic_type.aya:5: struct A[T] { T v }
example/test_nested_generic_type.aya:6: struct B[T] { T v }
example/test_nested_generic_type.aya:7: struct C[T] { T v }
example/test_nested_generic_type.aya:9: struct Pair[X, Y]
example/test_nested_generic_type.aya:14: fn mk() -> Wrap[Box[int]]
example/test_nested_generic_type.aya:18: fn deep() -> A[B[C[int]]]
example/test_nested_generic_type.aya:22: fn pair() -> Pair[A[int], B[int]]
example/test_nested_generic_type.aya:26: fn main() -> int
example/test_nested_ref_mut.aya:2: struct Inner { int n }
example/test_nested_ref_mut.aya:4: fn inc_free(ref mut Inner x) { x.n = x.n + 1; }
example/test_nested_ref_mut.aya:6: struct Outer
example/test_nested_ref_mut.aya:11: impl Inner
example/test_nested_ref_mut.aya:12: pub fn inc(ref mut self) { self.n = self.n + 1; }
example/test_nested_ref_mut.aya:15: impl Outer
example/test_nested_ref_mut.aya:16: pub fn go(ref mut self)
example/test_nested_ref_mut.aya:22: fn main() -> int
example/test_never.aya:2: enum Opt[T]
example/test_never.aya:8: fn die() -> !
example/test_never.aya:14: fn die2() -> void
example/test_never.aya:19: fn pick(bool c) -> int
example/test_never.aya:23: fn ret_never(bool c) -> int
example/test_never.aya:28: fn match_never(Opt[int] o) -> int
example/test_never.aya:35: fn all_div(bool c) -> int
example/test_never.aya:39: fn assign_never(bool c) -> int
example/test_never.aya:44: fn main() -> int
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
example/test_panic.aya:3: fn main() -> int
example/test_pass.aya:6: fn pure_add(int a, int b) -> int
example/test_pass.aya:11: fn impure_len() -> int
example/test_pass.aya:16: fn main() -> int
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
example/test_ref_array.aya:2: fn sum(ref [int] a, int n) -> int
example/test_ref_array.aya:10: fn fill(ref mut [int] a, int n, int base)
example/test_ref_array.aya:16: fn qsort(ref mut [int] a, int lo, int hi)
example/test_ref_array.aya:36: fn main() -> int
example/test_ref_auto.aya:5: fn bump(ref mut int i)
example/test_ref_auto.aya:9: fn bump_by(ref mut int i, int n) -> int
example/test_ref_auto.aya:16: fn add_one(int v) -> int
example/test_ref_auto.aya:20: fn via_value(ref mut int i) -> int
example/test_ref_auto.aya:24: fn total(ref int i, ref mut int acc)
example/test_ref_auto.aya:28: fn set_hi(ref mut String s)
example/test_ref_auto.aya:32: fn main() -> int
example/test_ref_param.aya:2: struct Box2
example/test_ref_param.aya:6: fn get(ref Box2 b) -> int
example/test_ref_param.aya:10: fn main() -> int
example/test_ref_return.aya:3: struct S
example/test_ref_return.aya:7: fn pick(ref S s) -> ref S
example/test_ref_return.aya:11: fn main() -> int
example/test_ref_self.aya:2: struct Counter
example/test_ref_self.aya:6: impl Counter
example/test_ref_self.aya:7: fn get(ref self) -> int
example/test_ref_self.aya:11: fn inc(ref mut self)
example/test_ref_self.aya:14: fn into(self) -> int
example/test_ref_self.aya:19: fn main() -> int
example/test_release_opt.aya:2: fn sum_squares(int n) -> int
example/test_release_opt.aya:12: fn bump(ref mut int x) -> int
example/test_release_opt.aya:18: fn sum_ref(ref [int] a, int n) -> int
example/test_release_opt.aya:27: fn make(int n) -> [int]
example/test_release_opt.aya:34: pub fn public_double(int x) -> int
example/test_release_opt.aya:38: fn main() -> int
example/test_requires.aya:3: fn dec(int n) -> int { return n - 1 }
example/test_requires.aya:7: fn clamp100(int x) -> int { return x }
example/test_requires.aya:9: fn main() -> int
example/test_self.aya:1: fn main() -> int
example/test_self_type.aya:2: interface Maker
example/test_self_type.aya:3: fn make(ref self) -> Self;
example/test_self_type.aya:4: fn combine(ref self, Self other) -> Self;
example/test_self_type.aya:7: struct P { int x }
example/test_self_type.aya:9: impl P
example/test_self_type.aya:10: fn make(ref self) -> Self { return P { x = self.x + 1 } }
example/test_self_type.aya:11: fn combine(ref self, Self other) -> Self { return P { x = self.x + other.x } }
example/test_self_type.aya:14: impl int
example/test_self_type.aya:15: fn make(ref self) -> Self { return self + 1 }
example/test_self_type.aya:16: fn combine(ref self, Self other) -> Self { return self + other }
example/test_self_type.aya:19: struct Box[T] { T v }
example/test_self_type.aya:21: impl[T] Box[T]
example/test_self_type.aya:22: fn make(ref self) -> Self { return Box[T] { v = self.v } }
example/test_self_type.aya:23: fn combine(ref self, Self other) -> Self { return Box[T] { v = other.v } }
example/test_self_type.aya:26: fn bump[T: Maker](ref T v) -> T { return v.make() }
example/test_self_type.aya:27: fn merge[T: Maker](ref T a, ref T b) -> T { return a.combine(b) }
example/test_self_type.aya:29: fn main() -> int
example/test_static.aya:6: fn bump() -> int
example/test_static.aya:11: fn inc(ref mut int x) -> int
example/test_static.aya:20: fn main() -> int
example/test_static_array.aya:6: fn total(ref [int; 4] t) -> int
example/test_static_array.aya:14: fn main() -> int
example/test_static_struct.aya:2: struct Point
example/test_static_struct.aya:7: struct Line
example/test_static_struct.aya:12: struct Buf
example/test_static_struct.aya:23: fn shift(ref mut Point p, int dx)
example/test_static_struct.aya:27: fn sum(ref Line l) -> int
example/test_static_struct.aya:31: fn main() -> int
example/test_std.aya:3: fn main() -> int
example/test_std_extra.aya:4: fn math_checks() -> int
example/test_std_extra.aya:16: fn list_checks() -> int
example/test_std_extra.aya:30: fn maybe(int x) -> Option[int]
example/test_std_extra.aya:35: fn option_checks() -> int
example/test_std_extra.aya:43: fn main() -> int
example/test_std_io.aya:3: fn main() -> int
example/test_std_text.aya:4: fn text_checks() -> int
example/test_std_text.aya:22: fn parse_checks() -> int
example/test_std_text.aya:31: fn char_checks() -> int
example/test_std_text.aya:44: fn main() -> int
example/test_struct.aya:1: struct Point
example/test_struct.aya:6: fn main() -> int
example/test_struct2.aya:1: struct Point
example/test_struct2.aya:6: fn main() -> int
example/test_struct_fn.aya:1: struct Point
example/test_struct_fn.aya:6: fn add_points(Point a, Point b) -> Point
example/test_struct_fn.aya:11: fn main() -> int
example/test_struct_impl.aya:1: struct Point
example/test_struct_impl.aya:6: impl Point
example/test_struct_impl.aya:7: fn get_x(self) -> int
example/test_struct_impl.aya:12: fn main() -> int
example/test_tail_expr.aya:2: enum E { A(int), B }
example/test_tail_expr.aya:4: fn add(int a, int b) -> int
example/test_tail_expr.aya:8: fn pick(int x) -> int
example/test_tail_expr.aya:15: fn unbox(E e) -> int
example/test_tail_expr.aya:22: fn nested(int x) -> int
example/test_tail_expr.aya:27: pub enum Pair[A, B] { P(A), Q(B) }
example/test_tail_expr.aya:29: impl[A, B] Pair[A, B]
example/test_tail_expr.aya:30: pub fn first(self) -> A
example/test_tail_expr.aya:38: fn mk_pair() -> Pair[int, int] { return Pair::P(5) }
example/test_tail_expr.aya:41: fn unit_tail(int x)
example/test_tail_expr.aya:45: fn main() -> int
example/test_track_caller.aya:3: fn where_am_i(int __line, int __col, String __file) -> int
example/test_track_caller.aya:10: fn main() -> int
example/test_try.aya:4: fn inner(int x) -> Result[int, int]
example/test_try.aya:9: fn outer(int x) -> Result[int, int]
example/test_try.aya:15: fn take(Result[int, int] r) -> int { return r._tag }
example/test_try.aya:17: fn main() -> int
example/test_two_phase.aya:1: struct S { int v }
example/test_two_phase.aya:2: impl S
example/test_two_phase.aya:4: fn add(ref mut self, int x) { self.v = self.v + x }
example/test_two_phase.aya:6: fn main() -> int
example/test_type_alias.aya:2: fn takes_i64(i64 x) -> i64 { return x }
example/test_type_alias.aya:3: fn takes_int(int x) -> int { return x }
example/test_type_alias.aya:4: fn main() -> int
example/test_unique_struct.aya:2: struct Point
example/test_unique_struct.aya:7: fn main() -> int
example/test_unit_type.aya:2: fn nothing() -> ()
example/test_unit_type.aya:6: fn main() -> int
example/test_usage_infer.aya:5: enum E
example/test_usage_infer.aya:10: fn make() -> E { return E::A(1); }
example/test_usage_infer.aya:12: fn main() -> int
example/test_usize_index.aya:2: pub struct S
example/test_usize_index.aya:6: fn field_index(ref S s) -> int
example/test_usize_index.aya:11: fn main() -> int
example/test_utf8_string.aya:2: fn main() -> int
example/test_vis.aya:1: pub fn main() -> int
example/test_vis2.aya:6: pub fn main() -> int
example/test_zero_arg_fn.aya:2: fn time() -> int { return 7 }
example/test_zero_arg_fn.aya:4: fn main() -> int
