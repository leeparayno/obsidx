# Plan: Query Expansion + RRF Tuning

## Goal
Improve recall with lightweight expansion before hybrid search.

## Requirements
- Expand query into 1–2 variants
- Run hybrid per variant
- Fuse via RRF with bonus for original query

## Design
- Heuristic expansion: synonyms, remove stopwords, phrase variants
- Optional small local model later

## Implementation Steps
1) Add expansion helper
2) Hybrid uses expanded queries
3) Update RRF scoring with original bonus

## Edge Cases
- Expansion generates empty query
- Duplicates
