# Details

Date : 2026-06-15 10:12:00

Directory /home/ayanami/Ayanami-language/src

Total : 57 files,  13026 codes, 875 comments, 832 blanks, all 14733 lines

[Summary](results.md) / Details / [Diff Summary](diff.md) / [Diff Details](diff-details.md)

## Files
| filename | language | code | comment | blank | total |
| :--- | :--- | ---: | ---: | ---: | ---: |
| [src/compiler/build.rs](/src/compiler/build.rs) | Rust | 526 | 10 | 63 | 599 |
| [src/compiler/check.rs](/src/compiler/check.rs) | Rust | 94 | 0 | 4 | 98 |
| [src/compiler/debug.rs](/src/compiler/debug.rs) | Rust | 372 | 1 | 11 | 384 |
| [src/compiler/import.rs](/src/compiler/import.rs) | Rust | 84 | 12 | 7 | 103 |
| [src/compiler/mod.rs](/src/compiler/mod.rs) | Rust | 156 | 51 | 23 | 230 |
| [src/compiler/symdef.rs](/src/compiler/symdef.rs) | Rust | 144 | 3 | 5 | 152 |
| [src/driver/mod.rs](/src/driver/mod.rs) | Rust | 139 | 25 | 26 | 190 |
| [src/formatter.rs](/src/formatter.rs) | Rust | 474 | 2 | 16 | 492 |
| [src/hir/display.rs](/src/hir/display.rs) | Rust | 292 | 0 | 10 | 302 |
| [src/hir/ir.rs](/src/hir/ir.rs) | Rust | 235 | 45 | 19 | 299 |
| [src/hir/lower/body.rs](/src/hir/lower/body.rs) | Rust | 1,943 | 158 | 83 | 2,184 |
| [src/hir/lower/helpers.rs](/src/hir/lower/helpers.rs) | Rust | 640 | 63 | 29 | 732 |
| [src/hir/lower/mod.rs](/src/hir/lower/mod.rs) | Rust | 196 | 74 | 28 | 298 |
| [src/hir/mod.rs](/src/hir/mod.rs) | Rust | 6 | 11 | 2 | 19 |
| [src/intern/interner.rs](/src/intern/interner.rs) | Rust | 28 | 0 | 5 | 33 |
| [src/intern/mod.rs](/src/intern/mod.rs) | Rust | 3 | 4 | 2 | 9 |
| [src/intern/symbol.rs](/src/intern/symbol.rs) | Rust | 20 | 0 | 7 | 27 |
| [src/lexer/delimiter.rs](/src/lexer/delimiter.rs) | Rust | 35 | 0 | 3 | 38 |
| [src/lexer/keyword.rs](/src/lexer/keyword.rs) | Rust | 91 | 0 | 3 | 94 |
| [src/lexer/lexer.rs](/src/lexer/lexer.rs) | Rust | 378 | 4 | 18 | 400 |
| [src/lexer/mod.rs](/src/lexer/mod.rs) | Rust | 11 | 7 | 2 | 20 |
| [src/lexer/test.rs](/src/lexer/test.rs) | Rust | 14 | 0 | 2 | 16 |
| [src/lexer/token.rs](/src/lexer/token.rs) | Rust | 43 | 0 | 6 | 49 |
| [src/lexer/token\_kind.rs](/src/lexer/token_kind.rs) | Rust | 30 | 0 | 4 | 34 |
| [src/lib.rs](/src/lib.rs) | Rust | 11 | 15 | 2 | 28 |
| [src/lir/display.rs](/src/lir/display.rs) | Rust | 121 | 1 | 5 | 127 |
| [src/lir/emit.rs](/src/lir/emit.rs) | Rust | 1,132 | 67 | 50 | 1,249 |
| [src/lir/ir.rs](/src/lir/ir.rs) | Rust | 207 | 52 | 9 | 268 |
| [src/lir/lower.rs](/src/lir/lower.rs) | Rust | 1,123 | 54 | 70 | 1,247 |
| [src/lir/mod.rs](/src/lir/mod.rs) | Rust | 9 | 11 | 2 | 22 |
| [src/lir/serialize.rs](/src/lir/serialize.rs) | Rust | 569 | 16 | 30 | 615 |
| [src/main.rs](/src/main.rs) | Rust | 348 | 27 | 28 | 403 |
| [src/mir/borrow.rs](/src/mir/borrow.rs) | Rust | 121 | 0 | 10 | 131 |
| [src/mir/display.rs](/src/mir/display.rs) | Rust | 303 | 0 | 10 | 313 |
| [src/mir/ir.rs](/src/mir/ir.rs) | Rust | 168 | 0 | 8 | 176 |
| [src/mir/lower.rs](/src/mir/lower.rs) | Rust | 534 | 47 | 45 | 626 |
| [src/mir/mem/mod.rs](/src/mir/mem/mod.rs) | Rust | 19 | 0 | 5 | 24 |
| [src/mir/mem/shared.rs](/src/mir/mem/shared.rs) | Rust | 17 | 0 | 3 | 20 |
| [src/mir/mem/unique.rs](/src/mir/mem/unique.rs) | Rust | 17 | 0 | 3 | 20 |
| [src/mir/mem/value.rs](/src/mir/mem/value.rs) | Rust | 17 | 0 | 3 | 20 |
| [src/mir/mod.rs](/src/mir/mod.rs) | Rust | 8 | 6 | 2 | 16 |
| [src/package/config.rs](/src/package/config.rs) | Rust | 74 | 9 | 6 | 89 |
| [src/package/mod.rs](/src/package/mod.rs) | Rust | 330 | 20 | 28 | 378 |
| [src/parser/ast/binary\_op.rs](/src/parser/ast/binary_op.rs) | Rust | 16 | 0 | 1 | 17 |
| [src/parser/ast/block.rs](/src/parser/ast/block.rs) | Rust | 12 | 0 | 3 | 15 |
| [src/parser/ast/expr.rs](/src/parser/ast/expr.rs) | Rust | 115 | 0 | 3 | 118 |
| [src/parser/ast/literal.rs](/src/parser/ast/literal.rs) | Rust | 20 | 0 | 3 | 23 |
| [src/parser/ast/mod.rs](/src/parser/ast/mod.rs) | Rust | 18 | 0 | 2 | 20 |
| [src/parser/ast/program.rs](/src/parser/ast/program.rs) | Rust | 10 | 0 | 3 | 13 |
| [src/parser/ast/stmt.rs](/src/parser/ast/stmt.rs) | Rust | 161 | 0 | 7 | 168 |
| [src/parser/ast/ty.rs](/src/parser/ast/ty.rs) | Rust | 48 | 0 | 4 | 52 |
| [src/parser/ast/unary\_op.rs](/src/parser/ast/unary_op.rs) | Rust | 5 | 0 | 1 | 6 |
| [src/parser/ast/vis.rs](/src/parser/ast/vis.rs) | Rust | 11 | 0 | 2 | 13 |
| [src/parser/mod.rs](/src/parser/mod.rs) | Rust | 4 | 5 | 2 | 11 |
| [src/parser/parser.rs](/src/parser/parser.rs) | Rust | 1,416 | 59 | 80 | 1,555 |
| [src/runtime.c](/src/runtime.c) | C | 46 | 13 | 16 | 75 |
| [src/span.rs](/src/span.rs) | Rust | 62 | 3 | 8 | 73 |

[Summary](results.md) / Details / [Diff Summary](diff.md) / [Diff Details](diff-details.md)