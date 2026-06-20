#!/bin/bash
set -e
cd "$(dirname "$0")/../asuka"
cargo build --release 2>&1 | tail -2
ASUKA_BIN="$(pwd)/target/release/asuka"
cd - > /dev/null
"$ASUKA_BIN" gen ayanami.grammar > src/generated/ayanami_parser.rs
echo "Parser regenerated: $(wc -l < src/generated/ayanami_parser.rs) lines"
