//! Hand-rolled BM25 retrieval over a compile-time-bundled corpus of
//! Concept notes and `DESIGN_*.md` documents. No embeddings, no
//! external crate, no runtime FS lookups — the corpus is baked into
//! the kernel image via `include_str!` (see `rag_corpus.rs`).
//!
//! Design choices forced by `no_std`:
//!
//! - IDF is precomputed at build time (`scripts/build_rag_corpus.py`)
//!   and embedded as a static `(hash, idf)` table. No runtime `ln()`.
//! - Document term-frequency is computed at query time by re-scanning
//!   each body. The corpus is ~160 KB, ~24k tokens total — well under
//!   a millisecond per query on the inference path.
//! - Term hashing uses FNV-1a 64-bit, identical to the Python builder.
//!
//! Tokenizer: `[A-Za-z0-9_]+`, lowercased. The Python side mirrors
//! exactly — if you change one, change the other.

#![allow(dead_code)]

use alloc::string::{String, ToString};
use alloc::vec::Vec;

use super::rag_corpus::{AVG_DOC_LEN, CORPUS, DOC_TOKEN_COUNTS, IDF_TABLE};

/// BM25 hyperparameters. Standard defaults; ablation lives in the
/// build script if we ever want to sweep them.
pub const BM25_K1: f32 = 1.2;
pub const BM25_B: f32  = 0.75;

/// Maximum bytes of body returned per hit. Long docs get truncated.
pub const SNIPPET_BYTES: usize = 1500;

/// One indexed corpus entry. Static lifetimes because the builder
/// emits these via `include_str!`.
pub struct CorpusEntry {
    pub title: &'static str,
    pub body: &'static str,
}

#[inline]
fn is_token_char(b: u8) -> bool {
    b.is_ascii_alphanumeric() || b == b'_'
}

/// Iterate tokens in `text` without allocating; yields owned `String`
/// only when a caller actually needs the value. The collected form is
/// what callers use; the streaming form is for inner loops.
fn for_each_token<F: FnMut(&str)>(text: &str, mut f: F) {
    let bytes = text.as_bytes();
    let mut i = 0;
    while i < bytes.len() {
        if !is_token_char(bytes[i]) {
            i += 1;
            continue;
        }
        let start = i;
        while i < bytes.len() && is_token_char(bytes[i]) {
            i += 1;
        }
        // SAFETY: the start..i slice is a contiguous run of bytes that
        // each pass `is_token_char` (ASCII), so it's valid UTF-8.
        let raw = unsafe { core::str::from_utf8_unchecked(&bytes[start..i]) };
        f(raw);
    }
}

fn fnv1a64_lower(token: &str) -> u64 {
    let mut h: u64 = 0xcbf29ce484222325;
    for &b in token.as_bytes() {
        let lower = if (b'A'..=b'Z').contains(&b) { b + 32 } else { b };
        h ^= lower as u64;
        h = h.wrapping_mul(0x100000001b3);
    }
    h
}

/// Binary search the precomputed IDF table for a term's IDF. Returns
/// 0.0 if the term is unseen in the corpus (treated as an out-of-vocab
/// no-op term in the BM25 sum).
fn idf(hash: u64) -> f32 {
    let table = IDF_TABLE;
    let mut lo = 0usize;
    let mut hi = table.len();
    while lo < hi {
        let mid = (lo + hi) / 2;
        let h = table[mid].0;
        if h < hash {
            lo = mid + 1;
        } else if h > hash {
            hi = mid;
        } else {
            return table[mid].1;
        }
    }
    0.0
}

fn score_doc(query_hashes: &[u64], doc_idx: usize) -> f32 {
    let doc_len = DOC_TOKEN_COUNTS[doc_idx] as f32;
    if doc_len == 0.0 {
        return 0.0;
    }
    let norm = 1.0 - BM25_B + BM25_B * (doc_len / AVG_DOC_LEN);
    let body = CORPUS[doc_idx].body;

    // For each unique query term, compute TF (occurrences in this
    // doc) by streaming the body. We dedupe query hashes ahead of
    // time so we don't double-count repeated query terms.
    let mut s = 0.0f32;
    for &q in query_hashes {
        let mut tf: u32 = 0;
        for_each_token(body, |t| {
            if fnv1a64_lower(t) == q {
                tf += 1;
            }
        });
        if tf == 0 {
            continue;
        }
        let tf_f = tf as f32;
        s += idf(q) * (tf_f * (BM25_K1 + 1.0)) / (tf_f + BM25_K1 * norm);
    }
    s
}

/// Top-k retrieval. Returns `(title, snippet)` ordered by BM25 score.
/// Snippet is the prefix of the body capped at `SNIPPET_BYTES`.
pub fn top_k(query: &str, k: usize) -> Vec<(String, String)> {
    if CORPUS.is_empty() || k == 0 {
        return Vec::new();
    }

    // Collect unique query term hashes.
    let mut q_hashes: Vec<u64> = Vec::with_capacity(16);
    for_each_token(query, |t| {
        let h = fnv1a64_lower(t);
        if !q_hashes.contains(&h) {
            q_hashes.push(h);
        }
    });
    if q_hashes.is_empty() {
        return Vec::new();
    }

    let mut scored: Vec<(f32, usize)> = Vec::with_capacity(CORPUS.len());
    for i in 0..CORPUS.len() {
        let s = score_doc(&q_hashes, i);
        if s > 0.0 {
            scored.push((s, i));
        }
    }
    // Largest score first. Stable insertion sort by score descending.
    scored.sort_by(|a, b| b.0.partial_cmp(&a.0).unwrap_or(core::cmp::Ordering::Equal));

    let n = scored.len().min(k);
    let mut out: Vec<(String, String)> = Vec::with_capacity(n);
    for &(_score, idx) in &scored[..n] {
        let entry = &CORPUS[idx];
        let snippet = if entry.body.len() > SNIPPET_BYTES {
            // Snip at a token boundary if we can find one nearby; else
            // fall back to a hard byte cut. We probe up to 64 bytes
            // back from the cap looking for whitespace.
            let mut cut = SNIPPET_BYTES;
            let probe_start = cut.saturating_sub(64);
            let bytes = entry.body.as_bytes();
            for i in (probe_start..cut).rev() {
                if bytes[i].is_ascii_whitespace() {
                    cut = i;
                    break;
                }
            }
            let mut s = String::with_capacity(cut + 16);
            s.push_str(&entry.body[..cut]);
            s.push_str("\n…(truncated)");
            s
        } else {
            entry.body.to_string()
        };
        out.push((entry.title.to_string(), snippet));
    }
    out
}

/// Render top-k retrieval as a markdown context block to prepend to
/// the user prompt. Returns `None` if no doc scored above zero.
pub fn context_block(query: &str, k: usize) -> Option<String> {
    let hits = top_k(query, k);
    if hits.is_empty() {
        return None;
    }
    let mut buf = String::from("Relevant context from the Bat_OS docs:\n\n");
    for (title, snippet) in hits {
        buf.push_str("## ");
        buf.push_str(&title);
        buf.push('\n');
        buf.push_str(&snippet);
        buf.push_str("\n\n");
    }
    buf.push_str("---\n");
    Some(buf)
}
