# obsidx Roadmap

This roadmap is informed by features observed in **qmd** and prioritized for obsidx.

## Now (0–2 weeks)
**Goal:** Better agent ergonomics + high‑impact retrieval improvements.

- **Collections**
  - `obsidx collection add <path> --name <name>`
  - `obsidx search --collection <name>`
- **Search filters**
  - `--min-score` for BM25/vector/hybrid
  - `--files` output list (paths only)
  - `--all` output (no limit)
- **Multi‑get**
  - `obsidx multi-get "glob"` or list of paths
- **Doc IDs**
  - Stable doc id hash for fast `get #docid`
- **Config file**
  - `~/.obsidx/config.toml` for defaults

## Next (2–6 weeks)
**Goal:** Improve search quality & integration stability.

- **Hybrid scoring refinement**
  - RRF tuning + optional normalization
  - position‑aware blending
- **Query expansion (lightweight)**
  - expansion via local model or heuristic expansion
- **MCP server (stdio)**
  - `obsidx mcp` exposing `search`, `vector`, `hybrid`, `get`, `multi-get`, `status`
- **Embedding cache**
  - LRU cache table to avoid recompute
- **Chunk‑level diffing**
  - Only re‑embed changed chunks

## Later (6–12 weeks)
**Goal:** Full parity with advanced local search tools.

- **HTTP MCP daemon** (long‑lived)
- **Reranker integration** (local ORT or llama‑cpp)
- **Context tree / hierarchy**
  - Parent context returned with hits
- **Index health / status**
  - `obsidx status` with index stats
- **Collection‑aware embeddings**

## Research / Decisions Needed
- Pick ORT model + tokenizer format
- Choose SQLite VSS extension strategy (bundled vs external)
- Decide doc ID hashing scheme (stable across moves?)

## Notes
- obsidx already has: BM25, vector pipeline (hash placeholder), hybrid RRF, backlinks, watch mode, JSON schema/tools.
- This roadmap assumes ORT + SQLite VSS for real embeddings.
