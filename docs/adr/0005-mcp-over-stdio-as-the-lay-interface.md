# 0005. MCP over stdio as the interface for non-technical users

Date: 2026-11-06
Status: Accepted

## Context

Many club mappers are not technical. They will use freemapper through an AI agent. The user named Claude Code,
CLI agents and any standard MCP client as the clients.

## Decision

The MCP server is a primary interface, not an add-on. `freemapper mcp` serves MCP over stdio with `rmcp`, the
official SDK. The tools mirror the CLI and describe themselves for agents: `find_place`, `create_project`,
`open_area_selector`, `estimate_download`, `start_pipeline`, `project_status`, `preview_layer`, `list_layers`.
Rulings from M6 on 2026-11-07:

- `start_pipeline` returns a job id at once, and agents poll `project_status`, which carries the last 50 lines.
  No MCP progress notifications are sent.
- Results are compact JSON text, which every client shows.
- Relative paths are refused, because the agent does not know the server's working folder.
- Nominatim calls are spaced at least 1 s apart server-wide, following its usage policy.

## Consequences

- Client timeouts on long downloads do not matter.
- No `.mcpb` bundle and no HTTP transport in the MVP.
- Cost if wrong: clients that show notifications show nothing while a job runs, and clients with structured
  output get no schema.

Source: spec §1, §3 (MCP), §4.6; plan-mode notes; plan `2026-11-07-m6-mcp-server.md`; M6 rulings (Task 4, Final).
