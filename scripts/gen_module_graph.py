#!/usr/bin/env python3
"""生成模块依赖知识图谱（docs/module-graph.md）。

用法: python3 scripts/gen_module_graph.py

规则：
- 扫描 src/**/*.rs（排除 src/generated/）
- 模块路径：src/foo/bar.rs -> foo::bar；src/foo/mod.rs -> foo；src/foo/bar/mod.rs -> foo::bar
- 提取 `use crate::X` 与 `use crate::{A, B}`，聚合到顶层模块（lexer/parser/hir/...）
- 输出 Mermaid 图 + 边统计表；确定性输出，无时间戳
"""
import glob
import os
import re

ROOT = os.path.dirname(os.path.dirname(os.path.abspath(__file__)))
OUT = os.path.join(ROOT, "docs", "module-graph.md")

USE_IDENT = re.compile(r"use\s+crate::([a-z_][a-z0-9_]*)")
USE_GROUP = re.compile(r"use\s+crate::\{([^}]*)\}")


def module_of(path: str) -> str:
    rel = os.path.relpath(path, os.path.join(ROOT, "src"))
    rel = rel[:-3]  # .rs
    if rel == "lib" or rel == "main":
        return "root"
    if rel.endswith("/mod"):
        return rel[:-4]
    return rel


def top_module(module: str) -> str:
    if module == "root":
        return "root"
    return module.split("/")[0]


def deps_of(path: str) -> set:
    text = open(path, encoding="utf-8").read()
    deps = set(USE_IDENT.findall(text))
    for group in USE_GROUP.findall(text):
        for item in group.split(","):
            item = item.strip().split("::")[0]
            if re.fullmatch(r"[a-z_][a-z0-9_]*", item):
                deps.add(item)
    return deps


def main() -> None:
    files = [
        f
        for f in sorted(glob.glob(os.path.join(ROOT, "src", "**", "*.rs"), recursive=True))
        if "/generated/" not in f
    ]
    edges: dict = {}
    for f in files:
        src = top_module(module_of(f))
        for dep in deps_of(f):
            if dep == src:
                continue
            edges[(src, dep)] = edges.get((src, dep), 0) + 1

    # 只保留两侧都是实际顶层模块的边
    tops = sorted({top_module(module_of(f)) for f in files})
    edges = {k: v for k, v in edges.items() if k[0] in tops and k[1] in tops}

    lines = []
    lines.append("# 模块依赖图（知识图谱 · 生成物）")
    lines.append("")
    lines.append("> 由 `python3 scripts/gen_module_graph.py` 生成，请勿手改。")
    lines.append("> 节点为顶层模块，边为 `use crate::...` 引用（排除 `src/generated/`）。")
    lines.append("")
    lines.append("```mermaid")
    lines.append("graph LR")
    for t in tops:
        if t != "root":
            lines.append(f"    {t}")
    for (a, b), count in sorted(edges.items()):
        label = f"|{count}|" if count > 1 else ""
        lines.append(f"    {a} -->{label} {b}")
    lines.append("```")
    lines.append("")
    lines.append("## 边统计")
    lines.append("")
    lines.append("| 来源 | 目标 | 引用数 |")
    lines.append("| --- | --- | ---: |")
    for (a, b), count in sorted(edges.items(), key=lambda kv: (-kv[1], kv[0])):
        lines.append(f"| `{a}` | `{b}` | {count} |")
    lines.append("")
    lines.append(f"共 {len(tops)} 个顶层模块、{len(edges)} 条依赖边。")
    lines.append("")

    os.makedirs(os.path.dirname(OUT), exist_ok=True)
    with open(OUT, "w", encoding="utf-8") as f:
        f.write("\n".join(lines))
    print(f"wrote {OUT}")


if __name__ == "__main__":
    main()
