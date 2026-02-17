# QMD Competitor Notes — Implementation Deep Dive

> Source: https://github.com/tobi/qmd

## 1) Architecture Overview
- CLI orchestrates indexing/search/MCP.
- Core pipeline in `src/store.ts` (FTS/BM25 + vector + expansion + rerank).
- Local LLM stack via `node-llama-cpp` in `src/llm.ts`.
- Data split: SQLite for index + YAML for collections/context.

**Key refs:**
- `src/qmd.ts` (CLI)
- `src/store.ts` (pipeline + DB)
- `src/llm.ts` (LLM/embeddings)
- `src/db.ts` (SQLite)
- `src/collections.ts` (YAML config)
- `src/mcp.ts` (MCP server)
- `README.md` (overview)

---

## 2) Data Storage & Index Schema
**Default paths**
- SQLite index: `~/.cache/qmd/index.sqlite` (configurable)
- Collections/contexts: `~/.config/qmd/index.yml`
- Model cache: `~/.cache/qmd/models/`

**SQLite tables (high‑level)**
- `content`: content‑addressable store
- `documents`: collection/path → content hash (+ active flag)
- `documents_fts`: FTS5 for filepath/title/body
- `content_vectors`: chunk metadata per hash/seq/position/model
- `vectors_vec`: sqlite‑vec virtual table (vec0, cosine distance)
- `llm_cache`: cached LLM results

**Refs:** `initializeDatabase()` + vec table setup in `src/store.ts`.

---

## 3) Chunking Strategy
- Default **~900 tokens** with **~15% overlap**.
- Uses **scored breakpoints** (headings, code fences, hr, blank lines, lists).
- Avoids splitting inside code fences.
- Char pre‑chunking + token‑aware recheck.

**Key functions:** `scanBreakPoints`, `findCodeFences`, `findBestCutoff`, `chunkDocument`, `chunkDocumentByTokens` in `src/store.ts`.

---

## 4) Embeddings & LLM Stack
- Local models via `node-llama-cpp` (GGUF).
- Embedding formatting:
  - Query: `task: search result | query: ...`
  - Doc: `title: ... | text: ...`
- Auto‑resolve models from HuggingFace + local cache.

**Default models**
- Embedding: `embeddinggemma-300M-Q8_0`
- Reranker: `qwen3-reranker-0.6b-q8_0`
- Query expansion: `qmd-query-expansion-1.7B-q4_k_m`

**Refs:** `src/llm.ts`, formatting helpers in `src/store.ts`.

---

## 5) Vector Search
- SQLite + `sqlite-vec` (vec0 virtual table).
- Distance → similarity: `1 - distance`.
- Two‑step query due to vec join limits:
  1) query `vectors_vec` for (hash_seq, distance)
  2) join via `content_vectors` + `documents` + `content`

**Refs:** `searchVec()` + vec table load in `src/store.ts` and `src/db.ts`.

---

## 6) Hybrid Search (Key Differentiator)
**Pipeline:**
1) BM25 probe (FTS)
2) Query expansion (typed: lex / vec / hyde)
3) Retrieval routing (lex→FTS, vec/hyde→vector)
4) RRF fusion (first lists weighted ~2x)
5) Chunk selection per doc (keyword overlap)
6) Rerank on best chunk only
7) Position‑aware blend to preserve retrieval rank

**Refs:** `hybridQuery()`, `reciprocalRankFusion()` in `src/store.ts`.

---

## 7) CLI Surface
Representative commands:
- `qmd collection add|list|remove|rename`
- `qmd context add|list|rm|check`
- `qmd embed`, `qmd search`, `qmd vsearch`, `qmd query`
- `qmd get`, `qmd multi-get`, `qmd ls`, `qmd status`
- `qmd mcp` (stdio + HTTP daemon)

**Refs:** `src/qmd.ts`.

---

## 8) MCP Server
- Tools: keyword search, vector search, deep search, get, multi_get, status.
- Supports stdio + HTTP daemon modes.
- Dynamic instructions based on index state.

**Refs:** `src/mcp.ts`.

---

## 9) Dependencies (Key)
- `node-llama-cpp` (local LLMs/embeddings/rerank)
- `better-sqlite3` (DB)
- `sqlite-vec` (vector virtual table)
- `@modelcontextprotocol/sdk` (MCP)
- `fast-glob`, `picomatch` (collection scanning)
- `yaml`, `zod`

**Refs:** `package.json`.
