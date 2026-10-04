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

// 8) 编译器类型输出解析 + 位置校验
const typesMod = require('./types');
const tout = 'warning something\n{"types":[{"name":"res","type":"String","line":11,"col":5},{"name":"i","type":"int","line":21,"col":30}]}\n';
const parsedTypes = typesMod.parseTypesOutput(tout);
eq(parsedTypes.length, 2, 'types parse count');
eq([parsedTypes[0].name, parsedTypes[0].type, parsedTypes[0].line, parsedTypes[0].col], ['res', 'String', 11, 5], 'types parse entry');
eq(typesMod.parseTypesOutput('garbage').length, 0, 'types parse garbage');
eq(typesMod.parseTypesOutput('').length, 0, 'types parse empty');
const tsrc = ['fn f() -> int {', '    res = String::new();', '    for i in (0, 3) {', '    }', '}'].join('\n');
const valid = typesMod.validateTypeEntries(tsrc, [
    { name: 'res', type: 'String', line: 2, col: 5 },
    { name: 'tmp', type: 'int', line: 4, col: 5 },
    { name: 'i', type: 'int', line: 3, col: 9 },
]);
eq(valid.length, 2, 'types validate count');
eq([valid[0].name, valid[0].line, valid[0].col], ['res', 2, 5], 'types validate keeps real decl');
eq(valid[1].name, 'i', 'types validate keeps for-var position when present');
const dup = typesMod.validateTypeEntries('    res = 1;\n    res = 2;\n', [
    { name: 'res', type: 'int', line: 1, col: 5 },
    { name: 'res', type: 'int', line: 1, col: 5 },
]);
eq(dup.length, 1, 'types validate dedupe');

// 9) Quick Fix：未知符号解析 + import 候选 + 插入位置
const qf = require('./quickfix');
eq(qf.parseUnknownSymbol('type `ArrayList` has no method `push` for argument types (TokenType) at 26:16（类型 `ArrayList` 未知：若来自包，请确认已 import 对应模块）'), { name: 'ArrayList', kind: 'type' }, 'qf parse method hint');
eq(qf.parseUnknownSymbol('undefined function `ArrayList.new` at 4:23'), { name: 'ArrayList', kind: 'type' }, 'qf parse ctor');
eq(qf.parseUnknownSymbol('undefined function `foo` at 1:1'), { name: 'foo', kind: 'fn' }, 'qf parse fn');
eq(qf.parseUnknownSymbol('type `ArrayList<TokenType>` has no method `x`'), { name: 'ArrayList', kind: 'type' }, 'qf parse generic');
eq(qf.parseUnknownSymbol('some unrelated error'), null, 'qf parse none');
const pkgs = [
    { stem: 'arraylist', functions: [], structs: [{ name: 'ArrayList' }], enums: [] },
    { stem: 'linkedlist', functions: [], structs: [{ name: 'LinkedList' }], enums: [] },
    { stem: 'string', functions: [{ name: 'int_to_string' }], structs: [{ name: 'String' }], enums: [] },
];
eq(qf.findImportCandidates({ name: 'ArrayList', kind: 'type' }, pkgs), ['arraylist'], 'qf candidates type');
eq(qf.findImportCandidates({ name: 'ArrayList', kind: 'type' }, [
    { stem: 'std', functions: [], structs: [{ name: 'ArrayList' }], enums: [] },
    { stem: 'arraylist', functions: [], structs: [{ name: 'ArrayList' }], enums: [] },
]), ['arraylist', 'std'], 'qf candidates prefer specific over std');
eq(qf.findImportCandidates({ name: 'int_to_string', kind: 'fn' }, pkgs), ['string'], 'qf candidates fn');
eq(qf.findImportCandidates({ name: 'Nope', kind: 'type' }, pkgs), [], 'qf candidates none');
eq(qf.importEditInfo('import "string";\n\nfn main() -> int { return 0; }\n', 'arraylist'), { line: 1, text: 'import "arraylist";\n' }, 'qf edit after imports');
eq(qf.importEditInfo('fn main() -> int { return 0; }\n', 'arraylist'), { line: 0, text: 'import "arraylist";\n' }, 'qf edit at top');
eq(qf.importEditInfo('import "arraylist";\n', 'arraylist'), null, 'qf edit already imported');

