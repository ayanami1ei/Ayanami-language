use super::*;
use super::mem::{action_to_stmt, strategy_for};

impl Ctx {
    pub(super) fn lower_if(
        &mut self,
        cond: &HirNodeBox,
        then_block: &HirBlock,
        elifs: &[(HirNodeBox, HirBlock)],
        else_block: &Option<HirBlock>,
        span: crate::span::Span,
    ) -> Vec<MirStmtBox> {
        cond.record_moves(&mut self.moved);
        let mut mir_cond = cond.lower_to_mir(&self.moved);
        // 首条件必然求值：借用临时量提升到 if 前赋值、if 后 drop
        let (pre, post) = self.hoist_ref_temps(&mut mir_cond, span);
        let base = self.moved.clone();
        // #116：分支必然 return 时其移动不合并到 if 之后（该路径不达后续代码）；
        // 其余分支按「可能移动」合并（任一可达路径移动即报）。
        let mut fall_moved = base.clone();

        self.moved = base.clone();
        let mir_then = self.lower_block(&then_block.stmts);
        if !self.block_diverges(&then_block.stmts) {
            fall_moved.extend(self.moved.iter().copied());
        }

        // 进入第 i 个 elif 的路径：前面分支体未执行，但条件已求值
        let mut cond_entry = base.clone();
        let mir_elifs: Vec<_> = elifs
            .iter()
            .map(|(c, b)| {
                self.moved = cond_entry.clone();
                c.record_moves(&mut self.moved);
                cond_entry = self.moved.clone();
                fall_moved.extend(self.moved.iter().copied());
                let cm = c.lower_to_mir(&self.moved);
                let bm = self.lower_block(&b.stmts);
                if !self.block_diverges(&b.stmts) {
                    fall_moved.extend(self.moved.iter().copied());
                }
                (cm, bm)
            })
            .collect();
        let mir_else = else_block
            .as_ref()
            .map(|b| {
                self.moved = cond_entry.clone();
                let bm = self.lower_block(&b.stmts);
                if !self.block_diverges(&b.stmts) {
                    fall_moved.extend(self.moved.iter().copied());
                }
                bm
            });

        self.moved = fall_moved;
        let mut stmts = pre;
        stmts.push(SMirIfStmt { cond: mir_cond, then_block: mir_then, elifs: mir_elifs, else_block: mir_else, span }.into());
        stmts.extend(post);
        stmts
    }

    pub(super) fn lower_while(&mut self, cond: &HirNodeBox, body: &HirBlock, span: crate::span::Span) -> Vec<MirStmtBox> {
        cond.record_moves(&mut self.moved);
        let mir_cond = cond.lower_to_mir(&self.moved);
        let mir_body = self.lower_block(&body.stmts);
        vec![SMirWhileStmt { cond: mir_cond, body: mir_body, span }.into()]
    }

    pub(super) fn lower_block(&mut self, stmts: &[HirStmt]) -> Vec<MirStmtBox> {
        let mut mir_stmts = Vec::new();
        let before: HashSet<VarId> = self.alive.clone();
        for stmt in stmts {
            let mut lowered = self.lower_stmt(stmt);
            mir_stmts.append(&mut lowered);
        }

        // 块结束：释放块内新声明且未被移动的变量（循环体内每轮都会执行）。
        // 以 break/continue 结尾的块跳过，避免在终结指令之后发射清理代码。
        let terminates = matches!(stmts.last(), Some(HirStmt::Break { .. } | HirStmt::Continue { .. }));
        if !terminates {
            let mut scoped: Vec<VarId> = self.alive.difference(&before).copied().collect();
            scoped.sort_by_key(|v| v.0);
            for var in scoped {
                self.alive.remove(&var);
                if self.moved.contains(&var) || self.result_vars.contains(&var) {
                    continue;
                }
                let ty = self.var_types[&var].clone();
                let strategy = strategy_for(&ty, &self.struct_defs);
                for action in strategy.on_scope_end(var, &ty) {
                    mir_stmts.push(action_to_stmt(var, &ty, &action));
                }
            }
        }
        mir_stmts
    }

    pub(super) fn mark_alive(&mut self, var: VarId) {
        self.alive.insert(var);
    }
}
