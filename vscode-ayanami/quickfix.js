// ─── Quick fix: 自动补 import ─────────────────────────────────────────
// 纯函数，便于 test.js 直接测试。

/// 从诊断消息里提取未知符号（类型/函数）。
function parseUnknownSymbol(message) {
    if (!message) return null;
    let m = message.match(/类型 `([A-Za-z_]\w*)` 未知/);
    if (m) return { name: m[1], kind: 'type' };
    m = message.match(/type `([^`]+)` has no method/);
    if (m) return { name: baseName(m[1]), kind: 'type' };
    m = message.match(/undefined function `([A-Za-z_]\w*)\.\w+`/);
    if (m) return { name: m[1], kind: 'type' };
    m = message.match(/undefined function `([A-Za-z_]\w*)`/);
    if (m) return { name: m[1], kind: 'fn' };
    return null;
}

function baseName(typeStr) {
    return String(typeStr).split('<')[0].split('[')[0].trim();
}

/// 在包符号表中查找定义该符号的包（返回 stem 列表，按推荐度排序）。
/// packages: [{ stem, functions:[{name}], structs:[{name}], enums:[{name}] }]
/// 注意：.lcl 符号表含依赖符号（std 里什么都有），因此排序：
///   1. 包名与符号同名（不区分大小写）优先
///   2. 非 std 优先（更具体的包）
///   3. 符号数少的包优先（依赖更少）
function findImportCandidates(symbol, packages) {
    if (!symbol) return [];
    const hits = [];
    for (const pkg of packages || []) {
        const hit = symbol.kind === 'type'
            ? (pkg.structs || []).some((s) => s.name === symbol.name)
                || (pkg.enums || []).some((e) => e.name === symbol.name)
            : (pkg.functions || []).some((f) => f.name === symbol.name);
        if (hit && !hits.some((h) => h.stem === pkg.stem)) hits.push(pkg);
    }
    const size = (p) => (p.functions || []).length + (p.structs || []).length + (p.enums || []).length;
    hits.sort((a, b) => {
        const aName = a.stem.toLowerCase() === symbol.name.toLowerCase() ? 0 : 1;
        const bName = b.stem.toLowerCase() === symbol.name.toLowerCase() ? 0 : 1;
        if (aName !== bName) return aName - bName;
        const aStd = a.stem === 'std' ? 1 : 0;
        const bStd = b.stem === 'std' ? 1 : 0;
        if (aStd !== bStd) return aStd - bStd;
        return size(a) - size(b);
    });
    return hits.map((p) => p.stem);
}

/// 生成插入 import 的编辑信息；已存在同模块 import 时返回 null。
function importEditInfo(documentText, stem) {
    const esc = stem.replace(/[.*+?^${}()|[\]\\]/g, '\\$&');
    if (new RegExp(`import\\s+"${esc}"`).test(documentText)) return null;
    const lines = documentText.split(/\r?\n/);
    let lastImport = -1;
    for (let i = 0; i < lines.length; i++) {
        if (/^\s*import\s+"/.test(lines[i])) lastImport = i;
    }
    const line = lastImport >= 0 ? lastImport + 1 : 0;
    return { line, text: `import "${stem}";\n` };
}

module.exports = { parseUnknownSymbol, baseName, findImportCandidates, importEditInfo };
