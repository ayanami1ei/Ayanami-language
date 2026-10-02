#!/bin/bash
set -e
cd "$(dirname "$0")/../asuka"
cargo build --release 2>&1 | tail -2
ASUKA_BIN="$(pwd)/target/release/asuka"
cd - > /dev/null

TMP_GEN="$(mktemp)"
trap 'rm -f "$TMP_GEN"' EXIT
"$ASUKA_BIN" gen ayanami.grammar > "$TMP_GEN"
python3 scripts/split_generated_parser.py "$TMP_GEN" src/generated/ayanami_parser
echo "Parser regenerated: $(ls src/generated/ayanami_parser/parser_*.rs | wc -l) files, max $(wc -l src/generated/ayanami_parser/*.rs | awk '$2 != "total" { print $1 }' | sort -rn | head -1) lines"
