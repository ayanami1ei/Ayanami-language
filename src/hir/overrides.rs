use crate::hir::{BlockId, FuncId, ObjId, SlotId, VarId};
use std::ops::{Add, AddAssign, Sub};

impl AddAssign<i32> for VarId {
    fn add_assign(&mut self, other: i32) {
        self.0 += other;
    }
}
impl Add<i32> for VarId {
    type Output = VarId;
    fn add(self, rhs: i32) -> Self::Output {
        VarId(self.0 + rhs)
    }
}
impl Sub<i32> for VarId {
    type Output = VarId;
    fn sub(self, rhs: i32) -> Self::Output {
        VarId(self.0 - rhs)
    }
}

impl AddAssign<i32> for ObjId {
    fn add_assign(&mut self, other: i32) {
        self.id += other;
    }
}
impl Add<i32> for ObjId {
    type Output = ObjId;
    fn add(self, rhs: i32) -> Self::Output {
        ObjId {
            id: self.id + rhs,
            home_level_id: self.home_level_id,
            cur_leve_id: self.cur_leve_id,
        }
    }
}
impl Sub<i32> for ObjId {
    type Output = ObjId;
    fn sub(self, rhs: i32) -> Self::Output {
        ObjId {
            id: self.id - rhs,
            home_level_id: self.home_level_id,
            cur_leve_id: self.cur_leve_id,
        }
    }
}

impl AddAssign<i32> for BlockId {
    fn add_assign(&mut self, other: i32) {
        self.id += other;
    }
}
impl Add<i32> for BlockId {
    type Output = BlockId;
    fn add(self, rhs: i32) -> Self::Output {
        BlockId {
            id: self.id + rhs,
            is_merge: false,
        }
    }
}
impl Sub<i32> for BlockId {
    type Output = BlockId;
    fn sub(self, rhs: i32) -> Self::Output {
        BlockId {
            id: self.id - rhs,
            is_merge: false,
        }
    }
}

impl AddAssign<i32> for SlotId {
    fn add_assign(&mut self, other: i32) {
        self.id += other;
    }
}
impl Add<i32> for SlotId {
    type Output = SlotId;
    fn add(self, rhs: i32) -> Self::Output {
        SlotId {
            id: self.id + rhs,
        }
    }
}
impl Sub<i32> for SlotId {
    type Output = SlotId;
    fn sub(self, rhs: i32) -> Self::Output {
        SlotId {
            id: self.id - rhs,
        }
    }
}

impl FuncId{
    pub(crate) fn get_id(self)->i32{
        let FuncId(x)=self;
        x
    }
}