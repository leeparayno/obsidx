# Plan: Multi‑get + Doc IDs

## Goal
Enable retrieval by stable doc IDs and batch retrieval via glob/list.

## Requirements
- `obsidx get #<docid>`
- `obsidx multi-get <glob>` or `--paths a,b,c`
- Optional `--collection <name>` filter

## Design
### Doc ID
- Deterministic hash of absolute path (or normalized vault‑relative path)
- Store as `doc_id` in Tantivy (STRING|STORED)
- Show doc_id in search results

### Multi‑get
- Accept glob patterns (e.g., `Notes/2026-*.md`)
- Accept list input (comma separated or stdin)
- Return concatenated JSON array of docs

## Implementation Steps
1) Add `doc_id` to schema + compute in indexer
2) Modify `search` output to include `doc_id`
3) `get` detects `#` prefix, resolves doc_id
4) Implement `multi-get` to resolve globs or list
5) Apply collection filter

## Edge Cases
- Doc ID collision (log + fallback to path)
- Globs that match no files

## Tests
- Known path -> doc_id -> get
- Multi‑get by glob
