#!/usr/bin/env bash
# check_warnings.sh — 校验 cargo check 零告警
#
# 说明：
#   - src/generated/ 为生成产物，已在模块级 #[allow(warnings)]；
#   - asuka/ 子仓同样要求零告警（独立提交）；
#   - 本脚本对主 crate（含所有 target）执行 cargo check 并统计 warning。
set -euo pipefail
cd "$(dirname "$0")/.."

out=$(cargo check --all-targets 2>&1 || true)
n=$(printf '%s\n' "$out" | grep -cE '^warning' || true)
if [ "$n" -ne 0 ]; then
    echo "发现 $n 条告警："
    printf '%s\n' "$out" | grep -E '^warning' | sort | uniq -c | sort -rn | head -20
    exit 1
fi
echo "OK: cargo check 零告警"
