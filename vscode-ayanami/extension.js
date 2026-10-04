const vscode = require('vscode');
const { parseCompilerOutput, computeRange } = require('./diagnostics');
const symbols = require('./symbols');
const typesMod = require('./types');
const quickfix = require('./quickfix');

const KEYWORD_DOCS = {
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
            'null': '**null** — nullable pointer value\n\n拥有指针可为 null，用 `== null` 比较。',
        };

function activate(context) {
    // ─── Output Channel ───────────────────────────────────────────────
    const outputChannel = vscode.window.createOutputChannel('Ayanami');
    context.subscriptions.push(outputChannel);
    outputChannel.appendLine('ayanami extension activating...');

    // ─── Debounced check queue ────────────────────────────────────────
    const pendingChecks = new Map(); // uri -> timer
    function scheduleCheck(doc) {
        const key = doc.uri.toString();
        if (pendingChecks.has(key)) {
            clearTimeout(pendingChecks.get(key));
        }
        pendingChecks.set(key, setTimeout(() => {
            pendingChecks.delete(key);
            runCheck(doc, diagCollection, context, setStatus, outputChannel);
        }, 400));
    }

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
            runCheck(doc, diagCollection, context, setStatus, outputChannel);
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

            // 标注补全：`#[` 后建议内置标注（A0–A4）
            const attrMatch = linePrefix.match(/#\[([\w:]*)$/);
            if (attrMatch) {
                const anns = [
                    ['inline', 'inline hint'], ['inline(always)', 'force inline'],
                    ['cold', 'cold path'], ['noreturn', 'does not return'],
                    ['pure', 'promise no effects (memory(none))'], ['readonly', 'readonly memory'],
                    ['nounwind', 'no unwind'], ['willreturn', 'always returns'],
                    ['noalias', 'pointer does not alias'], ['nonnull', 'non-null pointer'],
                    ['no_error', 'promise never fails'],
                    ['io', 'may perform IO'], ['state', 'may mutate state'], ['alloc', 'may allocate'],
                    ['cfg(target = "linux")', 'compile-time target'],
                    ['assume(cond)', 'assume condition'], ['requires(cond)', 'precondition'],
                    ['ensures(result)', 'postcondition'], ['invariant(cond)', 'loop invariant'],
                    ['throws(E)', 'may throw E'], ['throws(_)', 'unknown throws'],
                    ['throws()', 'no known errors (open slot)'],
                    ['follow_with(source)', 'reference lifetime source'],
                    ['macro', 'user macro'],
                ];
                for (const [label, detail] of anns) {
                    items.push(makeItem(label, vscode.CompletionItemKind.EnumMember, detail));
                }
                return items;
            }

            const dotMatch = linePrefix.match(/(\w+)\.$/);
            const nsMatch = linePrefix.match(/(\w+)::$/);
            const afterArrow = linePrefix.match(/->\s*$/);

            const folder = document.uri.scheme === 'file' ? require('path').dirname(document.uri.fsPath) : null;
            const filePath = document.uri.scheme === 'file' ? document.uri.fsPath : '';
            const ayanamiPathC = findAyanamiPath(context);
            const compilerDir = ayanamiPathC ? require('path').dirname(ayanamiPathC) : null;
            const workspaceFolders = (vscode.workspace.workspaceFolders || []).map((w) => w.uri.fsPath);
            const imported = filePath
                ? symbols.collectSymbols(document.getText(), filePath, { compilerDir, workspaceFolders })
                : { structs: [], namespaces: [], functions: [], enums: [], methods: [] };

            const localStructs = scanStructs(document);
            const localEnums = scanEnums(document);
            const localNss = scanNamespaces(document);
            const localFns = scanFunctions(document);

            const structs = [...localStructs, ...imported.structs];
            const namespaces = [...new Set([...localNss, ...(imported.namespaces || [])])];
            const functions = [...localFns, ...imported.functions];
            const fnByNs = groupByNamespace(functions);
            const varTypes = compilerVarTypes(document, context);
            const allEnums = [...localEnums, ...(imported.enums || [])];

            if (dotMatch) {
                const varName = dotMatch[1];
                const typeName = receiverTypeAt(varTypes, varName, position);
                const baseType = typeName ? String(typeName).split('<')[0].split('[')[0].trim() : null;
                if (baseType) {
                    const seenFields = new Set();
                    const st = structs.find((x) => x.name === baseType);
                    if (st && st.fields) {
                        for (const f of st.fields) {
                            if (seenFields.has(f.name)) continue;
                            seenFields.add(f.name);
                            items.push(makeItem(f.name, vscode.CompletionItemKind.Field, `${baseType}.${f.name}: ${f.type}`));
                        }
                    }
                    if (seenFields.size === 0) {
                        for (const f of getStructFields(document, baseType)) {
                            if (!seenFields.has(f)) { seenFields.add(f); items.push(makeItem(f, vscode.CompletionItemKind.Field, `${baseType} field`)); }
                        }
                        if (seenFields.size === 0 && folder) {
                            for (const f of getImportedStructFields(folder, baseType)) {
                                if (!seenFields.has(f)) { seenFields.add(f); items.push(makeItem(f, vscode.CompletionItemKind.Field, `${baseType} field`)); }
                            }
                        }
                    }
                    const seenMethods = new Set();
                    for (const m of (imported.methods || [])) {
                        if (m.type === baseType && !seenMethods.has(m.name)) {
                            seenMethods.add(m.name);
                            const item = makeItem(m.name, vscode.CompletionItemKind.Method, `${baseType}.${m.sig}`);
                            item.documentation = m.sig;
                            items.push(item);
                        }
                    }
                    for (const m of getImplMethods(document, baseType)) {
                        if (!seenMethods.has(m)) { seenMethods.add(m); items.push(makeItem(m, vscode.CompletionItemKind.Method, `${baseType} method`)); }
                    }
                    if (folder) {
                        for (const m of getImportedImplMethods(folder, baseType)) {
                            if (!seenMethods.has(m)) { seenMethods.add(m); items.push(makeItem(m, vscode.CompletionItemKind.Method, `${baseType} method`)); }
                        }
                    }
                }
                // Enum field completions: show _tag and _data_Variant fields
                const enumForType = allEnums.find(e => e.name === (baseType || varName));
                if (enumForType) {
                    items.push(makeItem('_tag', vscode.CompletionItemKind.Field, `${enumForType.name} enum tag`));
                    for (const v of enumForType.variants) {
                        items.push(makeItem('_data_' + v, vscode.CompletionItemKind.Field, `${enumForType.name} variant ${v}`));
                    }
                }
                return items;
            }

            if (nsMatch) {
                const nsName = nsMatch[1];
                // Check if this is an enum type — show variants
                const enumForNs = localEnums.find(e => e.name === nsName);
                if (enumForNs) {
                    for (const v of enumForNs.variants) {
                        items.push(makeItem(v, vscode.CompletionItemKind.EnumMember, `${nsName} variant`));
                    }
                    return items;
                }
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
                for (const e of localEnums) {
                    items.push(makeItem(e.name, vscode.CompletionItemKind.Enum, 'enum'));
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
                { label: 'while', kind: vscode.CompletionItemKind.Keyword, detail: 'while loop' },
                { label: 'break', kind: vscode.CompletionItemKind.Keyword, detail: 'break loop' },
                { label: 'continue', kind: vscode.CompletionItemKind.Keyword, detail: 'continue loop' },
                { label: 'mut', kind: vscode.CompletionItemKind.Keyword, detail: 'mutable' },
                { label: 'return', kind: vscode.CompletionItemKind.Keyword, detail: 'return' },
                { label: 'import', kind: vscode.CompletionItemKind.Keyword, detail: 'import module' },
                { label: 'struct', kind: vscode.CompletionItemKind.Keyword, detail: 'struct definition' },
                { label: 'enum', kind: vscode.CompletionItemKind.Keyword, detail: 'enum definition' },
                { label: 'match', kind: vscode.CompletionItemKind.Keyword, detail: 'match expression' },
                { label: 'namespace', kind: vscode.CompletionItemKind.Keyword, detail: 'namespace' },
                { label: 'interface', kind: vscode.CompletionItemKind.Keyword, detail: 'interface definition' },
                { label: 'impl', kind: vscode.CompletionItemKind.Keyword, detail: 'impl block' },
                { label: 'pub', kind: vscode.CompletionItemKind.Keyword, detail: 'public' },
                { label: 'ref', kind: vscode.CompletionItemKind.Keyword, detail: 'borrow (ref / ref mut)' },
                { label: 'extern', kind: vscode.CompletionItemKind.Keyword, detail: 'extern "C" declaration' },
                { label: 'inline', kind: vscode.CompletionItemKind.Keyword, detail: 'inline function' },
                { label: 'asm', kind: vscode.CompletionItemKind.Keyword, detail: 'inline assembly' },
                { label: 'true', kind: vscode.CompletionItemKind.Keyword, detail: 'boolean true' },
                { label: 'false', kind: vscode.CompletionItemKind.Keyword, detail: 'boolean false' },
                { label: 'null', kind: vscode.CompletionItemKind.Keyword, detail: 'null value' },
                { label: 'in', kind: vscode.CompletionItemKind.Keyword, detail: 'for iterator' },
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

            const seenStruct = new Set();
            for (const st of structs) {
                if (!st || !st.name || seenStruct.has(st.name)) continue;
                seenStruct.add(st.name);
                items.push(makeItem(st.name, vscode.CompletionItemKind.Struct, 'struct'));
            }

            for (const en of allEnums) {
                if (en && en.name) items.push(makeItem(en.name, vscode.CompletionItemKind.Enum, 'enum'));
            }

            for (const ns of namespaces) {
                items.push(makeItem(ns, vscode.CompletionItemKind.Module, 'namespace'));
            }

            for (const v of varTypes.keys()) {
                items.push(makeItem(v, vscode.CompletionItemKind.Variable, 'variable'));
            }

            const seenFn = new Set();
            for (const f of functions) {
                const name = typeof f === 'string' ? f : (f && f.name);
                if (!name || name.startsWith('__')) continue;   // 过滤内部辅助符号
                if (seenFn.has(name)) continue;
                seenFn.add(name);
                const detail = (f && typeof f === 'object' && f.sig) ? f.sig : 'function';
                items.push(makeItem(name, vscode.CompletionItemKind.Function, detail));
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
                { label: 'arr sized', insert: '[${1:int}; ${2:10}]' },
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
            try {
                const range = document.getWordRangeAtPosition(position, /[A-Za-z_][A-Za-z0-9_]*/);
                if (!range) return null;
                const word = document.getText(range);
                const path = require('path');

                if (KEYWORD_DOCS[word]) {
                    return new vscode.Hover(new vscode.MarkdownString(KEYWORD_DOCS[word]), range);
                }

                const filePath = document.uri.scheme === 'file' ? document.uri.fsPath : '';
                const folder = filePath ? path.dirname(filePath) : null;
                const ayanamiPath = findAyanamiPath(context);
                const compilerDir = ayanamiPath ? path.dirname(ayanamiPath) : null;
                const workspaceFolders = (vscode.workspace.workspaceFolders || []).map(w => w.uri.fsPath);
                const syms = symbols.collectSymbols(document.getText(), filePath, { compilerDir, workspaceFolders });

                const fn = syms.functions.find(f => f.name === word);
                if (fn) {
                    let md = `**fn** \`${fn.sig}\``;
                    if (fn.attrs && fn.attrs.length) md += `\n\n${fn.attrs.map(a => '`#' + a + '`').join(' ')}`;
                    if (fn.doc) md += `\n\n---\n\n${fn.doc}`;
                    if (fn.file) md += `\n\n*${symbols.shortPath(fn.file, folder)}:${fn.line}*`;
                    return new vscode.Hover(new vscode.MarkdownString(md), range);
                }

                const st = syms.structs.find(s => s.name === word);
                if (st) {
                    let md = `**struct** \`${st.name}\``;
                    if (st.fields && st.fields.length) {
                        md += '\n\n---\n\n';
                        for (const f of st.fields) md += `- \`${f.type} ${f.name}\`\n`;
                    }
                    if (st.file) md += `\n*${symbols.shortPath(st.file, folder)}:${st.line}*`;
                    return new vscode.Hover(new vscode.MarkdownString(md), range);
                }

                const en = syms.enums.find(e => e.name === word);
                if (en) {
                    let md = `**enum** \`${en.name}\``;
                    if (en.variants && en.variants.length) md += `\n\n变体：${en.variants.map(v => '`' + v + '`').join(', ')}`;
                    return new vscode.Hover(new vscode.MarkdownString(md), range);
                }

                for (const s of syms.structs) {
                    const f = (s.fields || []).find(x => x.name === word);
                    if (f) {
                        return new vscode.Hover(new vscode.MarkdownString(`**field** \`${s.name}.${f.name}: ${f.type}\``), range);
                    }
                }

                const vt = compilerVarTypes(document, context).get(word);
                const vtName = typeof vt === 'string' ? vt : (Array.isArray(vt) && vt.length ? vt[vt.length - 1].type : null);
                if (vtName) {
                    return new vscode.Hover(new vscode.MarkdownString(`**变量** \`${word}: ${vtName}\``), range);
                }
                // 方法 hover：lcl 方法表
                const method = (syms.methods || []).find((mm) => mm.name === word);
                if (method) {
                    return new vscode.Hover(new vscode.MarkdownString(`**method** \`${method.type}.${method.sig}\``), range);
                }

                return null;
            } catch (e) {
                return null;
            }
        }
    });
    context.subscriptions.push(hoverProvider);

    // ─── Inlay Hints Provider (type hints like rust-analyzer) ─────────
    const inlayHintsProvider = vscode.languages.registerInlayHintsProvider('ayanami', {
        provideInlayHints(document, range) {
            const hints = [];
            const text = document.getText();
            const excludeVars = new Set(['fn', 'for', 'if', 'elif', 'else', 'while', 'return', 'import', 'struct', 'namespace', 'impl', 'interface', 'pub', 'move', 'clone', 'ref', 'mut', 'true', 'false', 'null']);

            // 1. Variable assignments: name = expr → show : type after name
            //    优先使用编译器 `types` 输出（真实推断）；不可用时回退启发式
            const compilerTypes = getCompilerTypeEntries(document, context);
            if (compilerTypes) {
                for (const e of compilerTypes.entries) {
                    const line = e.line - 1;
                    const col = e.col - 1;
                    if (line < 0 || line >= document.lineCount || col < 0) continue;
                    const pos = new vscode.Position(line, col + e.name.length);
                    if (pos.isBefore(range.start) || pos.isAfter(range.end)) continue;
                    const hint = new vscode.InlayHint(pos, `: ${e.type}`, vscode.InlayHintKind.Type);
                    hint.paddingRight = true;
                    hints.push(hint);
                }
            } else {
                const varTypes = scanVariableTypes(document);
                const assignRe = /(?:^|\n)(\s*)(\w+)\s*=/gm;
                while ((m = assignRe.exec(text)) !== null) {
                    const varName = m[2];
                    if (excludeVars.has(varName)) continue;
                    const typeName = varTypes.get(varName);
                    if (!typeName) continue;
                    const nameStart = m[0].indexOf(varName, m[1].length);
                    const nameEnd = m.index + nameStart + varName.length;
                    const pos = document.positionAt(nameEnd);
                    if (pos.isBefore(range.start) || pos.isAfter(range.end)) continue;
                    const hint = new vscode.InlayHint(pos, `: ${typeName}`, vscode.InlayHintKind.Type);
                    hint.paddingRight = true;
                    hints.push(hint);
                }
            }

            // 2. self in method params: self → : TypeName (from enclosing impl block)
            const implRe = /impl\s+(\w+)\s*\{/g;
            while ((m = implRe.exec(text)) !== null) {
                const implType = m[1];
                let depth = 1, pos = m.index + m[0].length;
                const blockStart = pos;
                while (depth > 0 && pos < text.length) {
                    if (text[pos] === '{') depth++;
                    else if (text[pos] === '}') depth--;
                    pos++;
                }
                const blockBody = text.slice(blockStart, pos - 1);
                const fnRe = /fn\s+\w+\s*\(([^)]*)\)/g;
                let fm;
                while ((fm = fnRe.exec(blockBody)) !== null) {
                    const paramsStr = fm[1];
                    const selfIndex = paramsStr.indexOf('self');
                    if (selfIndex === -1) continue;
                    // Position of self end in blockBody-relative coordinates
                    const parenPos = fm[0].indexOf('(');
                    const absFnStart = blockStart + fm.index;
                    const nameEnd = absFnStart + parenPos + 1 + selfIndex + 4;
                    const p = document.positionAt(nameEnd);
                    if (p.isBefore(range.start) || p.isAfter(range.end)) continue;
                    const hint = new vscode.InlayHint(p, `: ${implType}`, vscode.InlayHintKind.Type);
                    hint.paddingRight = true;
                    hints.push(hint);
                }
            }

            // 3. For-loop variable: for i in ( → i: int
            const forRe = /\bfor\s+(\w+)\s+in\s*\(/g;
            while ((m = forRe.exec(text)) !== null) {
                const varName = m[1];
                const nameStart = m[0].indexOf(varName);
                const nameEnd = m.index + nameStart + varName.length;
                const p = document.positionAt(nameEnd);
                if (p.isBefore(range.start) || p.isAfter(range.end)) continue;
                const hint = new vscode.InlayHint(p, ': int', vscode.InlayHintKind.Type);
                hint.paddingRight = true;
                hints.push(hint);
            }

            return hints;
        }
    });
    context.subscriptions.push(inlayHintsProvider);

    // ─── Quick Fix: 自动补 import ─────────────────────────────────────
    const codeActionProvider = vscode.languages.registerCodeActionsProvider('ayanami', {
        async provideCodeActions(document, range, ctx) {
            const actions = [];
            try {
                const ayanamiPath = findAyanamiPath(context);
                if (!ayanamiPath) return actions;
                const packages = [
                    ...scanStdPackages(ayanamiPath),
                    ...(await scanProjectPackages(document)),
                ];
                const seen = new Set();
                for (const diag of ctx.diagnostics || []) {
                    const symbol = quickfix.parseUnknownSymbol(diag.message);
                    if (!symbol) continue;
                    for (const stem of quickfix.findImportCandidates(symbol, packages)) {
                        if (seen.has(stem)) continue;
                        seen.add(stem);
                        const info = quickfix.importEditInfo(document.getText(), stem);
                        if (!info) continue;
                        const action = new vscode.CodeAction(
                            `引入 import "${stem}"`, vscode.CodeActionKind.QuickFix);
                        action.diagnostics = [diag];
                        action.isPreferred = true;
                        const edit = new vscode.WorkspaceEdit();
                        edit.insert(document.uri, new vscode.Position(info.line, 0), info.text);
                        action.edit = edit;
                        actions.push(action);
                    }
                }
            } catch (_) { /* quick fix 失败不影响编辑 */ }
            return actions;
        }
    }, { providedCodeActionKinds: [vscode.CodeActionKind.QuickFix] });
    context.subscriptions.push(codeActionProvider);

    // ─── Diagnostic Provider (compiler check on save) ────────────────
    const diagCollection = vscode.languages.createDiagnosticCollection('ayanami');
    context.subscriptions.push(diagCollection);

    function runCheck(doc, collection, ctx, statusFn, log) {
        try {
            if (doc.languageId !== 'ayanami') return;
            const fs = require('fs');
            const path = require('path');
            const filePath = doc.uri.fsPath;

            // Skip files in node_modules, .git, etc.
            if (filePath.includes('node_modules') || filePath.includes('.git')) return;

            statusFn('checking');
            collection.clear();
            const diagnostics = [];

            let ayanamiPath = findAyanamiPath(ctx);
            if (!ayanamiPath) {
                if (log) log.appendLine('compiler not found');
                statusFn('not found');
                diagnostics.push(new vscode.Diagnostic(new vscode.Range(0, 0, 0, 10),
                    'ayanami: compiler not found (set ayanami.compilerPath in settings)', vscode.DiagnosticSeverity.Warning));
                collection.set(doc.uri, diagnostics);
                return;
            }

            const { spawnSync } = require('child_process');

            // Use project root as CWD (walk up from file to find ayanami.toml)
            let projectRoot = path.dirname(filePath);
            for (let i = 0; i < 10; i++) {
                if (fs.existsSync(path.join(projectRoot, 'ayanami.toml'))) break;
                const parent = path.dirname(projectRoot);
                if (parent === projectRoot) { projectRoot = path.dirname(filePath); break; }
                projectRoot = parent;
            }
            if (log) log.appendLine(`check: ${path.relative(projectRoot, filePath)} (cwd=${projectRoot})`);

            let out = '';
            try {
                // spawnSync：成功时也捕获 stderr（效应/注解告警走 stderr）
                const r = spawnSync(ayanamiPath, ['check', filePath], {
                    timeout: 15000,
                    encoding: 'utf8',
                    cwd: projectRoot,
                });
                out = (r.stdout || '') + (r.stderr || '');
                if (r.status === 0) {
                    statusFn('ok');
                    if (log) log.appendLine('check passed');
                } else if (log) {
                    log.appendLine(`check failed (status ${r.status})`);
                }
            } catch (e) {
                if (log) log.appendLine(`check error: ${String(e).slice(0, 200)}`);
            }

            if (out) {
                const parsed = parseCompilerOutput(out, filePath, projectRoot);
                const byFile = new Map();
                for (const d of parsed) {
                    const f = d.file || filePath;
                    let line = Math.max(0, (d.line || 1) - 1);
                    let col = Math.max(0, (d.col || 1) - 1);
                    if (f === filePath) {
                        line = Math.min(line, Math.max(0, doc.lineCount - 1));
                        const lineText = doc.lineAt(line).text;
                        col = Math.min(col, lineText.length);
                    }
                    let rng = { start: col, end: col + 1 };
                    if (f === filePath) {
                        rng = computeRange(doc.lineAt(line).text, col + 1, d.message);
                    }
                    const range = new vscode.Range(line, rng.start, line, rng.end);
                    const sev = d.severity === 1 ? vscode.DiagnosticSeverity.Warning : vscode.DiagnosticSeverity.Error;
                    if (!byFile.has(f)) byFile.set(f, []);
                    byFile.get(f).push(new vscode.Diagnostic(range, d.message, sev));
                }
                for (const [f, list] of byFile) {
                    if (f === filePath) {
                        diagnostics.push(...list);
                    } else {
                        collection.set(vscode.Uri.file(f), list);
                    }
                }
            }

            if (diagnostics.length > 0) {
                const errs = diagnostics.filter(d => d.severity === vscode.DiagnosticSeverity.Error).length;
                const warns = diagnostics.length - errs;
                statusFn(errs > 0 ? `${errs} error(s)` : `${warns} warning(s)`);
                if (log) log.appendLine(`${errs} error(s), ${warns} warning(s) found`);
            }

            collection.set(doc.uri, diagnostics);
        } catch (e) {
            if (log) log.appendLine(`diagnostic error: ${e.message}`);
            statusFn('error');
        }
    }

    // Check on open (debounced, 400ms delay to let editor settle)
    context.subscriptions.push(vscode.workspace.onDidOpenTextDocument(doc => {
        if (doc.languageId === 'ayanami') {
            scheduleCheck(doc);
        }
    }));

    // Check on save (immediate, no debounce)
    context.subscriptions.push(vscode.workspace.onDidSaveTextDocument(doc => {
        if (doc.languageId === 'ayanami') {
            runCheck(doc, diagCollection, context, setStatus, outputChannel);
        }
    }));

    // Check on edit (debounced, 600ms after last change)
    let editTimer = null;
    context.subscriptions.push(vscode.workspace.onDidChangeTextDocument(e => {
        if (e.document.languageId === 'ayanami') {
            if (editTimer) clearTimeout(editTimer);
            editTimer = setTimeout(() => {
                editTimer = null;
                runCheck(e.document, diagCollection, context, setStatus, outputChannel);
            }, 600);
        }
    }));

    // Check all open .aya files once after activation
    setTimeout(() => {
        for (const doc of vscode.workspace.textDocuments) {
            if (doc.languageId === 'ayanami') {
                scheduleCheck(doc);
            }
        }
        setStatus('ready');
    }, 1500);

    outputChannel.appendLine('ayanami extension activated');

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

// ─── Quick Fix: 包符号扫描（std .lcl + 工作区 .aya） ─────────────────
const stdPackageCache = new Map(); // ayanamiPath -> packages

function scanStdPackages(ayanamiPath) {
    const fs = require('fs');
    const path = require('path');
    if (stdPackageCache.has(ayanamiPath)) return stdPackageCache.get(ayanamiPath);
    const packages = [];
    const dir = path.join(path.dirname(ayanamiPath), 'std');
    try {
        for (const f of fs.readdirSync(dir)) {
            if (!f.endsWith('.lcl')) continue;
            const syms = { functions: [], structs: [], enums: [] };
            symbols.parseLclSymbols(fs.readFileSync(path.join(dir, f), 'utf8'), f, syms);
            packages.push({ stem: f.replace(/\.lcl$/, ''), ...syms });
        }
    } catch (_) { /* 无 std 目录 */ }
    stdPackageCache.set(ayanamiPath, packages);
    return packages;
}

const projectPackageCache = new Map(); // workspace key -> Promise<packages>

async function scanProjectPackages(currentDoc) {
    const fs = require('fs');
    const path = require('path');
    const folders = (vscode.workspace.workspaceFolders || []).map((f) => f.uri.fsPath);
    const key = folders.join('|');
    if (!projectPackageCache.has(key)) {
        projectPackageCache.set(key, (async () => {
            const packages = [];
            try {
                const uris = await vscode.workspace.findFiles(
                    '**/*.aya', '**/{node_modules,target,build,.git}/**', 300);
                for (const uri of uris) {
                    try {
                        const text = fs.readFileSync(uri.fsPath, 'utf8');
                        packages.push({
                            stem: '',
                            file: uri.fsPath,
                            functions: symbols.scanFnSigsFull(text, uri.fsPath),
                            structs: symbols.scanStructsFull(text, uri.fsPath),
                            enums: symbols.scanEnumsFull(text, uri.fsPath),
                        });
                    } catch (_) { /* skip */ }
                }
            } catch (_) { /* skip */ }
            return packages;
        })());
    }
    const packages = await projectPackageCache.get(key);
    const dir = path.dirname(currentDoc.uri.fsPath);
    return packages
        .filter((p) => p.file && p.file !== currentDoc.uri.fsPath)
        .map((p) => {
            let rel = path.relative(dir, p.file).replace(/\\/g, '/').replace(/\.aya$/, '');
            if (rel.startsWith('./')) rel = rel.slice(2);
            return { ...p, stem: rel };
        });
}

// ─── 补全辅助：编译器变量类型（按声明行选择最近的） ──────────────────
function compilerVarTypes(document, ctx) {
    const entries = getCompilerTypeEntries(document, ctx);
    if (!entries || !entries.entries.length) return scanVariableTypes(document);
    const map = new Map(); // name -> [{type, line}]
    for (const e of entries.entries) {
        if (!map.has(e.name)) map.set(e.name, []);
        map.get(e.name).push({ type: e.type, line: e.line });
    }
    return map;
}

function receiverTypeAt(varTypes, name, position) {
    if (!varTypes || !varTypes.get) return null;
    const val = varTypes.get(name);
    if (!val) return null;
    if (typeof val === 'string') return val;
    let best = null;
    for (const e of val) {
        if (e.line - 1 <= position.line) best = e.type;
    }
    return best || (val.length ? val[0].type : null);
}

// ─── Compiler variable types (cached per document version) ───────────
const typeCache = new Map(); // uri -> { version, entries }

function getCompilerTypeEntries(doc, ctx) {
    try {
        const key = doc.uri.toString();
        const cached = typeCache.get(key);
        if (cached && cached.version === doc.version) return cached;
        const ayanamiPath = findAyanamiPath(ctx);
        if (!ayanamiPath) return null;
        const fs = require('fs');
        const path = require('path');
        const filePath = doc.uri.fsPath;
        let projectRoot = path.dirname(filePath);
        for (let i = 0; i < 10; i++) {
            if (fs.existsSync(path.join(projectRoot, 'ayanami.toml'))) break;
            const parent = path.dirname(projectRoot);
            if (parent === projectRoot) { projectRoot = path.dirname(filePath); break; }
            projectRoot = parent;
        }
        const { execFileSync } = require('child_process');
        let out = '';
        try {
            out = execFileSync(ayanamiPath, ['types', filePath], {
                timeout: 5000,
                encoding: 'utf8',
                cwd: projectRoot,
                stdio: ['pipe', 'pipe', 'pipe'],
            });
        } catch (e) {
            out = e.stdout || '';
        }
        const parsed = typesMod.parseTypesOutput(out);
        if (!out || out.indexOf('{"types"') === -1) return null; // 旧编译器/命令不可用 → 回退
        const entries = typesMod.validateTypeEntries(doc.getText(), parsed);
        const result = { version: doc.version, entries };
        typeCache.set(key, result);
        return result;
    } catch (_) {
        return null;
    }
}

function findAyanamiPath(context) {
    const fs = require('fs');
    const path = require('path');
    const os = require('os');

    // 1. Setting
    try {
        const config = vscode.workspace.getConfiguration('ayanami');
        const setting = config.get('compilerPath', '');
        if (setting && fs.existsSync(setting)) {
            return setting;
        }
    } catch (_) {}

    // 2. Common home-directory locations
    const home = os.homedir();
    const homeCandidates = [
        path.join(home, 'ayanami', 'ayanami'),
        path.join(home, '.ayanami', 'ayanami'),
        path.join(home, 'bin', 'ayanami'),
        path.join(home, '.local', 'bin', 'ayanami'),
    ];
    for (const c of homeCandidates) {
        try { if (fs.existsSync(c)) return c; } catch (_) {}
    }

    // 3. PATH
    const envPath = (process.env.PATH || '').split(path.delimiter);
    for (const dir of envPath) {
        const candidate = path.join(dir, 'ayanami');
        try { if (fs.existsSync(candidate)) return candidate; } catch (_) {}
    }

    // 4. Workspace-relative build directories
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

// ─── Helper: scan enum definitions ─────────────────────────────────
function scanEnums(doc) {
    const enums = [];
    const text = doc.getText();
    const re = /enum\s+(\w+)\s*\{/g;
    let m;
    while ((m = re.exec(text)) !== null) {
        const ename = m[1];
        const variants = [];
        // Scan for variants within this enum block
        const blockStart = m.index;
        const afterBrace = text.indexOf('{', blockStart) + 1;
        let depth = 1;
        let pos = afterBrace;
        const varRe = /\b([A-Z]\w*)\s*(?:\(|\{)?/g;
        varRe.lastIndex = pos;
        let vm;
        while ((vm = varRe.exec(text)) !== null) {
            if (vm.index >= text.length) break;
            const ch = text[vm.index + vm[0].length];
            if (ch === ',' || ch === '}' || ch === '(' || ch === '{') {
                variants.push(vm[1]);
            }
            if (ch === '}') break;
        }
        enums.push({ name: ename, variants, line: m.index });
    }
    return enums;
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
    const re = /(?:pub\s+)?struct\s+(\w+)\s*\{/g;
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
    const re = new RegExp('(?:pub\\s+)?struct\\s+' + typeName + '\\s*\\{([^}]*)\\}', 'm');
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

// Get the type of a specific struct field
function getFieldType(doc, typeName, fieldName) {
    const text = doc.getText();
    const re = new RegExp('(?:pub\\s+)?struct\\s+' + typeName + '\\s*\\{([^}]*)\\}', 'm');
    const m = re.exec(text);
    if (!m) return null;
    const body = m[1];
    const lines = body.split('\n');
    for (const line of lines) {
        const trimmed = line.trim();
        // Pattern: [ownership] Type fieldname
        const fm = trimmed.match(/^(?:(?:ref\s+)?(\w+))\s+(\w+)$/);
        if (fm && fm[2] === fieldName) {
            return fm[1];
        }
    }
    return null;
}

// ─── Helper: scan variables with their inferred types ─────────────────
function scanVariableTypes(doc) {
    const varTypes = new Map();
    const text = doc.getText();

    // Assignment: v = expr (try to infer type from RHS)
    const assignRe = /(\w+)\s*=\s*(.*?)(?:;|$)/g;
    let m;
    while ((m = assignRe.exec(text)) !== null) {
        const varName = m[1];
        const rhs = m[2].trim();

        // 剥离借用前缀（ref），用于后续模式匹配
        let coreRhs = rhs.replace(/^ref\s+/, '');
        const ownership = rhs !== coreRhs ? rhs.split(/\s+/)[0] : null;

        // Struct constructor: String { ... }
        const structMatch = coreRhs.match(/^(\w+)\s*\{/);
        if (structMatch) {
            varTypes.set(varName, structMatch[1]);
            continue;
        }
        // Enum constructor: EnumType::Variant(args)
        const enumMatch = coreRhs.match(/^(\w+)::\w+\s*\(/);
        if (enumMatch) {
            varTypes.set(varName, enumMatch[1]);
            continue;
        }
        // String literal
        if (coreRhs.startsWith('"')) {
            varTypes.set(varName, 'String');
            continue;
        }
        // Int literal
        if (/^-?\d+$/.test(coreRhs)) {
            varTypes.set(varName, 'int');
            continue;
        }
        // Float literal
        if (/^-?\d+\.\d+$/.test(coreRhs)) {
            varTypes.set(varName, 'float');
            continue;
        }
        // Bool literal
        if (coreRhs === 'true' || coreRhs === 'false') {
            varTypes.set(varName, 'bool');
            continue;
        }
        // Unary not
        if (/^!\w+$/.test(coreRhs)) {
            varTypes.set(varName, 'bool');
            continue;
        }
        // List literal: [char; 10] or [int; n]
        const arrMatch = coreRhs.match(/^\[(\w+)\s*;/);
        if (arrMatch) {
            varTypes.set(varName, '[' + arrMatch[1] + ']');
            continue;
        }
        // Field access: weak obj.field or obj.field
        const fieldMatch = coreRhs.match(/^(\w+)\.(\w+)$/);
        if (fieldMatch) {
            const objName = fieldMatch[1];
            const fieldName = fieldMatch[2];
            // 优先从 struct 定义解析字段类型
            const objType = varTypes.get(objName) || varTypes.get('self');
            if (objType) {
                const fieldType = getFieldType(doc, objType, fieldName);
                if (fieldType) {
                    varTypes.set(varName, fieldType);
                    continue;
                }
            }
            // 后备：从对象类型推断方法链结果
            if (varTypes.has(objName)) {
                varTypes.set(varName, varTypes.get(objName));
                continue;
            }
        }
        // Variable copy: v = otherVar
        if (varTypes.has(coreRhs)) {
            varTypes.set(varName, varTypes.get(coreRhs));
            continue;
        }
        // to_string() call returns String
        if (coreRhs.endsWith('.to_string()')) {
            varTypes.set(varName, 'String');
            continue;
        }
        // .copy() returns same type as the receiver
        const copyMatch = coreRhs.match(/^(\w+)\.copy\(\)$/);
        if (copyMatch && varTypes.has(copyMatch[1])) {
            varTypes.set(varName, varTypes.get(copyMatch[1]));
            continue;
        }
        // .len() returns int
        if (coreRhs.match(/^\w+\.len\(\)$/)) {
            varTypes.set(varName, 'int');
            continue;
        }
        // .index() returns char
        if (coreRhs.match(/^\w+\.index\(/)) {
            varTypes.set(varName, 'char');
            continue;
        }
        // .neg() returns same type as receiver
        const negMatch = coreRhs.match(/^(\w+)\.neg\(\)$/);
        if (negMatch && varTypes.has(negMatch[1])) {
            varTypes.set(varName, varTypes.get(negMatch[1]));
            continue;
        }
        // .not() returns bool
        if (coreRhs.match(/^\w+\.not\(\)$/)) {
            varTypes.set(varName, 'bool');
            continue;
        }
        // .eq/.ne/.lt/.gt/.le/.ge returns bool
        if (coreRhs.match(/^\w+\.(eq|ne|lt|gt|le|ge)\(/)) {
            varTypes.set(varName, 'bool');
            continue;
        }
        // .add/.sub/.mul/.div/.rem returns same type as receiver (for primitives)
        const arithMatch = coreRhs.match(/^(\w+)\.(add|sub|mul|div|rem)\(/);
        if (arithMatch && varTypes.has(arithMatch[1])) {
            varTypes.set(varName, varTypes.get(arithMatch[1]));
            continue;
        }
        // Function call returns... for common known functions
        const callMatch = coreRhs.match(/^(\w+)\(/);
        if (callMatch) {
            const fnName = callMatch[1];
            if (fnName === 'to_string') {
                varTypes.set(varName, 'String');
                continue;
            }
            // Try to infer from function return type
            const retType = getReturnType(text, fnName);
            if (retType) {
                varTypes.set(varName, retType);
                continue;
            }
        }
    }

    // Collect function return types for the whole file first
    const fnRetTypes = collectReturnTypes(text);
    // Also collect from imported files
    const folder = doc.uri.scheme === 'file' ? require('path').dirname(doc.uri.fsPath) : null;
    if (folder) {
        const importRe = /import\s+"([^"]+\.aya)"/g;
        let im;
        while ((im = importRe.exec(text)) !== null) {
            const importPath = require('path').resolve(folder, im[1]);
            try {
                const importContent = require('fs').readFileSync(importPath, 'utf8');
                const importRetTypes = collectReturnTypes(importContent);
                for (const [k, v] of importRetTypes) {
                    fnRetTypes.set(k, v);
                }
            } catch (e) {}
        }
    }

    // Second pass: resolve call-based types from collected return types
    const assignRe2 = /(\w+)\s*=\s*(.*?)(?:;|$)/g;
    while ((m = assignRe2.exec(text)) !== null) {
        const varName = m[1];
        const rhs = m[2].trim();
        const callMatch2 = rhs.match(/^(\w+)\(/);
        if (callMatch2 && fnRetTypes.has(callMatch2[1]) && !varTypes.has(varName)) {
            varTypes.set(varName, fnRetTypes.get(callMatch2[1]));
        }
    }

    // Function params: fn foo(TypeName param)
    const paramRe = /fn\s+\w+\(([^)]*)\)/g;
    while ((m = paramRe.exec(text)) !== null) {
        const paramsStr = m[1];
        const params = paramsStr.split(',');
        for (const p of params) {
            const parts = p.trim().split(/\s+/);
            if (parts.length >= 2) {
                // Handle: ref T param / T param
                let typeName = parts[parts.length - 2];
                let paramName = parts[parts.length - 1];
                if (['ref'].includes(typeName) && parts.length >= 3) {
                    typeName = parts[parts.length - 3];
                    paramName = parts[parts.length - 1];
                }
                if (paramName && typeName && paramName !== '->' && !typeName.startsWith('//')) {
                    varTypes.set(paramName, typeName);
                }
            }
        }
    }

    // Self param in impl methods: type comes from impl block
    const implRe = /impl\s+(\w+)\s*\{/g;
    while ((m = implRe.exec(text)) !== null) {
        const implType = m[1];
        // self appears inside this impl block
        const blockStart = m.index + m[0].length;
        let depth = 1;
        let pos = blockStart;
        while (depth > 0 && pos < text.length) {
            if (text[pos] === '{') depth++;
            else if (text[pos] === '}') depth--;
            pos++;
        }
        const blockBody = text.slice(blockStart, pos - 1);
        const selfRe = /\bself\b/g;
        let sm;
        while ((sm = selfRe.exec(blockBody)) !== null) {
            varTypes.set('self', implType);
        }
    }

    // For loop: for v in (0, n) — v is int
    const forRe = /for\s+(\w+)\s+in\s*\(/g;
    while ((m = forRe.exec(text)) !== null) {
        varTypes.set(m[1], 'int');
    }

    return varTypes;
}

// ─── Helper: get method names from impl blocks for a type ────────────
function getImplMethods(doc, typeName) {
    const methods = [];
    const text = doc.getText();
    const re = new RegExp('impl\\s+' + typeName + '\\s*\\{', 'm');
    const m = re.exec(text);
    if (!m) return methods;
    const blockStart = m.index + m[0].length;
    let depth = 1;
    let pos = blockStart;
    while (depth > 0 && pos < text.length) {
        if (text[pos] === '{') depth++;
        else if (text[pos] === '}') depth--;
        pos++;
    }
    const blockBody = text.slice(blockStart, pos - 1);
    const fnRe = /(?:pub\s+)?fn\s+(\w+)\s*\(/g;
    let fm;
    while ((fm = fnRe.exec(blockBody)) !== null) {
        methods.push(fm[1]);
    }
    return methods;
}

// ─── Helper: collect function return types from source text ─────────
function collectReturnTypes(text) {
    const map = new Map();
    const re = /(?:pub\s+)?fn\s+(\w+)\s*\([^)]*\)\s*(?:->\s*(\w+))?/g;
    let m;
    while ((m = re.exec(text)) !== null) {
        if (m[2]) {
            map.set(m[1], m[2]);
        }
    }
    // Also check extern "C" declarations
    const externRe = /extern\s+"C"\s+fn\s+(\w+)\s*\([^)]*\)\s*(?:->\s*(\w+))?/g;
    while ((m = externRe.exec(text)) !== null) {
        if (m[2]) {
            map.set(m[1], m[2]);
        }
    }
    return map;
}

function getReturnType(text, fnName) {
    const types = collectReturnTypes(text);
    return types.get(fnName) || null;
}

// ─── Helper: get struct fields from imported files ──────────────────
function getImportedStructFields(folder, typeName) {
    const fs = require('fs');
    const path = require('path');
    const fields = [];
    try {
        const files = fs.readdirSync(folder);
        for (const f of files) {
            if (!f.endsWith('.aya')) continue;
            const content = fs.readFileSync(path.join(folder, f), 'utf8');
            const re = new RegExp('struct\\s+' + typeName + '\\s*\\{([^}]*)\\}', 'm');
            const m = re.exec(content);
            if (!m) continue;
            const body = m[1];
            const re2 = /(\w+)\s+(\w+)/g;
            let fm;
            while ((fm = re2.exec(body)) !== null) fields.push(fm[2]);
        }
    } catch (e) {}
    return fields;
}

// ─── Helper: get impl methods from imported files ───────────────────
function getImportedImplMethods(folder, typeName) {
    const fs = require('fs');
    const path = require('path');
    const methods = [];
    try {
        const files = fs.readdirSync(folder);
        for (const f of files) {
            if (!f.endsWith('.aya')) continue;
            const content = fs.readFileSync(path.join(folder, f), 'utf8');
            const re = new RegExp('impl\\s+' + typeName + '\\s*\\{', 'm');
            const m = re.exec(content);
            if (!m) continue;
            const blockStart = m.index + m[0].length;
            let depth = 1, pos = blockStart;
            while (depth > 0 && pos < content.length) {
                if (content[pos] === '{') depth++;
                else if (content[pos] === '}') depth--;
                pos++;
            }
            const blockBody = content.slice(blockStart, pos - 1);
            const fnRe = /(?:pub\s+)?fn\s+(\w+)\s*\(/g;
            let fm;
            while ((fm = fnRe.exec(blockBody)) !== null) methods.push(fm[1]);
        }
    } catch (e) {}
    return methods;
}

// ─── Helper: create completion item ─────────────────────────────────
function makeItem(label, kind, detail) {
    const item = new vscode.CompletionItem(label, kind);
    item.detail = detail;
    return item;
}

function deactivate() {}

module.exports = { activate, deactivate };