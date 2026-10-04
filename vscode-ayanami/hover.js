// ─── Hover 渲染（纯函数，便于 test.js 测试） ─────────────────────────
// 目标：类 rust-analyzer / clangd 的 Markdown 悬停：
//   1. 签名代码块
//   2. 标注列表（带说明）
//   3. 文档注释（原样 Markdown）
//   4. 来源位置

'use strict';

const ATTR_DOCS = {
    alloc: ['效应', '可能分配内存（堆分配）', '#[alloc]'],
    state: ['效应', '可能修改可观察状态（全局 / 堆 / ref mut）', '#[state]'],
    io: ['效应', '可能进行 IO', '#[io]'],
    pure: ['效应', '承诺无副作用（LLVM memory(none)）', '#[pure]'],
    readonly: ['效应', '只读内存（LLVM memory(read)）', '#[readonly]'],
    no_error: ['效应', '承诺不会失败（不返回错误）', '#[no_error]'],
    throws: ['效应', '可能抛出错误：throws(E) / throws(_) / throws()', '#[throws(E)]'],
    nounwind: ['属性', '不会 unwind（LLVM nounwind）', '#[nounwind]'],
    noreturn: ['属性', '不会返回（LLVM noreturn）', '#[noreturn]'],
    willreturn: ['属性', '总是返回（LLVM willreturn）', '#[willreturn]'],
    cold: ['属性', '冷路径（LLVM cold）', '#[cold]'],
    inline: ['属性', '内联提示：inline / inline(always)', '#[inline]'],
    noalias: ['属性', '指针不与其它指针别名', '#[noalias]'],
    nonnull: ['属性', '指针非空', '#[nonnull]'],
    cfg: ['编译期', '条件编译：cfg(target = "linux")', '#[cfg(target = "linux")]'],
    assume: ['契约', '假设条件成立（发射 llvm.assume）', '#[assume(cond)]'],
    requires: ['契约', '前置条件（默认运行时检查）', '#[requires(cond)]'],
    ensures: ['契约', '后置条件（result 绑定返回值）', '#[ensures(result)]'],
    invariant: ['契约', '循环不变式（每轮检查）', '#[invariant(cond)]'],
    follow_with: ['生命周期', '引用生命周期来源（参数名 / 类型名）', '#[follow_with(source)]'],
    macro: ['A5', '用户宏（源码级展开）', '#[macro]'],
    pass: ['A5', 'MIR 优化 pass（fn(ref mut MirFunction) -> void）', '#[pass]'],
    check: ['A5', 'MIR 只读检查（fn(ref MirFunction) -> void）', '#[check]'],
};

function fenced(code, lang = 'ayanami') {
    return '```' + lang + '\n' + code + '\n```';
}

function attrName(raw) {
    return String(raw || '').replace(/\(.*$/, '').trim();
}

/// 标注列表（带说明），缩进对齐
function renderAttrList(attrs) {
    if (!attrs || !attrs.length) return '';
    return attrs.map((a) => {
        const info = ATTR_DOCS[attrName(a)];
        const desc = info ? info[1] : '标注';
        return `- \`#[${a}]\` — ${desc}`;
    }).join('\n');
}

function locationLine(file, line, folder) {
    if (!file) return '';
    const p = require('path');
    const short = folder && file.startsWith(folder) ? p.relative(folder, file) : p.basename(file);
    return `*${short}${line ? ':' + line : ''}*`;
}

/// 函数 / 方法
function renderFnHover(fn, opts = {}) {
    const parts = [fenced('fn ' + fn.sig)];
    const attrs = renderAttrList(fn.attrs);
    if (attrs) parts.push(attrs);
    if (fn.doc) parts.push('---\n\n' + fn.doc);
    const loc = locationLine(fn.file, fn.line, opts.folder);
    if (loc) parts.push(loc);
    return parts.join('\n\n');
}

/// 方法（附带所属类型）
function renderMethodHover(m, opts = {}) {
    const parts = [fenced('fn ' + m.sig)];
    if (m.type) parts.push(`*\`${m.type}\` 的方法*`);
    const loc = locationLine(m.file, m.line, opts.folder);
    if (loc) parts.push(loc);
    return parts.join('\n\n');
}

/// 结构体：字段代码块（4 空格缩进）
function renderStructHover(st, opts = {}) {
    const generics = st.generics || '';
    const lines = (st.fields || []).map((f) => `    ${f.type} ${f.name}`);
    const body = ['struct ' + st.name + generics + ' {'].concat(lines, ['}']).join('\n');
    const parts = [fenced(body)];
    if (st.doc) parts.push('---\n\n' + st.doc);
    const loc = locationLine(st.file, st.line, opts.folder);
    if (loc) parts.push(loc);
    return parts.join('\n\n');
}

/// 枚举：变体代码块（4 空格缩进）
function renderEnumHover(en, opts = {}) {
    const generics = en.generics || '';
    const lines = (en.variants || []).map((v) => `    ${v}`);
    const body = ['enum ' + en.name + generics + ' {'].concat(lines, ['}']).join('\n');
    const parts = [fenced(body)];
    if (en.doc) parts.push('---\n\n' + en.doc);
    const loc = locationLine(en.file, en.line, opts.folder);
    if (loc) parts.push(loc);
    return parts.join('\n\n');
}

/// 变量
function renderVarHover(name, type) {
    return `**变量** \`${name}: ${type}\``;
}

/// 字段
function renderFieldHover(structName, field) {
    return `**字段** \`${structName}.${field.name}: ${field.type}\``;
}

/// 标注本身（悬停 `#[alloc]` 里的 alloc）
function attrDoc(name) {
    const info = ATTR_DOCS[name];
    if (!info) return null;
    const [group, desc, example] = info;
    return `**\`#[${name}]\`** — ${desc}\n\n*分组：${group}*\n\n用法：\`${example}\``;
}

module.exports = {
    ATTR_DOCS,
    fenced,
    renderAttrList,
    renderFnHover,
    renderMethodHover,
    renderStructHover,
    renderEnumHover,
    renderVarHover,
    renderFieldHover,
    attrDoc,
};
