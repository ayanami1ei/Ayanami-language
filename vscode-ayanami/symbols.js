// Ayanami VSCode 插件：源码符号扫描（函数签名/结构体/枚举/标准库 .lcl）。
// 纯 Node 模块（只依赖 fs/path），便于单测。

'use strict';

const fs = require('fs');
const path = require('path');

function matchParen(text, openIdx) {
    let depth = 0;
    for (let i = openIdx; i < text.length; i++) {
        const c = text[i];
        if (c === '(') depth++;
        else if (c === ')') { depth--; if (depth === 0) return i; }
    }
    return -1;
}

function matchBrace(text, openIdx) {
    let depth = 0;
    for (let i = openIdx; i < text.length; i++) {
        const c = text[i];
        if (c === '{') depth++;
        else if (c === '}') { depth--; if (depth === 0) return i; }
    }
    return -1;
}

function lineOf(text, idx) {
    return text.slice(0, idx).split('\n').length;
}

/// 扫描全部函数签名（含 impl 方法、泛型、返回类型、文档注释与标注）
function scanFnSigsFull(text, file) {
    const out = [];
    const seen = new Set();
    const fnRe = /\bfn\s+([A-Za-z_]\w*)\s*(\[[^\]]*\])?\s*\(/g;
    let m;
    while ((m = fnRe.exec(text)) !== null) {
        const name = m[1];
        const generics = m[2] || '';
        const openIdx = m.index + m[0].length - 1;
        const closeIdx = matchParen(text, openIdx);
        if (closeIdx < 0) continue;
        const params = text.slice(openIdx + 1, closeIdx).replace(/\s+/g, ' ').trim();
        const after = text.slice(closeIdx + 1);
        const arrow = after.match(/^\s*->\s*([^{\n;]+)/);
        const ret = arrow ? arrow[1].trim() : '';
        // 前置注释 / 标注
        let doc = '';
        const attrs = [];
        let cursor = text.lastIndexOf('\n', m.index);
        while (cursor > 0) {
            const prevStart = text.lastIndexOf('\n', cursor - 1) + 1;
            const line = text.slice(prevStart, cursor).trim();
            if (line.startsWith('//')) {
                doc = line.replace(/^\/\/\s?/, '') + (doc ? '\n' + doc : '');
                cursor = prevStart - 1;
            } else if (line.startsWith('#[')) {
                attrs.unshift(line.replace(/^#\[|\]$/g, ''));
                cursor = prevStart - 1;
            } else {
                break;
            }
        }
        const sig = `${name}${generics}(${params})${ret ? ' -> ' + ret : ''}`;
        if (seen.has(name + sig)) continue;
        seen.add(name + sig);
        out.push({ name, sig, params, ret, doc, attrs, line: lineOf(text, m.index), file });
    }
    return out;
}

/// 扫描结构体与字段（带类型）
function scanStructsFull(text, file) {
    const out = [];
    const re = /(?:pub\s+)?struct\s+([A-Za-z_]\w*)\s*(\[[^\]]*\])?\s*\{/g;
    let m;
    while ((m = re.exec(text)) !== null) {
        const open = text.indexOf('{', m.index);
        const close = matchBrace(text, open);
        const body = close > 0 ? text.slice(open + 1, close) : '';
        const fields = [];
        for (const raw of body.split('\n')) {
            const t = raw.trim();
            if (!t || t.startsWith('//')) continue;
            const fm = t.match(/^(?:(ref\s+mut|ref)\s+)?(.+?)\s+([A-Za-z_]\w*)$/);
            if (fm) fields.push({ type: (fm[1] ? fm[1] + ' ' : '') + fm[2].trim(), name: fm[3] });
        }
        out.push({ name: m[1], fields, line: lineOf(text, m.index), file });
    }
    return out;
}

/// 扫描枚举与变体名
function scanEnumsFull(text, file) {
    const out = [];
    const re = /(?:pub\s+)?enum\s+([A-Za-z_]\w*)\s*(\[[^\]]*\])?\s*\{/g;
    let m;
    while ((m = re.exec(text)) !== null) {
        const open = text.indexOf('{', m.index);
        const close = matchBrace(text, open);
        const body = close > 0 ? text.slice(open + 1, close) : '';
        const variants = [];
        for (const raw of body.split('\n')) {
            const vm = raw.trim().match(/^([A-Z]\w*)\s*(\(([^)]*)\))?/);
            if (vm) variants.push(vm[3] ? `${vm[1]}(${vm[3]})` : vm[1]);
        }
        out.push({ name: m[1], variants, line: lineOf(text, m.index), file });
    }
    return out;
}

/// 解析 .lcl 的符号表（fn="name,flags,sig" / struct="..." / method="Type,name,sig"）
function parseLclSymbols(text, file, syms) {
    if (!syms.methods) syms.methods = [];
    const fnRe = /^fn="([^"]*)"/gm;
    let m;
    while ((m = fnRe.exec(text)) !== null) {
        const entry = m[1];
        const parts = entry.split(',');
        const name = parts[0];
        // flags 与签名都以 ',' 分隔；签名从第一个含 '(' 或 '->' 的分段开始
        const idx = parts.findIndex((p, i) => i > 0 && (p.includes('(') || p.includes('->')));
        const rest = idx > 0 ? parts.slice(idx).join(',') : '';
        const sm = rest.match(/^([\w.]+)\(([^)]*)\)(?:\s*->\s*(.+))?$/);
        if (sm) {
            syms.functions.push({
                name, sig: `${name}(${sm[2]})${sm[3] ? ' -> ' + sm[3].trim() : ''}`,
                params: sm[2], ret: (sm[3] || '').trim(), doc: '', attrs: [], line: 0, file,
            });
        }
    }
    const meRe = /^method="([^"]*)"/gm;
    while ((m = meRe.exec(text)) !== null) {
        const parts = m[1].split(',');
        if (parts.length < 3) continue;
        const typeName = parts[0];
        const name = parts[1];
        const sigBody = parts.slice(2).join(',');
        const parsed = parseBalancedSig(name, sigBody);
        if (parsed) {
            syms.methods.push({
                type: typeName, name,
                sig: `${name}(${parsed.params})${parsed.ret ? ' -> ' + parsed.ret : ''}`,
                params: parsed.params, ret: parsed.ret, doc: '', attrs: [], line: 0, file,
            });
        }
    }
    const stRe = /^struct="([^"]*)"/gm;
    while ((m = stRe.exec(text)) !== null) {
        const nm = m[1].match(/^(\w+)\(([^)]*)\)/);
        if (nm) {
            const fields = nm[2].split(',').filter(Boolean).map(f => {
                const idx = f.indexOf(':');
                return idx >= 0 ? { name: f.slice(0, idx), type: f.slice(idx + 1) } : { name: f, type: '' };
            });
            syms.structs.push({ name: nm[1], fields, line: 0, file });
        }
    }
}

