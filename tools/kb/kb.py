#!/usr/bin/env python3
"""Ayanami 本地知识库：把教程 book/ 与文档切块、建向量索引、检索。

用法：
  python3 tools/kb/kb.py build [--root DIR]
  python3 tools/kb/kb.py search "QUERY" [-k N] [--json] [--root DIR]

仅用 Python 标准库。向量优先用 Ollama `nomic-embed-text`（HTTP API），
不可用时退回 TF-IDF（检索用 Jaccard token 重叠）。
"""

import argparse
import json
import re
import sys
import urllib.error
import urllib.request
from collections import defaultdict
from math import log, sqrt
from pathlib import Path

OLLAMA_URL = "http://127.0.0.1:11434/api/embeddings"
OLLAMA_MODEL = "nomic-embed-text"
MAX_PART = 1800
MERGE_LIMIT = 900

HEADING_RE = re.compile(r"^#{1,6}(\s|$)")
TOKEN_RE = re.compile(r"[a-z0-9_]+|[\u4e00-\u9fff]")


def sources_for(root: Path):
    out = []
    book = root / "book" / "src"
    if book.exists():
        out.extend(sorted(book.rglob("*.md")))
    for rel in ("README.md", "std/README.md", "docs/selfhost.md"):
        p = root / rel
        if p.exists():
            out.append(p)
    return out


def split_sections(text: str):
    """按 Markdown 标题切分；围栏代码块内的 # 不算标题。返回 [(heading, body)]。"""
    sections = []
    heading = ""
    body = []
    in_fence = False
    fence = ""
    for line in text.splitlines():
        stripped = line.strip()
        if stripped.startswith("```") or stripped.startswith("~~~"):
            marker = stripped[:3]
            if not in_fence:
                in_fence, fence = True, marker
            elif stripped.startswith(fence):
                in_fence, fence = False, ""
            body.append(line)
            continue
        if not in_fence and HEADING_RE.match(line):
            sections.append((heading, "\n".join(body)))
            heading = line.lstrip("#").strip()
            body = []
        else:
            body.append(line)
    sections.append((heading, "\n".join(body)))
    return sections


def split_long(text: str):
    """按空行分段，累计到不超过 MAX_PART；单段超限则整段保留。"""
    if len(text) <= MAX_PART:
        return [text]
    parts, cur = [], ""
    for para in text.split("\n\n"):
        if cur and len(cur) + len(para) + 2 > MAX_PART:
            parts.append(cur)
            cur = para
        else:
            cur = para if not cur else cur + "\n\n" + para
    if cur:
        parts.append(cur)
    return parts


def build_chunks(text: str, source: str):
    merged = []
    for heading, body in split_sections(text):
        if merged and merged[-1]["heading"] == heading and len(merged[-1]["text"]) + len(body) < MERGE_LIMIT:
            merged[-1]["text"] += "\n\n" + body
        else:
            merged.append({"heading": heading, "text": body})
    chunks = []
    for m in merged:
        for part in split_long(m["text"]):
            part = part.strip()
            if part:
                chunks.append({"source": source, "heading": m["heading"], "text": part})
    return chunks


def tokenize(text: str):
    return set(TOKEN_RE.findall(text.lower()))


def ollama_embed(texts):
    """返回向量列表；任一失败返回 None（调用方退回 TF-IDF）。"""
    vecs = []
    try:
        for t in texts:
            req = urllib.request.Request(
                OLLAMA_URL,
                data=json.dumps({"model": OLLAMA_MODEL, "prompt": t}).encode("utf-8"),
                headers={"Content-Type": "application/json"},
                method="POST",
            )
            with urllib.request.urlopen(req, timeout=30) as resp:
                vecs.append(json.loads(resp.read().decode("utf-8"))["embedding"])
    except Exception as e:  # 连接失败/超时/格式错误
        print(f"kb: ollama embedding unavailable ({e}); fallback to tf-idf", file=sys.stderr)
        return None
    return vecs


