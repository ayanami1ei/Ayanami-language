//! Ayanami 编译器统一错误类型。
//!
//! 所有跨模块、对外返回的错误一律使用 [`Error`]；禁止新增 `Result<_, String>`。
//! 错误按编译阶段分变体，消息保留原有诊断文本。

use std::path::PathBuf;

/// 编译各阶段错误的统一表示。
#[derive(Debug, thiserror::Error)]
pub enum Error {
    /// IO 错误（文件读写、目录创建等）。
    #[error("io error: {0}")]
    Io(#[from] std::io::Error),

    /// 词法/语法错误。
    #[error("parse error: {0}")]
    Parse(String),

    /// 名称解析、类型检查、HIR 降级错误。
    #[error("hir error: {0}")]
    Hir(String),

    /// MIR 借用检查错误。
    #[error("borrow error: {0}")]
    Borrow(String),

    /// LIR 序列化/反序列化错误。
    #[error("serialize error: {0}")]
    Serialize(String),

    /// 导入/依赖解析错误。
    #[error("import error: {0}")]
    Import(String),

    /// 编译管线错误（构建目标、链接编排、产物生成等）。
    #[error("compile error: {0}")]
    Compile(String),

    /// .lcl 包读写错误。
    #[error("package error: {0}")]
    Package(String),

    /// 外部工具（llc/gcc/ar）与代码生成驱动错误。
    #[error("driver error: {0}")]
    Driver(String),

    /// 代码格式化错误。
    #[error("format error: {0}")]
    Format(String),

    /// CLI 参数与文件读取错误。
    #[error("cli error: {0}")]
    Cli(String),

    /// 循环导入（结构化变体，便于程序化处理）。
    #[error("circular import: {path}")]
    CircularImport { path: PathBuf },
}

/// 编译器统一的 Result 别名。
pub type Result<T> = std::result::Result<T, Error>;
