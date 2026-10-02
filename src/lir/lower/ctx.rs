use super::*;

pub(super) struct LowerCtx<'a> {
    pub(super) tmp: u64,
    pub(super) block_id: u64,
    pub(super) current_label: String,
    pub(super) current_insts: Vec<LirNodeBox>,
    pub(super) blocks: Vec<LirBlock>,
    pub(super) str_map: &'a HashMap<String, u64>,
    pub(super) loop_stack: Vec<(String, String)>,
}

impl<'a> LowerCtx<'a> {
    pub(super) fn new(str_map: &'a HashMap<String, u64>) -> Self {
        Self {
            tmp: 0,
            block_id: 0,
            current_label: "entry".into(),
            current_insts: Vec::new(),
            blocks: Vec::new(),
            str_map,
            loop_stack: Vec::new(),
        }
    }

    pub(super) fn next_block_label(&mut self, prefix: &str) -> String {
        let id = self.block_id;
        self.block_id += 1;
        format!("{}{}", prefix, id)
    }

    pub(super) fn set_current_block(&mut self, label: String) {
        if !self.current_label.is_empty() {
            let old_label = std::mem::replace(&mut self.current_label, label);
            let insts = std::mem::take(&mut self.current_insts);
            if !insts.is_empty() || !self.blocks.is_empty() {
                self.blocks.push(LirBlock {
                    label: old_label,
                    insts,
                });
            }
        } else {
            self.current_label = label;
        }
    }

    pub(super) fn finish(&mut self) -> Vec<LirBlock> {
        if !self.current_label.is_empty() {
            let label = std::mem::take(&mut self.current_label);
            let insts = std::mem::take(&mut self.current_insts);
            self.blocks.push(LirBlock { label, insts });
        }
        std::mem::take(&mut self.blocks)
    }
}

impl LirLowerCtx for LowerCtx<'_> {
    fn next_tmp(&mut self) -> u64 {
        let t = self.tmp;
        self.tmp += 1;
        t
    }
    fn emit(&mut self, inst: LirNodeBox) {
        self.current_insts.push(inst);
    }
    fn str_map(&self) -> &HashMap<String, u64> {
        self.str_map
    }
    fn loop_stack(&self) -> &Vec<(String, String)> {
        &self.loop_stack
    }
    fn loop_stack_mut(&mut self) -> &mut Vec<(String, String)> {
        &mut self.loop_stack
    }
    fn next_block_label(&mut self, prefix: &str) -> String {
        self.next_block_label(prefix)
    }
    fn set_current_block(&mut self, label: String) {
        self.set_current_block(label)
    }
}
