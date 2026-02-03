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
}
