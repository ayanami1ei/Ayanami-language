# Details

Date : 2026-06-09 13:28:16

Directory /home/ayanami/Ayanami-language/src

Total : 57 files,  11261 codes, 566 comments, 783 blanks, all 12610 lines

[Summary](results.md) / Details / [Diff Summary](diff.md) / [Diff Details](diff-details.md)

## Files
| filename | language | code | comment | blank | total |
| :--- | :--- | ---: | ---: | ---: | ---: |
| [src/compiler/build.rs](/src/compiler/build.rs) | Rust | 514 | 8 | 63 | 585 |
| [src/compiler/check.rs](/src/compiler/check.rs) | Rust | 94 | 0 | 4 | 98 |
| [src/compiler/debug.rs](/src/compiler/debug.rs) | Rust | 354 | 1 | 11 | 366 |
| [src/compiler/import.rs](/src/compiler/import.rs) | Rust | 84 | 12 | 7 | 103 |
| [src/compiler/mod.rs](/src/compiler/mod.rs) | Rust | 75 | 2 | 7 | 84 |
| [src/compiler/symdef.rs](/src/compiler/symdef.rs) | Rust | 143 | 3 | 5 | 151 |
| [src/driver/mod.rs](/src/driver/mod.rs) | Rust | 140 | 24 | 26 | 190 |
| [src/formatter.rs](/src/formatter.rs) | Rust | 389 | 2 | 16 | 407 |
| [src/hir/display.rs](/src/hir/display.rs) | Rust | 276 | 0 | 10 | 286 |
| [src/hir/ir.rs](/src/hir/ir.rs) | Rust | 212 | 40 | 19 | 271 |
| [src/hir/lower/body.rs](/src/hir/lower/body.rs) | Rust | 1,321 | 102 | 74 | 1,497 |
| [src/hir/lower/helpers.rs](/src/hir/lower/helpers.rs) | Rust | 494 | 29 | 26 | 549 |
| [src/hir/lower/mod.rs](/src/hir/lower/mod.rs) | Rust | 126 | 12 | 22 | 160 |
| [src/hir/mod.rs](/src/hir/mod.rs) | Rust | 6 | 11 | 2 | 19 |
| [src/intern/interner.rs](/src/intern/interner.rs) | Rust | 28 | 0 | 5 | 33 |
| [src/intern/mod.rs](/src/intern/mod.rs) | Rust | 3 | 4 | 2 | 9 |
| [src/intern/symbol.rs](/src/intern/symbol.rs) | Rust | 20 | 0 | 7 | 27 |
| [src/lexer/delimiter.rs](/src/lexer/delimiter.rs) | Rust | 33 | 0 | 3 | 36 |
| [src/lexer/keyword.rs](/src/lexer/keyword.rs) | Rust | 83 | 0 | 3 | 86 |
| [src/lexer/lexer.rs](/src/lexer/lexer.rs) | Rust | 373 | 4 | 18 | 395 |
| [src/lexer/mod.rs](/src/lexer/mod.rs) | Rust | 11 | 7 | 2 | 20 |
| [src/lexer/test.rs](/src/lexer/test.rs) | Rust | 14 | 0 | 2 | 16 |
| [src/lexer/token.rs](/src/lexer/token.rs) | Rust | 43 | 0 | 6 | 49 |
| [src/lexer/token\_kind.rs](/src/lexer/token_kind.rs) | Rust | 30 | 0 | 4 | 34 |
| [src/lib.rs](/src/lib.rs) | Rust | 11 | 0 | 1 | 12 |
| [src/lir/display.rs](/src/lir/display.rs) | Rust | 116 | 0 | 5 | 121 |
| [src/lir/emit.rs](/src/lir/emit.rs) | Rust | 1,014 | 52 | 51 | 1,117 |
| [src/lir/ir.rs](/src/lir/ir.rs) | Rust | 199 | 50 | 9 | 258 |
| [src/lir/lower.rs](/src/lir/lower.rs) | Rust | 1,003 | 39 | 67 | 1,109 |
| [src/lir/mod.rs](/src/lir/mod.rs) | Rust | 9 | 11 | 2 | 22 |
| [src/lir/serialize.rs](/src/lir/serialize.rs) | Rust | 503 | 15 | 28 | 546 |
| [src/main.rs](/src/main.rs) | Rust | 348 | 10 | 27 | 385 |
| [src/mir/borrow.rs](/src/mir/borrow.rs) | Rust | 121 | 0 | 10 | 131 |
| [src/mir/display.rs](/src/mir/display.rs) | Rust | 283 | 0 | 10 | 293 |
| [src/mir/ir.rs](/src/mir/ir.rs) | Rust | 147 | 0 | 8 | 155 |
| [src/mir/lower.rs](/src/mir/lower.rs) | Rust | 510 | 26 | 44 | 580 |
| [src/mir/mem/mod.rs](/src/mir/mem/mod.rs) | Rust | 19 | 0 | 5 | 24 |
| [src/mir/mem/shared.rs](/src/mir/mem/shared.rs) | Rust | 17 | 0 | 3 | 20 |
| [src/mir/mem/unique.rs](/src/mir/mem/unique.rs) | Rust | 17 | 0 | 3 | 20 |
| [src/mir/mem/value.rs](/src/mir/mem/value.rs) | Rust | 17 | 0 | 3 | 20 |
| [src/mir/mod.rs](/src/mir/mod.rs) | Rust | 8 | 6 | 2 | 16 |
| [src/package/config.rs](/src/package/config.rs) | Rust | 74 | 9 | 6 | 89 |
| [src/package/mod.rs](/src/package/mod.rs) | Rust | 319 | 18 | 28 | 365 |
| [src/parser/ast/binary\_op.rs](/src/parser/ast/binary_op.rs) | Rust | 16 | 0 | 1 | 17 |
| [src/parser/ast/block.rs](/src/parser/ast/block.rs) | Rust | 12 | 0 | 3 | 15 |
| [src/parser/ast/expr.rs](/src/parser/ast/expr.rs) | Rust | 101 | 0 | 3 | 104 |
| [src/parser/ast/literal.rs](/src/parser/ast/literal.rs) | Rust | 20 | 0 | 3 | 23 |
| [src/parser/ast/mod.rs](/src/parser/ast/mod.rs) | Rust | 18 | 0 | 2 | 20 |
| [src/parser/ast/program.rs](/src/parser/ast/program.rs) | Rust | 10 | 0 | 3 | 13 |
| [src/parser/ast/stmt.rs](/src/parser/ast/stmt.rs) | Rust | 121 | 0 | 4 | 125 |
| [src/parser/ast/ty.rs](/src/parser/ast/ty.rs) | Rust | 46 | 0 | 4 | 50 |
| [src/parser/ast/unary\_op.rs](/src/parser/ast/unary_op.rs) | Rust | 5 | 0 | 1 | 6 |
| [src/parser/ast/vis.rs](/src/parser/ast/vis.rs) | Rust | 11 | 0 | 2 | 13 |
| [src/parser/mod.rs](/src/parser/mod.rs) | Rust | 4 | 5 | 2 | 11 |
| [src/parser/parser.rs](/src/parser/parser.rs) | Rust | 1,188 | 48 | 75 | 1,311 |
| [src/runtime.c](/src/runtime.c) | C | 46 | 13 | 16 | 75 |
| [src/span.rs](/src/span.rs) | Rust | 62 | 3 | 8 | 73 |

[Summary](results.md) / Details / [Diff Summary](diff.md) / [Diff Details](diff-details.md)