/// String interning: converts frequently-used strings to compact u32 IDs.
///
/// Symbol(u32) is used throughout the compiler for identifiers,
/// reducing string comparisons to integer comparisons.
pub mod interner;
pub mod symbol;

pub use symbol::Symbol;
