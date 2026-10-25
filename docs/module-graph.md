# 模块依赖图（知识图谱 · 生成物）

> 由 `python3 scripts/gen_module_graph.py` 生成，请勿手改。
> 节点为顶层模块，边为 `use crate::...` 引用。

```mermaid
graph LR
    annotation_table
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
    annotation_table --> error
    annotation_table --> intern
    annotation_table --> parser
    compiler --> annotation_table
    compiler -->|12| error
    compiler -->|4| hir
    compiler --> intern
    compiler --> lexer
    compiler --> lir
    compiler -->|3| mir
    compiler --> package
    compiler -->|9| parser
    driver -->|2| error
    formatter --> intern
    formatter --> lexer
    formatter -->|2| parser
    hir --> annotation_table
    hir -->|20| error
    hir -->|20| intern
    hir -->|2| mir
    hir -->|30| parser
    hir -->|5| span
    lexer --> span
    lir --> error
    lir -->|5| hir
    lir -->|3| intern
    lir -->|3| mir
    lir -->|3| parser
    mir -->|4| error
    mir -->|8| hir
    mir -->|3| intern
    mir --> lir
    mir --> parser
    mir -->|2| span
    package -->|2| error
    package --> hir
    package --> intern
    package --> parser
    parser -->|2| error
    parser -->|5| intern
    parser --> lexer
    parser -->|6| span
```

## 边统计

| 来源 | 目标 | 引用数 |
| --- | --- | ---: |
| `hir` | `parser` | 30 |
| `hir` | `error` | 20 |
| `hir` | `intern` | 20 |
| `compiler` | `error` | 12 |
| `compiler` | `parser` | 9 |
| `mir` | `hir` | 8 |
| `parser` | `span` | 6 |
| `hir` | `span` | 5 |
| `lir` | `hir` | 5 |
| `parser` | `intern` | 5 |
| `compiler` | `hir` | 4 |
| `mir` | `error` | 4 |
| `compiler` | `mir` | 3 |
| `lir` | `intern` | 3 |
| `lir` | `mir` | 3 |
| `lir` | `parser` | 3 |
| `mir` | `intern` | 3 |
| `driver` | `error` | 2 |
| `formatter` | `parser` | 2 |
| `hir` | `mir` | 2 |
| `mir` | `span` | 2 |
| `package` | `error` | 2 |
| `parser` | `error` | 2 |
| `annotation_table` | `error` | 1 |
| `annotation_table` | `intern` | 1 |
| `annotation_table` | `parser` | 1 |
| `compiler` | `annotation_table` | 1 |
| `compiler` | `intern` | 1 |
| `compiler` | `lexer` | 1 |
| `compiler` | `lir` | 1 |
| `compiler` | `package` | 1 |
| `formatter` | `intern` | 1 |
| `formatter` | `lexer` | 1 |
| `hir` | `annotation_table` | 1 |
| `lexer` | `span` | 1 |
| `lir` | `error` | 1 |
| `mir` | `lir` | 1 |
| `mir` | `parser` | 1 |
| `package` | `hir` | 1 |
| `package` | `intern` | 1 |
| `package` | `parser` | 1 |
| `parser` | `lexer` | 1 |

共 16 个顶层模块、42 条依赖边。
