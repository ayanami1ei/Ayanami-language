use std::collections::HashSet;
use crate::intern::Symbol;
use crate::mir::ir::*;
use crate::hir::*;
use crate::parser::ast::BinaryOp;

// ═══════════════════════════════════════════════════════════════════
//  impl HirNode for all 23 HIR struct types
// ═══════════════════════════════════════════════════════════════════

mod access;
mod basic;
mod call;
