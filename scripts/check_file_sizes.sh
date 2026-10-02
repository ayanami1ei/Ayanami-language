#!/usr/bin/env bash
# check_file_sizes.sh — 校验 src/ 下所有 Rust 文件不超过上限（默认 300 行）
#
# 用法: ./scripts/check_file_sizes.sh [LIMIT]
set -euo pipefail
cd "$(dirname "$0")/.."

limit="${1:-300}"
bad=$(find src -name '*.rs' -exec wc -l {} + \
    | awk -v lim="$limit" '$2 != "total" && $1 > lim { printf "%6d  %s\n", $1, $2 }' \
    | sort -rn)

if [[ -n "$bad" ]]; then
    echo "error: 以下文件超过 $limit 行，请拆分：" >&2
    echo "$bad" >&2
    exit 1
fi
echo "OK: src/ 下所有 .rs 文件不超过 $limit 行"
