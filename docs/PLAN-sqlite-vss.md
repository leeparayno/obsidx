# Plan: SQLite VSS Integration

## Goal
Replace brute‑force cosine scan with **SQLite VSS** for fast vector search.

## Requirements
- Local only.
- Efficient nearest‑neighbor search over embeddings.

## Design
- Add VSS virtual table (e.g., `vectors_vss`) with `rowid` + `embedding`.
- Keep `chunks` as metadata table; store `rowid` mapping.
- Query with VSS and join back to chunks for path/metadata.

## Implementation Steps
1) **Add VSS dependency**
   - Use `sqlite-vss` extension (dynamic load).
2) **Schema changes**
   - Create VSS table: `CREATE VIRTUAL TABLE vectors_vss USING vss0(embedding(384));`
   - Add `vss_rowid` to `chunks` if needed.
3) **Indexing**
   - Insert embedding into VSS table, store rowid in `chunks`.
4) **Search**
   - Query VSS for top K rowids + distance.
   - Join rowids to `chunks` to return path + chunk.
5) **Migration**
   - If VSS missing, fall back to brute‑force.
6) **CLI**
   - `--vector-backend {bruteforce|vss}`

## Edge Cases
- VSS extension not available.
- Dim mismatch vs model.
- Need to rebuild when model dims change.
