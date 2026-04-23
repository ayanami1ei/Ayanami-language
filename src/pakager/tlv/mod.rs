pub(crate) mod implement;

pub(crate) struct Tlv {
    pub(crate) ty: TlvType,
    pub(crate) value: Vec<u8>,
}

#[derive(Debug, Clone, Copy)]
pub(crate) enum TlvType {
    //header
    Magic = 0x00,
    PakagerVersion = 0x01,
    Flags = 0x02,

    //info
    Name = 0x03,
    Version = 0x04,

    UseMode = 0x05,

    Target = 0x06,

    Arch = 0x07,

    Bc = 0x08,

    System = 0x09,

    Symbol=0x0A,
    SymbolsName = 0x0B,
    SymbolIsFunc=0x0C,
    SymbolArgs=0x0D,
    SymbolIsArgc=0x0E,
    SymbolIsRef=0x0F,
    SymbolIsVar=0x10,
    SymbolItsType=0x11,
    SymbolIsArr=0x12,
    SymbolElemType=0x13,
    SymbolId=0x14,
    SymbolLevel=0x15,
    SymbolScopeId=0x16,
    SymbolBodyScopeId=0x17,
}
