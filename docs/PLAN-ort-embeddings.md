# Plan: ORT Embeddings (Real Vectors)

## Goal
Replace hash‑based placeholder embeddings with **real local embeddings** via **onnxruntime (ORT)**.

## Requirements
- Local model, no remote calls.
- Deterministic embeddings for doc chunks + queries.
- Cache embeddings per chunk.

## Design
- Add an **embedding provider** abstraction:
  - `HashProvider` (current)
  - `OrtProvider`
- Store model path + dims in config / env.
- Normalize vectors for cosine.

## Implementation Steps
1) **Add ORT deps**
   - Rust crates: `ort`, `ndarray` (or `ndarray` + `tract` if needed).
2) **Model loader**
   - Load ONNX model once; keep session in a singleton.
   - Accept `--model <path>` or env `OBSIDX_EMBED_MODEL`.
3) **Embedding API**
   - `embed_text(text) -> Vec<f32>`
   - Batch embed if model supports.
4) **Integrate into indexing**
   - When indexing chunks, call ORT provider.
   - Store vectors in SQLite (same schema) or VSS table if already available.
5) **Query embedding**
   - Use the same provider in `embed_search` and `hybrid`.
6) **Config / CLI**
   - `obsidx embed-index --model <path>` or `--embedding-backend {hash|ort}`.
7) **Tests / Verification**
   - Sanity: cosine(query, self) > cosine(query, unrelated).

## Edge Cases
- Model missing / incompatible dims.
- Tokenization mismatch (model expects specific preproc).
- Batch size memory limits.
