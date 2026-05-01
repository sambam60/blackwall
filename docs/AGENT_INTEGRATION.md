# Agent Integration

Blackwall is meant to work with any agent by sitting at the boundary where model intent becomes a side effect.

There are two generic integration paths today:

- `blackwall exec -- <agent>` wraps an agent process and injects shell shims into its environment.
- `blackwall proxy-mcp -- <server>` wraps a stdio MCP server and evaluates tool calls before forwarding them.

## Generic Agent Wrapper

```bash
blackwall exec -- <agent-command> <args>
```

This starts a gateway, creates a session-specific shim directory, prepends it to `PATH`, and runs the agent with `BLACKWALL_SESSION` and `BLACKWALL_SOCKET` set.

Use this for CLI agents, local IDE agent subprocesses, test harnesses, and custom agent runtimes.

## MCP Wrapper

```bash
blackwall proxy-mcp -- <stdio-mcp-server-command> <args>
```

Use this anywhere an agent config accepts a stdio MCP server command. The proxy inspects `tools/call` JSON-RPC requests. Read-oriented tools can be allowed, side-effecting tools can require confirmation, and dangerous payloads can be denied by policy.

## Codex Local Clients

Codex has its own sandbox and approval model. Keep those controls enabled and layer Blackwall around the local process:

```bash
blackwall exec -- codex --sandbox workspace-write --ask-for-approval on-request
```

For a stricter local posture, use Codex's untrusted approval mode:

```bash
blackwall exec -- codex --sandbox workspace-write --ask-for-approval untrusted
```

In `~/.codex/config.toml`, the same posture can be expressed as:

```toml
sandbox_mode = "workspace-write"
approval_policy = "on-request"

[shell_environment_policy]
inherit = "core"
ignore_default_excludes = false
```

Use `danger-full-access` only when the surrounding environment already provides isolation. Blackwall is a policy gateway, not a replacement for Codex's sandbox.

### Codex MCP

Codex supports stdio MCP servers in `config.toml`. Wrap each local MCP server with Blackwall:

```toml
[mcp_servers.example]
command = "blackwall"
args = ["proxy-mcp", "--", "npx", "-y", "@example/mcp-server"]
```

If a server is remote HTTP-only, put a Blackwall-aware proxy in front of it before treating it as protected.

## Cursor

Run `blackwall init` once so new terminals source `~/.blackwall/env`, then keep `blackwall` running while Cursor opens agent terminals.

Wrap MCP servers in `.cursor/mcp.json`:

```json
{
  "mcpServers": {
    "filesystem": {
      "command": "blackwall",
      "args": ["proxy-mcp", "--", "npx", "@modelcontextprotocol/server-filesystem", "/tmp"]
    }
  }
}
```

## Claude Code

```bash
blackwall exec -- claude
```

## Custom Agents

For custom runtimes, prefer a native adapter that emits Blackwall action envelopes before side effects. If that is not available, start with the shell and MCP wrappers and keep the runtime's own sandbox enabled.
