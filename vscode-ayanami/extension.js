const vscode = require('vscode');

function activate(context) {
    console.log('ayanami extension activating...');

    // ─── Status Bar ──────────────────────────────────────────────────
    const statusBar = vscode.window.createStatusBarItem(vscode.StatusBarAlignment.Left, 1000);
    statusBar.text = 'Ayanami';
    statusBar.tooltip = 'Ayanami Language — click to restart';
    statusBar.command = 'ayanami.restart';
    statusBar.show();
    context.subscriptions.push(statusBar);

    function setStatus(text, icon) {
        try {
            statusBar.text = text ? `Ayanami: ${text}` : 'Ayanami';
            statusBar.tooltip = text ? `Ayanami: ${text} — click to restart` : 'Ayanami Language — click to restart';
            statusBar.show();
        } catch (e) {
            console.error('ayanami setStatus error:', e);
        }
    }

    // ─── Restart command ─────────────────────────────────────────────
    const restartCmd = vscode.commands.registerCommand('ayanami.restart', () => {
        console.log('ayanami restart requested');
        setStatus('restarting');
        // Clear all diagnostics
        for (const coll of [diagCollection]) {
            if (coll) coll.clear();
        }
        // Re-check all open .aya files
        const files = vscode.workspace.textDocuments.filter(d => d.languageId === 'ayanami');
        for (const doc of files) {
            runCheck(doc, diagCollection, context, setStatus);
        }
        setStatus('ready');
        vscode.window.showInformationMessage('Ayanami: restarted');
    });
    context.subscriptions.push(restartCmd);

    // ─── Completion Provider ─────────────────────────────────────────
    const completionProvider = vscode.languages.registerCompletionItemProvider('ayanami', {
        provideCompletionItems(document, position) {
            const items = [];
            const linePrefix = document.lineAt(position).text.slice(0, position.character);
            const textBefore = document.getText(new vscode.Range(
                new vscode.Position(Math.max(0, position.line - 30), 0), position
            ));

            const dotMatch = linePrefix.match(/(\w+)\.$/);
            const nsMatch = linePrefix.match(/(\w+)::$/);
            const afterArrow = linePrefix.match(/->\s*$/);

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

            if (dotMatch) {
                const typeName = dotMatch[1];
                const fields = getStructFields(document, typeName);
                for (const f of fields) {
                    items.push(makeItem(f, vscode.CompletionItemKind.Field, 'struct field'));
                }
                for (const m of ['get_x', 'get_y', 'to_string']) {
                    items.push(makeItem(m, vscode.CompletionItemKind.Method, 'method (common)'));
                }
                return items;
            }

            if (nsMatch) {
                const nsName = nsMatch[1];
                const members = fnByNs[nsName] || [];
                for (const m of members) {
                    items.push(makeItem(m, vscode.CompletionItemKind.Function, 'namespace function'));
                }
                for (const ns of namespaces) {
                    if (ns.startsWith(nsName + '.')) {
                        const sub = ns.slice(nsName.length + 1);
                        items.push(makeItem(sub, vscode.CompletionItemKind.Module, 'namespace'));
                    }
                }
                return items;
            }

            if (afterArrow) {
                for (const t of ['int', 'float', 'char', 'bool', 'void']) {
                    items.push(makeItem(t, vscode.CompletionItemKind.TypeParameter, 'return type'));
                }
                for (const s of structs) {
                    items.push(makeItem(s.name, vscode.CompletionItemKind.Struct, 'struct'));
                }
                return items;
            }

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

            for (const t of ['int', 'float', 'char', 'bool', 'void']) {
                items.push(makeItem(t, vscode.CompletionItemKind.TypeParameter, 'type'));
            }

            for (const m of ['add', 'sub', 'mul', 'div', 'rem', 'neg', 'not', 'eq', 'ne', 'lt', 'gt', 'le', 'ge']) {
                items.push(makeItem(m, vscode.CompletionItemKind.Method, 'operator overload'));
            }

            for (const t of ['self', 'true', 'false']) {
                items.push(makeItem(t, vscode.CompletionItemKind.Constant, 'keyword'));
            }

            for (const s of structs) {
                items.push(makeItem(s.name, vscode.CompletionItemKind.Struct, 'struct'));
            }

            for (const ns of namespaces) {
                items.push(makeItem(ns, vscode.CompletionItemKind.Module, 'namespace'));
            }

            for (const v of vars) {
                items.push(makeItem(v, vscode.CompletionItemKind.Variable, 'variable'));
            }

            for (const f of functions) {
                items.push(makeItem(f, vscode.CompletionItemKind.Function, 'function'));
            }

            const snippets = [
                { label: 'fn main', insert: 'fn main() -> int {\n    ${1:return 0;}\n}' },
                { label: 'fn', insert: 'fn ${1:name}(${2:int param}) -> ${3:int} {\n    ${4}\n}' },
                { label: 'struct', insert: 'struct ${1:Name} {\n    ${2:int field}\n}' },
                { label: 'interface', insert: 'interface ${1:Name} {\n    fn ${2:method}(shared self) -> ${3:int}\n}' },
                { label: 'impl', insert: 'impl ${1:Type} {\n    fn ${2:method}(${3:shared self}) -> ${4:int} {\n        ${5}\n    }\n}' },
                { label: 'if', insert: 'if ${1:condition} {\n    ${2}\n}' },
                { label: 'ifelse', insert: 'if ${1:condition} {\n    ${2}\n} else {\n    ${3}\n}' },
                { label: 'elif', insert: 'if ${1:c1} {\n    ${2}\n} elif ${3:c2} {\n    ${4}\n} else {\n    ${5}\n}' },
                { label: 'while', insert: 'while ${1:condition} {\n    ${2}\n}' },
                { label: 'for', insert: 'for ${1:i} in (${2:start}, ${3:end}) {\n    ${4}\n}' },
                { label: 'namespace', insert: 'namespace ${1:name} {\n    ${2}\n}' },
                { label: 'import', insert: 'import "${1:path}"' },
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
    }, '.', ':', ...'abcdefghijklmnopqrstuvwxyz_');
    context.subscriptions.push(completionProvider);

    // ─── Hover Provider ──────────────────────────────────────────────
    const hoverProvider = vscode.languages.registerHoverProvider('ayanami', {
        provideHover(document, position) {
            const range = document.getWordRangeAtPosition(position, /[\w.]+/);
            if (!range) return null;
            const word = document.getText(range);

            const folder = document.uri.scheme === 'file' ? require('path').dirname(document.uri.fsPath) : null;
            const imported = folder ? resolveImports(document, folder) : { structs: [], namespaces: [], functions: [] };
            const localStructs = scanStructs(document);
            const localFns = scanFunctions(document);
            const structs = [...localStructs, ...imported.structs];
            const functions = [...localFns, ...imported.functions];

            // Check if it's a struct name
            for (const s of structs) {
                if (s.name === word) {
                    const fields = getStructFields(document, word);
                    let md = `**struct \`${word}\`**  \n`;
                    if (fields.length > 0) {
                        md += '---\n\n';
                        for (const f of fields) {
                            md += `- \`${f}\`\n`;
                        }
                    }
                    return new vscode.Hover(new vscode.MarkdownString(md));
                }
            }

            // Check if it's a function name
            for (const f of functions) {
                if (f === word) {
                    return new vscode.Hover(new vscode.MarkdownString(`**fn \`${word}\`**`));
                }
            }

            // Check for keywords
            const keywordDocs = {
                'fn': '**fn** — function declaration\n\n`fn name(params) -> ReturnType { body }`',
                'return': '**return** — return from function\n\n`return expr;`',
                'if': '**if** — conditional\n\n`if cond { ... } elif cond { ... } else { ... }`',
                'elif': '**elif** — else if\n\n`if c1 { ... } elif c2 { ... } else { ... }`',
                'else': '**else** — else branch',
                'for': '**for** — for loop\n\n`for var in (start, end) { ... }`\n`for var in (start, end, step) { ... }`',
                'while': '**while** — while loop\n\n`while cond { ... }`',
                'struct': '**struct** — struct definition\n\n`struct Name { type field }`',
                'interface': '**interface** — interface definition\n\n`interface Name { fn method(shared self) -> Ret; }`',
                'impl': '**impl** — impl block\n\n`impl Type { fn method(...) { ... } }`',
                'import': '**import** — import module\n\n`import "path"`',
                'namespace': '**namespace** — namespace\n\n`namespace name { fn ... }` — accessed via `name::fn()`',
                'shared': '**shared** — shared ownership (refcounted heap)\n\n`shared T` — multiple references, runtime refcounting',
                'unique': '**unique** — unique ownership\n\n`unique T` — single owner, deterministic free',
                'weak': '**weak** — weak reference (non-owning)\n\n`weak T` — does not affect refcount, must be promoted',
                'move': '**move** — transfer ownership\n\n`move x` — consumes the value, `x` becomes unavailable',
                'clone': '**clone** — deep copy\n\n`clone x` — creates an independent copy',
                'int': '**int** — 64-bit signed integer',
                'float': '**float** — 64-bit floating point',
                'char': '**char** — single character (8-bit)',
                'bool': '**bool** — boolean (true/false)',
                'void': '**void** — no return value',
                'self': '**self** — the receiver of a method call\n\nAvailable in method bodies within `impl` blocks.',
                'true': '**true** — boolean literal',
                'false': '**false** — boolean literal',
                'pub': '**pub** — make item visible outside the module',
                'null': '**null** — nullable pointer value\n\n`shared T` or `unique T` can be null. Compare with `== null`.',
            };
            if (keywordDocs[word]) {
                return new vscode.Hover(new vscode.MarkdownString(keywordDocs[word]));
            }

            return null;
        }
    });
    context.subscriptions.push(hoverProvider);

    // ─── Diagnostic Provider (compiler check on save) ────────────────
    const diagCollection = vscode.languages.createDiagnosticCollection('ayanami');
    context.subscriptions.push(diagCollection);

    function runCheck(doc, collection, ctx, statusFn) {
        try {
            if (doc.languageId !== 'ayanami') return;
            statusFn('checking');
            collection.clear();
            const diagnostics = [];
            const fs = require('fs');
            const path = require('path');

            let ayanamiPath = findAyanamiPath(ctx);
            if (!ayanamiPath) {
                statusFn('not found');
                diagnostics.push(new vscode.Diagnostic(new vscode.Range(0, 0, 0, 10),
                    'ayanami: compiler not found in PATH', vscode.DiagnosticSeverity.Warning));
                collection.set(doc.uri, diagnostics);
                return;
            }

            const { execFileSync } = require('child_process');
            const path = require('path');

            // Use project root as CWD (walk up from file to find ayanami.toml)
            let projectRoot = path.dirname(doc.uri.fsPath);
            for (let i = 0; i < 10; i++) {
                if (fs.existsSync(path.join(projectRoot, 'ayanami.toml'))) break;
                const parent = path.dirname(projectRoot);
                if (parent === projectRoot) { projectRoot = path.dirname(doc.uri.fsPath); break; }
                projectRoot = parent;
            }

            let out = '';
            try {
                out = execFileSync(ayanamiPath, ['check', doc.uri.fsPath], {
                    timeout: 15000,
                    encoding: 'utf8',
                    cwd: projectRoot,
                    stdio: ['pipe', 'pipe', 'pipe'],
                });
                statusFn('ok');
            } catch (e) {
                out = (e.stdout || '') + (e.stderr || '');
            }

            if (out) {
                for (const line of out.split('\n')) {
                    const t = line.trim();
                    if (!t || t.startsWith('stage') || t.startsWith('check passed') || t.startsWith('building') || t.startsWith('build ok')) continue;

                    // Format: file:line:col: error: message (primary)
                    const cl = t.match(/^([^:]+):(\d+):(\d+):\s*(.+)/);
                    if (cl) {
                        const l = Math.max(0, parseInt(cl[2]) - 1);
                        diagnostics.push(new vscode.Diagnostic(new vscode.Range(l, 0, l, 1000), cl[4], vscode.DiagnosticSeverity.Error));
                        continue;
                    }

                    // Format: ... (at line:col)
                    const at = t.match(/at (\d+):(\d+)\)?$/);
                    if (at) {
                        const l = Math.max(0, parseInt(at[1]) - 1);
                        const msg = t.replace(/\s*\(?at \d+:\d+\)?\s*$/, '').replace(/^[^:]+:\s*/, '');
                        diagnostics.push(new vscode.Diagnostic(new vscode.Range(l, 0, l, 1000), msg, vscode.DiagnosticSeverity.Error));
                        continue;
                    }

                    // Fallback
                    diagnostics.push(new vscode.Diagnostic(new vscode.Range(0, 0, 0, 10), t, vscode.DiagnosticSeverity.Error));
                }
            }

            if (diagnostics.length > 0) {
                statusFn(`${diagnostics.length} error(s)`);
            }

            collection.set(doc.uri, diagnostics);
        } catch (e) {
            console.error('ayanami diagnostic error:', e);
            statusFn('error');
        }
    }

    // Check on save
    context.subscriptions.push(vscode.workspace.onDidSaveTextDocument(doc => {
        runCheck(doc, diagCollection, context, setStatus);
    }));

    // Check on open (with small delay to let editor settle)
    context.subscriptions.push(vscode.workspace.onDidOpenTextDocument(doc => {
        setTimeout(() => runCheck(doc, diagCollection, context, setStatus), 500);
    }));

    console.log('ayanami extension activated');
    setTimeout(() => { setStatus('ready'); }, 1000);

    // ─── Defs Cache (for Go to Definition) ────────────────────────────
    const defsCache = {};

    function fetchDefs(filePath) {
        const cached = defsCache[filePath];
        if (cached && Array.isArray(cached)) return cached;
        try {
            const ayanamiPath = findAyanamiPath(context);
            if (!ayanamiPath) return [];
            const { execFileSync } = require('child_process');
            const out = execFileSync(ayanamiPath, ['defs', filePath], {
                timeout: 10000,
                encoding: 'utf8',
                cwd: require('path').dirname(filePath),
                stdio: ['pipe', 'pipe', 'pipe'],
            });
            const defs = JSON.parse(out);
            defsCache[filePath] = defs;
            return defs;
        } catch (e) {
            return [];
        }
    }

    // ─── Definition Provider (Go to Definition) ───────────────────────
    const defProvider = vscode.languages.registerDefinitionProvider('ayanami', {
        provideDefinition(document, position) {
            const range = document.getWordRangeAtPosition(position, /[\w.]+/);
            if (!range) return null;
            const word = document.getText(range);

            const allDefs = fetchDefs(document.uri.fsPath);

            for (const d of allDefs) {
                const defName = d.name;
                const simpleName = defName.includes('.') ? defName.split('.').pop() : defName;
                if (defName === word || simpleName === word) {
                    const line = Math.max(0, d.line - 1);
                    const col = Math.max(0, d.col - 1);
                    const defUri = vscode.Uri.file(d.file);
                    const defRange = new vscode.Range(line, col, line, col);
                    return new vscode.Location(defUri, defRange);
                }
            }
            return null;
        }
    });
    context.subscriptions.push(defProvider);

    // Invalidate defs cache on document change
    context.subscriptions.push(vscode.workspace.onDidChangeTextDocument(e => {
        delete defsCache[e.document.uri.fsPath];
    }));

    // Invalidate defs cache on document close
    context.subscriptions.push(vscode.workspace.onDidCloseTextDocument(doc => {
        delete defsCache[doc.uri.fsPath];
    }));

    // ─── Run Main command ─────────────────────────────────────────────
    const runMainCmd = vscode.commands.registerCommand('ayanami.runMain', async (filePath) => {
        try {
            const ayanamiPath = findAyanamiPath(context);
            if (!ayanamiPath) {
                vscode.window.showErrorMessage('ayanami: compiler not found');
                return;
            }
            const { execFileSync } = require('child_process');
            const path = require('path');
            const cwd = path.dirname(filePath);
            const terminal = vscode.window.createTerminal({ name: 'Ayanami Run' });
            terminal.show();
            terminal.sendText(`"${ayanamiPath}" run "${filePath}"`);
        } catch (e) {
            vscode.window.showErrorMessage(`ayanami run failed: ${e.message}`);
        }
    });
    context.subscriptions.push(runMainCmd);

    // ─── CodeLens Provider (Run button above fn main) ─────────────────
    const lensProvider = vscode.languages.registerCodeLensProvider('ayanami', {
        provideCodeLenses(document) {
            const lenses = [];
            const text = document.getText();
            const re = /(?:(?:pub\s+)?fn\s+main)\s*\(/g;
            let m;
            while ((m = re.exec(text)) !== null) {
                const pos = document.positionAt(m.index);
                const line = pos.line;
                const range = new vscode.Range(line, 0, line, 0);
                lenses.push(new vscode.CodeLens(range, {
                    title: '▶ Run',
                    command: 'ayanami.runMain',
                    arguments: [document.uri.fsPath],
                }));
            }
            return lenses;
        }
    });
    context.subscriptions.push(lensProvider);

    // ─── Format command ───────────────────────────────────────────────
    const fmtCmd = vscode.commands.registerCommand('ayanami.fmt', async (filePath) => {
        try {
            const ayanamiPath = findAyanamiPath(context);
            if (!ayanamiPath) {
                vscode.window.showErrorMessage('ayanami: compiler not found');
                return;
            }
            const { execFileSync } = require('child_process');
            const path = require('path');

            const editor = vscode.window.activeTextEditor;
            if (!editor || editor.document.languageId !== 'ayanami') {
                vscode.window.showErrorMessage('ayanami: not an .aya file');
                return;
            }

            const fpath = filePath || editor.document.uri.fsPath;
            const out = execFileSync(ayanamiPath, ['fmt', fpath], {
                timeout: 15000,
                encoding: 'utf8',
                cwd: path.dirname(fpath),
                stdio: ['pipe', 'pipe', 'pipe'],
            });

            // Replace the entire document with formatted output
            const fullRange = new vscode.Range(0, 0, editor.document.lineCount, 0);
            await editor.edit(editBuilder => {
                editBuilder.replace(fullRange, out);
            });
            vscode.window.showInformationMessage('ayanami: formatted');
        } catch (e) {
            const msg = (e.stdout || '') + (e.stderr || '') || e.message;
            vscode.window.showErrorMessage(`ayanami fmt failed: ${msg}`);
        }
    });
    context.subscriptions.push(fmtCmd);

    // ─── Format Document Provider (Shift+Alt+F) ───────────────────────
    const formatProvider = vscode.languages.registerDocumentFormattingEditProvider('ayanami', {
        provideDocumentFormattingEdits(document) {
            return new Promise((resolve, reject) => {
                try {
                    const ayanamiPath = findAyanamiPath(context);
                    if (!ayanamiPath) {
                        reject('ayanami: compiler not found');
                        return;
                    }
                    const { execFileSync } = require('child_process');
                    const path = require('path');

                    const out = execFileSync(ayanamiPath, ['fmt', document.uri.fsPath], {
                        timeout: 15000,
                        encoding: 'utf8',
                        cwd: path.dirname(document.uri.fsPath),
                        stdio: ['pipe', 'pipe', 'pipe'],
                    });

                    const lastLine = document.lineCount;
                    const fullRange = new vscode.Range(0, 0, lastLine - 1, document.lineAt(lastLine - 1).text.length);
                    resolve([new vscode.TextEdit(fullRange, out)]);
                } catch (e) {
                    const msg = (e.stdout || '') + (e.stderr || '') || e.message;
                    reject(msg);
                }
            });
        }
    });
    context.subscriptions.push(formatProvider);
}

function findAyanamiPath(context) {
    const fs = require('fs');
    const path = require('path');

    try {
        const config = vscode.workspace.getConfiguration('ayanami');
        const setting = config.get('compilerPath', '');
        if (setting && fs.existsSync(setting)) {
            return setting;
        }
    } catch (_) {}

    const envPath = (process.env.PATH || '').split(path.delimiter);
    for (const dir of envPath) {
        const candidate = path.join(dir, 'ayanami');
        try { if (fs.existsSync(candidate)) return candidate; } catch (_) {}
    }

    try {
        const workspaces = vscode.workspace.workspaceFolders || [];
        for (const ws of workspaces) {
            let dir = ws.uri.fsPath;
            for (let i = 0; i < 5; i++) {
                for (const sub of ['build/ayanami', 'target/release/ayanami', 'target/debug/ayanami']) {
                    const p = path.join(dir, sub);
                    if (fs.existsSync(p)) return p;
                }
                const parent = path.dirname(dir);
                if (parent === dir) break;
                dir = parent;
            }
        }
    } catch (_) {}

    return null;
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
    const re = /(?:(?:for\s+(\w+)\s+in)|(?:(\w+)\s*=(?!=)))/g;
    let m;
    while ((m = re.exec(text)) !== null) {
        if (m[1]) vars.add(m[1]);
        if (m[2]) vars.add(m[2]);
    }
    const re2 = /(\w+)\s+(\w+)(?=[,)]|\s*->)/g;
    while ((m = re2.exec(text)) !== null) {
        const typeName = m[1];
        const paramName = m[2];
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
        const fullPath = path.resolve(folder, importPath);
        try {
            const content = fs.readFileSync(fullPath, 'utf8');
            for (const s of scanStructsRaw(content)) result.structs.push(s);
            for (const ns of scanNamespacesRaw(content)) result.namespaces.push(ns);
            for (const f of scanFunctionsRaw(content)) result.functions.push(f);
        } catch (e) {}
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
    const re = new RegExp('struct\\s+' + typeName + '\\s*\\{([^}]*)\\}', 'm');
    const m = re.exec(text);
    if (!m) return [];
    const body = m[1];
    const fields = [];
    const re2 = /(\w+)\s+(\w+)/g;
    let fm;
    while ((fm = re2.exec(body)) !== null) {
        fields.push(fm[2]);
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