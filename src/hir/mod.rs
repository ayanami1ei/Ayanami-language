/// High-Level Intermediate Representation.
///
/// HIR is the first IR after parsing. It resolves:
/// - Variable names → VarId indices
/// - Function names → FnId indices (with overloading support)
/// - Interface definitions & vtable mapping
/// - Method calls → static Call or dynamic VirtualCall
/// - Concrete types → fat pointer wrapper (MakeFatPtr) for interface dispatch
/// - Memory ownership annotations (Move/Clone)
///
/// HIR still retains structured control flow (if/while/block).
pub mod ir;
pub mod lower;
pub mod display;

pub use ir::*;
pub use lower::lower_program;
pub use display::{display_hir_program, hir_program_to_string};
