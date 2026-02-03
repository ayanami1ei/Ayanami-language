use crate::hir::{HirGenerator, ObjSlot};

impl HirGenerator {
    fn can_delete(&self, slot: &ObjSlot) -> bool {
        !slot.escape && slot.last_use != 0
    }

    pub(super) fn analyze_lifetime(&mut self) {
        let mut offset = 0;
        let mut vec: Vec<(&super::SlotId, &ObjSlot)> = self.slot_registry.iter().collect();
        vec.sort_by_key(|x| x.1.last_use);

        for (id, slot) in vec {
            // skip temp slots: they are ephemeral loaded temps and should not get Delete
            if id.temp {
                continue;
            }
            if self.can_delete(&slot) {
                let mut insert_pos = slot.last_use + offset + 2;
                if insert_pos > self.hir.len() {
                    insert_pos = self.hir.len();
                }
                self.hir.insert(
                    insert_pos,
                    super::HIR::Inst(super::HIRInst::Delete { dst: *id }),
                );
                offset += 1;
            }
        }
    }
}
