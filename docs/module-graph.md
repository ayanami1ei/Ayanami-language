# 模块依赖图（知识图谱 · 生成物）

> 由 `python3 scripts/gen_module_graph.py` 生成，请勿手改。
> 节点为顶层模块，边为 `use crate::...` 引用（排除 `src/generated/`）。

```mermaid
graph LR
    cli
    compiler
    diagnostics
    driver
    error
    formatter
    hir
    intern
    lexer
    lir
    mir
    package
    parser
    span
    compiler -->|11| error
    compiler -->|4| hir
    compiler --> intern
    compiler --> lir
    compiler -->|3| mir
    compiler --> package
    compiler -->|9| parser
    driver --> error
    formatter --> intern
    formatter -->|2| parser
    hir -->|10| error
    hir -->|11| intern
    hir -->|2| mir
    hir -->|15| parser
    hir -->|5| span
    lexer --> span
    lir --> error
    lir -->|4| hir
    lir -->|3| intern
    lir -->|3| mir
    lir -->|3| parser
    mir -->|4| error
    mir -->|8| hir
    mir -->|3| intern
    mir --> lir
    mir --> parser
    mir --> span
    package --> error
    package --> parser
    parser -->|2| error
    parser -->|4| intern
    parser --> lexer
    parser -->|6| span
```

## 边统计

| 来源 | 目标 | 引用数 |
| --- | --- | ---: |
| `hir` | `parser` | 15 |
| `compiler` | `error` | 11 |
| `hir` | `intern` | 11 |
| `hir` | `error` | 10 |
| `compiler` | `parser` | 9 |
| `mir` | `hir` | 8 |
| `parser` | `span` | 6 |
| `hir` | `span` | 5 |
| `compiler` | `hir` | 4 |
| `lir` | `hir` | 4 |
| `mir` | `error` | 4 |
| `parser` | `intern` | 4 |
| `compiler` | `mir` | 3 |
| `lir` | `intern` | 3 |
| `lir` | `mir` | 3 |
| `lir` | `parser` | 3 |
| `mir` | `intern` | 3 |
| `formatter` | `parser` | 2 |
| `hir` | `mir` | 2 |
| `parser` | `error` | 2 |
| `compiler` | `intern` | 1 |
| `compiler` | `lir` | 1 |
| `compiler` | `package` | 1 |
| `driver` | `error` | 1 |
| `formatter` | `intern` | 1 |
| `lexer` | `span` | 1 |
| `lir` | `error` | 1 |
| `mir` | `lir` | 1 |
| `mir` | `parser` | 1 |
| `mir` | `span` | 1 |
| `package` | `error` | 1 |
| `package` | `parser` | 1 |
| `parser` | `lexer` | 1 |

共 15 个顶层模块、33 条依赖边。
