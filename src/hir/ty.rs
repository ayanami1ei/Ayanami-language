use crate::intern::Symbol;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct VarId(pub usize);

/// Index into the global function table (Ctx::fns).
/// Used to statically resolve function calls.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct FnId(pub usize);

/// HIR type system.
///
/// Represents both concrete types (int, float, char, void, bool)
/// and type constructors (Unique/Shared/Weak pointers, Named symbols,
/// FatPtr for interface dispatch).
/// Named corresponds to user-defined types (structs).
/// FatPtr { data_ptr, vtable_ptr } is used for interface dispatch.
#[derive(Debug, Clone, PartialEq)]
pub enum HirType {
    Int,
    Float,
    /// M1.4：32 位浮点（`f32`）；`float` / `f64` 为 `Float`
    F32,
    Char,
    Void,
    /// M1.9：never 类型 `!`（发散表达式；可强转到任意类型）
    Never,
    Bool,
    Unique(Box<HirType>),
    Named(Symbol),
    /// Fat pointer `{ data_ptr, vtable_ptr }` for interface dispatch.
    /// `name` is the interface name, `kind` preserves ownership (Shared/Unique).
    FatPtr { name: Symbol, kind: Box<HirType> },
    /// Array of elements of the inner type (unsized, heap)
    Array(Box<HirType>),
    /// Sized array of elements of the inner type (stack-allocated if value, heap if shared/unique)
    ArraySized(Box<HirType>, usize),
    /// Reference: Ref(inner, mutable)
    Ref(Box<HirType>, bool),
    FnPtr(Vec<HirType>, Box<HirType>),
    /// M2：闭包类型 `Fn(T) -> U` / `FnOnce(T) -> U`（胖值 `{ env, vtable }`）。
    /// `owns_env = false`：静态闭包（命名函数/非捕获 lambda），env 为 null，**Copy**、零分配；
    /// `owns_env = true`：捕获闭包，拥有环境（非 Copy，作用域结束释放）。
    /// `once = true`：FnOnce（调用消费闭包，环境由被调用方释放；仅捕获闭包可为 FnOnce）。
    Closure(Vec<HirType>, Box<HirType>, bool, bool),
    /// M1：定宽整数（i8..i128 / u8..u128 / isize / usize）
    IntN { bits: u8, signed: bool },
}

impl HirType {
    /// Copy 类型：赋值/传参时不移动（仅基元、函数指针、静态闭包与借用）。
    pub fn is_copy(&self) -> bool {
        matches!(
            self,
            HirType::Int
                | HirType::Float
                | HirType::F32
                | HirType::Char
                | HirType::Bool
                | HirType::Void
                | HirType::Never
                | HirType::FnPtr(..)
                | HirType::Closure(_, _, false, _)
                | HirType::Ref(..)
                | HirType::IntN { .. }
        )
    }
}

#[derive(Debug, Clone)]
pub enum HirLiteral {
    Int(i64),
    Float(f64),
    Char(char),
    String(String),
    Bool(bool),
    /// M6.2c：数组常量（元素为标量字面量；仅 static 初始化用）
    Array(Vec<HirLiteral>),
    /// M6.2c：结构体常量（按声明顺序的字段值）
    Struct(Vec<HirLiteral>),
}
