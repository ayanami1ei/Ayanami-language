#!/usr/bin/env bash
# check_all.sh — 提交前仓库不变量检查
#
# 依次检查：
#   1. 版本号一致性（Cargo.toml / std / vscode / 标签）
#   2. src/ 下文件行数（默认 ≤300）
#   3. SYMBOLS.md 符号地图是否过期
#   4. cargo check 零告警
set -euo pipefail
cd "$(dirname "$0")/.."

./scripts/check_version.sh
./scripts/check_file_sizes.sh
./scripts/gen_symbols.sh --check
./scripts/check_warnings.sh

if [ "${AYANAMI_SKIP_REGRESSION:-0}" != "1" ]; then
    ./scripts/regression.sh
fi

echo "all checks passed"
