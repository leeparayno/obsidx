# Plan: MCP (stdio) Server

## Goal
Expose obsidx via MCP tools for direct LLM integration.

## Requirements
- `obsidx mcp` stdio server
- Tools: search, vector, hybrid, get, multi-get, status

## Design
- Use MCP JSON RPC over stdin/stdout
- Map MCP tool names to obsidx handlers
- Keep process alive

## Implementation Steps
1) Add `mcp` subcommand
2) Implement dispatcher + JSON schema for tool inputs/outputs
3) Wire to existing functions

## Edge Cases
- Invalid JSON
- Long outputs (chunked)
