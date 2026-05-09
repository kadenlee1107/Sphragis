//! Hand-rolled BM25 retrieval over a compile-time-bundled corpus
//! of Concept notes and `DESIGN_*.md` documents. No embeddings, no
//! external crate, no runtime FS lookups — the corpus is baked into
//! the kernel image via `include_str!`.
//!
//! Phase 2 stub. The corpus build script (`scripts/build_rag_corpus.sh`)
//! and the actual scoring loop land in Phase 8.

#![allow(dead_code)]

use alloc::string::String;
use alloc::vec::Vec;

/// One indexed corpus entry. Title is the slug; body is the full
/// markdown text. We score over body, return body capped to a few KB.
pub struct CorpusEntry {
    pub title: &'static str,
    pub body: &'static str,
}

/// Compile-time corpus. Phase 8 generates this from the vault and
/// the top-level `DESIGN_*.md` files. Empty for now so `cargo check`
/// is happy.
pub const CORPUS: &[CorpusEntry] = &[];

/// Top-k retrieval. Returns `(title, snippet)` pairs ordered by
/// BM25 score. Snippet is the first 1.5 KB of the body — full text
/// is overkill for context injection.
pub fn top_k(_query: &str, _k: usize) -> Vec<(String, String)> {
    // Phase 2 stub.
    Vec::new()
}

/// BM25 scoring constants. Standard defaults from the literature.
/// Live here as `pub const` so a benchmark harness can sweep them
/// without surgery on the call sites.
pub const BM25_K1: f32 = 1.2;
pub const BM25_B: f32  = 0.75;
