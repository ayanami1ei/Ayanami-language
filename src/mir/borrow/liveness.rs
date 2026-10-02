use std::collections::HashSet;
use super::*;

/// 后向数据流：每个节点入口处的活跃变量集合。
/// NLL 语义下，引用的借用在其最后一次使用之后即失效。
pub fn live_in(cfg: &cfg::Cfg) -> Vec<HashSet<VarId>> {
    let n = cfg.nodes.len();
    let mut live_in: Vec<HashSet<VarId>> = vec![HashSet::new(); n];
    loop {
        let mut changed = false;
        for i in (0..n).rev() {
            let mut out: HashSet<VarId> = HashSet::new();
            for &s in &cfg.nodes[i].succ {
                out.extend(live_in[s].iter().copied());
            }
            let mut inn = cfg.nodes[i].uses.clone();
            for v in out {
                if !cfg.nodes[i].defs.contains(&v) {
                    inn.insert(v);
                }
            }
            if inn != live_in[i] {
                live_in[i] = inn;
                changed = true;
            }
        }
        if !changed {
            break;
        }
    }
    live_in
}
