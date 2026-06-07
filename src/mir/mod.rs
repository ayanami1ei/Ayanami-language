/// Mid-Level Intermediate Representation.
///
/// MIR lowers structured control flow (if/while/block in HIR) into
/// flat sequences of statements. It also inserts explicit memory
/// management actions (Drop/Retain/Release) based on ownership
/// strategies for each type (value, unique, shared).
pub mod ir;
pub mod lower;
pub mod display;
pub mod mem;
pub mod borrow;

pub use ir::*;
pub use lower::lower_program;
pub use display::{display_mir_program, mir_program_to_string};
