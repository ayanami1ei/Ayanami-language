const vscode = require('vscode');

function activate(context) {
    console.log('ayanami extension activating...');
    // ─── Status Bar ──────────────────────────────────────────────────
    const statusBar = vscode.window.createStatusBarItem(vscode.StatusBarAlignment.Right, 1000);
    statusBar.text = 'Ayanami';
    statusBar.tooltip = 'Ayanami Language';
    statusBar.show();
    context.subscriptions.push(statusBar);
    console.log('ayanami status bar shown');

    function setStatus(text, icon) {
        try {
            statusBar.text = text ? `Ayanami: ${text}` : 'Ayanami';
            statusBar.show();
        } catch (e) {
            console.error('ayanami setStatus error:', e);
        }
    }

    // ─── Completion Provider ─────────────────────────────────────────
    const provider = vscode.languages.registerCompletionItemProvider('ayanami', {
        provideCompletionItems(document, position) {
            const items = [];
            const linePrefix = document.lineAt(position).text.slice(0, position.character);
            const textBefore = document.getText(new vscode.Range(
                new vscode.Position(Math.max(0, position.line - 30), 0), position
            ));

            // ── Detect context ──
            const dotMatch = linePrefix.match(/(\w+)\.$/);
            const nsMatch = linePrefix.match(/(\w+)::$/);
            const afterArrow = linePrefix.match(/->\s*$/);
            const afterColon = linePrefix.match(/:\s*$/);

            // ── Scan documents (current + imported) for symbols ──
            const folder = document.uri.scheme === 'file' ? require('path').dirname(document.uri.fsPath) : null;
            const imported = folder ? resolveImports(document, folder) : { structs: [], namespaces: [], functions: [] };

            const localStructs = scanStructs(document);
            const localNss = scanNamespaces(document);
            const localFns = scanFunctions(document);

            const structs = [...localStructs, ...imported.structs];
            const namespaces = [...new Set([...localNss, ...imported.namespaces])];
            const functions = [...localFns, ...imported.functions];
            const fnByNs = groupByNamespace(functions);
            const vars = scanVariables(document);

            // ── After `.`: struct field completions ──
            if (dotMatch) {
                const typeName = dotMatch[1];
                const fields = getStructFields(document, typeName);
                for (const f of fields) {
                    items.push(makeItem(f, vscode.CompletionItemKind.Field, 'struct field'));
                }
                // Also suggest common methods
                for (const m of ['get_x', 'get_y', 'to_string']) {
                    items.push(makeItem(m, vscode.CompletionItemKind.Method, 'method (common)'));
                }
                return items;
            }

            // ── After `::`: namespace member completions ──
            if (nsMatch) {
                const nsName = nsMatch[1];
                const members = fnByNs[nsName] || [];
                for (const m of members) {
                    items.push(makeItem(m, vscode.CompletionItemKind.Function, 'namespace function'));
                }
                // Sub-namespaces
                for (const ns of namespaces) {
                    if (ns.startsWith(nsName + '.')) {
                        const sub = ns.slice(nsName.length + 1);
                        items.push(makeItem(sub, vscode.CompletionItemKind.Module, 'namespace'));
                    }
                }
                return items;
            }

            // ── After `->`: type completions ──
            if (afterArrow) {
                for (const t of ['int', 'float', 'char', 'bool', 'void']) {
                    items.push(makeItem(t, vscode.CompletionItemKind.TypeParameter, 'return type'));
                }
                for (const s of structs) {
                    items.push(makeItem(s.name, vscode.CompletionItemKind.Struct, 'struct'));
                }
                return items;
            }

            // ── General context: keywords + variables + types ──
            const keywords = [
                { label: 'fn', kind: vscode.CompletionItemKind.Keyword, detail: 'function declaration' },
                { label: 'return', kind: vscode.CompletionItemKind.Keyword, detail: 'return from function' },
                { label: 'if', kind: vscode.CompletionItemKind.Keyword, detail: 'if expression' },
                { label: 'elif', kind: vscode.CompletionItemKind.Keyword, detail: 'else if' },
                { label: 'else', kind: vscode.CompletionItemKind.Keyword, detail: 'else branch' },
                { label: 'for', kind: vscode.CompletionItemKind.Keyword, detail: 'for loop' },
                { label: 'in', kind: vscode.CompletionItemKind.Keyword, detail: 'for iterator' },
                { label: 'while', kind: vscode.CompletionItemKind.Keyword, detail: 'while loop' },
                { label: 'struct', kind: vscode.CompletionItemKind.Keyword, detail: 'struct definition' },
                { label: 'interface', kind: vscode.CompletionItemKind.Keyword, detail: 'interface' },
                { label: 'impl', kind: vscode.CompletionItemKind.Keyword, detail: 'impl block' },
                { label: 'import', kind: vscode.CompletionItemKind.Keyword, detail: 'import' },
                { label: 'namespace', kind: vscode.CompletionItemKind.Keyword, detail: 'namespace' },
                { label: 'pub', kind: vscode.CompletionItemKind.Keyword, detail: 'make public' },
                { label: 'pub(crate)', kind: vscode.CompletionItemKind.Keyword, detail: 'crate-public' },
                { label: 'shared', kind: vscode.CompletionItemKind.Keyword, detail: 'shared ownership' },
                { label: 'unique', kind: vscode.CompletionItemKind.Keyword, detail: 'unique ownership' },
                { label: 'weak', kind: vscode.CompletionItemKind.Keyword, detail: 'weak reference' },
                { label: 'move', kind: vscode.CompletionItemKind.Keyword, detail: 'move ownership' },
                { label: 'clone', kind: vscode.CompletionItemKind.Keyword, detail: 'clone value' },
            ];
            for (const kw of keywords) {
                items.push(new vscode.CompletionItem(kw.label, kw.kind));
            }

            // Type keywords
            for (const t of ['int', 'float', 'char', 'bool', 'void']) {
                items.push(makeItem(t, vscode.CompletionItemKind.TypeParameter, 'type'));
            }

            // Operator overloading method completions
            for (const m of ['add', 'sub', 'mul', 'div', 'rem', 'neg', 'not', 'eq', 'ne', 'lt', 'gt', 'le', 'ge']) {
                items.push(makeItem(m, vscode.CompletionItemKind.Method, 'operator overload'));
            }

            // Self / true / false
            for (const t of ['self', 'true', 'false']) {
                items.push(makeItem(t, vscode.CompletionItemKind.Constant, 'keyword'));
            }

            // Struct names as types
            for (const s of structs) {
                items.push(makeItem(s.name, vscode.CompletionItemKind.Struct, 'struct'));
            }

            // Namespace names (for `::` access)
            for (const ns of namespaces) {
                items.push(makeItem(ns, vscode.CompletionItemKind.Module, 'namespace'));
            }

            // Variables from current file
            for (const v of vars) {
                items.push(makeItem(v, vscode.CompletionItemKind.Variable, 'variable'));
            }

            // Functions as callable completions
            for (const f of functions) {
                items.push(makeItem(f, vscode.CompletionItemKind.Function, 'function'));
            }

            // ── Snippets ──
            const snippets = [
                { label: 'fn main', insert: 'fn main() -> int {\n    ${1:return 0;}\n}' },
                { label: 'fn', insert: 'fn ${1:name}(${2:int param}) -> ${3:int} {\n    ${4}\n}' },
                { label: 'struct', insert: 'struct ${1:Name} {\n    ${2:int field}\n}' },
                { label: 'interface', insert: 'interface ${1:Name} {\n    fn ${2:method}(shared self) -> ${3:int};\n}' },
                { label: 'impl', insert: 'impl ${1:Type} {\n    fn ${2:method}(${3:shared self}) -> ${4:int} {\n        ${5}\n    }\n}' },
                { label: 'if', insert: 'if ${1:condition} {\n    ${2}\n}' },
                { label: 'ifelse', insert: 'if ${1:condition} {\n    ${2}\n} else {\n    ${3}\n}' },
                { label: 'elif', insert: 'if ${1:c1} {\n    ${2}\n} elif ${3:c2} {\n    ${4}\n} else {\n    ${5}\n}' },
                { label: 'while', insert: 'while ${1:condition} {\n    ${2}\n}' },
                { label: 'for', insert: 'for ${1:i} in (${2:start}, ${3:end}) {\n    ${4}\n}' },
                { label: 'namespace', insert: 'namespace ${1:name} {\n    ${2}\n}' },
                { label: 'import', insert: 'import "${1:path}";' },
                { label: 'arr sized', insert: 'unique [${1:int}; ${2:10}]' },
                { label: 'impl add', insert: 'fn add(shared self, shared ${1:Type} other) -> ${1:Type} {\n    ${2}\n}' },
            ];
            for (const s of snippets) {
                const item = new vscode.CompletionItem(s.label, vscode.CompletionItemKind.Snippet);
                item.insertText = new vscode.SnippetString(s.insert);
                items.push(item);
            }

            return items;
        },
    }, '.', ':', ...'abcdefghijklmnopqrstuvwxyz_'); // trigger on . and : too

    context.subscriptions.push(provider);

    // ─── Diagnostic Provider (compiler check on save) ────────────────
    const diagCollection = vscode.languages.createDiagnosticCollection('ayanami');
    context.subscriptions.push(diagCollection);

    context.subscriptions.push(vscode.workspace.onDidSaveTextDocument(doc => {
        try {
            console.log('ayanami save event: lang=' + doc.languageId + ' file=' + doc.uri.fsPath);
            if (doc.languageId !== 'ayanami') return;
            setStatus('checking');
            diagCollection.clear();
            const diagnostics = [];
            const fs = require('fs');
            const path = require('path');

            let ayanamiPath = findAyanamiPath(context);
            if (!ayanamiPath) {
                setStatus('not found');
                console.log('ayanami: compiler not found');
                diagnostics.push(new vscode.Diagnostic(new vscode.Range(0, 0, 0, 10),
                    'ayanami: compiler not found in PATH', vscode.DiagnosticSeverity.Warning));
                diagCollection.set(doc.uri, diagnostics);
                return;
            }
            console.log('ayanami using:', ayanamiPath);

            const { execFileSync } = require('child_process');
            let out = '';
            try {
                out = execFileSync(ayanamiPath, ['check', doc.uri.fsPath], {
                    timeout: 15000,
                    encoding: 'utf8',
                    cwd: path.dirname(doc.uri.fsPath),
                    stdio: ['pipe', 'pipe', 'pipe'],
                });
                console.log('ayanami check ok');
                setStatus('ok');
            } catch (e) {
                out = (e.stdout || '') + (e.stderr || '');
                console.log('ayanami check failed, output:', out.slice(0, 300));
            }

            if (out) {
                for (const line of out.split('\n')) {
                    const t = line.trim();
                    if (!t) continue;
                    const msg = t.replace(/^[^:]*:\s*/, '');
                    if (!msg) continue;
                    const m = t.match(/:(\d+):(\d+)/);
                    const range = m
                        ? new vscode.Range(Math.max(0, parseInt(m[1]) - 1), 0, Math.max(0, parseInt(m[1]) - 1), 1000)
                        : new vscode.Range(0, 0, 0, 10);
                    diagnostics.push(new vscode.Diagnostic(range, msg, vscode.DiagnosticSeverity.Error));
                }
            }

            if (diagnostics.length > 0) {
                setStatus(`${diagnostics.length} error(s)`);
            }

            diagCollection.set(doc.uri, diagnostics);
        } catch (e) {
            console.error('ayanami diagnostic error:', e);
            setStatus('error');
        }
    }));
    console.log('ayanami extension activated');

    setTimeout(() => { setStatus('ready'); }, 1000);

function findAyanamiPath(context) {
    const fs = require('fs');
    const path = require('path');

    // 0. Check user setting first
    try {
        const config = vscode.workspace.getConfiguration('ayanami');
        const setting = config.get('compilerPath', '');
        if (setting && fs.existsSync(setting)) {
            console.log('ayanami from setting:', setting);
            return setting;
        }
    } catch (_) {}

    // 1. Search PATH
    const envPath = (process.env.PATH || '').split(path.delimiter);
    for (const dir of envPath) {
        const candidate = path.join(dir, 'ayanami');
        try {
            if (fs.existsSync(candidate)) {
                console.log('ayanami found in PATH:', candidate);
                return candidate;
            }
        } catch (_) {}
    }

    // 2. Check relative to workspace/project dirs
    try {
        const workspaces = vscode.workspace.workspaceFolders || [];
        for (const ws of workspaces) {
            let dir = ws.uri.fsPath;
            for (let i = 0; i < 5; i++) {
                for (const sub of ['build/ayanami', 'target/release/ayanami', 'target/debug/ayanami']) {
                    const p = path.join(dir, sub);
                    if (fs.existsSync(p)) { console.log('ayanami found in workspace:', p); return p; }
                }
                const parent = path.dirname(dir);
                if (parent === dir) break;
                dir = parent;
            }
        }
    } catch (_) {}

    console.log('ayanami: compiler not found');
    return null;
}
}

