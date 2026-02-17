# Plan: Chunk‑Level Diffing

## Goal
Speed up reindexing by **only re‑embedding changed chunks**.

## Requirements
- Stable chunking strategy.
- Hash‑based chunk identity.

## Design
- Keep `chunk_hash` as stable ID.
- On file update:
  - Rechunk content.
  - Compare hashes vs existing rows.
  - Delete removed chunks, insert new/changed chunks.

## Implementation Steps
1) **Fetch existing chunks** by `path` from SQLite.
2) **Rechunk** document and compute hashes.
3) **Diff**
   - `to_add = new_hashes - old_hashes`
   - `to_remove = old_hashes - new_hashes`
4) **Apply**
   - Delete `to_remove` rows.
   - Insert `to_add` rows (embed only these).
5) **Update mtime** for unchanged chunks or doc.

## Edge Cases
- Chunking strategy changes → force full reindex.
- Duplicate chunk hashes (rare; include seq index in hash if needed).
- Large docs → memory pressure during diff.
