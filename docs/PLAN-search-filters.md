# Plan: Search Filters (--min-score, --files, --all)

## Goal
Add output filters for agent workflows and large result sets.

## Requirements
- `--min-score <float>` for BM25/vector/hybrid
- `--files` outputs only paths
- `--all` ignores limit

## Design
- Apply `min_score` after scoring
- `--files` returns `[path...]` in JSON
- `--all` sets limit to a large cap (e.g., 10k)

## Implementation Steps
1) Add flags to `search`, `embed-search`, `hybrid`
2) Post‑filter results
3) Update tool spec + schema

## Edge Cases
- min_score with empty results
- all + huge result sets (cap)
