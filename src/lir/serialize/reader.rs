use super::*;

impl<'a> Reader<'a> {
    pub(super) fn read(&mut self, n: usize) -> Result<&'a [u8]> {
        if *self.pos + n > self.data.len() {
            return Err(Error::Serialize("unexpected EOF".into()));
        }
        let slice = &self.data[*self.pos..*self.pos + n];
        *self.pos += n;
        Ok(slice)
    }
    pub(super) fn u32(&mut self) -> Result<u32> {
        let b = self.read(4)?;
        Ok(u32::from_le_bytes([b[0], b[1], b[2], b[3]]))
    }
    pub(super) fn u64(&mut self) -> Result<u64> {
        let b = self.read(8)?;
        Ok(u64::from_le_bytes([b[0], b[1], b[2], b[3], b[4], b[5], b[6], b[7]]))
    }
    pub(super) fn str(&mut self) -> Result<String> {
        let len = self.u32()? as usize;
        let b = self.read(len)?;
        Ok(String::from_utf8(b.to_vec()).map_err(|e| Error::Serialize(format!("invalid string: {}", e)))?)
    }
    pub(super) fn ty(&mut self) -> Result<HirType> {
        let tag = self.read(1)?[0];
        match tag {
            0 => Ok(HirType::Int),
            1 => Ok(HirType::Float),
            2 => Ok(HirType::Char),
            3 => Ok(HirType::Void),
            4 => Ok(HirType::Bool),
            5 => { let s = Symbol::intern(&self.str()?); Ok(HirType::Named(s)) }
            6 => Ok(HirType::Unique(Box::new(self.ty()?))),
            7 => Ok(HirType::Shared(Box::new(self.ty()?))),
            8 => Ok(HirType::Weak(Box::new(self.ty()?))),
            9 => {
                let name = Symbol::intern(&self.str()?);
                let kind = Box::new(self.ty()?);
                Ok(HirType::FatPtr { name, kind })
            }
            10 => Ok(HirType::Array(Box::new(self.ty()?))),
            11 => { let inner = Box::new(self.ty()?); let mutable = self.read(1)?[0] != 0; Ok(HirType::Ref(inner, mutable)) }
            12 => {
                let pc = self.u32()? as usize;
                let mut params = Vec::with_capacity(pc);
                for _ in 0..pc { params.push(self.ty()?); }
                let ret = Box::new(self.ty()?);
                Ok(HirType::FnPtr(params, ret))
            }
            _ => Err(Error::Serialize(format!("unknown type tag: {}", tag))),
        }
    }
}
