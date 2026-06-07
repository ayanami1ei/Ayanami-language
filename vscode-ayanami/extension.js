const vscode = require('vscode');

function activate(context) {
    const provider = vscode.languages.registerCompletionItemProvider('ayanami', {
        provideCompletionItems(document, position) {
            const items = [];
            const linePrefix = document.lineAt(position).text.slice(0, position.character);
            const isAfterDot = linePrefix.endsWith('.');
            const isAfterImport = /import\s*"$/.test(linePrefix);

            // === Keywords ===
            const keywords = [
                { label: 'fn', detail: 'function declaration' },
                { label: 'return', detail: 'return from function' },
                { label: 'if', detail: 'if expression' },
                { label: 'elif', detail: 'else if' },
                { label: 'else', detail: 'else branch' },
                { label: 'for', detail: 'for loop' },
                { label: 'in', detail: 'for loop iterator' },
                { label: 'while', detail: 'while loop' },
                { label: 'struct', detail: 'struct definition' },
                { label: 'interface', detail: 'interface definition' },
                { label: 'impl', detail: 'implementation block' },
                { label: 'import', detail: 'import package or file' },
                { label: 'namespace', detail: 'namespace declaration' },
                { label: 'pub', detail: 'make public' },
                { label: 'pub(crate)', detail: 'make crate-public' },
                { label: 'shared', detail: 'shared ownership (refcount)' },
                { label: 'unique', detail: 'unique ownership' },
                { label: 'weak', detail: 'weak reference' },
                { label: 'move', detail: 'move ownership' },
                { label: 'clone', detail: 'clone value' },
                { label: 'self', detail: 'self parameter' },
                { label: 'true', detail: 'boolean true' },
                { label: 'false', detail: 'boolean false' },
                { label: 'int', detail: '64-bit integer type' },
                { label: 'float', detail: '64-bit float type' },
                { label: 'char', detail: 'character type' },
                { label: 'bool', detail: 'boolean type' },
                { label: 'void', detail: 'void type' },
                { label: 'mut', detail: 'mutable variable' },
            ];
            for (const kw of keywords) {
                const item = new vscode.CompletionItem(kw.label, vscode.CompletionItemKind.Keyword);
                item.detail = kw.detail;
                items.push(item);
            }

            // === Snippets ===
            const snippets = [
                {
                    label: 'fn main',
                    detail: 'main function',
                    insertText: new vscode.SnippetString('fn main() -> int {\n    ${1:return 0;}\n}'),
                },
                {
                    label: 'fn',
                    detail: 'function with return type',
                    insertText: new vscode.SnippetString('fn ${1:name}(${2:int param}) -> ${3:int} {\n    ${4}\n}'),
                },
                {
                    label: 'struct',
                    detail: 'struct definition',
                    insertText: new vscode.SnippetString('struct ${1:Name} {\n    ${2:int field}\n}'),
                },
                {
                    label: 'interface',
                    detail: 'interface definition',
                    insertText: new vscode.SnippetString('interface ${1:Name} {\n    fn ${2:method}(shared self) -> ${3:int};\n}'),
                },
                {
                    label: 'impl',
                    detail: 'implementation block',
                    insertText: new vscode.SnippetString('impl ${1:Type} {\n    fn ${2:method}(${3:shared self}) -> ${4:int} {\n        ${5}\n    }\n}'),
                },
                {
                    label: 'if',
                    detail: 'if expression',
                    insertText: new vscode.SnippetString('if ${1:condition} {\n    ${2}\n}'),
                },
                {
                    label: 'ifelse',
                    detail: 'if-else expression',
                    insertText: new vscode.SnippetString('if ${1:condition} {\n    ${2}\n} else {\n    ${3}\n}'),
                },
                {
                    label: 'elif',
                    detail: 'else-if chain',
                    insertText: new vscode.SnippetString('if ${1:cond1} {\n    ${2}\n} elif ${3:cond2} {\n    ${4}\n} else {\n    ${5}\n}'),
                },
                {
                    label: 'while',
                    detail: 'while loop',
                    insertText: new vscode.SnippetString('while ${1:condition} {\n    ${2}\n}'),
                },
                {
                    label: 'for',
                    detail: 'for loop',
                    insertText: new vscode.SnippetString('for ${1:i} in (${2:start}, ${3:end}) {\n    ${4}\n}'),
                },
                {
                    label: 'namespace',
                    detail: 'namespace',
                    insertText: new vscode.SnippetString('namespace ${1:name} {\n    ${2}\n}'),
                },
                {
                    label: 'import',
                    detail: 'import package or file',
                    insertText: new vscode.SnippetString('import "${1:path}";'),
                },
            ];
            for (const s of snippets) {
                const item = new vscode.CompletionItem(s.label, vscode.CompletionItemKind.Snippet);
                item.detail = s.detail;
                item.insertText = s.insertText;
                items.push(item);
            }

            return items;
        },
    }, ...'abcdefghijklmnopqrstuvwxyz_'); // trigger characters

    context.subscriptions.push(provider);
}

function deactivate() {}

module.exports = { activate, deactivate };
