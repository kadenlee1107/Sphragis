"""Hand-rolled BM25 retrieval over `docs/rag_corpus/`.

Standalone Python implementation so the eval harness can measure the
RAG win against ollama before the Rust port lands. Mirrors the Rust
implementation in `src/ai/rag.rs` byte-for-byte semantically so the
two stay in sync (same tokenizer, same k1/b, same scoring).

Usage:
    from evals.rag import Corpus
    corpus = Corpus.load()
    for title, snippet in corpus.top_k("What is V8-ROOT-1?", k=3):
        print(title, snippet[:80])
"""
from __future__ import annotations

import json
import math
import re
from dataclasses import dataclass
from pathlib import Path

K1 = 1.2
B = 0.75

REPO    = Path(__file__).resolve().parent.parent
CORPUS  = REPO / "docs" / "rag_corpus"
INDEX   = CORPUS / "INDEX.json"
TOKEN   = re.compile(r"[A-Za-z0-9_]+")


def tokenize(text: str) -> list[str]:
    """Lowercase + split on non-alphanumeric. Same as the Rust side."""
    return [m.group(0).lower() for m in TOKEN.finditer(text)]


@dataclass
class Doc:
    slug: str
    title: str
    body: str
    tokens: list[str]


class Corpus:
    """In-memory BM25 corpus. Build once, call top_k repeatedly."""

    def __init__(self, docs: list[Doc]):
        self.docs = docs
        self.df: dict[str, int] = {}
        self.avg_len: float = 0.0
        if docs:
            self.avg_len = sum(len(d.tokens) for d in docs) / len(docs)
        for d in docs:
            for term in set(d.tokens):
                self.df[term] = self.df.get(term, 0) + 1
        self.n = len(docs)

    @classmethod
    def load(cls, corpus_dir: Path = CORPUS) -> "Corpus":
        index = json.loads((corpus_dir / "INDEX.json").read_text(encoding="utf-8"))
        docs: list[Doc] = []
        for entry in index:
            body = (corpus_dir / f"{entry['slug']}.md").read_text(encoding="utf-8")
            docs.append(Doc(
                slug=entry["slug"],
                title=entry["title"],
                body=body,
                tokens=tokenize(body),
            ))
        return cls(docs)

    def idf(self, term: str) -> float:
        df = self.df.get(term, 0)
        if df == 0:
            return 0.0
        # The standard Robertson-Sparck-Jones IDF used in BM25.
        return math.log(1.0 + (self.n - df + 0.5) / (df + 0.5))

    def score(self, query_tokens: list[str], doc: Doc) -> float:
        if not doc.tokens:
            return 0.0
        # Per-doc term frequencies are computed lazily — the corpus is
        # small enough that a fresh count per query is cheaper than a
        # precomputed map.
        tf: dict[str, int] = {}
        for t in doc.tokens:
            tf[t] = tf.get(t, 0) + 1
        norm = 1 - B + B * (len(doc.tokens) / self.avg_len)
        s = 0.0
        for q in set(query_tokens):
            f = tf.get(q, 0)
            if f == 0:
                continue
            s += self.idf(q) * (f * (K1 + 1)) / (f + K1 * norm)
        return s

    def top_k(self, query: str, k: int = 3, snippet_bytes: int = 1500
              ) -> list[tuple[str, str]]:
        """Return [(title, snippet)] for the top-k docs by BM25.

        Snippet is the prefix of the body capped at `snippet_bytes`.
        Long docs get truncated; short ones return the whole body.
        """
        q = tokenize(query)
        if not q or self.n == 0:
            return []
        scored = [(self.score(q, d), d) for d in self.docs]
        scored.sort(key=lambda x: x[0], reverse=True)
        out: list[tuple[str, str]] = []
        for s, d in scored[:k]:
            if s <= 0:
                continue
            snippet = d.body[:snippet_bytes]
            if len(d.body) > snippet_bytes:
                snippet += "\n…(truncated)"
            out.append((d.title, snippet))
        return out

    def context_block(self, query: str, k: int = 3) -> str:
        """Render top_k as a markdown context block to prepend to the
        user prompt. Empty string if nothing scores above zero."""
        hits = self.top_k(query, k=k)
        if not hits:
            return ""
        parts = ["Relevant context from the Bat_OS docs:\n"]
        for title, snippet in hits:
            parts.append(f"## {title}\n{snippet}\n")
        return "\n".join(parts) + "\n---\n"
