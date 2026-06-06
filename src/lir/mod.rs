/// Low-Level Intermediate Representation (three-address code).
///
/// LIR flattens MIR into basic blocks with three-address instructions,
/// each operating on virtual temporaries (Tmp). This is a linear IR
/// close to LLVM IR — the emit pass is a straightforward 1:1 mapping.
///
/// Key transformations in LIR:
/// - Expressions broken into sequences of loads, binops, calls
/// - Fat pointer construction (MakeFatPtr) → malloc + store + insertvalue
/// - Virtual dispatch (VirtualCall) → extractvalue + GEP + load + indirect call
/// - Vtable globals generated as `constant [N x ptr]` arrays
pub mod ir;
pub mod lower;
pub mod display;
pub mod emit;

pub use ir::LirProgram;
pub use lower::lower_program;
pub use display::lir_program_to_string;
pub use emit::emit_program;
