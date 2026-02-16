# Plan: Collections + Scoped Search (obsidx)

## Goal
Add named collections that map to vault subpaths (or external folders) and enable scoped search and indexing.

## Requirements
- `collection add <path> --name <name>`
- `collection list`
- `collection remove <name>`
- Search scoping: `obsidx search --collection <name>`
- Index only collection paths when specified
- Collections stored in config (local, per‑user)

## Design
### Storage
- Config file: `~/.obsidx/config.toml`
- Format:
  ```toml
  [collections]
  notes = "/Users/leeparayno/OpenClaw/Jarvis/Notes"
  meetings = "/Users/leeparayno/OpenClaw/Jarvis/Meetings"
  ```

### Indexing changes
- Add optional `--collection <name>` to `index`, `search`, `embed-index`, `embed-search`, `hybrid`, `get`, `multi-get`
- When collection is set:
  - Resolve path from config
  - Restrict scan + retrieval to that root

### Schema impact
- Add stored field `collection` (STRING|STORED)
- Populate during indexing
- Filter queries by collection field when requested

### CLI additions
```
obsidx collection add <path> --name <name>
obsidx collection list
obsidx collection remove <name>

obsidx index --collection notes
obsidx search --collection notes "query"
obsidx embed-index --collection notes
```

## Implementation Steps
1) Add config loader/saver (TOML) for `~/.obsidx/config.toml`
2) Add `collection` subcommands
3) Update scanning to accept optional root override
4) Add collection field to Tantivy schema + embed DB
5) Filter search results by collection
6) Update docs + tool spec

## Edge Cases
- Missing collection name → error with list of valid collections
- Path outside vault → allow (for external folders)
- Renamed/missing paths → warning + skip

## Tests
- Add/remove/list collections
- Index + search within collection
- Ensure other collections excluded
