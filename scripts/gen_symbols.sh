#!/usr/bin/env bash
set -euo pipefail

cd "$(dirname "$0")/.."

if ! command -v rg &> /dev/null; then
    echo "Error: ripgrep (rg) is required but not found." >&2
    exit 1
fi

check_mode=false
if [[ "${1:-}" == "--check" ]]; then
    check_mode=true
fi

tmpfile=$(mktemp)
trap 'rm -f "$tmpfile"' EXIT

{
    echo "# SYMBOLS.md — Ayanami 符号地图"
    echo ""
    echo "> 本文件由 scripts/gen_symbols.sh 自动生成，请勿手改。重新生成：./scripts/gen_symbols.sh"
    echo "> 查询：rg \"关键词\" SYMBOLS.md"
    echo "> 标准库（std/）符号由 [Ayanami-std](https://github.com/ayanami1ei/Ayanami-std) 子仓自行维护，见其 SYMBOLS.md。"
    echo ""
    echo "## Rust"
} > "$tmpfile"

rg --line-number --sort path --no-heading --color never \
    '^[[:space:]]*(pub(\([^)]*\))?[[:space:]]+)?(async[[:space:]]+)?(unsafe[[:space:]]+)?(extern[[:space:]]+"[^"]*"[[:space:]]+)?(fn|struct|enum|trait|impl|mod|type|const|static)([^A-Za-z0-9_]|$)' \
    --glob="*.rs" \
    --glob="!src/generated/**" \
    src \
    | sed -E 's/^([^:]+:[0-9]+:)[[:space:]]*/\1 /; s/[[:space:]]+$//; s/[[:space:]]*\{$//' \
    >> "$tmpfile" || true

{
    echo ""
    echo "## Ayanami (.aya)"
} >> "$tmpfile"

if [[ -d "example" ]]; then
    rg --line-number --sort path --no-heading --color never \
        '^[[:space:]]*(pub[[:space:]]+)?(fn|struct|enum|interface|impl|type)([^A-Za-z0-9_]|$)' \
        --glob="*.aya" \
        example \
        | sed -E 's/^([^:]+:[0-9]+:)[[:space:]]*/\1 /; s/[[:space:]]+$//; s/[[:space:]]*\{$//' \
        >> "$tmpfile" || true
fi

if [[ "$check_mode" == true ]]; then
    if [[ ! -f "SYMBOLS.md" ]]; then
        echo "Error: SYMBOLS.md does not exist." >&2
        exit 1
    fi
    # 判定用 cmp（本机 PATH 中的 diff 可能是玩具版：不支持选项且永远 exit 0）
    if ! cmp -s "SYMBOLS.md" "$tmpfile"; then
        echo "Error: SYMBOLS.md is out of date (run ./scripts/gen_symbols.sh)." >&2
        diff -u "SYMBOLS.md" "$tmpfile" >&2 || true
        exit 1
    fi
    echo "SYMBOLS.md is up to date"
    exit 0
else
    cp "$tmpfile" SYMBOLS.md
    chmod 644 SYMBOLS.md
fi
