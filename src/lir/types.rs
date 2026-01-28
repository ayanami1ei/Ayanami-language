#[derive(Debug, Clone, Eq, PartialEq, Hash)]
#[repr(u32)]
pub(crate) enum VarType {
    Int = 1,
    Float = 2,
    Bool = 3,
    Char = 4,
    String = 5,
    Unknown = 6,
}

#[repr(C)]
pub(crate) struct Object {
    pub(crate) ty: VarType,
    pub(crate) refcnt: i32,
}

#[repr(C)]
pub struct IntObject {
    pub header: Object,
    pub value: i64,
}

#[repr(C)]
pub struct FloatObject {
    pub header: Object,
    pub value: f64,
}

#[repr(C)]
pub struct CharObject {
    pub header: Object,
    pub value: u32,
}

#[repr(C)]
pub struct BoolObject {
    pub header: Object,
    pub value: bool,
}

#[repr(C)]
pub struct StringObject {
    pub header: Object,
    pub len: usize,
    pub data: *const u8,
}