/// 解析 `name(params)->ret`，参数允许嵌套括号（如 fn(T)）
function parseBalancedSig(name, body) {
    const open = body.indexOf('(');
    if (open < 0) return null;
    let depth = 0, close = -1;
    for (let i = open; i < body.length; i++) {
        if (body[i] === '(') depth++;
        else if (body[i] === ')') { depth--; if (depth === 0) { close = i; break; } }
    }
    if (close < 0) return null;
    const params = body.slice(open + 1, close).trim();
    const rest = body.slice(close + 1);
    const arrow = rest.match(/^\s*->\s*(.+)$/);
    return { params, ret: arrow ? arrow[1].trim() : '' };
}

const cache = new Map();

/// 收集当前文件 + import（.aya 源码或 .lcl 符号表）的符号
function collectSymbols(text, filePath, opts) {
    const key = filePath + ':' + text.length + ':' + (opts.compilerDir || '');
    const hit = cache.get(key);
    if (hit) return hit;

    const syms = { functions: [], structs: [], enums: [], methods: [] };
    const addText = (t, f) => {
        syms.functions.push(...scanFnSigsFull(t, f));
        syms.structs.push(...scanStructsFull(t, f));
        syms.enums.push(...scanEnumsFull(t, f));
    };
    addText(text, filePath);

    const folder = filePath ? path.dirname(filePath) : null;
    const stdDirs = [];
    if (folder) stdDirs.push(path.resolve(folder, '../std/src'), path.resolve(folder, '../std'));
    if (opts.compilerDir) stdDirs.push(path.join(opts.compilerDir, 'std'), path.join(opts.compilerDir, 'std/src'));
    for (const wf of opts.workspaceFolders || []) {
        stdDirs.push(path.join(wf, 'std/src'), path.join(wf, 'std'));
    }

    const re = /import\s+"([^"]+)"/g;
    let m;
    while ((m = re.exec(text)) !== null) {
        const imp = m[1];
        const cands = [];
        if (imp.endsWith('.aya') || imp.endsWith('.lcl')) {
            if (folder) cands.push(path.resolve(folder, imp));
        } else {
            if (folder) cands.push(path.resolve(folder, imp + '.aya'));
            for (const d of stdDirs) cands.push(path.join(d, imp + '.aya'), path.join(d, imp + '.lcl'));
        }
        for (const c of cands) {
            try {
                const content = fs.readFileSync(c, 'utf8');
                if (c.endsWith('.lcl')) parseLclSymbols(content, c, syms);
                else addText(content, c);
                break;
            } catch (e) { /* 继续尝试下一个候选路径 */ }
        }
    }

    if (cache.size > 64) cache.clear();
    cache.set(key, syms);
    return syms;
}

function shortPath(file, folder) {
    if (!file) return '';
    if (folder && file.startsWith(folder)) return path.relative(folder, file);
    return path.basename(file);
}

module.exports = { scanFnSigsFull, scanStructsFull, scanEnumsFull, parseLclSymbols, collectSymbols, shortPath };
