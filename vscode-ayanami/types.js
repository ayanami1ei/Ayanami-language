// ─── Compiler-provided variable types (`ayanami types <file>`) ───────
// 纯函数，便于 test.js 直接测试。

/// 解析编译器 stdout 中的 JSON（容忍前后噪声）。
function parseTypesOutput(stdout) {
    if (!stdout) return [];
    const start = stdout.indexOf('{"types"');
    if (start === -1) return [];
    const end = stdout.lastIndexOf('}');
    if (end <= start) return [];
    try {
        const obj = JSON.parse(stdout.slice(start, end + 1));
        if (!obj || !Array.isArray(obj.types)) return [];
        return obj.types.filter((t) => t
            && typeof t.name === 'string'
            && typeof t.type === 'string'
            && Number.isInteger(t.line)
            && Number.isInteger(t.col));
    } catch (_) {
        return [];
    }
}

/// 校验条目位置处确实是该变量名（1-based line/col），并去重。
function validateTypeEntries(text, entries) {
    const lines = text.split(/\r?\n/);
    const out = [];
    const seen = new Set();
    for (const e of entries) {
        const lineText = lines[e.line - 1];
        if (lineText === undefined) continue;
        const idx = e.col - 1;
        if (idx < 0) continue;
        const rest = lineText.slice(idx);
        if (!rest.startsWith(e.name)) continue;
        const next = rest.charAt(e.name.length);
        if (next && /[\w]/.test(next)) continue;
        const key = `${e.line}:${e.col}:${e.name}`;
        if (seen.has(key)) continue;
        seen.add(key);
        out.push(e);
    }
    return out;
}

module.exports = { parseTypesOutput, validateTypeEntries };
