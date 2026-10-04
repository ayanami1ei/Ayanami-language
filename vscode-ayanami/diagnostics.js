// Ayanami VSCode 插件：编译器输出 → 结构化诊断（纯函数，可单测）。
//
// 支持的编译器输出形态：
//   error: check failed: compile error: /path/file.aya: hir error: undefined function `f` at 3:5
//   error: check failed: compile error: /path/file.aya: parse error: ... (at 3:5)
//   warning: function `print` may have effect `alloc`; consider adding #[alloc]
//     --> std/src/io.aya:66:11
//      |
//   66 |     buf = [c]
//      |           ^
// 返回 [{ file, line, col, message, severity }]，severity: 0=error, 1=warning；
// line/col 为 1-based（调用方转 0-based）。

'use strict';

const WRAPPER_PREFIXES = [
    /^(check|build|run|package|install)\s+failed:\s*/i,
    /^compile error:\s*/i,
    /^(hir|mir|lir|parse|import|driver|package|borrow|serialize)\s+error:\s*/i,
];

function cleanMessage(msg) {
    let m = (msg || '').trim();
    // 反复剥离，直到稳定（前缀与文件路径可能交替出现）
    let prev;
    do {
        prev = m;
        for (const re of WRAPPER_PREFIXES) m = m.replace(re, '');
        m = m.replace(/^\S+\.aya:\s*/, '');
    } while (m !== prev);
    return m.trim();
}

function extractPos(msg) {
    let m = msg.match(/\(at\s+(\d+):(\d+)\)/);
    if (m) return { line: +m[1], col: +m[2] };
    m = msg.match(/\bat\s+(\d+):(\d+)/);
    if (m) return { line: +m[1], col: +m[2] };
    m = msg.match(/(\S+\.aya):(\d+):(\d+)/);
    if (m) return { file: m[1], line: +m[2], col: +m[3] };
    // 形如 `error: 12:1: error: ...`（report_error 剥离路径后的形态）
    m = msg.match(/(?:^|\s)(\d+):(\d+):\s*(?:error|warning|note):/);
    if (m) return { line: +m[1], col: +m[2] };
    return { line: 0, col: 0 };
}

/// 计算波浪线范围（0-based，同一行内 [start, end)）。
/// - 声明级诊断（缺 return / 注解缺失）→ 整条 `fn ...` 声明
/// - 消息里反引号符号名 → 行内该名字
/// - 其余 → 位置处的完整标识符；再不行 → 整行（去缩进）
function computeRange(lineText, col, message) {
    const line = lineText || '';
    const start = Math.max(0, Math.min((col || 1) - 1, line.length));
    const decl = (message || '').match(/function `([^`]+)` (?:has non-void return type|may have effect|may throw|is #\[)/);
    if (decl) {
        const brace = line.indexOf('{');
        const end = brace > 0 ? brace : line.replace(/\s+$/, '').length;
        const fnIdx = line.search(/\b(?:pub\s+)?fn\b/);
        const s0 = fnIdx >= 0 ? fnIdx : start;
        return { start: s0, end: Math.max(end, s0 + 1) };
    }
    const bt = (message || '').match(/`([A-Za-z_]\w*)`/);
    if (bt) {
        let idx = line.indexOf(bt[1], start);
        if (idx < 0) idx = line.indexOf(bt[1]);
        if (idx >= 0) return { start: idx, end: idx + bt[1].length };
    }
    const isWord = (c) => /\w/.test(c);
    if (start < line.length && isWord(line[start])) {
        let s0 = start, e0 = start;
        while (s0 > 0 && isWord(line[s0 - 1])) s0--;
        while (e0 < line.length && isWord(line[e0])) e0++;
        return { start: s0, end: e0 };
    }
    const trimmedStart = line.search(/\S/);
    const trimmedEnd = line.replace(/\s+$/, '').length;
    if (trimmedStart >= 0) return { start: trimmedStart, end: trimmedEnd };
    return { start, end: start + 1 };
}

const SKIP_LINE = /^(stage\b|check passed|building\b|build ok|running:|package:|installing\b|note:)/;
const SNIPPET_LINE = /^(\|.*|\d+\s*\|.*|\^+\s*|\|\s*\^+.*)$/;

function parseCompilerOutput(out, defaultFile, projectRoot) {
    const path = require('path');
    const diags = [];
    let pending = null;

    const flush = (loc) => {
        if (!pending) return;
        const file = (loc && loc.file) || pending.file || defaultFile;
        const line = (loc && loc.line) ? loc.line : (pending.line || 0);
        const col = (loc && loc.col) ? loc.col : (pending.col || 0);
        diags.push({
            file,
            line: line > 0 ? line : 1,
            col: col > 0 ? col : 1,
            message: pending.message,
            severity: pending.severity,
        });
        pending = null;
    };

    for (const raw of (out || '').split('\n')) {
        const t = raw.trim();
        if (!t) continue;
        if (SKIP_LINE.test(t)) continue;
        if (SNIPPET_LINE.test(t)) continue;

        // 位置行：--> file:line:col（给上一条挂位置）
        const loc = t.match(/^-->\s*(.+?):(\d+):(\d+)\s*$/);
        if (loc) {
            flush({ file: loc[1], line: +loc[2], col: +loc[3] });
            continue;
        }

        // warning:/error: 前缀
        const sev = t.match(/^(warning|error):\s*(.*)$/);
        if (sev) {
            flush(null);
            const rawMsg = sev[2];
            const pos = extractPos(rawMsg);
            pending = {
                message: cleanMessage(rawMsg),
                severity: sev[1] === 'warning' ? 1 : 0,
                file: pos.file,
                line: pos.line,
                col: pos.col,
            };
            continue;
        }

        // 裸 "file.aya:line:col: message"
        const cl = t.match(/^(\S+\.aya):(\d+):(\d+):\s*(.+)$/);
        if (cl) {
            flush(null);
            diags.push({ file: cl[1], line: +cl[2], col: +cl[3], message: cleanMessage(cl[4]), severity: 0 });
            continue;
        }

        // 其它行：若前一条还没有位置信息，作为其补充（如 borrow 错误的说明行）
        if (pending && !pending.line) {
            pending.message += (pending.message ? ' ' : '') + cleanMessage(t);
        }
    }
    flush(null);

    // 规范化文件路径
    for (const d of diags) {
        if (d.file && !path.isAbsolute(d.file)) {
            d.file = path.resolve(projectRoot || '.', d.file);
        }
    }
    return diags;
}

module.exports = { parseCompilerOutput, cleanMessage, extractPos, computeRange };
