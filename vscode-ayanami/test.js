const { parseCompilerOutput } = require('./diagnostics');
const syms = require('./symbols');

let fails = 0;
function eq(actual, expected, label) {
    if (JSON.stringify(actual) !== JSON.stringify(expected)) {
        console.log(`FAIL ${label}: got ${JSON.stringify(actual)} want ${JSON.stringify(expected)}`);
        fails++;
    } else {
        console.log(`ok   ${label}`);
    }
}

// 1) HIR 错误（at 无括号）
const hir = "error: check failed: compile error: /tmp/opencode/diag1.aya: hir error: undefined function `undefined_fn` at 2:9\n";
let d = parseCompilerOutput(hir, '/tmp/opencode/diag1.aya', '/tmp/opencode');
eq(d.length, 1, 'hir count');
eq([d[0].file, d[0].line, d[0].col, d[0].severity], ['/tmp/opencode/diag1.aya', 2, 9, 0], 'hir pos');
eq(d[0].message, 'undefined function `undefined_fn` at 2:9', 'hir msg');

// 2) 解析错误（(at l:c)）
const parse = "error: check failed: compile error: /tmp/opencode/diag2.aya: parse error: expected `)`, found `Keyword(return)` (at 3:5)\n";
d = parseCompilerOutput(parse, '/tmp/opencode/diag2.aya', '/tmp/opencode');
eq([d[0].line, d[0].col], [3, 5], 'parse pos');

// 3) 借用错误（无位置 → 1:1，但保留消息）
const borrow = "error: check failed: compile error: /tmp/opencode/diag3.aya: borrow error: reference local `r` must be initialized from `ref`, another reference, or a call returning a reference\n";
d = parseCompilerOutput(borrow, '/tmp/opencode/diag3.aya', '/tmp/opencode');
eq([d[0].line, d[0].col, d[0].severity], [1, 1, 0], 'borrow fallback');
eq(d[0].message.startsWith('reference local'), true, 'borrow msg');

// 4) 警告 + --> + 片段
const warn = [
    "warning: function `print` may have effect `alloc`; consider adding #[alloc]",
    "  --> std/src/io.aya:66:11",
    "   |",
    "66 |     buf = [c]",
    "   |           ^",
    "warning: function `println` may have effect `alloc`; consider adding #[alloc]",
    "  --> std/src/io.aya:29:5",
    "   |",
    "29 | pub fn println(ref String s) {",
    "   |     ^",
].join("\n") + "\n";
d = parseCompilerOutput(warn, '/x/main.aya', '/proj');
eq(d.length, 2, 'warn count');
eq([d[0].file, d[0].line, d[0].col, d[0].severity], ['/proj/std/src/io.aya', 66, 11, 1], 'warn1 pos');
eq(d[1].line, 29, 'warn2 line');

// 5) 符号扫描：函数签名 / 结构体 / 枚举
const src = [
    "// 计算两数之和",
    "#[pure]",
    "pub fn add(int a, int b) -> int {",
    "    return a + b",
    "}",
    "",
    "struct Point {",
    "    float x",
    "    int y",
    "}",
    "",
    "enum Color {",
    "    Red,",
    "    Blue(int),",
    "}",
    "",
    "impl Point {",
    "    fn norm(ref self, fn(int) f) -> float {",
    "        return 0.0",
    "    }",
    "}",
].join("\n");
const fns = syms.scanFnSigsFull(src, 'x.aya');
const add = fns.find(f => f.name === 'add');
eq(add.sig, 'add(int a, int b) -> int', 'fn sig');
eq(add.doc, '计算两数之和', 'fn doc');
eq(add.attrs, ['pure'], 'fn attrs');
const norm = fns.find(f => f.name === 'norm');
eq(norm.sig, 'norm(ref self, fn(int) f) -> float', 'method sig with nested parens');
const st = syms.scanStructsFull(src, 'x.aya').find(s => s.name === 'Point');
eq(st.fields.map(f => `${f.type} ${f.name}`), ['float x', 'int y'], 'struct fields');
const en = syms.scanEnumsFull(src, 'x.aya').find(e => e.name === 'Color');
eq(en.variants, ['Red', 'Blue(int)'], 'enum variants');

// 6) .lcl 符号表解析
const lcl = 'fn="add,d:alloc,add(int,int)->int"\nfn="to_string,,to_string(int)->String"\nstruct="Point(x:float,y:int)"\n';
const out = { functions: [], structs: [], enums: [] };
syms.parseLclSymbols(lcl, 'io.lcl', out);
eq(out.functions.map(f => f.sig), ['add(int,int) -> int', 'to_string(int) -> String'], 'lcl fns');
eq(out.structs[0].fields.map(f => `${f.type} ${f.name}`), ['float x', 'int y'], 'lcl struct');

// 7) 终端新格式：error + --> + 源码片段
const term = [
    "error: undefined function `undefined_fn`",
    "  --> /tmp/opencode/diag1.aya:2:9",
    "  |",
    "2 |     x = undefined_fn(3)",
    "  |         ^",
].join("\n") + "\n";
d = parseCompilerOutput(term, '/tmp/opencode/diag1.aya', '/tmp/opencode');
eq([d.length, d[0].line, d[0].col, d[0].message], [1, 2, 9, 'undefined function `undefined_fn`'], 'terminal error format');

console.log(fails === 0 ? 'ALL EXT TESTS PASS' : `${fails} FAILURES`);
process.exit(fails === 0 ? 0 : 1);