// ─── Helper: scan struct definitions ─────────────────────────────────
function scanStructs(doc) {
    const structs = [];
    const text = doc.getText();
    const re = /struct\s+(\w+)\s*\{/g;
    let m;
    while ((m = re.exec(text)) !== null) {
        structs.push({ name: m[1], line: m.index });
    }
    return structs;
}

// ─── Helper: scan namespace definitions ─────────────────────────────
function scanNamespaces(doc) {
    const nss = [];
    const text = doc.getText();
    const re = /namespace\s+(\w+)\s*\{/g;
    let m;
    while ((m = re.exec(text)) !== null) {
        nss.push(m[1]);
    }
    return nss;
}

// ─── Helper: scan function signatures ────────────────────────────────
function scanFunctions(doc) {
    const fns = [];
    const text = doc.getText();
    const re = /(?:pub\s+)?fn\s+(\w+)\s*\(/g;
    let m;
    while ((m = re.exec(text)) !== null) {
        fns.push(m[1]);
    }
    return fns;
}

// ─── Helper: group functions by namespace ────────────────────────────
function groupByNamespace(fns) {
    const groups = {};
    for (const f of fns) {
        const parts = f.split('.');
        if (parts.length > 1) {
            const ns = parts.slice(0, -1).join('.');
            const name = parts[parts.length - 1];
            if (!groups[ns]) groups[ns] = [];
            groups[ns].push(name);
        }
    }
    return groups;
}

// ─── Helper: scan variable assignments ──────────────────────────────
function scanVariables(doc) {
    const vars = new Set();
    const text = doc.getText();
    // Match:  name =  or  for name in
    const re = /(?:(?:for\s+(\w+)\s+in)|(?:(\w+)\s*=(?!=)))/g;
    let m;
    while ((m = re.exec(text)) !== null) {
        if (m[1]) vars.add(m[1]);
        if (m[2]) vars.add(m[2]);
    }
    // Match function params:  type name, type name)
    const re2 = /(\w+)\s+(\w+)(?=[,)]|\s*->)/g;
    while ((m = re2.exec(text)) !== null) {
        const typeName = m[1];
        const paramName = m[2];
        // Skip if first word is a type keyword
        if (!['int', 'float', 'char', 'bool', 'void', 'shared', 'unique', 'weak'].includes(typeName)) continue;
        vars.add(paramName);
    }
    return [...vars];
}

// ─── Helper: resolve imported .aya files ─────────────────────────────
function resolveImports(doc, folder) {
    const fs = require('fs');
    const path = require('path');
    const result = { structs: [], namespaces: [], functions: [] };
    const text = doc.getText();
    const re = /import\s+"([^"]+\.aya)"/g;
    let m;
    while ((m = re.exec(text)) !== null) {
        const importPath = m[1];
        // Try relative to current file's directory
        const fullPath = path.resolve(folder, importPath);
        try {
            const content = fs.readFileSync(fullPath, 'utf8');
            const structs = scanStructsRaw(content);
            for (const s of structs) result.structs.push(s);
            const nss = scanNamespacesRaw(content);
            for (const ns of nss) result.namespaces.push(ns);
            const fns = scanFunctionsRaw(content);
            for (const f of fns) result.functions.push(f);
        } catch (e) {
            // File not found, skip
        }
    }
    return result;
}

