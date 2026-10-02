# 模块依赖图（知识图谱 · 生成物）

> 由 `python3 scripts/gen_module_graph.py` 生成，请勿手改。
> 节点为顶层模块，边为 `use crate::...` 引用（排除 `src/generated/`）。

```mermaid
graph LR
    cli
    compiler
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
    compiler -->|3| error
    compiler --> hir
    compiler --> lir
    compiler --> package
    compiler -->|4| parser
    driver --> error
    formatter --> intern
    formatter --> parser
    hir -->|2| error
    hir -->|7| intern
    hir -->|2| mir
    hir -->|7| parser
    hir -->|5| span
    lexer --> span
    lir --> error
    lir -->|4| hir
    lir -->|4| intern
    lir -->|3| mir
    lir -->|3| parser
    mir -->|4| error
    mir -->|7| hir
    mir -->|2| intern
    mir --> lir
    mir --> parser
    package --> error
    package --> parser
    parser -->|3| error
    parser -->|5| intern
    parser --> lexer
    parser -->|7| span
```

## 边统计

| 来源 | 目标 | 引用数 |
| --- | --- | ---: |
| `hir` | `intern` | 7 |
| `hir` | `parser` | 7 |
| `mir` | `hir` | 7 |
| `parser` | `span` | 7 |
| `hir` | `span` | 5 |
| `parser` | `intern` | 5 |
| `compiler` | `parser` | 4 |
| `lir` | `hir` | 4 |
| `lir` | `intern` | 4 |
| `mir` | `error` | 4 |
| `compiler` | `error` | 3 |
| `lir` | `mir` | 3 |
| `lir` | `parser` | 3 |
| `parser` | `error` | 3 |
| `hir` | `error` | 2 |
| `hir` | `mir` | 2 |
| `mir` | `intern` | 2 |
| `compiler` | `hir` | 1 |
| `compiler` | `lir` | 1 |
| `compiler` | `package` | 1 |
| `driver` | `error` | 1 |
| `formatter` | `intern` | 1 |
| `formatter` | `parser` | 1 |
| `lexer` | `span` | 1 |
| `lir` | `error` | 1 |
| `mir` | `lir` | 1 |
| `mir` | `parser` | 1 |
| `package` | `error` | 1 |
| `package` | `parser` | 1 |
| `parser` | `lexer` | 1 |

共 14 个顶层模块、30 条依赖边。