// 10) 诊断位置：`12:1: error:` 前缀形态
const missingRet = 'error: 12:1: error: function `digit` has non-void return type but no return statement\n';
d = parseCompilerOutput(missingRet, '/tmp/opencode/x.aya', '/tmp/opencode');
eq([d.length, d[0].line, d[0].col], [1, 12, 1], 'line:col prefix pos');

// 11) 波浪线范围
const { computeRange } = require('./diagnostics');
const declLine = 'fn digit(ref mut int i, ref String code) -> TokenType {';
const rDecl = computeRange(declLine, 1, 'function `digit` has non-void return type but no return statement');
eq([rDecl.start, rDecl.end], [0, declLine.indexOf('{')], 'range declaration');
const rTok = computeRange('    res = String::new();', 11, 'undefined variable `res`');
eq([rTok.start, rTok.end], [4, 7], 'range token by name');
const rWord = computeRange('    foo.bar', 5, 'some error');
eq([rWord.start, rWord.end], [4, 7], 'range identifier at col');
const rLine = computeRange('    = 1', 5, 'some error');
eq([rLine.start, rLine.end], [4, 7], 'range whole line fallback');

// 12) lcl 方法表
const mout = { functions: [], structs: [], enums: [], methods: [] };
syms.parseLclSymbols('method="ArrayList,push,push(ref mut ArrayList[T],T)->void"\nmethod="ArrayList,iter,iter(ref ArrayList[T],fn(T))->void"\n', 'a.lcl', mout);
eq(mout.methods.length, 2, 'lcl methods count');
eq([mout.methods[0].type, mout.methods[0].name, mout.methods[0].sig], ['ArrayList', 'push', 'push(ref mut ArrayList[T],T) -> void'], 'lcl method push');
eq(mout.methods[1].sig, 'iter(ref ArrayList[T],fn(T)) -> void', 'lcl method nested paren');

// 13) Hover 渲染（Markdown / 缩进 / 标注说明 / 文档）
const hov = require('./hover');
const fnHover = hov.renderFnHover({
    sig: 'greet(ref String n) -> int',
    attrs: ['alloc', 'state'],
    doc: '打招呼\n\n- 支持 markdown',
    file: '/w/src/main.aya', line: 5,
}, { folder: '/w/src' });
eq(fnHover.includes('```ayanami\nfn greet(ref String n) -> int\n```'), true, 'hover fn code block');
eq(fnHover.includes('`#[alloc]` — 可能分配内存'), true, 'hover fn attr desc');
eq(fnHover.includes('`#[state]` — 可能修改可观察状态'), true, 'hover fn attr state');
eq(fnHover.includes('打招呼'), true, 'hover fn doc');
eq(fnHover.includes('main.aya:5'), true, 'hover fn location');
const stHover = hov.renderStructHover({
    name: 'Point', generics: '', fields: [{ type: 'int', name: 'x' }, { type: 'float', name: 'y' }],
    doc: '坐标', file: 'p.aya', line: 1,
}, {});
eq(stHover.includes('    int x'), true, 'hover struct indent x');
eq(stHover.includes('    float y'), true, 'hover struct indent y');
eq(stHover.includes('坐标'), true, 'hover struct doc');
const enHover = hov.renderEnumHover({ name: 'Color', variants: ['Red', 'Blue(int)'] }, {});
eq(enHover.includes('    Blue(int)'), true, 'hover enum variant indent');
eq(hov.attrDoc('alloc').includes('可能分配内存'), true, 'attr doc alloc');
eq(hov.attrDoc('throws').includes('throws'), true, 'attr doc throws');
eq(hov.attrDoc('nope'), null, 'attr doc unknown');
eq(hov.renderVarHover('i', 'int'), '**变量** `i: int`', 'hover var');
eq(hov.renderMethodHover({ sig: 'push(ref mut ArrayList[T],T) -> void', type: 'ArrayList' }, {}).includes('ArrayList'), true, 'hover method type');
// 文档提取：`//` 与 `///`，跳过标注
const docSrc = '/// 第一行\n///\n/// 列表\n#[alloc]\nfn f() -> int { return 0 }';
eq(syms.scanFnSigsFull(docSrc, 'x.aya')[0].doc, '第一行\n\n列表', 'doc extraction slash');
eq(syms.scanFnSigsFull(docSrc, 'x.aya')[0].attrs, ['alloc'], 'doc extraction attrs');

console.log(fails === 0 ? 'ALL EXT TESTS PASS' : `${fails} FAILURES`);
process.exit(fails === 0 ? 0 : 1);
