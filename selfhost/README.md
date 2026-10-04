# Ayanami 自举编译器（selfhost）

用 Ayanami 语言重写 Ayanami 编译器（stage-1）。设计与里程碑见 `docs/selfhost.md`。

## 命令

```bash
./scripts/selfhost.sh sync          # 拉取/合并 rust、更新 book、重建知识库
./scripts/selfhost.sh run src/main.aya
./scripts/selfhost.sh test          # selfhost/tests/*_test.aya
./scripts/selfhost.sh kb "所有权 移动"   # 检索教程知识库
```

## 目录

- `src/base/` 编译器内部基础库（文件 IO、StringBuf、Map、驻留、诊断）
- `src/lexer/` `src/parser/` `src/hir/` `src/mir/` `src/lir/` `src/driver/`
- `tests/` 每个模块的自测（stage-0 运行，非 0 = 失败）

## 约束

- 只用 `book/` 记录、且经 stage-0 验证过的 Ayanami 语法/API；生成代码前先检索知识库。
- 不改 `src/**.rs`（Rust stage-0 由 `rust` 主线维护），差异通过 `sync` 合并。
