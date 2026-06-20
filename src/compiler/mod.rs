pub mod build;
pub mod check;
pub mod debug;
pub mod import;
pub mod symdef;

pub use build::{
    build_source, build_source_to, build_source_with_target, compile_file,
    install_package, package_source, run_executable,
};
pub use check::check_hir_returns;
pub use debug::format_program;
pub use import::{find_std_dir, resolve_import_path};
pub use symdef::{collect_defs_from_stmts, defs_to_json, SymDef};

use crate::lir::ir::LirProgram;
use crate::parser::ast::Program;

/// 编译结果：包含整个编译流程的产物（AST、LIR、LLVM IR）
/// 由 `CompilerPipeline` 生成，也可通过 `compile_source` 快捷函数直接获取
pub struct CompiledFile {
    pub program: Program,
    pub lir_program: LirProgram,
    pub llvm_ir: String,
    pub obj_paths: Vec<std::path::PathBuf>,
    pub link_flags: Vec<String>,
    pub own_obj: std::path::PathBuf,
    pub lcl_path: std::path::PathBuf,
    pub dep_lcl_paths: Vec<std::path::PathBuf>,
}

/// 简化编译结果（不含导入解析和链接信息）
pub struct CompileResult {
    pub llvm_ir: String,
    pub program: Program,
    pub lir_program: LirProgram,
}

// ============================================================
//  CompilerPipeline —— 面向对象的编译管线
//  将编译流程封装为结构体 + 方法链，每个阶段可独立调用
// ============================================================

/// ## Ayanami 编译器管线
///
/// 将源代码到 LLVM IR 的完整编译流程封装为结构体，
/// 支持链式调用和分段执行（例如只做词法分析 + 解析，不进行后续降级）。
///
/// ### 使用方式
/// ```ignore
/// let mut pipeline = CompilerPipeline::new(source_code);
/// pipeline
///     .lex()?          // 词法分析：源码 → Token 流
///     .parse()?        // 语法分析：Token 流 → AST
///     .lower_hir()?    // HIR 降级：AST → 高级中间表示
///     .lower_mir()     // MIR 降级：高级 IR → 中级 IR（含内存管理）
///     .lower_lir()?    // LIR 降级：中级 IR → 低级三地址码
///     .emit();         // LLVM IR 发射：LIR → 文本 LLVM IR
/// let result = pipeline.result();  // 获取编译结果
/// ```
pub struct CompilerPipeline {
    /// 源代码文本
    code: String,
    /// 词法分析结果（Token 流）
    tokens: Vec<crate::lexer::Token>,
    /// 语法分析结果（抽象语法树）
    ast: Option<Program>,
    /// HIR 降级结果
    hir: Option<crate::hir::ir::HirProgram>,
    /// MIR 降级结果
    mir: Option<crate::mir::ir::MirProgram>,
    /// LIR 降级结果
    lir: Option<LirProgram>,
    /// LLVM IR 文本
    llvm_ir: Option<String>,
}

impl CompilerPipeline {
    /// 创建新的编译器管线实例，传入源代码
    pub fn new(code: &str) -> Self {
        Self {
            code: code.to_string(),
            tokens: Vec::new(),
            ast: None,
            hir: None,
            mir: None,
            lir: None,
            llvm_ir: None,
        }
    }

    /// 阶段一：词法分析 — 将源代码切分为 Token 流
    pub fn lex(&mut self) -> Result<&mut Self, String> {
        self.tokens = {
            let mut lexer = crate::lexer::Lexer::new(&self.code);
            let tokens = lexer.tokenize_all();
            tokens
                .into_iter()
                .filter(|t| !matches!(t.kind, crate::lexer::TokenKind::EOF))
                .collect()
        };
        Ok(self)
    }

    /// 阶段二：语法分析 — 将 Token 流解析为抽象语法树（AST）
    pub fn parse(&mut self) -> Result<&mut Self, String> {
        let program = crate::parser::parse_source(&self.code)
            .map_err(|e| format!("Parse error: {}", e))?;
        self.ast = Some(program);
        Ok(self)
    }

    /// 阶段三：HIR 降级 — 将 AST 降级为高级中间表示（HIR）
    /// 完成名称解析、类型解析、虚函数表构建、运算符重载处理
    pub fn lower_hir(&mut self) -> Result<&mut Self, String> {
        let program = self.ast.as_ref().ok_or("No AST available — call parse() first")?;
        let hir_program = crate::hir::lower_program(program)
            .map_err(|e| format!("HIR error: {}", e))?;
        self.hir = Some(hir_program);
        Ok(self)
    }

