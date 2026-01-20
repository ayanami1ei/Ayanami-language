use crate::hir::{BlockId, ObjId, VarId};
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
        self.0 += other;
    }
}
impl Add<i32> for ObjId {
    type Output = ObjId;
    fn add(self, rhs: i32) -> Self::Output {
        ObjId(self.0 + rhs)
    }
}
impl Sub<i32> for ObjId {
    type Output = ObjId;
    fn sub(self, rhs: i32) -> Self::Output {
        ObjId(self.0 - rhs)
    }
}

impl AddAssign<i32> for BlockId {
    fn add_assign(&mut self, other: i32) {
        self.0 += other;
    }
}
impl Add<i32> for BlockId {
    type Output = BlockId;
    fn add(self, rhs: i32) -> Self::Output {
        BlockId(self.0 + rhs)
    }
}
impl Sub<i32> for BlockId {
    type Output = BlockId;
    fn sub(self, rhs: i32) -> Self::Output {
        BlockId(self.0 - rhs)
    }
}


