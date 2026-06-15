// ============================================================
//  Ayanami 编译器 —— 库入口
//  模块结构（按编译流程）：
//  1. lexer      → 词法分析（源码 → Token 流）
//  2. parser     → 语法分析（Token 流 → AST）
//  3. hir        → 高级中间表示（名称解析、类型检查、接口分发）
//  4. mir        → 中级中间表示（内存管理插入、借用检查）
//  5. lir        → 低级中间表示（三地址码、基本块）
//  6. compiler   → 编译管线编排（解析、降级、构建、打包）
//  7. driver     → 代码生成驱动（llc 编译、gcc 链接）
//  8. package    → .lcl 包格式（序列化/反序列化）
//  9. intern     → 字符串驻留（Symbol 类型）
// 10. formatter  → 代码格式化
// 11. span       → 源代码位置跟踪
// ============================================================

pub mod generated;
pub mod compiler;
pub mod driver;
pub mod formatter;
pub mod hir;
pub mod intern;
pub mod lexer;
pub mod lir;
pub mod mir;
pub mod package;
pub mod parser;
pub mod span;