function scanStructsRaw(text) {
    const structs = [];
    const re = /struct\s+(\w+)\s*\{/g;
    let m;
    while ((m = re.exec(text)) !== null) structs.push({ name: m[1], line: m.index });
    return structs;
}

function scanNamespacesRaw(text) {
    const nss = [];
    const re = /namespace\s+(\w+)\s*\{/g;
    let m;
    while ((m = re.exec(text)) !== null) nss.push(m[1]);
    return nss;
}

function scanFunctionsRaw(text) {
    const fns = [];
    const re = /(?:pub\s+)?fn\s+(\w+)\s*\(/g;
    let m;
    while ((m = re.exec(text)) !== null) fns.push(m[1]);
    return fns;
}

// ─── Helper: get struct fields ──────────────────────────────────────
function getStructFields(doc, typeName) {
    const text = doc.getText();
    // Find struct definition
    const re = new RegExp('struct\\s+' + typeName + '\\s*\\{([^}]*)\\}', 'm');
    const m = re.exec(text);
    if (!m) return [];
    const body = m[1];
    const fields = [];
    const re2 = /(\w+)\s+(\w+)/g;
    let fm;
    while ((fm = re2.exec(body)) !== null) {
        fields.push(fm[2]); // field name
    }
    return fields;
}

// ─── Helper: create completion item ─────────────────────────────────
function makeItem(label, kind, detail) {
    const item = new vscode.CompletionItem(label, kind);
    item.detail = detail;
    return item;
}

function deactivate() {}

module.exports = { activate, deactivate };
