# SYMBOLS.md — Ayanami 符号地图

> 本文件由 scripts/gen_symbols.sh 自动生成，请勿手改。重新生成：./scripts/gen_symbols.sh
> 查询：rg "关键词" SYMBOLS.md

## Rust
src/compiler/build.rs:13: pub fn compile_file(
src/compiler/build.rs:133: fn parse_and_check(code: &str, src_path: &Path) -> Result<Program, String>
src/compiler/build.rs:147: fn resolve_dependencies(
src/compiler/build.rs:261: fn lower_to_lir(program: &Program, src_path: &Path) -> Result<crate::lir::ir::LirProgram, String>
src/compiler/build.rs:277: fn merge_dep_struct_defs(
src/compiler/build.rs:294: fn build_target_artifact(
src/compiler/build.rs:324: fn merge_symbols(
src/compiler/build.rs:363: fn emit_lcl_package(
src/compiler/build.rs:380: pub fn package_source(src_path: &str, _code: &str) -> Result<(), String>
src/compiler/build.rs:430: pub fn install_package(lcl_path: &str, target_type: Option<&str>) -> Result<(), String>
src/compiler/build.rs:475: pub fn build_source(src_path: &str, code: &str) -> Result<(), String>
src/compiler/build.rs:480: pub fn build_source_to(src_path: &str, _code: &str, out_dir: &str) -> Result<(), String>
src/compiler/build.rs:485: pub fn build_source_with_target(
src/compiler/build.rs:586: pub fn run_executable(exe_name: &str) -> Result<i32, String>
src/compiler/check.rs:4: pub fn check_hir_returns(hir: &HirProgram, src_path: &Path) -> Result<(), String>
src/compiler/check.rs:35: fn has_return_in_item(item: &HirItem) -> bool
src/compiler/check.rs:69: fn has_return_in_stmt(s: &HirStmt) -> bool
src/compiler/debug.rs:5: fn pad(n: usize) -> String
src/compiler/debug.rs:9: fn format_type(ty: &Type) -> String
src/compiler/debug.rs:41: fn format_op(op: &BinaryOp) -> &str
src/compiler/debug.rs:59: fn format_unary(op: &UnaryOp) -> &str
src/compiler/debug.rs:66: fn format_literal(lit: &Literal) -> String
src/compiler/debug.rs:77: pub fn format_program(program: &Program) -> String
src/compiler/debug.rs:86: fn write_expr(expr: &Expr, level: usize, w: &mut impl Write)
src/compiler/debug.rs:225: fn write_block(block: &Block, level: usize, w: &mut impl Write)
src/compiler/debug.rs:233: fn write_stmt(stmt: &Stmt, level: usize, w: &mut impl Write)
src/compiler/import.rs:2: pub fn find_std_dir() -> Option<std::path::PathBuf>
src/compiler/import.rs:20: pub fn resolve_import_path(
src/compiler/import.rs:84: pub fn load_config_for_file(file_path: &std::path::Path, _out_dir: &std::path::Path) -> Option<String>
src/compiler/mod.rs:1: pub mod build;
src/compiler/mod.rs:2: pub mod check;
src/compiler/mod.rs:3: pub mod debug;
src/compiler/mod.rs:4: pub mod import;
src/compiler/mod.rs:5: pub mod symdef;
src/compiler/mod.rs:21: pub struct CompiledFile
src/compiler/mod.rs:33: pub struct CompileResult
src/compiler/mod.rs:61: pub struct CompilerPipeline
src/compiler/mod.rs:78: impl CompilerPipeline
src/compiler/mod.rs:80: pub fn new(code: &str) -> Self
src/compiler/mod.rs:93: pub fn lex(&mut self) -> Result<&mut Self, String>
src/compiler/mod.rs:106: pub fn parse(&mut self) -> Result<&mut Self, String>
src/compiler/mod.rs:115: pub fn lower_hir(&mut self) -> Result<&mut Self, String>
src/compiler/mod.rs:124: pub fn check_returns(&mut self) -> Result<&mut Self, String>
src/compiler/mod.rs:132: pub fn lower_mir(&mut self) -> &mut Self
src/compiler/mod.rs:141: pub fn check_borrows(&mut self) -> Result<&mut Self, String>
src/compiler/mod.rs:154: pub fn lower_lir(&mut self) -> &mut Self
src/compiler/mod.rs:163: pub fn emit(&mut self) -> &mut Self
src/compiler/mod.rs:171: pub fn compile(&mut self) -> Result<&mut Self, String>
src/compiler/mod.rs:182: pub fn result(&self) -> CompileResult
src/compiler/mod.rs:191: pub fn hir_program(&self) -> Option<&crate::hir::ir::HirProgram>
src/compiler/mod.rs:196: pub fn mir_program(&self) -> Option<&crate::mir::ir::MirProgram>
src/compiler/mod.rs:201: pub fn ast_program(&self) -> Option<&Program>
src/compiler/mod.rs:207: pub fn compile_source(code: &str) -> Result<CompileResult, String>
src/compiler/mod.rs:215: pub fn check_source(code: &str, _out_dir: &str) -> Result<(), String>
src/compiler/symdef.rs:5: pub struct SymDef
src/compiler/symdef.rs:14: pub fn collect_defs_from_stmts(
src/compiler/symdef.rs:25: fn collect_defs_from_stmt(stmt: &Stmt, file: &str, prefix: &str, defs: &mut Vec<SymDef>)
src/compiler/symdef.rs:135: pub fn defs_to_json(defs: &[SymDef]) -> String
src/driver/mod.rs:10: fn find_llc() -> Result<(PathBuf, PathBuf), String>
src/driver/mod.rs:25: pub fn ir_to_object(llvm_ir: &str, obj_path: impl AsRef<Path>) -> Result<(), String>
src/driver/mod.rs:58: pub fn objects_to_exe(obj_paths: &[PathBuf], exe_path: impl AsRef<Path>) -> Result<(), String>
src/driver/mod.rs:63: pub fn objects_to_exe_with_flags(obj_paths: &[PathBuf], extra_flags: &[String], exe_path: impl AsRef<Path>) -> Result<(), String>
src/driver/mod.rs:76: pub fn object_to_exe(obj_path: impl AsRef<Path>, exe_path: impl AsRef<Path>) -> Result<(), String>
src/driver/mod.rs:82: fn find_runtime_c() -> Result<String, String>
src/driver/mod.rs:110: pub fn object_to_static_lib(obj_path: impl AsRef<Path>, lib_path: impl AsRef<Path>) -> Result<(), String>
src/driver/mod.rs:125: pub fn objects_to_shared_lib(obj_paths: &[PathBuf], lib_path: impl AsRef<Path>) -> Result<(), String>
src/driver/mod.rs:136: pub fn object_to_shared_lib(obj_path: impl AsRef<Path>, lib_path: impl AsRef<Path>) -> Result<(), String>
src/driver/mod.rs:153: pub fn ir_to_library(llvm_ir: &str, lib_path: impl AsRef<Path>, lib_type: &str) -> Result<(), String>
src/driver/mod.rs:173: pub fn ir_to_object_keep(llvm_ir: &str, obj_path: impl AsRef<Path>) -> Result<(), String>
src/driver/mod.rs:178: pub fn ir_to_executable(llvm_ir: &str, exe_path: impl AsRef<Path>) -> Result<(), String>
src/formatter.rs:7: const INDENT: &str = "    ";
src/formatter.rs:9: pub fn format_program(program: &Program) -> String
src/formatter.rs:21: fn write_stmt_separator(out: &mut String, stmt: &Stmt)
src/formatter.rs:31: fn indent(level: usize) -> String
src/formatter.rs:35: fn write_stmt(out: &mut String, stmt: &Stmt, level: usize)
src/formatter.rs:248: fn write_block_same_line(out: &mut String, block: &Block, level: usize)
src/formatter.rs:261: fn write_generic_params(out: &mut String, params: &[(Symbol, Option<Symbol>)])
src/formatter.rs:274: fn write_params(out: &mut String, params: &[(Symbol, Type)])
src/formatter.rs:297: fn write_return_type(out: &mut String, ty: &Type)
src/formatter.rs:306: fn write_type(ty: &Type) -> String
src/formatter.rs:342: fn write_expr(expr: &Expr) -> String
src/formatter.rs:442: fn write_literal(lit: &Literal) -> String
src/formatter.rs:462: fn write_bin_op(op: &BinaryOp) -> &str
src/formatter.rs:480: fn vis_str(vis: &Visibility) -> &str
src/formatter.rs:489: pub fn format_file(code: &str) -> Result<String, String>
src/hir/display.rs:5: pub fn display_hir_program(program: &HirProgram)
src/hir/display.rs:9: pub fn hir_program_to_string(program: &HirProgram) -> String
src/hir/display.rs:18: pub(crate) fn pad(n: usize) -> String
src/hir/display.rs:22: pub(crate) fn display_type(ty: &HirType) -> String
src/hir/display.rs:46: fn write_item(item: &HirItem, level: usize, w: &mut impl Write) -> std::fmt::Result
src/hir/display.rs:91: fn write_block(block: &HirBlock, level: usize, w: &mut impl Write) -> std::fmt::Result
src/hir/display.rs:101: pub(crate) fn write_expr(expr: &HirNodeBox, level: usize, w: &mut impl Write) -> std::fmt::Result
src/hir/display.rs:105: fn write_stmt(stmt: &HirStmt, level: usize, w: &mut impl Write) -> std::fmt::Result
src/hir/item.rs:7: pub struct HirStructField
src/hir/item.rs:13: pub struct HirStructDef
src/hir/item.rs:19: pub struct HirLocal
src/hir/item.rs:25: impl HirLocal
src/hir/item.rs:26: pub fn new(name: Symbol, ty: HirType, mutable: bool) -> Self
src/hir/item.rs:34: pub struct VtableEntry
src/hir/item.rs:42: pub struct HirInterfaceMethod
src/hir/item.rs:50: pub struct HirFn
src/hir/item.rs:63: pub enum HirItem
src/hir/item.rs:79: pub struct ImportedFnSig
src/hir/item.rs:87: pub struct HirProgram
src/hir/lower/body.rs:9: impl super::Ctx
src/hir/lower/body.rs:14: pub(super) fn collect_fns(&mut self, stmts: &[Stmt]) -> Result<(), String>
src/hir/lower/body.rs:22: pub(super) fn collect_fns_with_ns(&mut self, stmts: &[Stmt], ns_prefix: &str) -> Result<(), String>
src/hir/lower/body.rs:355: pub(super) fn build_vtables(&mut self) -> Result<(), String>
src/hir/lower/body.rs:419: pub(super) fn try_match_interface(
src/hir/lower/body.rs:478: pub(super) fn try_match_generic_interface(
src/hir/lower/body.rs:566: pub(super) fn infer_iface_generic(
src/hir/lower/body.rs:579: pub(super) fn substitute_iface_type(ty: &HirType, subst: &HashMap<Symbol, HirType>, gp_names: &[Symbol]) -> HirType
src/hir/lower/body.rs:594: pub(super) fn type_matches(a: &HirType, b: &HirType) -> bool
src/hir/lower/body.rs:607: pub(super) fn find_fn_by_sig(&self, name: Symbol, param_types: &[HirType]) -> Option<FnId>
src/hir/lower/body.rs:620: fn extract_concrete_type_name(ty: &HirType) -> Option<Symbol>
src/hir/lower/body.rs:635: pub(super) fn is_fatptr_compatible(&self, param_ty: &HirType, arg_ty: &HirType) -> bool
src/hir/lower/body.rs:673: fn check_generic_fns_for_iface(&self, type_name: &Symbol, iface_name: &Symbol) -> bool
src/hir/lower/body.rs:708: fn ensure_specialized_interface(&mut self, specialized_name: &Symbol) -> Result<(), String>
src/hir/lower/body.rs:745: fn register_generic_vtable(
src/hir/lower/body.rs:779: pub(super) fn param_compatible(&self, param_ty: &HirType, arg_ty: &HirType) -> bool
src/hir/lower/body.rs:798: pub(super) fn resolve_fn_call(&self, name: &Symbol, arg_types: &[HirType]) -> Option<FnId>
src/hir/lower/body.rs:830: pub(super) fn receiver_matches_param(receiver: &HirType, param: &HirType) -> bool
src/hir/lower/body.rs:860: pub(super) fn resolve_method(&self, receiver_type: &HirType, method_name: &Symbol, arg_types: &[HirType]) -> Option<FnId>
src/hir/lower/body.rs:882: pub(super) fn specialize_generic_call(&mut self, name: &Symbol, arg_types: &[HirType], span: &crate::span::Span) -> Result<FnId, String>
src/hir/lower/body.rs:1101: pub(super) fn lower_items(&mut self, stmts: &[Stmt]) -> Result<Vec<HirItem>, String>
src/hir/lower/body.rs:1105: pub(super) fn lower_items_with_ns(&mut self, stmts: &[Stmt], ns_prefix: &str) -> Result<Vec<HirItem>, String>
src/hir/lower/body.rs:1212: pub(super) fn collect_gp_from_type(ty: &HirType, out: &mut Vec<Symbol>)
src/hir/lower/body.rs:1243: pub(super) fn lower_fn(
src/hir/lower/body.rs:1293: pub(super) fn lower_block(&mut self, block: &Block) -> Result<HirBlock, String>
src/hir/lower/body.rs:1303: pub(super) fn lower_stmt(&mut self, stmt: &Stmt) -> Result<HirStmt, String>
src/hir/lower/body.rs:1458: pub(super) fn lower_for(
src/hir/lower/body.rs:1517: pub(super) fn lower_expr(&mut self, expr: &Expr) -> Result<HirNodeBox, String>
src/hir/lower/body.rs:2207: pub(super) fn lower_literal(&mut self, lit: &Literal) -> Result<HirNodeBox, String>
src/hir/lower/helpers.rs:17: pub(crate) fn implicit_move(expr: HirNodeBox) -> HirNodeBox
src/hir/lower/helpers.rs:29: pub(crate) fn wrap_arg_for_param(arg: HirNodeBox, param_ty: &HirType) -> HirNodeBox
src/hir/lower/helpers.rs:84: pub(crate) fn wrap_for_unique_param(expr: HirNodeBox, param_ty: &HirType) -> HirNodeBox
src/hir/lower/helpers.rs:101: pub(crate) fn hir_type_to_ast_type(ty: &HirType) -> Type
src/hir/lower/helpers.rs:129: pub(crate) fn infer_generic_from_param<'a>(param_ty: &'a Type, arg_ty: &'a HirType) -> Option<(Symbol, HirType)>
src/hir/lower/helpers.rs:177: pub(crate) fn substitute_type_in_type(ty: &Type, subst: &HashMap<Symbol, Type>) -> Type
src/hir/lower/helpers.rs:205: pub(crate) fn substitute_type_in_expr(expr: &Expr, subst: &HashMap<Symbol, Type>) -> Expr
src/hir/lower/helpers.rs:302: pub(crate) fn substitute_type_in_block(block: &Block, subst: &HashMap<Symbol, Type>) -> Block
src/hir/lower/helpers.rs:310: pub(crate) fn substitute_type_in_stmt(stmt: &Stmt, subst: &HashMap<Symbol, Type>) -> Stmt
src/hir/lower/helpers.rs:417: pub(crate) fn type_to_string_generic(ty: &Type, interfaces: &HashMap<Symbol, InterfaceReg>) -> String
src/hir/lower/helpers.rs:443: pub(crate) fn substitute_hir_type(ty: &HirType, subst: &HashMap<Symbol, HirType>) -> HirType
src/hir/lower/helpers.rs:489: pub(crate) fn sig_str_to_hir(s: &str) -> HirType
src/hir/lower/helpers.rs:512: fn is_iface_type(inner_hir: &HirType, interfaces: &HashMap<Symbol, super::InterfaceReg>) -> bool
src/hir/lower/helpers.rs:523: pub(crate) fn ast_type_to_hir(ty: &Type, interfaces: &HashMap<Symbol, InterfaceReg>) -> HirType
src/hir/lower/helpers.rs:578: pub(crate) fn extract_named(ty: &HirType) -> Option<&Symbol>
src/hir/lower/helpers.rs:586: pub(crate) fn hir_type_display(ty: &HirType) -> String
src/hir/lower/helpers.rs:611: pub(crate) fn needs_deep_copy(ty: &HirType) -> bool
src/hir/lower/helpers.rs:616: pub(crate) fn binary_op_to_fn_name(op: &BinaryOp) -> Option<&'static str>
src/hir/lower/helpers.rs:635: pub(crate) fn unary_op_to_fn_name(op: &UnaryOp) -> Option<&'static str>
src/hir/lower/helpers.rs:643: pub(crate) fn strip_ownership(ty: HirType) -> HirType
src/hir/lower/helpers.rs:651: pub(crate) fn strip_ownership_ref(ty: &HirType) -> &HirType
src/hir/lower/helpers.rs:659: pub(crate) fn expr_type(expr: &HirNodeBox) -> HirType
src/hir/lower/helpers.rs:664: pub(crate) fn is_null_literal(expr: &HirNodeBox) -> bool
src/hir/lower/helpers.rs:669: pub(crate) fn is_pointer_type_for_cmp(ty: &HirType) -> bool
src/hir/lower/mod.rs:1: pub mod body;
src/hir/lower/mod.rs:2: pub mod helpers;
src/hir/lower/mod.rs:3: pub mod to_mir;
src/hir/lower/mod.rs:14: pub(super) fn strip_generic_name(name: &Symbol) -> Symbol
src/hir/lower/mod.rs:30: pub(crate) struct FnSig
src/hir/lower/mod.rs:41: pub(crate) struct InterfaceReg
src/hir/lower/mod.rs:66: pub(crate) struct Ctx
src/hir/lower/mod.rs:99: impl Ctx
src/hir/lower/mod.rs:101: pub fn new() -> Self
src/hir/lower/mod.rs:126: pub fn push_scope(&mut self) { self.scopes.push(HashMap::new()); }
src/hir/lower/mod.rs:129: pub fn pop_scope(&mut self) { self.scopes.pop(); }
src/hir/lower/mod.rs:132: pub fn bind_var(&mut self, name: Symbol, id: VarId, ty: HirType, mutable: bool)
src/hir/lower/mod.rs:137: pub fn lookup_var(&self, name: &Symbol) -> Option<(VarId, HirType, bool)>
src/hir/lower/mod.rs:145: pub fn find_field_index(&self, struct_ty: &HirType, field: &Symbol, span: &Span) -> Result<usize, String>
src/hir/lower/mod.rs:158: fn find_field_index_by_name(&self, type_name: &Symbol, field: &Symbol, span: &Span) -> Result<usize, String>
src/hir/lower/mod.rs:176: pub fn find_field_type(&self, struct_ty: &HirType, field: &Symbol, span: &Span) -> Result<HirType, String>
src/hir/lower/mod.rs:194: fn find_field_type_by_name(&self, type_name: &Symbol, field: &Symbol, span: &Span) -> Result<HirType, String>
src/hir/lower/mod.rs:213: fn build_generic_subst(&self, type_name: &Symbol, base: &Symbol) -> HashMap<Symbol, HirType>
src/hir/lower/mod.rs:232: pub fn collected_generic_params(&self, type_name: &Symbol) -> Vec<(Symbol, Option<Symbol>)>
src/hir/lower/mod.rs:237: pub fn register_or_lookup(&mut self, name: Symbol, inferred_ty: HirType) -> (VarId, HirType, bool)
src/hir/lower/mod.rs:246: pub fn update_var_type(&mut self, var_id: VarId, new_ty: HirType)
src/hir/lower/mod.rs:261: pub fn is_enum_type(&self, type_name: &Symbol) -> bool
src/hir/lower/mod.rs:282: pub fn lower_program(program: &Program) -> Result<HirProgram, String>
src/hir/lower/to_mir.rs:11: impl HirNode for SVar
src/hir/lower/to_mir.rs:12: fn clone_node(&self) -> Box<dyn HirNode> { Box::new(self.clone()) }
src/hir/lower/to_mir.rs:13: fn lower_to_mir(&self, moved: &HashSet<VarId>) -> MirNodeBox
src/hir/lower/to_mir.rs:16: fn display(&self, level: usize, w: &mut dyn std::fmt::Write) -> std::fmt::Result
src/hir/lower/to_mir.rs:19: fn expr_type(&self) -> HirType { self.ty.clone() }
src/hir/lower/to_mir.rs:20: fn as_local(&self) -> Option<VarId> { Some(self.var) }
src/hir/lower/to_mir.rs:21: fn collect_var_ids(&self, vars: &mut HashSet<VarId>) { vars.insert(self.var); }
src/hir/lower/to_mir.rs:24: impl HirNode for SConst
src/hir/lower/to_mir.rs:25: fn clone_node(&self) -> Box<dyn HirNode> { Box::new(self.clone()) }
src/hir/lower/to_mir.rs:26: fn lower_to_mir(&self, _moved: &HashSet<VarId>) -> MirNodeBox
src/hir/lower/to_mir.rs:29: fn display(&self, level: usize, w: &mut dyn std::fmt::Write) -> std::fmt::Result
src/hir/lower/to_mir.rs:39: fn expr_type(&self) -> HirType { self.ty.clone() }
src/hir/lower/to_mir.rs:40: fn as_const(&self) -> Option<&HirLiteral> { Some(&self.val) }
src/hir/lower/to_mir.rs:43: impl HirNode for SBin
src/hir/lower/to_mir.rs:44: fn clone_node(&self) -> Box<dyn HirNode> { Box::new(self.clone()) }
src/hir/lower/to_mir.rs:45: fn lower_to_mir(&self, moved: &HashSet<VarId>) -> MirNodeBox
src/hir/lower/to_mir.rs:53: fn display(&self, level: usize, w: &mut dyn std::fmt::Write) -> std::fmt::Result
src/hir/lower/to_mir.rs:61: fn expr_type(&self) -> HirType { self.ty.clone() }
src/hir/lower/to_mir.rs:64: impl HirNode for SUn
src/hir/lower/to_mir.rs:65: fn clone_node(&self) -> Box<dyn HirNode> { Box::new(self.clone()) }
src/hir/lower/to_mir.rs:66: fn lower_to_mir(&self, moved: &HashSet<VarId>) -> MirNodeBox
src/hir/lower/to_mir.rs:69: fn display(&self, level: usize, w: &mut dyn std::fmt::Write) -> std::fmt::Result
src/hir/lower/to_mir.rs:74: fn expr_type(&self) -> HirType { self.ty.clone() }
src/hir/lower/to_mir.rs:77: impl HirNode for SCall
src/hir/lower/to_mir.rs:78: fn clone_node(&self) -> Box<dyn HirNode> { Box::new(self.clone()) }
src/hir/lower/to_mir.rs:79: fn lower_to_mir(&self, moved: &HashSet<VarId>) -> MirNodeBox
src/hir/lower/to_mir.rs:86: fn display(&self, level: usize, w: &mut dyn std::fmt::Write) -> std::fmt::Result
src/hir/lower/to_mir.rs:91: fn expr_type(&self) -> HirType { self.ty.clone() }
src/hir/lower/to_mir.rs:94: impl HirNode for SMove
src/hir/lower/to_mir.rs:95: fn clone_node(&self) -> Box<dyn HirNode> { Box::new(self.clone()) }
src/hir/lower/to_mir.rs:96: fn lower_to_mir(&self, moved: &HashSet<VarId>) -> MirNodeBox
src/hir/lower/to_mir.rs:99: fn display(&self, level: usize, w: &mut dyn std::fmt::Write) -> std::fmt::Result
src/hir/lower/to_mir.rs:104: fn expr_type(&self) -> HirType { self.ty.clone() }
src/hir/lower/to_mir.rs:105: fn is_move_or_clone(&self) -> bool { true }
src/hir/lower/to_mir.rs:106: fn as_move(&self) -> Option<&HirNodeBox> { Some(&self.expr) }
src/hir/lower/to_mir.rs:107: fn record_moves(&self, moved: &mut HashSet<VarId>)
src/hir/lower/to_mir.rs:113: impl HirNode for SClone
src/hir/lower/to_mir.rs:114: fn clone_node(&self) -> Box<dyn HirNode> { Box::new(self.clone()) }
src/hir/lower/to_mir.rs:115: fn lower_to_mir(&self, moved: &HashSet<VarId>) -> MirNodeBox
src/hir/lower/to_mir.rs:118: fn display(&self, level: usize, w: &mut dyn std::fmt::Write) -> std::fmt::Result
src/hir/lower/to_mir.rs:123: fn expr_type(&self) -> HirType { self.ty.clone() }
src/hir/lower/to_mir.rs:124: fn is_move_or_clone(&self) -> bool { true }
src/hir/lower/to_mir.rs:125: fn as_clone(&self) -> Option<&HirNodeBox> { Some(&self.expr) }
src/hir/lower/to_mir.rs:128: impl HirNode for SToUnique
src/hir/lower/to_mir.rs:129: fn clone_node(&self) -> Box<dyn HirNode> { Box::new(self.clone()) }
src/hir/lower/to_mir.rs:130: fn lower_to_mir(&self, moved: &HashSet<VarId>) -> MirNodeBox
src/hir/lower/to_mir.rs:133: fn display(&self, level: usize, w: &mut dyn std::fmt::Write) -> std::fmt::Result
src/hir/lower/to_mir.rs:138: fn expr_type(&self) -> HirType { self.ty.clone() }
src/hir/lower/to_mir.rs:141: impl HirNode for SToShared
src/hir/lower/to_mir.rs:142: fn clone_node(&self) -> Box<dyn HirNode> { Box::new(self.clone()) }
src/hir/lower/to_mir.rs:143: fn lower_to_mir(&self, moved: &HashSet<VarId>) -> MirNodeBox
src/hir/lower/to_mir.rs:146: fn display(&self, level: usize, w: &mut dyn std::fmt::Write) -> std::fmt::Result
src/hir/lower/to_mir.rs:151: fn expr_type(&self) -> HirType { self.ty.clone() }
src/hir/lower/to_mir.rs:154: impl HirNode for SToWeak
src/hir/lower/to_mir.rs:155: fn clone_node(&self) -> Box<dyn HirNode> { Box::new(self.clone()) }
src/hir/lower/to_mir.rs:156: fn lower_to_mir(&self, moved: &HashSet<VarId>) -> MirNodeBox
src/hir/lower/to_mir.rs:159: fn display(&self, level: usize, w: &mut dyn std::fmt::Write) -> std::fmt::Result
src/hir/lower/to_mir.rs:164: fn expr_type(&self) -> HirType { self.ty.clone() }
src/hir/lower/to_mir.rs:167: impl HirNode for SField
src/hir/lower/to_mir.rs:168: fn clone_node(&self) -> Box<dyn HirNode> { Box::new(self.clone()) }
src/hir/lower/to_mir.rs:169: fn lower_to_mir(&self, moved: &HashSet<VarId>) -> MirNodeBox
src/hir/lower/to_mir.rs:177: fn display(&self, level: usize, w: &mut dyn std::fmt::Write) -> std::fmt::Result
src/hir/lower/to_mir.rs:182: fn expr_type(&self) -> HirType { self.ty.clone() }
src/hir/lower/to_mir.rs:185: impl HirNode for SStruct
src/hir/lower/to_mir.rs:186: fn clone_node(&self) -> Box<dyn HirNode> { Box::new(self.clone()) }
src/hir/lower/to_mir.rs:187: fn lower_to_mir(&self, moved: &HashSet<VarId>) -> MirNodeBox
src/hir/lower/to_mir.rs:194: fn display(&self, level: usize, w: &mut dyn std::fmt::Write) -> std::fmt::Result
src/hir/lower/to_mir.rs:202: fn expr_type(&self) -> HirType { self.ty.clone() }
src/hir/lower/to_mir.rs:205: impl HirNode for SArrLit
src/hir/lower/to_mir.rs:206: fn clone_node(&self) -> Box<dyn HirNode> { Box::new(self.clone()) }
src/hir/lower/to_mir.rs:207: fn lower_to_mir(&self, moved: &HashSet<VarId>) -> MirNodeBox
src/hir/lower/to_mir.rs:213: fn display(&self, level: usize, w: &mut dyn std::fmt::Write) -> std::fmt::Result
src/hir/lower/to_mir.rs:218: fn expr_type(&self) -> HirType { self.ty.clone() }
src/hir/lower/to_mir.rs:221: impl HirNode for SArrSz
src/hir/lower/to_mir.rs:222: fn clone_node(&self) -> Box<dyn HirNode> { Box::new(self.clone()) }
src/hir/lower/to_mir.rs:223: fn lower_to_mir(&self, moved: &HashSet<VarId>) -> MirNodeBox
src/hir/lower/to_mir.rs:230: fn display(&self, level: usize, w: &mut dyn std::fmt::Write) -> std::fmt::Result
src/hir/lower/to_mir.rs:236: fn expr_type(&self) -> HirType { self.ty.clone() }
src/hir/lower/to_mir.rs:239: impl HirNode for SRef
src/hir/lower/to_mir.rs:240: fn clone_node(&self) -> Box<dyn HirNode> { Box::new(self.clone()) }
src/hir/lower/to_mir.rs:241: fn lower_to_mir(&self, moved: &HashSet<VarId>) -> MirNodeBox
src/hir/lower/to_mir.rs:244: fn display(&self, level: usize, w: &mut dyn std::fmt::Write) -> std::fmt::Result
src/hir/lower/to_mir.rs:250: fn expr_type(&self) -> HirType { self.ty.clone() }
src/hir/lower/to_mir.rs:253: impl HirNode for SIdx
src/hir/lower/to_mir.rs:254: fn clone_node(&self) -> Box<dyn HirNode> { Box::new(self.clone()) }
src/hir/lower/to_mir.rs:255: fn lower_to_mir(&self, moved: &HashSet<VarId>) -> MirNodeBox
src/hir/lower/to_mir.rs:262: fn display(&self, level: usize, w: &mut dyn std::fmt::Write) -> std::fmt::Result
src/hir/lower/to_mir.rs:270: fn expr_type(&self) -> HirType { self.ty.clone() }
src/hir/lower/to_mir.rs:273: impl HirNode for SAsm
src/hir/lower/to_mir.rs:274: fn clone_node(&self) -> Box<dyn HirNode> { Box::new(self.clone()) }
src/hir/lower/to_mir.rs:275: fn lower_to_mir(&self, moved: &HashSet<VarId>) -> MirNodeBox
src/hir/lower/to_mir.rs:283: fn display(&self, level: usize, w: &mut dyn std::fmt::Write) -> std::fmt::Result
src/hir/lower/to_mir.rs:295: fn expr_type(&self) -> HirType { self.ty.clone() }
src/hir/lower/to_mir.rs:298: impl HirNode for SVCall
src/hir/lower/to_mir.rs:299: fn clone_node(&self) -> Box<dyn HirNode> { Box::new(self.clone()) }
src/hir/lower/to_mir.rs:300: fn lower_to_mir(&self, moved: &HashSet<VarId>) -> MirNodeBox
src/hir/lower/to_mir.rs:309: fn display(&self, level: usize, w: &mut dyn std::fmt::Write) -> std::fmt::Result
src/hir/lower/to_mir.rs:316: fn expr_type(&self) -> HirType { self.ty.clone() }
src/hir/lower/to_mir.rs:319: impl HirNode for SMFP
src/hir/lower/to_mir.rs:320: fn clone_node(&self) -> Box<dyn HirNode> { Box::new(self.clone()) }
src/hir/lower/to_mir.rs:321: fn lower_to_mir(&self, moved: &HashSet<VarId>) -> MirNodeBox
src/hir/lower/to_mir.rs:329: fn display(&self, level: usize, w: &mut dyn std::fmt::Write) -> std::fmt::Result
src/hir/lower/to_mir.rs:334: fn expr_type(&self) -> HirType { self.ty.clone() }
src/hir/lower/to_mir.rs:337: impl HirNode for SFnPtr
src/hir/lower/to_mir.rs:338: fn clone_node(&self) -> Box<dyn HirNode> { Box::new(self.clone()) }
src/hir/lower/to_mir.rs:339: fn lower_to_mir(&self, _moved: &HashSet<VarId>) -> MirNodeBox
src/hir/lower/to_mir.rs:342: fn display(&self, level: usize, w: &mut dyn std::fmt::Write) -> std::fmt::Result
src/hir/lower/to_mir.rs:345: fn expr_type(&self) -> HirType { self.ty.clone() }
src/hir/lower/to_mir.rs:348: impl HirNode for SCallP
src/hir/lower/to_mir.rs:349: fn clone_node(&self) -> Box<dyn HirNode> { Box::new(self.clone()) }
src/hir/lower/to_mir.rs:350: fn lower_to_mir(&self, moved: &HashSet<VarId>) -> MirNodeBox
src/hir/lower/to_mir.rs:357: fn display(&self, level: usize, w: &mut dyn std::fmt::Write) -> std::fmt::Result
src/hir/lower/to_mir.rs:360: fn expr_type(&self) -> HirType { self.ty.clone() }
src/hir/lower/to_mir.rs:363: impl HirNode for SEnumC
src/hir/lower/to_mir.rs:364: fn clone_node(&self) -> Box<dyn HirNode> { Box::new(self.clone()) }
src/hir/lower/to_mir.rs:365: fn lower_to_mir(&self, moved: &HashSet<VarId>) -> MirNodeBox
src/hir/lower/to_mir.rs:374: fn display(&self, level: usize, w: &mut dyn std::fmt::Write) -> std::fmt::Result
src/hir/lower/to_mir.rs:377: fn expr_type(&self) -> HirType { self.ty.clone() }
src/hir/lower/to_mir.rs:380: impl HirNode for SEnumM
src/hir/lower/to_mir.rs:381: fn clone_node(&self) -> Box<dyn HirNode> { Box::new(self.clone()) }
src/hir/lower/to_mir.rs:382: fn lower_to_mir(&self, moved: &HashSet<VarId>) -> MirNodeBox
src/hir/lower/to_mir.rs:389: fn display(&self, level: usize, w: &mut dyn std::fmt::Write) -> std::fmt::Result
src/hir/lower/to_mir.rs:395: fn expr_type(&self) -> HirType { self.ty.clone() }
src/hir/mod.rs:1: pub mod ir;
src/hir/mod.rs:2: pub mod ty;
src/hir/mod.rs:3: pub mod node;
src/hir/mod.rs:4: pub mod stmt;
src/hir/mod.rs:5: pub mod item;
src/hir/mod.rs:6: pub mod lower;
src/hir/mod.rs:7: pub mod display;
src/hir/mod.rs:25: pub struct $name
src/hir/node.rs:8: pub trait HirNode: std::fmt::Debug
src/hir/node.rs:9: fn clone_node(&self) -> Box<dyn HirNode>;
src/hir/node.rs:10: fn lower_to_mir(&self, moved: &HashSet<VarId>) -> MirNodeBox;
src/hir/node.rs:11: fn display(&self, level: usize, w: &mut dyn FmtWrite) -> std::fmt::Result;
src/hir/node.rs:12: fn expr_type(&self) -> HirType;
src/hir/node.rs:15: fn as_local(&self) -> Option<VarId> { None }
src/hir/node.rs:16: fn is_move_or_clone(&self) -> bool { false }
src/hir/node.rs:17: fn collect_var_ids(&self, vars: &mut HashSet<VarId>) {}
src/hir/node.rs:18: fn record_moves(&self, moved: &mut HashSet<VarId>) {}
src/hir/node.rs:19: fn as_const(&self) -> Option<&HirLiteral> { None }
src/hir/node.rs:20: fn as_move(&self) -> Option<&HirNodeBox> { None }
src/hir/node.rs:21: fn as_clone(&self) -> Option<&HirNodeBox> { None }
src/hir/node.rs:26: pub struct HirNodeBox(pub Box<dyn HirNode>);
src/hir/node.rs:27: impl Clone for HirNodeBox
src/hir/node.rs:28: fn clone(&self) -> Self { HirNodeBox(self.0.clone_node()) }
src/hir/node.rs:30: impl std::ops::Deref for HirNodeBox
src/hir/node.rs:31: type Target = dyn HirNode;
src/hir/node.rs:32: fn deref(&self) -> &Self::Target { &*self.0 }
src/hir/node.rs:34: impl From<HirNodeBox> for Box<dyn HirNode>
src/hir/node.rs:35: fn from(b: HirNodeBox) -> Self { b.0 }
src/hir/node.rs:44: fn from(v: $ty) -> Self { HirNodeBox(Box::new(v) as Box<dyn HirNode>) }
src/hir/stmt.rs:4: pub enum HirStmt
src/hir/stmt.rs:41: pub struct HirBlock
src/hir/stmt.rs:45: impl HirBlock
src/hir/stmt.rs:46: pub fn new(stmts: Vec<HirStmt>) -> Self
src/hir/ty.rs:7: pub struct VarId(pub usize);
src/hir/ty.rs:12: pub struct FnId(pub usize);
src/hir/ty.rs:22: pub enum HirType
src/hir/ty.rs:45: pub enum HirLiteral
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
src/lexer/keyword.rs:47: impl fmt::Display for Keyword
src/lexer/keyword.rs:48: fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result
src/lexer/lexer.rs:6: pub struct Lexer<'a>
src/lexer/lexer.rs:15: impl<'a> Lexer<'a>
src/lexer/lexer.rs:16: pub fn new(src: &'a str) -> Self
src/lexer/lexer.rs:27: fn peek(&self) -> Option<char>
src/lexer/lexer.rs:30: fn peek_next(&self) -> Option<char>
src/lexer/lexer.rs:34: fn bump(&mut self) -> Option<char>
src/lexer/lexer.rs:47: fn skip_whitespace_and_comments(&mut self)
src/lexer/lexer.rs:75: fn is_ident_start(c: char) -> bool
src/lexer/lexer.rs:78: fn is_ident_continue(c: char) -> bool
src/lexer/lexer.rs:82: fn read_identifier_or_keyword(&mut self) -> (String, usize, usize, usize)
src/lexer/lexer.rs:102: fn read_number(&mut self) -> (String, bool, usize, usize, usize)
src/lexer/lexer.rs:132: fn read_char_literal(&mut self) -> (String, usize, usize, usize)
src/lexer/lexer.rs:159: fn read_string_literal(&mut self) -> (String, usize, usize, usize)
src/lexer/lexer.rs:185: pub fn next_token(&mut self) -> Token
src/lexer/lexer.rs:364: impl<'a> Lexer<'a>
src/lexer/lexer.rs:366: pub fn tokenize_all(&mut self) -> Vec<Token>
src/lexer/lexer.rs:381: mod tests
src/lexer/lexer.rs:386: fn smoke()
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
src/lib.rs:19: pub mod driver;
src/lib.rs:20: pub mod formatter;
src/lib.rs:21: pub mod hir;
src/lib.rs:22: pub mod intern;
src/lib.rs:23: pub mod lexer;
src/lib.rs:24: pub mod lir;
src/lib.rs:25: pub mod mir;
src/lib.rs:26: pub mod package;
src/lib.rs:27: pub mod parser;
src/lib.rs:28: pub mod span;
src/lib.rs:31: mod test_parse
src/lib.rs:33: fn test_struct_literal_parse()
src/lir/display.rs:5: pub fn lir_program_to_string(prog: &LirProgram) -> String
src/lir/display.rs:19: fn write_fn(f: &LirFn, w: &mut impl Write) -> std::fmt::Result
src/lir/display.rs:34: fn write_inst(inst: &LirNodeBox, w: &mut impl Write) -> std::fmt::Result
src/lir/emit.rs:19: pub fn emit_program(prog: &LirProgram) -> String
src/lir/emit.rs:29: struct Emitter<'a>
src/lir/emit.rs:37: impl<'a> Emitter<'a>
src/lir/emit.rs:38: fn new(prog: &'a LirProgram) -> Self
src/lir/emit.rs:48: fn finish(self) -> String
src/lir/emit.rs:52: fn wln(&mut self, s: &str)
src/lir/emit.rs:60: fn wln_fmt(&mut self, fmt: std::fmt::Arguments<'_>)
src/lir/emit.rs:68: fn llvm_type(&self, ty: &HirType) -> String
src/lir/emit.rs:100: fn emit(&mut self)
src/lir/emit.rs:145: fn emit_vtable_globals(&mut self)
src/lir/emit.rs:179: fn fn_needs_vtable_wrapper(&self, fn_id: FnId) -> bool
src/lir/emit.rs:191: fn fn_ret_type(&self, fn_id: FnId) -> String
src/lir/emit.rs:198: fn fn_first_param_unwrapped(&self, fn_id: FnId) -> String
src/lir/emit.rs:214: fn fn_extra_params(&self, fn_id: FnId) -> (Vec<String>, Vec<String>)
src/lir/emit.rs:228: fn emit_vtable_wrappers(&mut self)
src/lir/emit.rs:278: fn emit_struct_defs(&mut self)
src/lir/emit.rs:294: fn struct_llvm_name(&self, name: &Symbol) -> Option<String>
src/lir/emit.rs:310: fn emit_string_globals(&mut self)
src/lir/emit.rs:330: fn emit_fn(&mut self, f: &LirFn)
src/lir/emit.rs:369: fn emit_inst(&mut self, inst: &LirNodeBox)
src/lir/emit.rs:386: fn tmp(&mut self) -> u64
src/lir/emit.rs:392: fn value_ref(&self, val: &LirValue, expected_ty: &HirType) -> String
src/lir/emit.rs:408: fn escape_llvm_string(s: &str) -> String
src/lir/ir.rs:12: pub struct IrNode
src/lir/ir.rs:18: pub enum IrValue
src/lir/ir.rs:27: impl IrNode
src/lir/ir.rs:28: pub fn new(kind: &str) -> Self
src/lir/ir.rs:31: pub fn set(&mut self, name: &str, val: IrValue) -> &mut Self
src/lir/ir.rs:35: pub fn get(&self, name: &str) -> Option<&IrValue>
src/lir/ir.rs:41: pub enum ConvKind
src/lir/ir.rs:48: pub enum LirValue
src/lir/ir.rs:60: pub struct LirEmitCtx<'a>
src/lir/ir.rs:66: impl LirEmitCtx<'_>
src/lir/ir.rs:67: pub fn llvm_type(&self, ty: &HirType) -> String
src/lir/ir.rs:95: pub fn value_ref(&self, val: &LirValue, expected_ty: &HirType) -> String
src/lir/ir.rs:104: pub fn tmp(&mut self) -> u64
src/lir/ir.rs:110: pub fn struct_llvm_name(&self, name: &Symbol) -> Option<String>
src/lir/ir.rs:130: pub trait LirNode: std::fmt::Debug
src/lir/ir.rs:131: fn clone_node(&self) -> Box<dyn LirNode>;
src/lir/ir.rs:132: fn kind(&self) -> &'static str;
src/lir/ir.rs:133: fn as_any(&self) -> &dyn std::any::Any;
src/lir/ir.rs:134: fn emit(&self, ctx: &mut LirEmitCtx) -> Vec<String>;
src/lir/ir.rs:135: fn display(&self, f: &mut dyn Write) -> std::fmt::Result;
src/lir/ir.rs:136: fn serialize(&self, buf: &mut Vec<u8>);
src/lir/ir.rs:144: pub struct LirNodeBox(pub Box<dyn LirNode>);
src/lir/ir.rs:146: impl Clone for LirNodeBox
src/lir/ir.rs:147: fn clone(&self) -> Self
src/lir/ir.rs:152: impl std::ops::Deref for LirNodeBox
src/lir/ir.rs:153: type Target = dyn LirNode;
src/lir/ir.rs:154: fn deref(&self) -> &Self::Target
src/lir/ir.rs:159: impl From<SLirCustom> for LirNodeBox
src/lir/ir.rs:160: fn from(v: SLirCustom) -> Self { LirNodeBox(Box::new(v)) }
src/lir/ir.rs:170: pub struct $name { $(pub $field: $ty),* }
src/lir/ir.rs:214: fn from(v: $ty) -> Self { LirNodeBox(Box::new(v) as Box<dyn LirNode>) }
src/lir/ir.rs:233: pub fn sanitize_name(name: &str) -> String
src/lir/ir.rs:238: pub(crate) fn lit_to_string(lit: &HirLiteral, expected_ty: &HirType) -> String
src/lir/ir.rs:253: pub(crate) fn llvm_type_size(ty: &HirType) -> &'static str
src/lir/ir.rs:269: pub(crate) fn struct_llvm_size(ty: &HirType, struct_defs: &std::collections::HashMap<Symbol, Vec<(Symbol, HirType)>>) -> String
src/lir/ir.rs:286: fn needs_heap_ops(ty: &HirType) -> bool
src/lir/ir.rs:299: fn is_pointer_type(ty: &HirType) -> bool
src/lir/ir.rs:311: impl LirNode for SLirAlloca
src/lir/ir.rs:312: fn clone_node(&self) -> Box<dyn LirNode> { Box::new(self.clone()) }
src/lir/ir.rs:313: fn kind(&self) -> &'static str { "Alloca" }
src/lir/ir.rs:314: fn as_any(&self) -> &dyn std::any::Any { self }
src/lir/ir.rs:315: fn emit(&self, ctx: &mut LirEmitCtx) -> Vec<String>
src/lir/ir.rs:319: fn display(&self, f: &mut dyn Write) -> std::fmt::Result
src/lir/ir.rs:322: fn serialize(&self, buf: &mut Vec<u8>)
src/lir/ir.rs:329: impl LirNode for SLirStore
src/lir/ir.rs:330: fn clone_node(&self) -> Box<dyn LirNode> { Box::new(self.clone()) }
src/lir/ir.rs:331: fn kind(&self) -> &'static str { "Store" }
src/lir/ir.rs:332: fn as_any(&self) -> &dyn std::any::Any { self }
src/lir/ir.rs:333: fn emit(&self, ctx: &mut LirEmitCtx) -> Vec<String>
src/lir/ir.rs:349: fn display(&self, f: &mut dyn Write) -> std::fmt::Result
src/lir/ir.rs:352: fn serialize(&self, buf: &mut Vec<u8>)
src/lir/ir.rs:360: impl LirNode for SLirLoad
src/lir/ir.rs:361: fn clone_node(&self) -> Box<dyn LirNode> { Box::new(self.clone()) }
src/lir/ir.rs:362: fn kind(&self) -> &'static str { "Load" }
src/lir/ir.rs:363: fn as_any(&self) -> &dyn std::any::Any { self }
src/lir/ir.rs:364: fn emit(&self, ctx: &mut LirEmitCtx) -> Vec<String>
src/lir/ir.rs:368: fn display(&self, f: &mut dyn Write) -> std::fmt::Result
src/lir/ir.rs:371: fn serialize(&self, buf: &mut Vec<u8>)
src/lir/ir.rs:379: impl LirNode for SLirBinOp
src/lir/ir.rs:380: fn clone_node(&self) -> Box<dyn LirNode> { Box::new(self.clone()) }
src/lir/ir.rs:381: fn kind(&self) -> &'static str { "BinOp" }
src/lir/ir.rs:382: fn as_any(&self) -> &dyn std::any::Any { self }
src/lir/ir.rs:383: fn emit(&self, ctx: &mut LirEmitCtx) -> Vec<String>
src/lir/ir.rs:449: fn display(&self, f: &mut dyn Write) -> std::fmt::Result
src/lir/ir.rs:452: fn serialize(&self, buf: &mut Vec<u8>)
src/lir/ir.rs:463: impl LirNode for SLirUnaryOp
src/lir/ir.rs:464: fn clone_node(&self) -> Box<dyn LirNode> { Box::new(self.clone()) }
src/lir/ir.rs:465: fn kind(&self) -> &'static str { "UnaryOp" }
src/lir/ir.rs:466: fn as_any(&self) -> &dyn std::any::Any { self }
src/lir/ir.rs:467: fn emit(&self, ctx: &mut LirEmitCtx) -> Vec<String>
src/lir/ir.rs:476: fn display(&self, f: &mut dyn Write) -> std::fmt::Result
src/lir/ir.rs:479: fn serialize(&self, buf: &mut Vec<u8>)
src/lir/ir.rs:488: impl LirNode for SLirCall
src/lir/ir.rs:489: fn clone_node(&self) -> Box<dyn LirNode> { Box::new(self.clone()) }
src/lir/ir.rs:490: fn kind(&self) -> &'static str { "Call" }
src/lir/ir.rs:491: fn as_any(&self) -> &dyn std::any::Any { self }
src/lir/ir.rs:492: fn emit(&self, ctx: &mut LirEmitCtx) -> Vec<String>
src/lir/ir.rs:508: fn display(&self, f: &mut dyn Write) -> std::fmt::Result
src/lir/ir.rs:513: fn serialize(&self, buf: &mut Vec<u8>)
src/lir/ir.rs:523: impl LirNode for SLirCallPtr
src/lir/ir.rs:524: fn clone_node(&self) -> Box<dyn LirNode> { Box::new(self.clone()) }
src/lir/ir.rs:525: fn kind(&self) -> &'static str { "CallPtr" }
src/lir/ir.rs:526: fn as_any(&self) -> &dyn std::any::Any { self }
src/lir/ir.rs:527: fn emit(&self, ctx: &mut LirEmitCtx) -> Vec<String>
src/lir/ir.rs:538: fn display(&self, f: &mut dyn Write) -> std::fmt::Result
src/lir/ir.rs:542: fn serialize(&self, buf: &mut Vec<u8>)
src/lir/ir.rs:552: impl LirNode for SLirFnAddr
src/lir/ir.rs:553: fn clone_node(&self) -> Box<dyn LirNode> { Box::new(self.clone()) }
src/lir/ir.rs:554: fn kind(&self) -> &'static str { "FnAddr" }
src/lir/ir.rs:555: fn as_any(&self) -> &dyn std::any::Any { self }
src/lir/ir.rs:556: fn emit(&self, ctx: &mut LirEmitCtx) -> Vec<String>
src/lir/ir.rs:560: fn display(&self, f: &mut dyn Write) -> std::fmt::Result
src/lir/ir.rs:563: fn serialize(&self, buf: &mut Vec<u8>)
src/lir/ir.rs:570: impl LirNode for SLirStrGlobal
src/lir/ir.rs:571: fn clone_node(&self) -> Box<dyn LirNode> { Box::new(self.clone()) }
src/lir/ir.rs:572: fn kind(&self) -> &'static str { "StrGlobal" }
src/lir/ir.rs:573: fn as_any(&self) -> &dyn std::any::Any { self }
src/lir/ir.rs:574: fn emit(&self, _ctx: &mut LirEmitCtx) -> Vec<String>
src/lir/ir.rs:577: fn display(&self, f: &mut dyn Write) -> std::fmt::Result
src/lir/ir.rs:580: fn serialize(&self, buf: &mut Vec<u8>)
src/lir/ir.rs:587: impl LirNode for SLirConv
src/lir/ir.rs:588: fn clone_node(&self) -> Box<dyn LirNode> { Box::new(self.clone()) }
src/lir/ir.rs:589: fn kind(&self) -> &'static str { "Conv" }
src/lir/ir.rs:590: fn as_any(&self) -> &dyn std::any::Any { self }
src/lir/ir.rs:591: fn emit(&self, ctx: &mut LirEmitCtx) -> Vec<String>
src/lir/ir.rs:660: fn display(&self, f: &mut dyn Write) -> std::fmt::Result
src/lir/ir.rs:663: fn serialize(&self, buf: &mut Vec<u8>)
src/lir/ir.rs:670: impl LirNode for SLirDropValue
src/lir/ir.rs:671: fn clone_node(&self) -> Box<dyn LirNode> { Box::new(self.clone()) }
src/lir/ir.rs:672: fn kind(&self) -> &'static str { "DropValue" }
src/lir/ir.rs:673: fn as_any(&self) -> &dyn std::any::Any { self }
src/lir/ir.rs:674: fn emit(&self, ctx: &mut LirEmitCtx) -> Vec<String>
src/lir/ir.rs:684: fn display(&self, f: &mut dyn Write) -> std::fmt::Result
src/lir/ir.rs:687: fn serialize(&self, buf: &mut Vec<u8>)
src/lir/ir.rs:694: impl LirNode for SLirRetainValue
src/lir/ir.rs:695: fn clone_node(&self) -> Box<dyn LirNode> { Box::new(self.clone()) }
src/lir/ir.rs:696: fn kind(&self) -> &'static str { "RetainValue" }
src/lir/ir.rs:697: fn as_any(&self) -> &dyn std::any::Any { self }
src/lir/ir.rs:698: fn emit(&self, ctx: &mut LirEmitCtx) -> Vec<String>
src/lir/ir.rs:708: fn display(&self, f: &mut dyn Write) -> std::fmt::Result
src/lir/ir.rs:711: fn serialize(&self, buf: &mut Vec<u8>)
src/lir/ir.rs:718: impl LirNode for SLirReleaseValue
src/lir/ir.rs:719: fn clone_node(&self) -> Box<dyn LirNode> { Box::new(self.clone()) }
src/lir/ir.rs:720: fn kind(&self) -> &'static str { "ReleaseValue" }
src/lir/ir.rs:721: fn as_any(&self) -> &dyn std::any::Any { self }
src/lir/ir.rs:722: fn emit(&self, ctx: &mut LirEmitCtx) -> Vec<String>
src/lir/ir.rs:732: fn display(&self, f: &mut dyn Write) -> std::fmt::Result
src/lir/ir.rs:735: fn serialize(&self, buf: &mut Vec<u8>)
src/lir/ir.rs:742: impl LirNode for SLirBr
src/lir/ir.rs:743: fn clone_node(&self) -> Box<dyn LirNode> { Box::new(self.clone()) }
src/lir/ir.rs:744: fn kind(&self) -> &'static str { "Br" }
src/lir/ir.rs:745: fn as_any(&self) -> &dyn std::any::Any { self }
src/lir/ir.rs:746: fn emit(&self, _ctx: &mut LirEmitCtx) -> Vec<String>
src/lir/ir.rs:749: fn display(&self, f: &mut dyn Write) -> std::fmt::Result
src/lir/ir.rs:752: fn serialize(&self, buf: &mut Vec<u8>)
src/lir/ir.rs:758: impl LirNode for SLirBrCond
src/lir/ir.rs:759: fn clone_node(&self) -> Box<dyn LirNode> { Box::new(self.clone()) }
src/lir/ir.rs:760: fn kind(&self) -> &'static str { "BrCond" }
src/lir/ir.rs:761: fn as_any(&self) -> &dyn std::any::Any { self }
src/lir/ir.rs:762: fn emit(&self, ctx: &mut LirEmitCtx) -> Vec<String>
src/lir/ir.rs:766: fn display(&self, f: &mut dyn Write) -> std::fmt::Result
src/lir/ir.rs:769: fn serialize(&self, buf: &mut Vec<u8>)
src/lir/ir.rs:777: impl LirNode for SLirRet
src/lir/ir.rs:778: fn clone_node(&self) -> Box<dyn LirNode> { Box::new(self.clone()) }
src/lir/ir.rs:779: fn kind(&self) -> &'static str { "Ret" }
src/lir/ir.rs:780: fn as_any(&self) -> &dyn std::any::Any { self }
src/lir/ir.rs:781: fn emit(&self, ctx: &mut LirEmitCtx) -> Vec<String>
src/lir/ir.rs:791: fn display(&self, f: &mut dyn Write) -> std::fmt::Result
src/lir/ir.rs:797: fn serialize(&self, buf: &mut Vec<u8>)
src/lir/ir.rs:806: impl LirNode for SLirMakeFatPtr
src/lir/ir.rs:807: fn clone_node(&self) -> Box<dyn LirNode> { Box::new(self.clone()) }
src/lir/ir.rs:808: fn kind(&self) -> &'static str { "MakeFatPtr" }
src/lir/ir.rs:809: fn as_any(&self) -> &dyn std::any::Any { self }
src/lir/ir.rs:810: fn emit(&self, ctx: &mut LirEmitCtx) -> Vec<String>
src/lir/ir.rs:833: fn display(&self, f: &mut dyn Write) -> std::fmt::Result
src/lir/ir.rs:836: fn serialize(&self, buf: &mut Vec<u8>)
src/lir/ir.rs:844: impl LirNode for SLirFieldAccess
src/lir/ir.rs:845: fn clone_node(&self) -> Box<dyn LirNode> { Box::new(self.clone()) }
src/lir/ir.rs:846: fn kind(&self) -> &'static str { "FieldAccess" }
src/lir/ir.rs:847: fn as_any(&self) -> &dyn std::any::Any { self }
src/lir/ir.rs:848: fn emit(&self, ctx: &mut LirEmitCtx) -> Vec<String>
src/lir/ir.rs:869: fn display(&self, f: &mut dyn Write) -> std::fmt::Result
src/lir/ir.rs:872: fn serialize(&self, buf: &mut Vec<u8>)
src/lir/ir.rs:880: impl LirNode for SLirAsm
src/lir/ir.rs:881: fn clone_node(&self) -> Box<dyn LirNode> { Box::new(self.clone()) }
src/lir/ir.rs:882: fn kind(&self) -> &'static str { "Asm" }
src/lir/ir.rs:883: fn as_any(&self) -> &dyn std::any::Any { self }
src/lir/ir.rs:884: fn emit(&self, ctx: &mut LirEmitCtx) -> Vec<String>
src/lir/ir.rs:901: fn display(&self, f: &mut dyn Write) -> std::fmt::Result
src/lir/ir.rs:904: fn serialize(&self, buf: &mut Vec<u8>)
src/lir/ir.rs:918: impl LirNode for SLirRefInst
src/lir/ir.rs:919: fn clone_node(&self) -> Box<dyn LirNode> { Box::new(self.clone()) }
src/lir/ir.rs:920: fn kind(&self) -> &'static str { "RefInst" }
src/lir/ir.rs:921: fn as_any(&self) -> &dyn std::any::Any { self }
src/lir/ir.rs:922: fn emit(&self, _ctx: &mut LirEmitCtx) -> Vec<String>
src/lir/ir.rs:925: fn display(&self, f: &mut dyn Write) -> std::fmt::Result
src/lir/ir.rs:929: fn serialize(&self, buf: &mut Vec<u8>)
src/lir/ir.rs:938: impl LirNode for SLirArraySized
src/lir/ir.rs:939: fn clone_node(&self) -> Box<dyn LirNode> { Box::new(self.clone()) }
src/lir/ir.rs:940: fn kind(&self) -> &'static str { "ArraySized" }
src/lir/ir.rs:941: fn as_any(&self) -> &dyn std::any::Any { self }
src/lir/ir.rs:942: fn emit(&self, ctx: &mut LirEmitCtx) -> Vec<String>
src/lir/ir.rs:953: fn display(&self, f: &mut dyn Write) -> std::fmt::Result
src/lir/ir.rs:956: fn serialize(&self, buf: &mut Vec<u8>)
src/lir/ir.rs:965: impl LirNode for SLirArrayLit
src/lir/ir.rs:966: fn clone_node(&self) -> Box<dyn LirNode> { Box::new(self.clone()) }
src/lir/ir.rs:967: fn kind(&self) -> &'static str { "ArrayLit" }
src/lir/ir.rs:968: fn as_any(&self) -> &dyn std::any::Any { self }
src/lir/ir.rs:969: fn emit(&self, ctx: &mut LirEmitCtx) -> Vec<String>
src/lir/ir.rs:985: fn display(&self, f: &mut dyn Write) -> std::fmt::Result
src/lir/ir.rs:988: fn serialize(&self, buf: &mut Vec<u8>)
src/lir/ir.rs:999: impl LirNode for SLirIndexAccess
src/lir/ir.rs:1000: fn clone_node(&self) -> Box<dyn LirNode> { Box::new(self.clone()) }
src/lir/ir.rs:1001: fn kind(&self) -> &'static str { "IndexAccess" }
src/lir/ir.rs:1002: fn as_any(&self) -> &dyn std::any::Any { self }
src/lir/ir.rs:1003: fn emit(&self, ctx: &mut LirEmitCtx) -> Vec<String>
src/lir/ir.rs:1013: fn display(&self, f: &mut dyn Write) -> std::fmt::Result
src/lir/ir.rs:1016: fn serialize(&self, buf: &mut Vec<u8>)
src/lir/ir.rs:1024: impl LirNode for SLirStructLit
src/lir/ir.rs:1025: fn clone_node(&self) -> Box<dyn LirNode> { Box::new(self.clone()) }
src/lir/ir.rs:1026: fn kind(&self) -> &'static str { "StructLit" }
src/lir/ir.rs:1027: fn as_any(&self) -> &dyn std::any::Any { self }
src/lir/ir.rs:1028: fn emit(&self, ctx: &mut LirEmitCtx) -> Vec<String>
src/lir/ir.rs:1041: fn display(&self, f: &mut dyn Write) -> std::fmt::Result
src/lir/ir.rs:1044: fn serialize(&self, buf: &mut Vec<u8>)
src/lir/ir.rs:1055: impl LirNode for SLirVirtualCall
src/lir/ir.rs:1056: fn clone_node(&self) -> Box<dyn LirNode> { Box::new(self.clone()) }
src/lir/ir.rs:1057: fn kind(&self) -> &'static str { "VirtualCall" }
src/lir/ir.rs:1058: fn as_any(&self) -> &dyn std::any::Any { self }
src/lir/ir.rs:1059: fn emit(&self, ctx: &mut LirEmitCtx) -> Vec<String>
src/lir/ir.rs:1080: fn display(&self, f: &mut dyn Write) -> std::fmt::Result
src/lir/ir.rs:1084: fn serialize(&self, buf: &mut Vec<u8>)
src/lir/ir.rs:1096: impl LirNode for SLirFieldStore
src/lir/ir.rs:1097: fn clone_node(&self) -> Box<dyn LirNode> { Box::new(self.clone()) }
src/lir/ir.rs:1098: fn kind(&self) -> &'static str { "FieldStore" }
src/lir/ir.rs:1099: fn as_any(&self) -> &dyn std::any::Any { self }
src/lir/ir.rs:1100: fn emit(&self, ctx: &mut LirEmitCtx) -> Vec<String>
src/lir/ir.rs:1129: fn display(&self, f: &mut dyn Write) -> std::fmt::Result
src/lir/ir.rs:1132: fn serialize(&self, buf: &mut Vec<u8>)
src/lir/ir.rs:1142: impl LirNode for SLirIndexStore
src/lir/ir.rs:1143: fn clone_node(&self) -> Box<dyn LirNode> { Box::new(self.clone()) }
src/lir/ir.rs:1144: fn kind(&self) -> &'static str { "IndexStore" }
src/lir/ir.rs:1145: fn as_any(&self) -> &dyn std::any::Any { self }
src/lir/ir.rs:1146: fn emit(&self, ctx: &mut LirEmitCtx) -> Vec<String>
src/lir/ir.rs:1155: fn display(&self, f: &mut dyn Write) -> std::fmt::Result
src/lir/ir.rs:1158: fn serialize(&self, buf: &mut Vec<u8>)
src/lir/ir.rs:1166: impl LirNode for SLirCustom
src/lir/ir.rs:1167: fn clone_node(&self) -> Box<dyn LirNode> { Box::new(self.clone()) }
src/lir/ir.rs:1168: fn kind(&self) -> &'static str { "Custom" }
src/lir/ir.rs:1169: fn as_any(&self) -> &dyn std::any::Any { self }
src/lir/ir.rs:1170: fn emit(&self, _ctx: &mut LirEmitCtx) -> Vec<String>
src/lir/ir.rs:1174: fn display(&self, _f: &mut dyn Write) -> std::fmt::Result { Ok(()) }
src/lir/ir.rs:1175: fn serialize(&self, _buf: &mut Vec<u8>) {}
src/lir/ir.rs:1183: pub struct LirBlock
src/lir/ir.rs:1189: pub struct LirFn
src/lir/ir.rs:1204: pub struct VtableDesc
src/lir/ir.rs:1210: pub struct LirProgram
src/lir/ir.rs:1224: pub(crate) fn put_u32(buf: &mut Vec<u8>, v: u32) { buf.extend_from_slice(&v.to_le_bytes()); }
src/lir/ir.rs:1225: pub(crate) fn put_u64(buf: &mut Vec<u8>, v: u64) { buf.extend_from_slice(&v.to_le_bytes()); }
src/lir/ir.rs:1226: pub(crate) fn put_str(buf: &mut Vec<u8>, s: &str)
src/lir/ir.rs:1232: pub(crate) fn put_type(buf: &mut Vec<u8>, ty: &HirType)
src/lir/ir.rs:1259: pub(crate) fn put_value(buf: &mut Vec<u8>, v: &LirValue)
src/lir/ir.rs:1272: pub(crate) fn put_literal(buf: &mut Vec<u8>, lit: &HirLiteral)
src/lir/ir.rs:1287: pub trait LirLowerCtx
src/lir/ir.rs:1288: fn next_tmp(&mut self) -> u64;
src/lir/ir.rs:1289: fn emit(&mut self, inst: LirNodeBox);
src/lir/ir.rs:1290: fn str_map(&self) -> &HashMap<String, u64>;
src/lir/ir.rs:1291: fn loop_stack(&self) -> &Vec<(String, String)>;
src/lir/ir.rs:1292: fn loop_stack_mut(&mut self) -> &mut Vec<(String, String)>;
src/lir/ir.rs:1293: fn next_block_label(&mut self, prefix: &str) -> String;
src/lir/ir.rs:1294: fn set_current_block(&mut self, label: String);
src/lir/lower.rs:9: pub fn lower_program(mir: &MirProgram) -> LirProgram
src/lir/lower.rs:56: fn collect_fn_names(mir: &MirProgram) -> HashMap<FnId, String>
src/lir/lower.rs:62: fn collect_fn_names_items(items: &[MirItem], _prefix: &str, map: &mut HashMap<FnId, String>)
src/lir/lower.rs:81: fn mangle(prefix: &str, name: &str, params: &[(crate::intern::Symbol, HirType)]) -> String
src/lir/lower.rs:99: fn type_to_mangle(ty: &HirType) -> String
src/lir/lower.rs:118: fn lower_items(item: &MirItem, str_map: &HashMap<String, u64>) -> Vec<LirFn>
src/lir/lower.rs:128: fn lower_fn(f: &MirFn, str_map: &HashMap<String, u64>) -> LirFn
src/lir/lower.rs:184: struct LowerCtx<'a>
src/lir/lower.rs:194: impl<'a> LowerCtx<'a>
src/lir/lower.rs:195: fn new(str_map: &'a HashMap<String, u64>) -> Self
src/lir/lower.rs:207: fn next_block_label(&mut self, prefix: &str) -> String
src/lir/lower.rs:213: fn set_current_block(&mut self, label: String)
src/lir/lower.rs:228: fn finish(&mut self) -> Vec<LirBlock>
src/lir/lower.rs:238: impl LirLowerCtx for LowerCtx<'_>
src/lir/lower.rs:239: fn next_tmp(&mut self) -> u64
src/lir/lower.rs:244: fn emit(&mut self, inst: LirNodeBox)
src/lir/lower.rs:247: fn str_map(&self) -> &HashMap<String, u64>
src/lir/lower.rs:250: fn loop_stack(&self) -> &Vec<(String, String)>
src/lir/lower.rs:253: fn loop_stack_mut(&mut self) -> &mut Vec<(String, String)>
src/lir/lower.rs:256: fn next_block_label(&mut self, prefix: &str) -> String
src/lir/lower.rs:259: fn set_current_block(&mut self, label: String)
src/lir/lower.rs:264: fn lower_stmts(ctx: &mut LowerCtx, stmts: &[MirStmtBox])
src/lir/lower.rs:296: fn lower_expr(ctx: &mut dyn LirLowerCtx, expr: &MirNodeBox) -> LirValue
src/lir/lower.rs:300: fn default_ret_value(ty: &HirType) -> Option<(LirValue, HirType)>
src/lir/lower.rs:313: fn strip_ownership(ty: HirType) -> HirType
src/lir/lower.rs:320: fn type_size(ty: &HirType) -> u64
src/lir/lower.rs:331: fn collect_strings(mir: &MirProgram) -> Vec<String>
src/lir/lower.rs:339: fn collect_strings_items(items: &[MirItem], out: &mut Vec<String>)
src/lir/lower.rs:349: fn collect_strings_stmts(stmts: &[MirStmtBox], out: &mut Vec<String>)
src/lir/lower.rs:357: fn collect_strings_dyn(node: &dyn MirNode, out: &mut Vec<String>)
src/lir/lower.rs:366: fn block_ends_with_ret(stmts: &[MirStmtBox]) -> bool
src/lir/lower.rs:374: impl MirNode for SMirLocal
src/lir/lower.rs:375: fn clone_node(&self) -> Box<dyn MirNode> { Box::new(self.clone()) }
src/lir/lower.rs:376: fn lower_to_lir(&self, ctx: &mut dyn LirLowerCtx) -> LirValue
src/lir/lower.rs:381: fn display(&self, level: usize, w: &mut dyn std::fmt::Write) -> std::fmt::Result
src/lir/lower.rs:385: fn expr_type(&self) -> HirType { self.ty.clone() }
src/lir/lower.rs:386: fn as_local(&self) -> Option<VarId> { Some(self.var) }
src/lir/lower.rs:387: fn collect_var_ids(&self, vars: &mut std::collections::HashSet<VarId>) { vars.insert(self.var); }
src/lir/lower.rs:390: impl MirNode for SMirLiteral
src/lir/lower.rs:391: fn clone_node(&self) -> Box<dyn MirNode> { Box::new(self.clone()) }
src/lir/lower.rs:392: fn lower_to_lir(&self, ctx: &mut dyn LirLowerCtx) -> LirValue
src/lir/lower.rs:435: fn display(&self, level: usize, w: &mut dyn std::fmt::Write) -> std::fmt::Result
src/lir/lower.rs:445: fn expr_type(&self) -> HirType { self.ty.clone() }
src/lir/lower.rs:446: fn as_string_literal(&self) -> Option<&str>
src/lir/lower.rs:451: impl MirNode for SMirBinary
src/lir/lower.rs:452: fn clone_node(&self) -> Box<dyn MirNode> { Box::new(self.clone()) }
src/lir/lower.rs:453: fn lower_to_lir(&self, ctx: &mut dyn LirLowerCtx) -> LirValue
src/lir/lower.rs:464: fn display(&self, level: usize, w: &mut dyn std::fmt::Write) -> std::fmt::Result
src/lir/lower.rs:472: fn expr_type(&self) -> HirType { self.ty.clone() }
src/lir/lower.rs:473: fn for_each_child(&self, f: &mut dyn FnMut(&dyn MirNode)) { f(&*self.lhs); f(&*self.rhs); }
src/lir/lower.rs:476: impl MirNode for SMirUnary
src/lir/lower.rs:477: fn clone_node(&self) -> Box<dyn MirNode> { Box::new(self.clone()) }
src/lir/lower.rs:478: fn lower_to_lir(&self, ctx: &mut dyn LirLowerCtx) -> LirValue
src/lir/lower.rs:484: fn display(&self, level: usize, w: &mut dyn std::fmt::Write) -> std::fmt::Result
src/lir/lower.rs:489: fn expr_type(&self) -> HirType { self.ty.clone() }
src/lir/lower.rs:490: fn for_each_child(&self, f: &mut dyn FnMut(&dyn MirNode)) { f(&*self.arg); }
src/lir/lower.rs:493: impl MirNode for SMirCall
src/lir/lower.rs:494: fn clone_node(&self) -> Box<dyn MirNode> { Box::new(self.clone()) }
src/lir/lower.rs:495: fn lower_to_lir(&self, ctx: &mut dyn LirLowerCtx) -> LirValue
src/lir/lower.rs:507: fn display(&self, level: usize, w: &mut dyn std::fmt::Write) -> std::fmt::Result
src/lir/lower.rs:512: fn expr_type(&self) -> HirType { self.ty.clone() }
src/lir/lower.rs:513: fn for_each_child(&self, f: &mut dyn FnMut(&dyn MirNode)) { for a in &self.args { f(&**a); } }
src/lir/lower.rs:516: impl MirNode for SMirMove
src/lir/lower.rs:517: fn clone_node(&self) -> Box<dyn MirNode> { Box::new(self.clone()) }
src/lir/lower.rs:518: fn lower_to_lir(&self, ctx: &mut dyn LirLowerCtx) -> LirValue { self.expr.lower_to_lir(ctx) }
src/lir/lower.rs:519: fn display(&self, level: usize, w: &mut dyn std::fmt::Write) -> std::fmt::Result
src/lir/lower.rs:524: fn expr_type(&self) -> HirType { self.ty.clone() }
src/lir/lower.rs:525: fn for_each_child(&self, f: &mut dyn FnMut(&dyn MirNode)) { f(&*self.expr); }
src/lir/lower.rs:528: impl MirNode for SMirClone
src/lir/lower.rs:529: fn clone_node(&self) -> Box<dyn MirNode> { Box::new(self.clone()) }
src/lir/lower.rs:530: fn lower_to_lir(&self, ctx: &mut dyn LirLowerCtx) -> LirValue { self.expr.lower_to_lir(ctx) }
src/lir/lower.rs:531: fn display(&self, level: usize, w: &mut dyn std::fmt::Write) -> std::fmt::Result
src/lir/lower.rs:536: fn expr_type(&self) -> HirType { self.ty.clone() }
src/lir/lower.rs:537: fn for_each_child(&self, f: &mut dyn FnMut(&dyn MirNode)) { f(&*self.expr); }
src/lir/lower.rs:540: impl MirNode for SMirToUnique
src/lir/lower.rs:541: fn clone_node(&self) -> Box<dyn MirNode> { Box::new(self.clone()) }
src/lir/lower.rs:542: fn lower_to_lir(&self, ctx: &mut dyn LirLowerCtx) -> LirValue
src/lir/lower.rs:555: fn display(&self, level: usize, w: &mut dyn std::fmt::Write) -> std::fmt::Result
src/lir/lower.rs:560: fn expr_type(&self) -> HirType { self.ty.clone() }
src/lir/lower.rs:561: fn for_each_child(&self, f: &mut dyn FnMut(&dyn MirNode)) { f(&*self.expr); }
src/lir/lower.rs:564: impl MirNode for SMirToShared
src/lir/lower.rs:565: fn clone_node(&self) -> Box<dyn MirNode> { Box::new(self.clone()) }
src/lir/lower.rs:566: fn lower_to_lir(&self, ctx: &mut dyn LirLowerCtx) -> LirValue
src/lir/lower.rs:579: fn display(&self, level: usize, w: &mut dyn std::fmt::Write) -> std::fmt::Result
src/lir/lower.rs:584: fn expr_type(&self) -> HirType { self.ty.clone() }
src/lir/lower.rs:585: fn for_each_child(&self, f: &mut dyn FnMut(&dyn MirNode)) { f(&*self.expr); }
src/lir/lower.rs:588: impl MirNode for SMirToWeak
src/lir/lower.rs:589: fn clone_node(&self) -> Box<dyn MirNode> { Box::new(self.clone()) }
src/lir/lower.rs:590: fn lower_to_lir(&self, ctx: &mut dyn LirLowerCtx) -> LirValue
src/lir/lower.rs:603: fn display(&self, level: usize, w: &mut dyn std::fmt::Write) -> std::fmt::Result
src/lir/lower.rs:608: fn expr_type(&self) -> HirType { self.ty.clone() }
src/lir/lower.rs:609: fn for_each_child(&self, f: &mut dyn FnMut(&dyn MirNode)) { f(&*self.expr); }
src/lir/lower.rs:612: impl MirNode for SMirVirtualCall
src/lir/lower.rs:613: fn clone_node(&self) -> Box<dyn MirNode> { Box::new(self.clone()) }
src/lir/lower.rs:614: fn lower_to_lir(&self, ctx: &mut dyn LirLowerCtx) -> LirValue
src/lir/lower.rs:631: fn display(&self, level: usize, w: &mut dyn std::fmt::Write) -> std::fmt::Result
src/lir/lower.rs:638: fn expr_type(&self) -> HirType { self.ty.clone() }
src/lir/lower.rs:639: fn for_each_child(&self, f: &mut dyn FnMut(&dyn MirNode)) { f(&*self.receiver); for a in &self.args { f(&**a); } }
src/lir/lower.rs:642: impl MirNode for SMirMakeFatPtr
src/lir/lower.rs:643: fn clone_node(&self) -> Box<dyn MirNode> { Box::new(self.clone()) }
src/lir/lower.rs:644: fn lower_to_lir(&self, ctx: &mut dyn LirLowerCtx) -> LirValue
src/lir/lower.rs:658: fn display(&self, level: usize, w: &mut dyn std::fmt::Write) -> std::fmt::Result
src/lir/lower.rs:663: fn expr_type(&self) -> HirType { self.ty.clone() }
src/lir/lower.rs:664: fn for_each_child(&self, f: &mut dyn FnMut(&dyn MirNode)) { f(&*self.value); }
src/lir/lower.rs:667: impl MirNode for SMirEnumConstruct
src/lir/lower.rs:668: fn clone_node(&self) -> Box<dyn MirNode> { Box::new(self.clone()) }
src/lir/lower.rs:669: fn lower_to_lir(&self, _ctx: &mut dyn LirLowerCtx) -> LirValue
src/lir/lower.rs:672: fn display(&self, level: usize, w: &mut dyn std::fmt::Write) -> std::fmt::Result
src/lir/lower.rs:675: fn expr_type(&self) -> HirType { self.ty.clone() }
src/lir/lower.rs:676: fn for_each_child(&self, f: &mut dyn FnMut(&dyn MirNode)) { for a in &self.args { f(&**a); } }
src/lir/lower.rs:679: impl MirNode for SMirFnPtr
src/lir/lower.rs:680: fn clone_node(&self) -> Box<dyn MirNode> { Box::new(self.clone()) }
src/lir/lower.rs:681: fn lower_to_lir(&self, ctx: &mut dyn LirLowerCtx) -> LirValue
src/lir/lower.rs:686: fn display(&self, level: usize, w: &mut dyn std::fmt::Write) -> std::fmt::Result
src/lir/lower.rs:689: fn expr_type(&self) -> HirType { self.ty.clone() }
src/lir/lower.rs:692: impl MirNode for SMirCallPtr
src/lir/lower.rs:693: fn clone_node(&self) -> Box<dyn MirNode> { Box::new(self.clone()) }
src/lir/lower.rs:694: fn lower_to_lir(&self, ctx: &mut dyn LirLowerCtx) -> LirValue
src/lir/lower.rs:707: fn display(&self, level: usize, w: &mut dyn std::fmt::Write) -> std::fmt::Result
src/lir/lower.rs:710: fn expr_type(&self) -> HirType { self.ty.clone() }
src/lir/lower.rs:711: fn for_each_child(&self, f: &mut dyn FnMut(&dyn MirNode)) { f(&*self.fn_ptr); for a in &self.args { f(&**a); } }
src/lir/lower.rs:714: impl MirNode for SMirEnumMatch
src/lir/lower.rs:715: fn clone_node(&self) -> Box<dyn MirNode> { Box::new(self.clone()) }
src/lir/lower.rs:716: fn lower_to_lir(&self, ctx: &mut dyn LirLowerCtx) -> LirValue
src/lir/lower.rs:749: fn display(&self, level: usize, w: &mut dyn std::fmt::Write) -> std::fmt::Result
src/lir/lower.rs:755: fn expr_type(&self) -> HirType { self.ty.clone() }
src/lir/lower.rs:756: fn for_each_child(&self, f: &mut dyn FnMut(&dyn MirNode))
src/lir/lower.rs:762: impl MirNode for SMirFieldAccess
src/lir/lower.rs:763: fn clone_node(&self) -> Box<dyn MirNode> { Box::new(self.clone()) }
src/lir/lower.rs:764: fn lower_to_lir(&self, ctx: &mut dyn LirLowerCtx) -> LirValue
src/lir/lower.rs:774: fn display(&self, level: usize, w: &mut dyn std::fmt::Write) -> std::fmt::Result
src/lir/lower.rs:779: fn expr_type(&self) -> HirType { self.ty.clone() }
src/lir/lower.rs:780: fn for_each_child(&self, f: &mut dyn FnMut(&dyn MirNode)) { f(&*self.object); }
src/lir/lower.rs:783: impl MirNode for SMirStructLiteral
src/lir/lower.rs:784: fn clone_node(&self) -> Box<dyn MirNode> { Box::new(self.clone()) }
src/lir/lower.rs:785: fn lower_to_lir(&self, ctx: &mut dyn LirLowerCtx) -> LirValue
src/lir/lower.rs:794: fn display(&self, level: usize, w: &mut dyn std::fmt::Write) -> std::fmt::Result
src/lir/lower.rs:802: fn expr_type(&self) -> HirType { self.ty.clone() }
src/lir/lower.rs:803: fn for_each_child(&self, f: &mut dyn FnMut(&dyn MirNode)) { for (_, e) in &self.fields { f(&**e); } }
src/lir/lower.rs:806: impl MirNode for SMirArrayLiteral
src/lir/lower.rs:807: fn clone_node(&self) -> Box<dyn MirNode> { Box::new(self.clone()) }
src/lir/lower.rs:808: fn lower_to_lir(&self, ctx: &mut dyn LirLowerCtx) -> LirValue
src/lir/lower.rs:818: fn display(&self, level: usize, w: &mut dyn std::fmt::Write) -> std::fmt::Result
src/lir/lower.rs:823: fn expr_type(&self) -> HirType { self.ty.clone() }
src/lir/lower.rs:824: fn for_each_child(&self, f: &mut dyn FnMut(&dyn MirNode)) { for e in &self.elems { f(&**e); } }
src/lir/lower.rs:827: impl MirNode for SMirArraySized
src/lir/lower.rs:828: fn clone_node(&self) -> Box<dyn MirNode> { Box::new(self.clone()) }
src/lir/lower.rs:829: fn lower_to_lir(&self, ctx: &mut dyn LirLowerCtx) -> LirValue
src/lir/lower.rs:837: fn display(&self, level: usize, w: &mut dyn std::fmt::Write) -> std::fmt::Result
src/lir/lower.rs:843: fn expr_type(&self) -> HirType { self.ty.clone() }
src/lir/lower.rs:844: fn for_each_child(&self, f: &mut dyn FnMut(&dyn MirNode)) { f(&*self.count); }
src/lir/lower.rs:847: impl MirNode for SMirRef
src/lir/lower.rs:848: fn clone_node(&self) -> Box<dyn MirNode> { Box::new(self.clone()) }
src/lir/lower.rs:849: fn lower_to_lir(&self, ctx: &mut dyn LirLowerCtx) -> LirValue
src/lir/lower.rs:858: fn display(&self, level: usize, w: &mut dyn std::fmt::Write) -> std::fmt::Result
src/lir/lower.rs:864: fn expr_type(&self) -> HirType { self.ty.clone() }
src/lir/lower.rs:865: fn for_each_child(&self, f: &mut dyn FnMut(&dyn MirNode)) { f(&*self.expr); }
src/lir/lower.rs:866: fn as_ref(&self) -> Option<(VarId, bool)>
src/lir/lower.rs:871: impl MirNode for SMirIndex
src/lir/lower.rs:872: fn clone_node(&self) -> Box<dyn MirNode> { Box::new(self.clone()) }
src/lir/lower.rs:873: fn lower_to_lir(&self, ctx: &mut dyn LirLowerCtx) -> LirValue
src/lir/lower.rs:886: fn display(&self, level: usize, w: &mut dyn std::fmt::Write) -> std::fmt::Result
src/lir/lower.rs:894: fn expr_type(&self) -> HirType { self.ty.clone() }
src/lir/lower.rs:895: fn for_each_child(&self, f: &mut dyn FnMut(&dyn MirNode)) { f(&*self.object); f(&*self.index); }
src/lir/lower.rs:898: impl MirNode for SMirAsm
src/lir/lower.rs:899: fn clone_node(&self) -> Box<dyn MirNode> { Box::new(self.clone()) }
src/lir/lower.rs:900: fn lower_to_lir(&self, ctx: &mut dyn LirLowerCtx) -> LirValue
src/lir/lower.rs:912: fn display(&self, level: usize, w: &mut dyn std::fmt::Write) -> std::fmt::Result
src/lir/lower.rs:924: fn expr_type(&self) -> HirType { self.ty.clone() }
src/lir/lower.rs:925: fn for_each_child(&self, f: &mut dyn FnMut(&dyn MirNode))
src/lir/lower.rs:935: impl MirStmtNode for SMirAssignStmt
src/lir/lower.rs:936: fn clone_stmt(&self) -> Box<dyn MirStmtNode> { Box::new(self.clone()) }
src/lir/lower.rs:937: fn lower_to_lir_stmt(&self, ctx: &mut dyn LirLowerCtx)
src/lir/lower.rs:943: fn display_stmt(&self, level: usize, w: &mut dyn std::fmt::Write) -> std::fmt::Result
src/lir/lower.rs:951: fn for_each_child_expr(&self, f: &mut dyn FnMut(&dyn MirNode)) { f(&*self.target); f(&*self.value); }
src/lir/lower.rs:954: impl MirStmtNode for SMirFieldAssignStmt
src/lir/lower.rs:955: fn clone_stmt(&self) -> Box<dyn MirStmtNode> { Box::new(self.clone()) }
src/lir/lower.rs:956: fn lower_to_lir_stmt(&self, ctx: &mut dyn LirLowerCtx)
src/lir/lower.rs:974: fn display_stmt(&self, level: usize, w: &mut dyn std::fmt::Write) -> std::fmt::Result
src/lir/lower.rs:982: fn for_each_child_expr(&self, f: &mut dyn FnMut(&dyn MirNode)) { f(&*self.object); f(&*self.value); }
src/lir/lower.rs:985: impl MirStmtNode for SMirIndexAssignStmt
src/lir/lower.rs:986: fn clone_stmt(&self) -> Box<dyn MirStmtNode> { Box::new(self.clone()) }
src/lir/lower.rs:987: fn lower_to_lir_stmt(&self, ctx: &mut dyn LirLowerCtx)
src/lir/lower.rs:1000: fn display_stmt(&self, level: usize, w: &mut dyn std::fmt::Write) -> std::fmt::Result
src/lir/lower.rs:1010: fn for_each_child_expr(&self, f: &mut dyn FnMut(&dyn MirNode)) { f(&*self.object); f(&*self.index); f(&*self.value); }
src/lir/lower.rs:1013: impl MirStmtNode for SMirReturnStmt
src/lir/lower.rs:1014: fn clone_stmt(&self) -> Box<dyn MirStmtNode> { Box::new(self.clone()) }
src/lir/lower.rs:1015: fn lower_to_lir_stmt(&self, ctx: &mut dyn LirLowerCtx)
src/lir/lower.rs:1023: fn display_stmt(&self, level: usize, w: &mut dyn std::fmt::Write) -> std::fmt::Result
src/lir/lower.rs:1029: fn for_each_child_expr(&self, f: &mut dyn FnMut(&dyn MirNode))
src/lir/lower.rs:1032: fn is_return(&self) -> bool { true }
src/lir/lower.rs:1033: fn return_value(&self) -> Option<&MirNodeBox> { self.value.as_ref() }
src/lir/lower.rs:1036: impl MirStmtNode for SMirIfStmt
src/lir/lower.rs:1037: fn clone_stmt(&self) -> Box<dyn MirStmtNode> { Box::new(self.clone()) }
src/lir/lower.rs:1038: fn lower_to_lir_stmt(&self, ctx: &mut dyn LirLowerCtx)
src/lir/lower.rs:1055: fn display_stmt(&self, level: usize, w: &mut dyn std::fmt::Write) -> std::fmt::Result
src/lir/lower.rs:1073: fn for_each_child_expr(&self, f: &mut dyn FnMut(&dyn MirNode)) { f(&*self.cond); }
src/lir/lower.rs:1074: fn for_each_child_stmt(&self, f: &mut dyn FnMut(&dyn MirStmtNode))
src/lir/lower.rs:1081: fn lower_elifs(ctx: &mut dyn LirLowerCtx, elifs: &[(MirNodeBox, Vec<MirStmtBox>)], else_block: &Option<Vec<MirStmtBox>>, merge_lbl: &str)
src/lir/lower.rs:1102: impl MirStmtNode for SMirWhileStmt
src/lir/lower.rs:1103: fn clone_stmt(&self) -> Box<dyn MirStmtNode> { Box::new(self.clone()) }
src/lir/lower.rs:1104: fn lower_to_lir_stmt(&self, ctx: &mut dyn LirLowerCtx)
src/lir/lower.rs:1124: fn display_stmt(&self, level: usize, w: &mut dyn std::fmt::Write) -> std::fmt::Result
src/lir/lower.rs:1132: fn for_each_child_expr(&self, f: &mut dyn FnMut(&dyn MirNode)) { f(&*self.cond); }
src/lir/lower.rs:1133: fn for_each_child_stmt(&self, f: &mut dyn FnMut(&dyn MirStmtNode)) { for s in &self.body { f(&**s); } }
src/lir/lower.rs:1136: impl MirStmtNode for SMirBreakStmt
src/lir/lower.rs:1137: fn clone_stmt(&self) -> Box<dyn MirStmtNode> { Box::new(self.clone()) }
src/lir/lower.rs:1138: fn lower_to_lir_stmt(&self, ctx: &mut dyn LirLowerCtx)
src/lir/lower.rs:1143: fn display_stmt(&self, level: usize, w: &mut dyn std::fmt::Write) -> std::fmt::Result
src/lir/lower.rs:1148: impl MirStmtNode for SMirContinueStmt
src/lir/lower.rs:1149: fn clone_stmt(&self) -> Box<dyn MirStmtNode> { Box::new(self.clone()) }
src/lir/lower.rs:1150: fn lower_to_lir_stmt(&self, ctx: &mut dyn LirLowerCtx)
src/lir/lower.rs:1155: fn display_stmt(&self, level: usize, w: &mut dyn std::fmt::Write) -> std::fmt::Result
src/lir/lower.rs:1160: impl MirStmtNode for SMirExprStmt
src/lir/lower.rs:1161: fn clone_stmt(&self) -> Box<dyn MirStmtNode> { Box::new(self.clone()) }
src/lir/lower.rs:1162: fn lower_to_lir_stmt(&self, ctx: &mut dyn LirLowerCtx)
src/lir/lower.rs:1165: fn display_stmt(&self, level: usize, w: &mut dyn std::fmt::Write) -> std::fmt::Result
src/lir/lower.rs:1170: fn for_each_child_expr(&self, f: &mut dyn FnMut(&dyn MirNode)) { f(&*self.expr); }
src/lir/lower.rs:1173: impl MirStmtNode for SMirBlockStmt
src/lir/lower.rs:1174: fn clone_stmt(&self) -> Box<dyn MirStmtNode> { Box::new(self.clone()) }
src/lir/lower.rs:1175: fn lower_to_lir_stmt(&self, ctx: &mut dyn LirLowerCtx)
src/lir/lower.rs:1178: fn display_stmt(&self, level: usize, w: &mut dyn std::fmt::Write) -> std::fmt::Result
src/lir/lower.rs:1184: fn for_each_child_stmt(&self, f: &mut dyn FnMut(&dyn MirStmtNode)) { for s in &self.stmts { f(&**s); } }
src/lir/lower.rs:1187: impl MirStmtNode for SMirDropStmt
src/lir/lower.rs:1188: fn clone_stmt(&self) -> Box<dyn MirStmtNode> { Box::new(self.clone()) }
src/lir/lower.rs:1189: fn lower_to_lir_stmt(&self, ctx: &mut dyn LirLowerCtx)
src/lir/lower.rs:1192: fn display_stmt(&self, level: usize, w: &mut dyn std::fmt::Write) -> std::fmt::Result
src/lir/lower.rs:1195: fn as_drop(&self) -> Option<(VarId, &HirType)> { Some((self.var, &self.ty)) }
src/lir/lower.rs:1198: impl MirStmtNode for SMirRetainStmt
src/lir/lower.rs:1199: fn clone_stmt(&self) -> Box<dyn MirStmtNode> { Box::new(self.clone()) }
src/lir/lower.rs:1200: fn lower_to_lir_stmt(&self, ctx: &mut dyn LirLowerCtx)
src/lir/lower.rs:1203: fn display_stmt(&self, level: usize, w: &mut dyn std::fmt::Write) -> std::fmt::Result
src/lir/lower.rs:1206: fn as_retain(&self) -> Option<(VarId, &HirType)> { Some((self.var, &self.ty)) }
src/lir/lower.rs:1209: impl MirStmtNode for SMirReleaseStmt
src/lir/lower.rs:1210: fn clone_stmt(&self) -> Box<dyn MirStmtNode> { Box::new(self.clone()) }
src/lir/lower.rs:1211: fn lower_to_lir_stmt(&self, ctx: &mut dyn LirLowerCtx)
src/lir/lower.rs:1214: fn display_stmt(&self, level: usize, w: &mut dyn std::fmt::Write) -> std::fmt::Result
src/lir/lower.rs:1217: fn as_release(&self) -> Option<(VarId, &HirType)> { Some((self.var, &self.ty)) }
src/lir/lower.rs:1220: fn write_stmt_block(stmts: &[MirStmtBox], level: usize, w: &mut dyn std::fmt::Write) -> std::fmt::Result
src/lir/lower.rs:1227: fn display_hir_type(ty: &HirType) -> String
src/lir/lower.rs:1231: fn extract_var(val: &LirValue) -> VarId
src/lir/mod.rs:12: pub mod ir;
src/lir/mod.rs:13: pub mod lower;
src/lir/mod.rs:14: pub mod display;
src/lir/mod.rs:15: pub mod emit;
src/lir/mod.rs:16: pub mod serialize;
src/lir/serialize.rs:10: pub fn program_to_bytes(p: &LirProgram) -> Vec<u8>
src/lir/serialize.rs:71: pub fn program_from_bytes(data: &[u8]) -> Result<LirProgram, String>
src/lir/serialize.rs:144: fn put_inst(buf: &mut Vec<u8>, inst: &LirNodeBox)
src/lir/serialize.rs:148: fn put_fn(buf: &mut Vec<u8>, f: &LirFn)
src/lir/serialize.rs:177: struct Reader<'a>
src/lir/serialize.rs:182: impl<'a> Reader<'a>
src/lir/serialize.rs:183: fn read(&mut self, n: usize) -> Result<&'a [u8], String>
src/lir/serialize.rs:191: fn u32(&mut self) -> Result<u32, String>
src/lir/serialize.rs:195: fn u64(&mut self) -> Result<u64, String>
src/lir/serialize.rs:199: fn str(&mut self) -> Result<String, String>
src/lir/serialize.rs:204: fn ty(&mut self) -> Result<HirType, String>
src/lir/serialize.rs:233: fn literal(&mut self) -> Result<HirLiteral, String>
src/lir/serialize.rs:244: fn value(&mut self) -> Result<LirValue, String>
src/lir/serialize.rs:254: fn inst(&mut self) -> Result<LirNodeBox, String>
src/lir/serialize.rs:415: fn read_fn(&mut self) -> Result<LirFn, String>
src/main.rs:19: fn main()
src/main.rs:56: fn find_project(dir: &Path) -> Option<(PathBuf, String)>
src/main.rs:71: fn project_entry(project_dir: &Path) -> Option<PathBuf>
src/main.rs:80: fn resolve_path(arg: Option<&str>) -> PathBuf
src/main.rs:108: fn cmd_new(args: &[String])
src/main.rs:148: fn cmd_fmt(args: &[String])
src/main.rs:181: fn do_check(path: &Path) -> Result<(), String>
src/main.rs:192: fn cmd_check(args: &[String])
src/main.rs:209: fn watch_file(path: &Path)
src/main.rs:251: fn cmd_package(args: &[String])
src/main.rs:264: fn cmd_install(args: &[String])
src/main.rs:276: fn cmd_defs(args: &[String])
src/main.rs:307: fn resolve_import_defs(stmts: &[ayanami::parser::ast::Stmt], base_path: &str, visited: &mut std::collections::HashSet<std::path::PathBuf>, defs: &mut Vec<ayanami::compiler::SymDef>)
src/main.rs:344: fn load_config() -> Option<(PathBuf, ayanami::package::config::ProjectConfig)>
src/main.rs:353: fn cmd_clean()
src/main.rs:366: fn cmd_build(args: &[String])
src/main.rs:385: fn cmd_run(args: &[String])
src/mir/borrow.rs:6: pub fn check_borrows(mir_fn: &MirFn) -> Result<(), String>
src/mir/borrow.rs:11: struct Borrow
src/mir/borrow.rs:17: struct BorrowChecker<'a>
src/mir/borrow.rs:22: impl<'a> BorrowChecker<'a>
src/mir/borrow.rs:23: fn new(mir_fn: &'a MirFn) -> Self
src/mir/borrow.rs:27: fn check(&self) -> Result<(), String>
src/mir/borrow.rs:32: fn check_stmts(&self, stmts: &[MirStmtBox], base: usize)
src/mir/borrow.rs:49: fn check_expr(&self, expr: &dyn MirNode) -> Result<(), String>
src/mir/borrow.rs:67: fn add_borrow(&self, var: VarId, mutable: bool) -> Result<(), String>
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
src/mir/ir.rs:16: fn collect_var_ids(&self, vars: &mut HashSet<VarId>) {}
src/mir/ir.rs:17: fn record_moves(&self, moved: &mut HashSet<VarId>) {}
src/mir/ir.rs:18: fn for_each_child(&self, f: &mut dyn FnMut(&dyn MirNode)) {}
src/mir/ir.rs:19: fn as_string_literal(&self) -> Option<&str> { None }
src/mir/ir.rs:20: fn as_ref(&self) -> Option<(VarId, bool)> { None }
src/mir/ir.rs:24: pub struct MirNodeBox(pub Box<dyn MirNode>);
src/mir/ir.rs:25: impl Clone for MirNodeBox
src/mir/ir.rs:26: fn clone(&self) -> Self { MirNodeBox(self.0.clone_node()) }
src/mir/ir.rs:28: impl std::ops::Deref for MirNodeBox
src/mir/ir.rs:29: type Target = dyn MirNode;
src/mir/ir.rs:30: fn deref(&self) -> &Self::Target { &*self.0 }
src/mir/ir.rs:33: pub trait MirStmtNode: std::fmt::Debug
src/mir/ir.rs:34: fn clone_stmt(&self) -> Box<dyn MirStmtNode>;
src/mir/ir.rs:35: fn lower_to_lir_stmt(&self, ctx: &mut dyn LirLowerCtx);
src/mir/ir.rs:36: fn display_stmt(&self, level: usize, w: &mut dyn FmtWrite) -> std::fmt::Result;
src/mir/ir.rs:37: fn for_each_child_expr(&self, f: &mut dyn FnMut(&dyn MirNode)) {}
src/mir/ir.rs:38: fn for_each_child_stmt(&self, f: &mut dyn FnMut(&dyn MirStmtNode)) {}
src/mir/ir.rs:39: fn is_return(&self) -> bool { false }
src/mir/ir.rs:40: fn return_value(&self) -> Option<&MirNodeBox> { None }
src/mir/ir.rs:41: fn as_drop(&self) -> Option<(VarId, &HirType)> { None }
src/mir/ir.rs:42: fn as_retain(&self) -> Option<(VarId, &HirType)> { None }
src/mir/ir.rs:43: fn as_release(&self) -> Option<(VarId, &HirType)> { None }
src/mir/ir.rs:47: pub struct MirStmtBox(pub Box<dyn MirStmtNode>);
src/mir/ir.rs:48: impl Clone for MirStmtBox
src/mir/ir.rs:49: fn clone(&self) -> Self { MirStmtBox(self.0.clone_stmt()) }
src/mir/ir.rs:51: impl std::ops::Deref for MirStmtBox
src/mir/ir.rs:52: type Target = dyn MirStmtNode;
src/mir/ir.rs:53: fn deref(&self) -> &Self::Target { &*self.0 }
src/mir/ir.rs:59: pub struct $name { $(pub $field: $ty),* }
src/mir/ir.rs:67: pub struct $name { $(pub $field: $ty),* }
src/mir/ir.rs:115: fn from(v: $ty) -> Self { MirNodeBox(Box::new(v) as Box<dyn MirNode>) }
src/mir/ir.rs:124: fn from(v: $ty) -> Self { MirStmtBox(Box::new(v) as Box<dyn MirStmtNode>) }
src/mir/ir.rs:131: pub struct MirLocal
src/mir/ir.rs:137: impl MirLocal
src/mir/ir.rs:138: pub fn new(name: Symbol, ty: HirType, mutable: bool) -> Self
src/mir/ir.rs:144: pub struct MirFn
src/mir/ir.rs:156: pub enum MirItem
src/mir/ir.rs:169: pub struct MirProgram
src/mir/lower.rs:8: fn strategy_for(ty: &HirType, struct_defs: &HashMap<Symbol, Vec<(Symbol, HirType)>>) -> Box<dyn MemStrategy>
src/mir/lower.rs:35: fn action_to_stmt(_var: VarId, ty: &HirType, action: &MemAction) -> MirStmtBox
src/mir/lower.rs:43: pub fn lower_program(hir: &HirProgram) -> MirProgram
src/mir/lower.rs:61: fn lower_item(item: &HirItem, struct_defs: &HashMap<Symbol, Vec<(Symbol, HirType)>>) -> Vec<MirItem>
src/mir/lower.rs:76: fn lower_fn(f: &HirFn, struct_defs: &HashMap<Symbol, Vec<(Symbol, HirType)>>) -> MirFn
src/mir/lower.rs:142: struct Ctx
src/mir/lower.rs:150: impl Ctx
src/mir/lower.rs:151: fn new(f: &HirFn, struct_defs: &HashMap<Symbol, Vec<(Symbol, HirType)>>) -> Self
src/mir/lower.rs:172: fn emit_assign_cleanup(&self, var: &VarId) -> Vec<MirStmtBox>
src/mir/lower.rs:184: fn lower_stmt(&mut self, stmt: &HirStmt) -> Vec<MirStmtBox>
src/mir/lower.rs:219: fn track_stmt_moves(&mut self, stmt: &HirStmt)
src/mir/lower.rs:252: fn lower_assign(&mut self, target: &HirNodeBox, value: &HirNodeBox) -> Vec<MirStmtBox>
src/mir/lower.rs:296: fn lower_return(&mut self, value: &Option<HirNodeBox>) -> Vec<MirStmtBox>
src/mir/lower.rs:341: fn lower_if(
src/mir/lower.rs:361: fn lower_while(&mut self, cond: &HirNodeBox, body: &HirBlock) -> Vec<MirStmtBox>
src/mir/lower.rs:367: fn lower_block(&mut self, stmts: &[HirStmt]) -> Vec<MirStmtBox>
src/mir/lower.rs:376: fn mark_alive(&mut self, var: VarId)
src/mir/mem/mod.rs:4: pub enum MemAction
src/mir/mem/mod.rs:10: pub trait MemStrategy
src/mir/mem/mod.rs:11: fn on_scope_end(&self, var: VarId, ty: &HirType) -> Vec<MemAction>;
src/mir/mem/mod.rs:12: fn on_move_out(&self, var: VarId, ty: &HirType) -> Vec<MemAction>;
src/mir/mem/mod.rs:13: fn on_clone(&self, var: VarId, ty: &HirType) -> Vec<MemAction>;
src/mir/mem/mod.rs:14: fn on_assign_overwrite(&self, var: VarId, ty: &HirType) -> Vec<MemAction>;
src/mir/mem/mod.rs:17: pub mod value;
src/mir/mem/mod.rs:18: pub mod unique;
src/mir/mem/mod.rs:19: pub mod shared;
src/mir/mem/mod.rs:20: pub mod struct_strategy;
src/mir/mem/shared.rs:4: pub struct SharedStrategy;
src/mir/mem/shared.rs:6: impl MemStrategy for SharedStrategy
src/mir/mem/shared.rs:7: fn on_scope_end(&self, var: VarId, _ty: &HirType) -> Vec<MemAction>
src/mir/mem/shared.rs:10: fn on_move_out(&self, _var: VarId, _ty: &HirType) -> Vec<MemAction>
src/mir/mem/shared.rs:13: fn on_clone(&self, var: VarId, _ty: &HirType) -> Vec<MemAction>
src/mir/mem/shared.rs:16: fn on_assign_overwrite(&self, var: VarId, _ty: &HirType) -> Vec<MemAction>
src/mir/mem/struct_strategy.rs:7: pub struct StructStrategy
src/mir/mem/struct_strategy.rs:12: impl MemStrategy for StructStrategy
src/mir/mem/struct_strategy.rs:13: fn on_scope_end(&self, var: VarId, _ty: &HirType) -> Vec<MemAction>
src/mir/mem/struct_strategy.rs:21: fn on_move_out(&self, var: VarId, _ty: &HirType) -> Vec<MemAction>
src/mir/mem/struct_strategy.rs:26: fn on_clone(&self, var: VarId, _ty: &HirType) -> Vec<MemAction>
src/mir/mem/struct_strategy.rs:35: fn on_assign_overwrite(&self, var: VarId, _ty: &HirType) -> Vec<MemAction>
src/mir/mem/unique.rs:4: pub struct UniqueStrategy;
src/mir/mem/unique.rs:6: impl MemStrategy for UniqueStrategy
src/mir/mem/unique.rs:7: fn on_scope_end(&self, var: VarId, _ty: &HirType) -> Vec<MemAction>
src/mir/mem/unique.rs:10: fn on_move_out(&self, _var: VarId, _ty: &HirType) -> Vec<MemAction>
src/mir/mem/unique.rs:13: fn on_clone(&self, _var: VarId, _ty: &HirType) -> Vec<MemAction>
src/mir/mem/unique.rs:16: fn on_assign_overwrite(&self, var: VarId, _ty: &HirType) -> Vec<MemAction>
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
src/package/config.rs:5: pub struct ProjectConfig
src/package/config.rs:13: impl ProjectConfig
src/package/config.rs:14: pub fn load(toml_content: &str) -> Self
src/package/config.rs:60: pub fn resolve_import<'a>(&'a self, import_path: &str, base_dir: &Path) -> Option<String>
src/package/config.rs:77: pub fn resolve_target(&self, file_path: &Path) -> &str
src/package/mod.rs:1: pub mod config;
src/package/mod.rs:7: pub enum TargetType
src/package/mod.rs:13: impl TargetType
src/package/mod.rs:14: pub fn as_str(&self) -> &'static str
src/package/mod.rs:22: pub fn from_str(s: &str) -> Option<Self>
src/package/mod.rs:33: pub struct Package
src/package/mod.rs:43: pub enum PackageSymbol
src/package/mod.rs:59: impl Package
src/package/mod.rs:60: pub fn new(name: String, version: String) -> Self
src/package/mod.rs:73: pub fn collect_symbols(&mut self, stmts: &[Stmt])
src/package/mod.rs:78: pub fn collect_all_symbols(&mut self, stmts: &[Stmt])
src/package/mod.rs:82: fn collect_symbols_with_prefix(&mut self, stmts: &[Stmt], all: bool, ns_prefix: &str)
src/package/mod.rs:88: fn collect_stmt_symbols(&mut self, stmt: &Stmt, all: bool, ns_prefix: &str)
src/package/mod.rs:192: pub fn to_bytes(&self) -> Vec<u8>
src/package/mod.rs:252: pub fn write_to_file(&self, path: &str) -> Result<(), String>
src/package/mod.rs:260: pub enum ImportedSymbol
src/package/mod.rs:277: pub fn load_package(path: &str) -> Result<(Vec<ImportedSymbol>, Vec<String>, Vec<u8>, Vec<TargetType>), String>
src/package/mod.rs:345: fn parse_ini_value(s: &str) -> String
src/package/mod.rs:354: fn type_to_string(ty: &Type) -> String
src/parser/ast/binary_op.rs:2: pub enum BinaryOp
src/parser/ast/block.rs:5: pub struct Block
src/parser/ast/block.rs:10: impl Block
src/parser/ast/block.rs:11: pub fn new(stmts: Vec<Stmt>, span: Span) -> Self
src/parser/ast/expr.rs:9: pub enum Expr
src/parser/ast/expr.rs:95: impl Expr
src/parser/ast/expr.rs:96: pub fn span(&self) -> Span
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
src/parser/ast/stmt.rs:9: pub struct InterfaceMethod
src/parser/ast/stmt.rs:17: pub enum EnumFields
src/parser/ast/stmt.rs:24: pub struct EnumVariant
src/parser/ast/stmt.rs:30: pub struct MatchArm
src/parser/ast/stmt.rs:37: pub enum Stmt
src/parser/ast/stmt.rs:144: impl Stmt
src/parser/ast/stmt.rs:145: pub fn span(&self) -> Span
src/parser/ast/ty.rs:5: pub enum Type
src/parser/ast/ty.rs:24: impl Type
src/parser/ast/ty.rs:25: pub fn span(&self) -> Span
src/parser/ast/ty.rs:45: pub fn inner(&self) -> Option<&Type>
src/parser/ast/unary_op.rs:2: pub enum UnaryOp
src/parser/ast/vis.rs:2: pub enum Visibility
src/parser/ast/vis.rs:8: impl Visibility
src/parser/ast/vis.rs:9: pub fn is_public(&self) -> bool
src/parser/gen_bridge.rs:9: pub fn node_to_program(node: &Node) -> Result<crate::parser::ast::block::Block, String>
src/parser/gen_bridge.rs:18: pub fn node_to_stmt(node: &Node) -> Result<Stmt, String>
src/parser/gen_bridge.rs:93: pub fn node_to_expr(node: &Node) -> Result<Expr, String>
src/parser/gen_bridge.rs:243: pub fn type_from_node(node: &Node) -> Result<Type, String>
src/parser/gen_bridge.rs:286: fn get_str(node: &Node, field: &str) -> Result<String, String>
src/parser/gen_bridge.rs:291: fn str_to_binop(s: &str) -> Result<BinaryOp, String>
src/parser/gen_bridge.rs:303: fn params_from_node(node: &Node) -> Result<Vec<(Symbol, Type)>, String>
src/parser/gen_bridge.rs:313: fn block_from_node(node: &Node) -> Result<crate::parser::ast::block::Block, String>
src/parser/gen_bridge.rs:318: fn default_span() -> Span
src/parser/mod.rs:7: pub mod parser;
src/parser/mod.rs:8: pub mod ast;
src/parser/mod.rs:9: pub mod gen_bridge;
src/parser/mod.rs:15: pub fn parse_source(source: &str) -> Result<crate::parser::ast::program::Program, String>
src/parser/mod.rs:32: fn fallback_parse(source: &str) -> Result<crate::parser::ast::program::Program, String>
src/parser/parser.rs:7: pub struct Parser
src/parser/parser.rs:12: impl Parser
src/parser/parser.rs:13: pub fn new(tokens: Vec<Token>) -> Self
src/parser/parser.rs:17: fn peek(&self) -> Option<&Token>
src/parser/parser.rs:21: fn advance(&mut self) -> Option<Token>
src/parser/parser.rs:27: fn error(&self, msg: &str) -> String
src/parser/parser.rs:35: fn expect_keyword(&mut self, kw: Keyword) -> Result<(), String>
src/parser/parser.rs:46: fn expect_delimiter(&mut self, d: Delimiter) -> Result<(), String>
src/parser/parser.rs:57: fn expect_operator(&mut self, op: &str) -> Result<(), String>
src/parser/parser.rs:69: fn is_stmt_only_keyword(kind: &TokenKind) -> bool
src/parser/parser.rs:80: fn is_stmt_start(kind: &TokenKind) -> bool
src/parser/parser.rs:93: fn try_semicolon(&mut self) -> Result<(), String>
src/parser/parser.rs:104: fn parse_visibility(&mut self) -> Visibility
src/parser/parser.rs:125: fn expect_identifier(&mut self) -> Result<String, String>
src/parser/parser.rs:143: pub fn parse_program(&mut self) -> Result<Program, String>
src/parser/parser.rs:153: fn parse_stmt(&mut self) -> Result<Stmt, String>
src/parser/parser.rs:217: fn parse_any_assign_or_expr(&mut self) -> Result<Stmt, String>
src/parser/parser.rs:243: fn is_type_start(&self, pos: usize) -> bool
src/parser/parser.rs:251: fn parse_lambda(&mut self) -> Result<Expr, String>
src/parser/parser.rs:286: fn parse_fn_decl(&mut self, vis: Visibility, is_inline: bool, extern_c: bool) -> Result<Stmt, String>
src/parser/parser.rs:356: fn parse_return(&mut self) -> Result<Stmt, String>
src/parser/parser.rs:372: fn parse_if(&mut self) -> Result<Stmt, String>
src/parser/parser.rs:404: fn parse_for(&mut self) -> Result<Stmt, String>
src/parser/parser.rs:431: fn parse_while(&mut self) -> Result<Stmt, String>
src/parser/parser.rs:441: fn parse_match_stmt(&mut self) -> Result<Stmt, String>
src/parser/parser.rs:472: fn parse_namespace(&mut self, vis: Visibility) -> Result<Stmt, String>
src/parser/parser.rs:495: fn parse_struct_def(&mut self, vis: Visibility) -> Result<Stmt, String>
src/parser/parser.rs:534: fn parse_enum_def(&mut self, vis: Visibility) -> Result<Stmt, String>
src/parser/parser.rs:602: fn parse_interface_def(&mut self) -> Result<Stmt, String>
src/parser/parser.rs:645: fn parse_interface_method(&mut self) -> Result<InterfaceMethod, String>
src/parser/parser.rs:703: fn parse_import(&mut self) -> Result<Stmt, String>
src/parser/parser.rs:718: fn parse_impl_block(&mut self) -> Result<Stmt, String>
src/parser/parser.rs:744: fn extract_type_name(ty: &Type) -> Symbol
src/parser/parser.rs:780: fn parse_impl_method(&mut self, impl_type: &Symbol, impl_generic_params: &[(Symbol, Option<Symbol>)]) -> Result<Stmt, String>
src/parser/parser.rs:912: fn parse_block(&mut self) -> Result<Block, String>
src/parser/parser.rs:928: fn parse_expr(&mut self) -> Result<Expr, String>
src/parser/parser.rs:932: fn parse_or(&mut self) -> Result<Expr, String>
src/parser/parser.rs:948: fn parse_and(&mut self) -> Result<Expr, String>
src/parser/parser.rs:964: fn parse_compare(&mut self) -> Result<Expr, String>
src/parser/parser.rs:997: fn parse_sum(&mut self) -> Result<Expr, String>
src/parser/parser.rs:1026: fn parse_product(&mut self) -> Result<Expr, String>
src/parser/parser.rs:1056: fn parse_unary(&mut self) -> Result<Expr, String>
src/parser/parser.rs:1115: fn parse_postfix(&mut self) -> Result<Expr, String>
src/parser/parser.rs:1242: fn parse_atom(&mut self) -> Result<Expr, String>
src/parser/parser.rs:1515: fn parse_type(&mut self) -> Result<Type, String>
src/parser/parser.rs:1545: fn parse_base_type(&mut self) -> Result<Type, String>
src/parser/parser.rs:1619: fn handle_path_sep(&mut self, name_str: &mut String, name_sym: &mut Symbol) -> Result<(), String>
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
std/io.aya:8: pub fn getchar() -> int
std/io.aya:11: pub fn putchar(int c)
std/io.aya:14: pub fn print(unique String n)
std/io.aya:17: pub fn println()
std/io.aya:20: pub fn println(unique String s)
std/main.aya:4: fn main()->int
std/math.aya:4: pub fn abs(int x) -> int
std/math.aya:9: pub fn min(int a, int b) -> int
std/math.aya:14: pub fn max(int a, int b) -> int
std/math.aya:19: pub fn clamp(int x, int lo, int hi) -> int
std/math.aya:25: pub fn pow(int base, int exp) -> int
std/src/arraylist.aya:4: pub struct ArrayList[T:ToString]
std/src/arraylist.aya:10: impl[T:ToString] ArrayList[T]
std/src/arraylist.aya:11: fn expand(shared self)
std/src/arraylist.aya:23: pub fn push(shared self, T val)
std/src/arraylist.aya:29: pub fn index(shared self, int index)->T{ return self.data[index] }
std/src/arraylist.aya:30: pub fn to_string(shared self)->unique String
std/src/arraylist.aya:39: pub fn len(shared self)->int{ return self.len }
std/src/arraylist.aya:41: pub fn iter(shared self, fn(T) f)
std/src/io.aya:8: pub fn getchar() -> int
std/src/io.aya:11: pub fn putchar(int c)
std/src/io.aya:14: pub fn print(unique String n)
std/src/io.aya:17: pub fn println()
std/src/io.aya:20: pub fn println(unique String s)
std/src/linkedlist.aya:4: struct LinkedListNode[T:ToString]
std/src/linkedlist.aya:10: pub struct LinkedList[T:ToString]
std/src/linkedlist.aya:17: pub fn new[T:ToString]()->shared LinkedList[T]
std/src/linkedlist.aya:22: impl[T:ToString] LinkedList[T]
std/src/linkedlist.aya:23: pub fn push(shared self, T val)
std/src/linkedlist.aya:37: pub fn index(shared self, int index)->T
std/src/linkedlist.aya:43: pub fn len(shared self)->int{ return self.len }
std/src/linkedlist.aya:45: pub fn iter(shared self, fn(T) f)
std/src/linkedlist.aya:53: pub fn to_string(shared self) -> unique String
std/src/list.aya:1: pub interface List[T:ToString]
std/src/list.aya:2: fn push(shared self, T val);
std/src/list.aya:3: fn index(shared self, int index)->T;
std/src/list.aya:4: fn to_string(shared self)->unique String;
std/src/list.aya:5: fn len(shared self)->int;
std/src/list.aya:6: fn iter(shared self, fn(T) f);
std/src/math.aya:4: pub fn abs(int x) -> int
std/src/math.aya:9: pub fn min(int a, int b) -> int
std/src/math.aya:14: pub fn max(int a, int b) -> int
std/src/math.aya:19: pub fn clamp(int x, int lo, int hi) -> int
std/src/math.aya:25: pub fn pow(int base, int exp) -> int
std/src/std.aya:9: pub interface Error
std/src/std.aya:10: fn what(shared self) -> shared String;
std/src/std.aya:13: pub enum Result[T, E]
std/src/std.aya:18: impl[T, E] Result[T, E]
std/src/std.aya:19: pub fn try_unwrap(unique self) -> T
std/src/string.aya:1: pub struct String
std/src/string.aya:6: pub interface ToString
std/src/string.aya:7: fn to_string(unique self)->unique String;
std/src/string.aya:10: pub interface Error
std/src/string.aya:11: fn what(shared self) -> shared String;
std/src/string.aya:14: pub enum Result[T, E]
std/src/string.aya:19: impl[T, E] Result[T, E]
std/src/string.aya:20: pub fn try_unwrap(unique self) -> T
std/src/string.aya:28: impl int
std/src/string.aya:29: fn to_string(unique self) -> unique String
std/src/string.aya:65: impl float
std/src/string.aya:66: fn to_string(unique self) -> unique String
std/src/string.aya:73: impl char
std/src/string.aya:74: fn to_string(unique self) -> unique String
std/src/string.aya:79: impl bool
std/src/string.aya:80: fn to_string(unique self) -> unique String
std/src/string.aya:93: impl int
std/src/string.aya:94: pub fn add(unique self, int other) -> int { return self + other }
std/src/string.aya:95: pub fn sub(unique self, int other) -> int { return self - other }
std/src/string.aya:96: pub fn mul(unique self, int other) -> int { return self * other }
std/src/string.aya:97: pub fn div(unique self, int other) -> int { return self / other }
std/src/string.aya:98: pub fn rem(unique self, int other) -> int { return self % other }
std/src/string.aya:100: pub fn eq(unique self, int other) -> bool { return self == other }
std/src/string.aya:101: pub fn ne(unique self, int other) -> bool { return self != other }
std/src/string.aya:102: pub fn lt(unique self, int other) -> bool { return self < other }
std/src/string.aya:103: pub fn gt(unique self, int other) -> bool { return self > other }
std/src/string.aya:104: pub fn le(unique self, int other) -> bool { return self <= other }
std/src/string.aya:105: pub fn ge(unique self, int other) -> bool { return self >= other }
std/src/string.aya:107: pub fn neg(unique self) -> int { return -self }
std/src/string.aya:114: impl float
std/src/string.aya:115: pub fn add(unique self, float other) -> float { return self + other }
std/src/string.aya:116: pub fn sub(unique self, float other) -> float { return self - other }
std/src/string.aya:117: pub fn mul(unique self, float other) -> float { return self * other }
std/src/string.aya:118: pub fn div(unique self, float other) -> float { return self / other }
std/src/string.aya:120: pub fn eq(unique self, float other) -> bool { return self == other }
std/src/string.aya:121: pub fn ne(unique self, float other) -> bool { return self != other }
std/src/string.aya:122: pub fn lt(unique self, float other) -> bool { return self < other }
std/src/string.aya:123: pub fn gt(unique self, float other) -> bool { return self > other }
std/src/string.aya:124: pub fn le(unique self, float other) -> bool { return self <= other }
std/src/string.aya:125: pub fn ge(unique self, float other) -> bool { return self >= other }
std/src/string.aya:127: pub fn neg(unique self) -> float { return -self }
std/src/string.aya:134: impl char
std/src/string.aya:135: pub fn eq(unique self, char other) -> bool { return self == other }
std/src/string.aya:136: pub fn ne(unique self, char other) -> bool { return self != other }
std/src/string.aya:137: pub fn lt(unique self, char other) -> bool { return self < other }
std/src/string.aya:138: pub fn gt(unique self, char other) -> bool { return self > other }
std/src/string.aya:139: pub fn le(unique self, char other) -> bool { return self <= other }
std/src/string.aya:140: pub fn ge(unique self, char other) -> bool { return self >= other }
std/src/string.aya:147: impl bool
std/src/string.aya:148: pub fn eq(unique self, bool other) -> bool { return self == other }
std/src/string.aya:149: pub fn ne(unique self, bool other) -> bool { return self != other }
std/src/string.aya:156: impl String
std/src/string.aya:157: pub fn to_string(unique self) -> unique String
std/src/string.aya:161: pub fn index(unique self, int i) -> char
std/src/string.aya:165: pub fn len(unique self) -> int
std/src/string.aya:169: pub fn add(unique self, unique String other) -> unique String
std/src/string.aya:183: pub fn add[T:ToString](unique self, T a) -> unique String
std/src/string.aya:188: pub fn eq(unique self, unique String other) -> bool
std/src/string.aya:200: pub fn ne(unique self, unique String other) -> bool
std/src/string.aya:204: pub fn copy(unique self) -> unique String
std/string.aya:1: pub struct String
std/string.aya:6: pub interface ToString
std/string.aya:7: fn to_string(unique self)->unique String;
std/string.aya:10: impl int
std/string.aya:11: fn to_string(unique self) -> unique String
std/string.aya:47: impl float
std/string.aya:48: fn to_string(unique self) -> unique String
std/string.aya:55: impl char
std/string.aya:56: fn to_string(unique self) -> unique String
std/string.aya:61: impl bool
std/string.aya:62: fn to_string(unique self) -> unique String
std/string.aya:75: impl int
std/string.aya:76: pub fn add(unique self, int other) -> int { return self + other }
std/string.aya:77: pub fn sub(unique self, int other) -> int { return self - other }
std/string.aya:78: pub fn mul(unique self, int other) -> int { return self * other }
std/string.aya:79: pub fn div(unique self, int other) -> int { return self / other }
std/string.aya:80: pub fn rem(unique self, int other) -> int { return self % other }
std/string.aya:82: pub fn eq(unique self, int other) -> bool { return self == other }
std/string.aya:83: pub fn ne(unique self, int other) -> bool { return self != other }
std/string.aya:84: pub fn lt(unique self, int other) -> bool { return self < other }
std/string.aya:85: pub fn gt(unique self, int other) -> bool { return self > other }
std/string.aya:86: pub fn le(unique self, int other) -> bool { return self <= other }
std/string.aya:87: pub fn ge(unique self, int other) -> bool { return self >= other }
std/string.aya:89: pub fn neg(unique self) -> int { return -self }
std/string.aya:96: impl float
std/string.aya:97: pub fn add(unique self, float other) -> float { return self + other }
std/string.aya:98: pub fn sub(unique self, float other) -> float { return self - other }
std/string.aya:99: pub fn mul(unique self, float other) -> float { return self * other }
std/string.aya:100: pub fn div(unique self, float other) -> float { return self / other }
std/string.aya:102: pub fn eq(unique self, float other) -> bool { return self == other }
std/string.aya:103: pub fn ne(unique self, float other) -> bool { return self != other }
std/string.aya:104: pub fn lt(unique self, float other) -> bool { return self < other }
std/string.aya:105: pub fn gt(unique self, float other) -> bool { return self > other }
std/string.aya:106: pub fn le(unique self, float other) -> bool { return self <= other }
std/string.aya:107: pub fn ge(unique self, float other) -> bool { return self >= other }
std/string.aya:109: pub fn neg(unique self) -> float { return -self }
std/string.aya:116: impl char
std/string.aya:117: pub fn eq(unique self, char other) -> bool { return self == other }
std/string.aya:118: pub fn ne(unique self, char other) -> bool { return self != other }
std/string.aya:119: pub fn lt(unique self, char other) -> bool { return self < other }
std/string.aya:120: pub fn gt(unique self, char other) -> bool { return self > other }
std/string.aya:121: pub fn le(unique self, char other) -> bool { return self <= other }
std/string.aya:122: pub fn ge(unique self, char other) -> bool { return self >= other }
std/string.aya:129: impl bool
std/string.aya:130: pub fn eq(unique self, bool other) -> bool { return self == other }
std/string.aya:131: pub fn ne(unique self, bool other) -> bool { return self != other }
std/string.aya:138: impl String
std/string.aya:139: pub fn to_string(unique self) -> unique String
std/string.aya:143: pub fn index(unique self, int i) -> char
std/string.aya:147: pub fn len(unique self) -> int
std/string.aya:151: pub fn add(unique self, unique String other) -> unique String
std/string.aya:165: pub fn add[T:ToString](unique self, T a) -> unique String
std/string.aya:170: pub fn eq(unique self, unique String other) -> bool
std/string.aya:182: pub fn ne(unique self, unique String other) -> bool
std/string.aya:186: pub fn copy(unique self) -> unique String
example/math_lib.aya:1: pub fn add(int a, int b) -> int
example/test.aya:1: fn main()->int
example/test.aya:11: fn foo(unique int x, shared float y, weak char z)->unique int
example/test.aya:16: fn add(int a, int b)->int
example/test.aya:19: fn sub(int a, int b)->int
example/test_array.aya:1: fn main() -> int
example/test_array_param.aya:1: fn sum(unique [int] arr) -> int
example/test_array_param.aya:5: fn main() -> int
example/test_asm.aya:1: fn main() -> int
example/test_bool.aya:1: fn main() -> int
example/test_bool2.aya:1: fn main() -> int
example/test_comments.aya:3: fn main() -> int
example/test_conv.aya:1: fn main() -> int
example/test_import.aya:3: fn main() -> int
example/test_ns.aya:2: fn add(int a, int b) -> int
example/test_ns.aya:7: fn main() -> int
example/test_ns_main.aya:2: fn add(int a, int b) -> int
example/test_ns_main.aya:7: fn main() -> int
example/test_oop.aya:1: interface Drawable
example/test_oop.aya:2: fn draw(shared self) -> void;
example/test_oop.aya:3: fn get_id(shared self) -> int;
example/test_oop.aya:6: interface Resizable
example/test_oop.aya:7: fn resize(shared self, int factor) -> int;
example/test_oop.aya:10: impl int
example/test_oop.aya:11: fn draw(shared self) -> void {}
example/test_oop.aya:12: fn get_id(shared self) -> int { return self; }
example/test_oop.aya:13: fn resize(shared self, int factor) -> int { return self + factor; }
example/test_oop.aya:14: fn triple(unique self) -> int { return self * 3; }
example/test_oop.aya:17: impl float
example/test_oop.aya:18: fn draw(shared self) -> void {}
example/test_oop.aya:19: fn get_id(shared self) -> int { return 0; }
example/test_oop.aya:20: fn resize(shared self, int factor) -> int { return 0; }
example/test_oop.aya:23: fn render_drawable(shared Drawable d) -> void
example/test_oop.aya:27: fn get_any_id(shared Drawable d) -> int
example/test_oop.aya:31: fn scale(shared Resizable r, int f) -> int
example/test_oop.aya:35: fn main() -> int
example/test_op_overload.aya:1: struct Point
example/test_op_overload.aya:6: impl Point
example/test_op_overload.aya:7: fn add(shared self, shared Point other) -> Point
example/test_op_overload.aya:14: fn main() -> int
example/test_overload.aya:1: fn main()->int
example/test_overload.aya:5: fn add(int a, int b)->int
example/test_overload.aya:9: fn add(float a, float b)->float
example/test_poly.aya:1: interface Shape
example/test_poly.aya:2: fn area(shared self) -> int;
example/test_poly.aya:3: fn describe(shared self) -> void;
example/test_poly.aya:6: impl int
example/test_poly.aya:7: fn area(shared self) -> int
example/test_poly.aya:10: fn describe(shared self) -> void {}
example/test_poly.aya:13: impl float
example/test_poly.aya:14: fn area(shared self) -> int
example/test_poly.aya:17: fn describe(shared self) -> void {}
example/test_poly.aya:20: fn double_area(shared Shape s) -> int
example/test_poly.aya:25: fn main() -> int
example/test_self.aya:1: fn main() -> int
example/test_shared_struct.aya:1: struct Point
example/test_shared_struct.aya:6: fn main() -> int
example/test_std.aya:3: fn main() -> int
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
example/test_unique_struct.aya:1: struct Point
example/test_unique_struct.aya:6: fn main() -> int
example/test_vis.aya:1: pub fn main() -> int
example/test_vis2.aya:6: pub fn main() -> int