def tfidf_vectors(chunks):
    df = defaultdict(int)
    tfs = []
    for c in chunks:
        tokens = re.findall(r"[a-z0-9_]+|[\u4e00-\u9fff]", c["text"].lower())
        tfs.append(tokens)
        for t in set(tokens):
            df[t] += 1
    n = max(len(chunks), 1)
    vectors = []
    for tokens in tfs:
        counts = defaultdict(int)
        for t in tokens:
            counts[t] += 1
        total = max(len(tokens), 1)
        vectors.append({t: (cnt / total) * (log((n + 1) / (df[t] + 1)) + 1) for t, cnt in counts.items()})
    return vectors


def cosine(v1, v2):
    if isinstance(v1, dict):  # tf-idf 稀疏
        keys = set(v1) & set(v2)
        dot = sum(v1[k] * v2[k] for k in keys)
        n1 = sqrt(sum(x * x for x in v1.values()))
        n2 = sqrt(sum(x * x for x in v2.values()))
    else:
        dot = sum(a * b for a, b in zip(v1, v2))
        n1 = sqrt(sum(a * a for a in v1))
        n2 = sqrt(sum(b * b for b in v2))
    if n1 == 0 or n2 == 0:
        return 0.0
    return dot / (n1 * n2)


def cmd_build(root: Path):
    index_path = root / "tools" / "kb" / "index.json"
    index_path.parent.mkdir(parents=True, exist_ok=True)
    chunks = []
    for src in sources_for(root):
        try:
            text = src.read_text(encoding="utf-8")
        except Exception:
            continue
        chunks.extend(build_chunks(text, src.relative_to(root).as_posix()))
    if not chunks:
        print("kb: no source documents found", file=sys.stderr)
        return 1
    vectors = ollama_embed([c["text"] for c in chunks])
    backend = "ollama"
    if vectors is None:
        backend = "tfidf"
        vectors = tfidf_vectors(chunks)
    for i, c in enumerate(chunks):
        c["id"] = i
    payload = {"backend": backend, "chunks": chunks, "vectors": vectors}
    index_path.write_text(json.dumps(payload, ensure_ascii=False, indent=1), encoding="utf-8")
    print(f"kb: {len(chunks)} chunks backend={backend} -> tools/kb/index.json")
    return 0


def cmd_search(query: str, k: int, as_json: bool, root: Path):
    index_path = root / "tools" / "kb" / "index.json"
    if not index_path.exists():
        print("kb: index missing, run: python3 tools/kb/kb.py build", file=sys.stderr)
        return 2
    index = json.loads(index_path.read_text(encoding="utf-8"))
    chunks, vectors, backend = index["chunks"], index["vectors"], index["backend"]

    qvec = None
    if backend == "ollama":
        got = ollama_embed([query])
        qvec = got[0] if got else None
    scores = []
    if qvec is not None:
        for c, v in zip(chunks, vectors):
            scores.append((cosine(qvec, v), c))
    else:  # TF-IDF/回退：Jaccard token 重叠
        qt = tokenize(query)
        for c in chunks:
            ct = tokenize(c["text"])
            union = len(qt | ct)
            scores.append(((len(qt & ct) / union) if union else 0.0, c))
    scores.sort(key=lambda x: x[0], reverse=True)
    results = [(s, c) for s, c in scores if s > 0][:k]
    if not results:
        print("kb: no results")
        return 1
    if as_json:
        print(json.dumps([{"score": s, "source": c["source"], "heading": c["heading"], "text": c["text"]} for s, c in results], ensure_ascii=False, indent=1))
        return 0
    for i, (s, c) in enumerate(results):
        text = c["text"]
        if len(text) > 700:
            text = text[:697] + "..."
        print(f"[{i + 1}] score={s:.2f} {c['source']} › {c['heading']}")
        print(text)
        print()
    return 0


def main():
    ap = argparse.ArgumentParser(prog="kb")
    sub = ap.add_subparsers(dest="cmd", required=True)
    b = sub.add_parser("build")
    b.add_argument("--root", default=".")
    b.add_argument("--force", action="store_true")
    s = sub.add_parser("search")
    s.add_argument("query")
    s.add_argument("-k", type=int, default=5)
    s.add_argument("--json", action="store_true")
    s.add_argument("--root", default=".")
    args = ap.parse_args()
    root = Path(args.root).resolve()
    if args.cmd == "build":
        return cmd_build(root)
    return cmd_search(args.query, args.k, args.json, root)


if __name__ == "__main__":
    sys.exit(main())
