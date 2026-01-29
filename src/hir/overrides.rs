use crate::hir::{BlockId, FloatKey, FuncId, ObjId, SlotId, VarId};
use std::cmp::Ordering;
use std::ops::{Add, AddAssign, Sub};
use std::hash::Hasher;
use std::hash::Hash;

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


impl From<f64> for FloatKey {
    fn from(v: f64) -> Self {
        FloatKey(v)
    }
}

impl PartialEq for FloatKey {
    fn eq(&self, other: &Self) -> bool {
        self.0.to_bits() == other.0.to_bits()
    }
}
impl Eq for FloatKey {}

impl Hash for FloatKey {
    fn hash<H: Hasher>(&self, state: &mut H) {
        state.write_u64(self.0.to_bits());
    }
}

impl PartialOrd for FloatKey {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for FloatKey {
    fn cmp(&self, other: &Self) -> Ordering {
        self.0.total_cmp(&other.0)
    }
}
