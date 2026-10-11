#!/usr/bin/env python3
"""按编译器 effect 检查输出，为 selfhold 源码补 `#[alloc]` / `#[state]` 标注。

用法：
    python3 scripts/selfhost-annotate.py            # 编译 selfhost/src/main.aya 并补标注（幂等）
    python3 scripts/selfhost-annotate.py FILE.aya   # 只针对指定入口编译
    python3 scripts/selfhost-annotate.py --dry      # 只打印将要改动的位置

实现：解析 "warning: function `NAME` may have effect `EFF`" 与随后的
`--> path:line:col`；在声明的起始行前插入缺失的 effect 注解，保持既有注解。
"""
import re
import subprocess
import sys
from pathlib import Path

ROOT = Path(__file__).resolve().parent.parent
ENTRY = ROOT / "selfhost/src/main.aya"
STAGE0 = ROOT / "target/release/ayanami"
RUNTIME = ROOT / "src/runtime.c"

DECL_RE = re.compile(
    r"^(?P<indent>\s*)(?:(?:pub|priv|async)\s+)*fn\s+(?P<name>[A-Za-z_][A-Za-z0-9_]*)\s*\("
)


def compile_entry(entry: Path) -> str:
    env = {
        "AYANAMI_RUNTIME": str(RUNTIME),
        "PATH": "/usr/bin:/bin",
        "HOME": str(Path.home()),
    }
    proc = subprocess.run(
        [str(STAGE0), "check", str(entry)],
        cwd=ROOT, env=env, capture_output=True, text=True,
    )
    return proc.stdout + proc.stderr


def collect(output: str) -> dict[tuple[str, int, str], set[str]]:
    """(path, 声明行, 函数名) -> 缺失的 effects（逐行解析，避免长正则）"""
    want: dict[tuple[str, int, str], set[str]] = {}
    name = ""
    eff = ""
    for ln in output.split("\n"):
        if ln.startswith("warning: function `"):
            rest = ln[len("warning: function `"):]
            name = rest.split("`", 1)[0]
            eff = ""
            if "may have effect `" in ln:
                eff = ln.split("may have effect `", 1)[1].split("`", 1)[0]
            continue
        loc = ln.strip()
        if name and eff and loc.startswith("--> "):
            target = loc[4:]
            parts = target.rsplit(":", 2)
            if len(parts) == 3 and parts[1].isdigit():
                key = (parts[0], int(parts[1]), name)
                want.setdefault(key, set()).add(eff)
            name = ""
            eff = ""
    return want


def find_decl(lines: list[str], line: int, name: str) -> int | None:
    """找到 name 的声明行（0 基）；告警行号可能落在注解行/函数体/声明本身"""
    leaf = name.rsplit(".", 1)[-1]   # `Ast.new` / `Parser.new` -> new
    pat = re.compile(r"^\s*(?:(?:pub|priv|async)\s+)*fn\s+" + re.escape(leaf) + r"\s*\(")
    for i in range(min(line, len(lines)) - 1, -1, -1):
        if pat.match(lines[i]):
            return i
    for i in range(line, min(line + 400, len(lines))):
        if pat.match(lines[i]):
            return i
    return None


def annotate(path: str, line: int, name: str, effects: set[str], dry: bool) -> bool:
    p = Path(path)
    if not p.is_absolute():
        p = ROOT / p
    lines = p.read_text().split("\n")
    idx = find_decl(lines, line, name)
    if idx is None:
        print(f"  skip（找不到声明）{path}:{line} {name}")
        return False
    have = set()
    j = idx - 1
    while j >= 0 and lines[j].strip().startswith("#["):
        have.add(lines[j].strip().strip("#[]").split("(")[0])
        j -= 1
    missing = effects - have
    if not missing:
        return False
    indent = re.match(r"\s*", lines[idx]).group(0)
    tags = [f"{indent}#[{e}]" for e in ("io", "alloc", "state") if e in (have | missing)]
    if dry:
        print(f"  {path}:{idx + 1} {name} <- {' '.join(t.strip() for t in tags)}")
        return True
    lines[idx - len(have):idx] = tags
    p.write_text("\n".join(lines))
    return True


def main() -> int:
    args = [a for a in sys.argv[1:] if a != "--dry"]
    dry = "--dry" in sys.argv
    entry = Path(args[0]) if args else ENTRY
    out = compile_entry(entry)
    want = collect(out)
    errors = len(re.findall(r"^error: ", out, re.M))
    print(f"编译 {entry.relative_to(ROOT)}：warning 组 {len(want)}，error {errors}")
    changed = 0
    for (path, line, name), effects in sorted(want.items()):
        if annotate(path, line, name, effects, dry):
            changed += 1
    print(f"{'将改动' if dry else '已改动'} {changed} 处")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
