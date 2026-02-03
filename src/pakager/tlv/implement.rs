use crate::pakager::tlv::{Tlv, TlvType};

impl Tlv{
    pub(crate) fn new(ty: TlvType,value:Vec<u8>)->Self{
        Tlv{
            ty,
            value,
        }
    }

    pub(crate) fn as_bytes(&self)->Vec<u8>{
        let mut bytes = Vec::new();
        bytes.extend(&(self.ty as u8).to_le_bytes());
        bytes.extend(&(self.value.len() as u32).to_le_bytes());
        bytes.extend(&self.value);
        bytes
    }
}