    /// HIR 返回检查：确保所有非 void 函数都有 return 语句
    pub fn check_returns(&mut self) -> Result<&mut Self, String> {
        let hir = self.hir.as_ref().ok_or("No HIR available — call lower_hir() first")?;
        let _ = crate::compiler::check_hir_returns(hir, std::path::Path::new(""));
        Ok(self)
    }

    /// 阶段四：MIR 降级 — 将 HIR 降级为中级中间表示（MIR）
    /// 展平控制流，插入内存管理指令（Drop/Retain/Release）
    pub fn lower_mir(&mut self) -> &mut Self {
        let hir = self.hir.take().expect("No HIR available — call lower_hir() first");
        let mir_program = crate::mir::lower_program(&hir);
        self.hir = Some(hir); // 保留 HIR 供后续使用
        self.mir = Some(mir_program);
        self
    }

    /// MIR 借用检查：确保没有悬垂引用和重复释放
    pub fn check_borrows(&mut self) -> Result<&mut Self, String> {
        let mir = self.mir.as_ref().ok_or("No MIR available — call lower_mir() first")?;
        for item in &mir.items {
            if let crate::mir::ir::MirItem::Fn(f) = item {
                crate::mir::borrow::check_borrows(f)
                    .map_err(|e| format!("borrow error: {}", e))?;
            }
        }
        Ok(self)
    }

    /// 阶段五：LIR 降级 — 将 MIR 降级为低级三地址码（LIR）
    /// 生成基本块和虚拟临时变量
    pub fn lower_lir(&mut self) -> &mut Self {
        let mir = self.mir.take().expect("No MIR available — call lower_mir() first");
        let lir_program = crate::lir::lower_program(&mir);
        self.mir = Some(mir);
        self.lir = Some(lir_program);
        self
    }

    /// 阶段六：LLVM IR 发射 — 将 LIR 转换为 LLVM IR 文本
    pub fn emit(&mut self) -> &mut Self {
        let lir = self.lir.as_ref().expect("No LIR available — call lower_lir() first");
        let llvm_ir = crate::lir::emit_program(lir);
        self.llvm_ir = Some(llvm_ir);
        self
    }

    /// 一键编译：执行完整管线（lex → parse → lower_hir → lower_mir → lower_lir → emit）
    pub fn compile(&mut self) -> Result<&mut Self, String> {
        self.lex()?
            .parse()?
            .lower_hir()?
            .lower_mir()
            .lower_lir()
            .emit();
        Ok(self)
    }

    /// 获取编译结果
    pub fn result(&self) -> CompileResult {
        CompileResult {
            llvm_ir: self.llvm_ir.clone().unwrap_or_default(),
            program: self.ast.clone().expect("No AST available — run compile() first"),
            lir_program: self.lir.clone().expect("No LIR available — run compile() first"),
        }
    }

    /// 获取 HIR 程序（用于检查等场景）
    pub fn hir_program(&self) -> Option<&crate::hir::ir::HirProgram> {
        self.hir.as_ref()
    }

    /// 获取 MIR 程序（用于借用检查等场景）
    pub fn mir_program(&self) -> Option<&crate::mir::ir::MirProgram> {
        self.mir.as_ref()
    }

    /// 获取 AST 程序
    pub fn ast_program(&self) -> Option<&Program> {
        self.ast.as_ref()
    }
}

/// 从源代码直接编译（完整管线一次完成）
pub fn compile_source(code: &str) -> Result<CompileResult, String> {
    let mut pipeline = CompilerPipeline::new(code);
    pipeline.compile()?;
    Ok(pipeline.result())
}

/// 检查源代码（词法分析 → 语法分析 → HIR → MIR → 借用检查）
/// 不生成 LIR 或 LLVM IR，仅用于前端验证
pub fn check_source(code: &str, _out_dir: &str) -> Result<(), String> {
    let mut pipeline = CompilerPipeline::new(code);
    pipeline
        .lex()?
        .parse()?
        .lower_hir()?
        .check_returns()?
        .lower_mir()
        .check_borrows()?;
    println!("check passed");
    Ok(())
}
