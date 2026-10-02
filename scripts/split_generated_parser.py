#!/usr/bin/env python3
"""Split an asuka-generated `ayanami_parser.rs` into a module directory.

Usage: split_generated_parser.py GEN_FILE OUT_DIR [MAX_LINES]

Deterministic: same input produces the same files. Fails loudly if the
generated file does not have the expected `impl Parser { ... }` structure.
"""
import os
import re
import sys


def fail(msg: str) -> None:
    sys.exit(f"split_generated_parser: error: {msg}")


def main() -> None:
    if len(sys.argv) < 3:
        fail("usage: split_generated_parser.py GEN_FILE OUT_DIR [MAX_LINES]")
    src, outdir = sys.argv[1], sys.argv[2]
    max_lines = int(sys.argv[3]) if len(sys.argv) > 3 else 180

    text = open(src, encoding="utf-8").read()
    if not text.startswith("// @generated"):
        fail(f"{src} does not start with '// @generated'")
    lines = text.split("\n")

    impl_start = next((i for i, l in enumerate(lines) if l == "impl Parser {"), None)
    if impl_start is None:
        fail("'impl Parser {' not found")
    impl_end = next(
        (i for i in range(len(lines) - 1, -1, -1) if lines[i] == "}"), None
    )
    if impl_end is None or impl_end <= impl_start:
        fail("closing brace of 'impl Parser' not found")

    methods = [
        i
        for i in range(impl_start + 1, impl_end)
        if re.match(r"^    (pub )?fn ", lines[i])
    ]
    if not methods:
        fail("no methods found inside 'impl Parser'")

    ends = methods[1:] + [impl_end]
    files, cur, cur_len = [], [], 0
    wrapper = 4  # use line + blank + impl header + closing brace
    for start, end in zip(methods, ends):
        size = end - start
        if cur and cur_len + size + wrapper > max_lines:
            files.append(cur)
            cur, cur_len = [], 0
        cur.append((start, end))
        cur_len += size
    if cur:
        files.append(cur)

    os.makedirs(outdir, exist_ok=True)
    for name in os.listdir(outdir):
        if re.match(r"^parser_\d+\.rs$", name):
            os.remove(os.path.join(outdir, name))

    mods = []
    for idx, chunks in enumerate(files, 1):
        name = f"parser_{idx:02d}"
        mods.append(name)
        body = []
        for start, end in chunks:
            body.extend(lines[start:end])
        body_text = re.sub(r"^    fn ", "    pub(super) fn ", "\n".join(body), flags=re.M)
        with open(os.path.join(outdir, name + ".rs"), "w", encoding="utf-8") as f:
            f.write("use super::*;\n\nimpl Parser {\n")
            f.write(body_text.rstrip() + "\n}\n")

    header = "\n".join(lines[:impl_start]).rstrip()
    with open(os.path.join(outdir, "mod.rs"), "w", encoding="utf-8") as f:
        f.write(header + "\n\n")
        for name in mods:
            f.write(f"mod {name};\n")

    print(f"split {src} -> {outdir}: {len(methods)} methods in {len(files)} files")


if __name__ == "__main__":
    main()
