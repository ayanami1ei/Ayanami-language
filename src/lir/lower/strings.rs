use super::*;

pub(super) fn collect_strings(mir: &MirProgram) -> Vec<String> {
    let mut strings = Vec::new();
    collect_strings_items(&mir.items, &mut strings);
    let mut seen = std::collections::HashSet::new();
    strings.retain(|s| seen.insert(s.clone()));
    strings
}

pub(super) fn collect_strings_items(items: &[MirItem], out: &mut Vec<String>) {
    for item in items {
        match item {
            MirItem::Fn(f) => collect_strings_stmts(&f.body, out),
            MirItem::StructDef { .. } => {}
            MirItem::Namespace { items, .. } => collect_strings_items(items, out),
        }
    }
}

pub(super) fn collect_strings_stmts(stmts: &[MirStmtBox], out: &mut Vec<String>) {
    for stmt in stmts {
        stmt.for_each_child_expr(&mut |child| {
            collect_strings_dyn(child, out);
        });
    }
}

pub(super) fn collect_strings_dyn(node: &dyn MirNode, out: &mut Vec<String>) {
    if let Some(s) = node.as_string_literal() {
        out.push(s.to_string());
    }
    node.for_each_child(&mut |child| {
        collect_strings_dyn(child, out);
    });
}
