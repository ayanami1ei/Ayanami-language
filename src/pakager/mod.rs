use crate::symbol_table::Symbol;

pub(crate) mod implement;
pub(crate) mod tlv;

pub(crate) struct PackageHeader {
    magic: String,   // 文件标识
    version: String, // 格式版本
    flags: u16,      // 特性位
}

pub(crate) struct PakageInfo {
    pub(crate) name: String,
    pub(crate) version: String,
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
#[allow(dead_code)]
pub enum UseMode {
    AsDeveloper,
    #[default]
    AsUser,
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
#[allow(dead_code)]
pub enum Target {
    #[default]
    Executable,
    StaticLib,
    DynamicLib,
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
#[allow(dead_code)]
pub enum Arch {
    X86X64,
    X86,
    AArch64,
    Arm,
    RiscV64,
    Mips64,
    PowerPC64,
    S390x,
    #[default]
    Default,
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
#[allow(dead_code)]
pub enum System {
    Windows,
    Linux,
    #[default]
    Default,
}

pub(crate) struct Pakage {
    pub(crate) header: PackageHeader,
    pub(crate) info: PakageInfo,
    pub(crate) use_mode: UseMode,
    pub(crate) target: Target,
    pub(crate) arch: Arch,
    pub(crate) system: System,
    pub(crate) symbols: Vec<Symbol>,
    pub(crate) bc: Vec<u8>,
}
