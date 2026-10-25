use super::*;
use super::mem::needs_drop;
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
        // 分支内 return 会清空 alive（lower_return 作用域清理）；保存并在落空路径恢复，
        // 否则后续路径不再释放函数级拥有局部（参数等）→ 泄漏。
        let base_alive = self.alive.clone();
        let mut fall_alive = base_alive.clone();
        // #116：分支必然 return 时其移动不合并到 if 之后（该路径不达后续代码）；
        // 其余分支按「可能移动」合并（任一可达路径移动即报）。
        let mut fall_moved = base.clone();

        self.moved = base.clone();
        self.alive = base_alive.clone();
        let mut mir_then = self.lower_block(&then_block.stmts);
        // #184：分支 delta-moved（用于条件 drop；块末 drop 对「可能移动」整体跳过）
        let mut then_delta: Vec<VarId> = Vec::new();
        let then_diverges = self.block_diverges(&then_block.stmts);
        if !then_diverges {
            then_delta = self.moved.difference(&base).copied().collect();
            fall_moved.extend(self.moved.iter().copied());
            fall_alive.extend(self.alive.iter().copied());
        }

        // 进入第 i 个 elif 的路径：前面分支体未执行，但条件已求值
        let mut cond_entry = base.clone();
        let mut elif_deltas: Vec<Vec<VarId>> = Vec::new();
        let mut mir_elifs: Vec<_> = elifs
            .iter()
            .map(|(c, b)| {
                self.moved = cond_entry.clone();
                c.record_moves(&mut self.moved);
                cond_entry = self.moved.clone();
                fall_moved.extend(self.moved.iter().copied());
                let mut cm = c.lower_to_mir(&self.moved);
                // elif 条件按需（前序条件失败才求值）：借用临时量在链内求值后 drop
                let (epre, epost) = self.hoist_ref_temps(&mut cm, span);
                self.alive = base_alive.clone();
                let bm = self.lower_block(&b.stmts);
                let mut delta: Vec<VarId> = Vec::new();
                if !self.block_diverges(&b.stmts) {
                    delta = self.moved.difference(&base).copied().collect();
                    fall_moved.extend(self.moved.iter().copied());
                    fall_alive.extend(self.alive.iter().copied());
                }
                elif_deltas.push(delta);
                (cm, epre, epost, bm)
            })
            .collect();
        let mut else_delta: Vec<VarId> = Vec::new();
        let mut mir_else = else_block
            .as_ref()
            .map(|b| {
                self.moved = cond_entry.clone();
                self.alive = base_alive.clone();
                let bm = self.lower_block(&b.stmts);
                if !self.block_diverges(&b.stmts) {
                    else_delta = self.moved.difference(&base).copied().collect();
                    fall_moved.extend(self.moved.iter().copied());
                    fall_alive.extend(self.alive.iter().copied());
                }
                bm
            });
        // #184：分支级条件 drop —— 仅 ToUnique 等非清零转移需要在未移动路径补 drop；
        // 普通 SMove 已清零源槽，块末可无条件 drop（不再按 moved 跳过）。
        {
            // 仅限 if 入口已存活的局部（分支内新声明的变量在其它分支不存在，drop 会读垃圾）
            let mut union: Vec<VarId> = then_delta.iter().copied()
                .filter(|v| base_alive.contains(v)).collect();
            for d in &elif_deltas {
                union.extend(d.iter().copied().filter(|v| base_alive.contains(v)));
            }
            union.extend(else_delta.iter().copied().filter(|v| base_alive.contains(v)));
            union.sort_by_key(|v| v.0);
            union.dedup();
            let drops_for = |delta: &[VarId], this: &Self| -> Vec<MirStmtBox> {
                union.iter()
                    .filter(|v| this.nonzero.contains(v))
                    .filter(|v| !delta.contains(v))
                    .filter_map(|v| {
                        let ty = this.var_types.get(v)?.clone();
                        needs_drop(&ty, &this.struct_defs)
                            .then(|| SMirDropStmt { var: *v, ty, span }.into())
                    })
                    .collect()
            };
            if !then_diverges {
                mir_then.extend(drops_for(&then_delta, self));
            }
            for ((_, _, _, bm), d) in mir_elifs.iter_mut().zip(elif_deltas.iter()) {
                let drops = drops_for(d, self);
                bm.extend(drops);
            }
            let else_drops = drops_for(&else_delta, self);
            if !else_drops.is_empty() {
                match &mut mir_else {
                    Some(b) => b.extend(else_drops),
                    None => mir_else = Some(else_drops),
                }
            }
        }

        self.moved = fall_moved;
        self.alive = fall_alive;
        let mut stmts = pre;
        stmts.push(SMirIfStmt { cond: mir_cond, then_block: mir_then, elifs: mir_elifs, else_block: mir_else, span }.into());
        stmts.extend(post);
        stmts
    }

    pub(super) fn lower_while(&mut self, cond: &HirNodeBox, body: &HirBlock, span: crate::span::Span) -> Vec<MirStmtBox> {
        cond.record_moves(&mut self.moved);
        let mut mir_cond = cond.lower_to_mir(&self.moved);
        // 条件每轮求值：借用临时量在循环头赋值、求值后 drop（不能提升到循环外）
        let (pre_cond, post_cond) = self.hoist_ref_temps(&mut mir_cond, span);
        // 循环体内 return 会清空 alive；循环后恢复外层 alive（体内新变量已按作用域释放）
        let base_alive = self.alive.clone();
        let mir_body = self.lower_block(&body.stmts);
        self.alive = base_alive;
        vec![SMirWhileStmt { cond: mir_cond, pre_cond, post_cond, body: mir_body, span }.into()]
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
                if self.nonzero.contains(&var) || self.result_vars.contains(&var) {
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
