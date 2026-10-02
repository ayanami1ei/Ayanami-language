use std::collections::HashSet;
use super::*;

/// CFG 节点载荷
pub enum Payload<'a> {
    /// 叶子语句
    Stmt(&'a dyn MirStmtNode),
    /// 分支/循环条件表达式
    Cond(&'a MirNodeBox),
    /// 连接点/空节点
    Empty,
}

pub struct Node<'a> {
    pub payload: Payload<'a>,
    pub succ: Vec<usize>,
    /// 本节点定义的局部变量（赋值目标）
    pub defs: HashSet<VarId>,
    /// 本节点写入的局部变量（赋值/字段写/索引写/drop）
    pub writes: HashSet<VarId>,
    /// 本节点读取的局部变量
    pub uses: HashSet<VarId>,
}

pub struct Cfg<'a> {
    pub nodes: Vec<Node<'a>>,
}

/// 把结构化 MIR 展平为 CFG（每条语句/条件一个节点）。
pub fn build(body: &[MirStmtBox]) -> Cfg<'_> {
    let mut b = Builder { nodes: Vec::new() };
    let exit = b.push(Payload::Empty, vec![]);
    b.seq(body, exit, &[]);
    Cfg { nodes: b.nodes }
}

struct Builder<'a> {
    nodes: Vec<Node<'a>>,
}

impl<'a> Builder<'a> {
    fn push(&mut self, payload: Payload<'a>, succ: Vec<usize>) -> usize {
        let mut uses = HashSet::new();
        let (defs, writes) = match &payload {
            Payload::Stmt(s) => stmt_io(*s, &mut uses),
            Payload::Cond(c) => {
                c.collect_var_ids(&mut uses);
                (HashSet::new(), HashSet::new())
            }
            Payload::Empty => (HashSet::new(), HashSet::new()),
        };
        self.nodes.push(Node { payload, succ, defs, writes, uses });
        self.nodes.len() - 1
    }

    fn seq(&mut self, stmts: &'a [MirStmtBox], next: usize, loops: &[(usize, usize)]) -> usize {
        let mut next = next;
        for s in stmts.iter().rev() {
            next = self.stmt(&**s, next, loops);
        }
        next
    }

    fn stmt(&mut self, s: &'a dyn MirStmtNode, next: usize, loops: &[(usize, usize)]) -> usize {
        if let Some((cond, then_b, elifs, else_b)) = s.as_if() {
            let else_entry = match else_b {
                Some(eb) => self.seq(eb, next, loops),
                None => next,
            };
            let mut else_target = else_entry;
            for (c, b) in elifs.iter().rev() {
                let body = self.seq(b, next, loops);
                else_target = self.push(Payload::Cond(c), vec![body, else_target]);
            }
            let then_entry = self.seq(then_b, next, loops);
            return self.push(Payload::Cond(cond), vec![then_entry, else_target]);
        }
        if let Some((cond, body)) = s.as_while() {
            let cond_id = self.push(Payload::Cond(cond), vec![]);
            let mut inner_loops = vec![(cond_id, next)];
            inner_loops.extend_from_slice(loops);
            let body_entry = self.seq(body, cond_id, &inner_loops);
            self.nodes[cond_id].succ = vec![body_entry, next];
            return cond_id;
        }
        if let Some(inner) = s.as_block() {
            return self.seq(inner, next, loops);
        }
        if s.is_break() {
            let target = loops.first().map(|l| l.1).unwrap_or(next);
            return self.push(Payload::Stmt(s), vec![target]);
        }
        if s.is_continue() {
            let target = loops.first().map(|l| l.0).unwrap_or(next);
            return self.push(Payload::Stmt(s), vec![target]);
        }
        if s.is_return() {
            return self.push(Payload::Stmt(s), vec![]);
        }
        self.push(Payload::Stmt(s), vec![next])
    }
}

/// 收集语句的读取/定义/写入变量。
fn stmt_io(s: &dyn MirStmtNode, uses: &mut HashSet<VarId>) -> (HashSet<VarId>, HashSet<VarId>) {
    let mut defs = HashSet::new();
    let mut writes = HashSet::new();
    if let Some((target, value)) = s.assign_parts() {
        value.collect_var_ids(uses);
        if let Some(v) = target.as_local() {
            defs.insert(v);
            writes.insert(v);
        }
    }
    if let Some((obj, value)) = s.field_assign_parts() {
        obj.collect_var_ids(uses);
        value.collect_var_ids(uses);
        if let Some(v) = obj.as_local() {
            writes.insert(v);
        }
    }
    if let Some((obj, idx, value)) = s.index_assign_parts() {
        obj.collect_var_ids(uses);
        idx.collect_var_ids(uses);
        value.collect_var_ids(uses);
        if let Some(v) = obj.as_local() {
            writes.insert(v);
        }
    }
    if let Some(v) = s.return_value() {
        v.collect_var_ids(uses);
    }
    if let Some(e) = s.expr_part() {
        e.collect_var_ids(uses);
    }
    if let Some((v, _)) = s.as_drop() {
        writes.insert(v);
    }
    (defs, writes)
}
