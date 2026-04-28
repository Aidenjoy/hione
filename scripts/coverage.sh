#!/usr/bin/env bash
# 运行 cargo-llvm-cov 收集工作空间覆盖率，产出 HTML + lcov + 摘要。
# 覆盖率低于阈值则退出码非 0（便于 CI 集成）。

set -euo pipefail

THRESHOLD=${COVERAGE_THRESHOLD:-80}

# 需要从工作空间根运行
cd "$(dirname "$0")/.."

# 检查依赖
if ! command -v cargo-llvm-cov >/dev/null 2>&1; then
    echo "[coverage] cargo-llvm-cov 未安装，正在安装..."
    cargo install cargo-llvm-cov --locked
fi

# llvm-tools-preview 需要装过；若无则尝试安装
if ! rustup component list --installed 2>/dev/null | grep -q llvm-tools-preview; then
    echo "[coverage] 添加 llvm-tools-preview 组件..."
    rustup component add llvm-tools-preview
fi

OUT=target/coverage
rm -rf "$OUT"
mkdir -p "$OUT"

# 排除 bootstrap/glue 代码
# - main.rs / build.rs：进程入口与 Cargo 构建脚本
# - src-tauri/src/{commands,pty_handler,window_manager,main}.rs：Tauri glue 与系统调用
# - hi-cli/src/commands/start.rs：派生子进程，无法离线测试
# - tests/：集成测试文件自身不参与覆盖率统计
IGNORE_REGEX='(src/main\.rs|build\.rs|src-tauri/src/(commands|pty_handler|window_manager|main)\.rs|hi-cli/src/commands/start\.rs|tests/)'

echo "[coverage] 运行 cargo llvm-cov --workspace ..."
cargo llvm-cov clean --workspace

# HTML 报告
cargo llvm-cov --workspace --html --output-dir "$OUT/html" \
    --ignore-filename-regex "$IGNORE_REGEX" \
    --no-fail-fast

# lcov 报告（给 CI / Codecov 用）
cargo llvm-cov --workspace --lcov --output-path "$OUT/lcov.info" \
    --ignore-filename-regex "$IGNORE_REGEX" \
    --no-fail-fast

# 摘要
SUMMARY_FILE="$OUT/coverage-summary.txt"
cargo llvm-cov report \
    --ignore-filename-regex "$IGNORE_REGEX" \
    | tee "$SUMMARY_FILE"

# 提取 TOTAL 行覆盖率百分比（line coverage 列）
# llvm-cov 输出格式示例：
# TOTAL     123     45   67.89%   ...   80.12%   ...   70.00%
# 列：regions, missed_regions, cover% | functions, missed_functions, cover% | lines, missed_lines, cover%
# 取最后一个百分比列（line coverage）更稳。
LINE_COV=$(awk '/^TOTAL/ {
    for (i=NF; i>=1; i--) {
        if ($i ~ /%$/) { print $i; exit }
    }
}' "$SUMMARY_FILE" | tr -d '%')

if [[ -z "${LINE_COV:-}" ]]; then
    echo "[coverage] 无法解析 TOTAL 行覆盖率" >&2
    exit 2
fi

echo "[coverage] 总行覆盖率：${LINE_COV}%（阈值 ${THRESHOLD}%）"
echo "[coverage] HTML 报告：$OUT/html/index.html"
echo "[coverage] lcov 报告：$OUT/lcov.info"

# 浮点比较用 awk
PASSED=$(awk -v cov="$LINE_COV" -v th="$THRESHOLD" 'BEGIN { print (cov+0 >= th+0) ? 1 : 0 }')
if [[ "$PASSED" != "1" ]]; then
    echo "[coverage] ❌ 覆盖率低于阈值" >&2
    exit 1
fi

echo "[coverage] ✅ 达标"
