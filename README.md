# Blackwall

```
  .-#+-+#--#.  +-.         ..-##..    .+.##-+##-..-.-.  .##-.  -##   .###   .##.  ####.   .##.     +#+     
  .+#+--++##-..##.         -+###+-.   +#++-..-+...++#+--##-.    ##+  ####-  ##-  ###-##.  .##.     ###     
  .+#+.---+#-..##.        -##+--###   #+.     .  .+######+      .##.-## ## ###  ###  -##. .##.     ###     
  -###-+#+###--###++-....-##########..###+.. -#---####++###..    .####. +####  ##########..#######-########
   .-......-.  .----.... ...      ..  ...--..--. ....    ...      ....   ...   ..      ... ....... ........
```

**A deterministic execution firewall for AI agents.**

Blackwall sits between AI agents and their tools. Every action — file write, shell command, network request, MCP tool call — passes through a deterministic policy engine before it executes. No silent side effects. Every invocation auditable. The gateway is rule-based, not AI. It can't be prompt-injected. It can't be socially engineered. It runs at the speed of pattern matching, not inference.

## Quick Start

```bash
# Install from source
cargo install --path crates/blackwall-cli

# One-time: add shell hook so agent terminals are protected
blackwall init

# Start the gateway
blackwall
```

When the gateway is running, every new terminal session automatically gets Blackwall's shell shims injected into its PATH. Agent commands are intercepted and evaluated against your policy before they execute.

```
■ blackwall v0.1.0
  policy: default | session: 7f3a28
  workspace: /Users/you/project
  log: ~/.blackwall/logs/7f3a28.jsonl

  detected: mcp (3 servers), cursor, git

  ● shell hook active — new terminals are protected
  mcp proxy: blackwall proxy-mcp -- <server-command>

  gateway active — press ctrl+c to stop
```

## How It Works

Blackwall operates at the execution boundary — the moment an agent's intent becomes a real side effect.

**Shell interception**: Blackwall creates shim scripts for common commands (`git`, `curl`, `npm`, `python`, `sudo`, etc.) and prepends them to PATH. When a command runs, the shim evaluates it through the gateway's Unix socket before executing the real binary:

```
14:23:01 ✓ shell.exec        git status
14:23:03 ✓ shell.exec        cargo build
14:23:15 ✗ shell.exec        sudo rm -rf /    [permissions.shell.deny]
```

**MCP interception**: For agents using the Model Context Protocol, Blackwall proxies the stdio connection between client and server, intercepting `tools/call` JSON-RPC requests:

```
14:23:20 · mcp.tool_call     read_file
14:23:22 ✗ mcp.tool_call     execute_command  [permissions.shell.deny]
```

**Escalation**: When the gateway is uncertain (a `pause` decision), it prompts the human inline:

```
  ⏸ PAUSE — confirmation required
  │ shell.exec rm -rf node_modules
  │ reason: matches confirmation rule 'rm -rf'
  │
  └─ allow (a) | deny (d) | end session (x) >
```

## Integration

### Cursor

**Shell commands** — run `blackwall init` once to install the shell hook. Then whenever `blackwall` is running, every terminal Cursor opens (including ones the agent uses) will have commands intercepted.

**MCP tools** — edit `.cursor/mcp.json` to wrap your MCP servers:

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

### Claude Code

Wrap the entire process:

```bash
blackwall exec -- claude
```

This injects shell shims into Claude Code's environment. Every command it spawns is intercepted. Your own terminal is unaffected.

### Any Agent

```bash
# Wrap any agent process
blackwall exec -- python my_agent.py
blackwall exec -- node agent.js

# Or start the gateway and source the env in your agent's shell
blackwall
# In agent's terminal:
source ~/.blackwall/env
```

## Commands


| Command                            | What it does                                             |
| ---------------------------------- | -------------------------------------------------------- |
| `blackwall`                        | Start the gateway. Shell shims + IPC server + audit log. |
| `blackwall init`                   | Add shell hook to `~/.zshrc` (one-time setup).           |
| `blackwall exec -- <cmd>`          | Wrap a specific process with full protection.            |
| `blackwall proxy-mcp -- <cmd>`     | Proxy an MCP server with tool call interception.         |
| `blackwall off`                    | Stop all running gateways.                               |
| `blackwall status`                 | Show active sessions and recent logs.                    |
| `blackwall logs`                   | Print the most recent audit log.                         |
| `blackwall logs --follow`          | Live tail of the audit stream.                           |
| `blackwall logs -s <id>`           | Print a specific session's log.                          |
| `blackwall --policy strict`        | Start with the strict policy.                            |
| `blackwall --policy ./custom.yaml` | Start with a custom policy file.                         |


## Policies

Three built-in profiles:

- `**default**` — sensible defaults for development. Standard dev commands allowed, dangerous patterns blocked, unknown network denied.
- `**strict**` — maximum restriction. Every write requires confirmation. No outbound network.
- `**permissive**` — log-only mode. Nothing blocked, everything recorded. Good for auditing what agents actually do.

### Default Policy Summary


|                | Allowed                                                    | Blocked                                | Asks First                                 |
| -------------- | ---------------------------------------------------------- | -------------------------------------- | ------------------------------------------ |
| **File read**  | Workspace files                                            | `.env`, secrets, `/proc`, SSH keys     | —                                          |
| **File write** | Workspace files                                            | System dirs, `.git/config`, SSH        | `package.json`, lockfiles, Dockerfile      |
| **Shell**      | `git`, `npm`, `cargo`, `python`, `node`, `make`, `curl`... | `sudo`, `su`, `chmod +s`, `dd`, `mkfs` | `rm -rf`, `git push --force`, `DROP TABLE` |
| **Network**    | Package registries, GitHub                                 | Everything else (`*`)                  | New domains (first time)                   |
| **Process**    | —                                                          | `/proc/*/mem`, `/proc/*/environ`       | —                                          |


Plus pipe-to-shell patterns are always blocked: `curl ... | sh`, `wget ... | bash`.

### Custom Policies

Write your own in YAML:

```yaml
version: "blackwall/policy-1.0"
name: "my-policy"
description: "Locked down for production deploys"

permissions:
  filesystem:
    read:
      allow: ["${WORKSPACE}/**"]
      deny: ["**/.env", "**/*.key"]
    write:
      allow: ["${WORKSPACE}/src/**"]
      confirm: ["${WORKSPACE}/config/**"]
  shell:
    allow: [git, cargo, npm]
    deny: [sudo, su]
    confirm: ["rm -rf", "git push --force"]
    deny_patterns:
      - "curl * | *sh"
  network:
    allow: [github.com, crates.io]
    deny: ["*"]

patterns:
  - name: secret_then_network
    sequence:
      - action: "filesystem.read"
        path: "**/.env"
      - action: "network.request"
    within: 10
    risk: 90
    on_match: halt
```

The `${WORKSPACE}` variable is resolved to the current working directory at startup.

### Temporal Anti-Patterns

The default policy includes five patterns derived from documented frontier model behavior (Anthropic's [Mythos system card](https://www.anthropic.com/research/the-claude-model-spec)):


| Pattern                 | Detects                                          | Action |
| ----------------------- | ------------------------------------------------ | ------ |
| `credential_harvesting` | Reading `/proc` then environment data            | HALT   |
| `sandbox_probing`       | Process info → network config → external request | HALT   |
| `self_modification`     | Reading then writing MCP/tool config             | HALT   |
| `public_exfiltration`   | Workspace reads then unknown network requests    | PAUSE  |
| `history_tampering`     | Git operations then writing `.git` internals     | PAUSE  |


Patterns match over a sliding window of recent actions. A HALT trips the circuit breaker and blocks all further actions. A PAUSE prompts the human.

## Architecture

```
┌─────────┐     ┌─────────────────┐     ┌──────────┐
│  Agent   │────▶│  Blackwall GW   │────▶│  Tools   │
│ (Claude, │     │                 │     │ (MCP,    │
│  Cursor, │     │  ┌───────────┐  │     │  Shell,  │
│  GPT,    │     │  │  Policy   │  │     │  FS,     │
│  etc.)   │◀────│  │  Engine   │  │◀────│  Net)    │
│          │     │  └───────────┘  │     └──────────┘
└─────────┘     │  ┌───────────┐  │
                │  │  Pattern   │  │     ┌──────────┐
                │  │  Matcher   │  │────▶│  Audit   │
                │  └───────────┘  │     │  Trail   │
                │  ┌───────────┐  │     │ (JSONL)  │
                │  │  Risk      │  │     └──────────┘
                │  │  Scorer    │  │
                │  └───────────┘  │     ┌──────────┐
                │  ┌───────────┐  │     │  Human   │
                │  │  Circuit   │  │────▶│  Prompt  │
                │  │  Breaker   │  │     │  (⏸)     │
                │  └───────────┘  │     └──────────┘
                └─────────────────┘
```

Each action is evaluated in order:

1. **Circuit breakers** — is the session already halted? Rate limit exceeded?
2. **Permissions** — deny → confirm → allow → default deny
3. **Pattern matching** — sliding window anti-pattern detection over recent history
4. **Risk scoring** — cumulative score over rolling window, with pause/halt thresholds

All evaluation is deterministic. Sub-millisecond latency per decision.

### Interception Layers


| Layer           | Mechanism                                                  | What it catches                                   |
| --------------- | ---------------------------------------------------------- | ------------------------------------------------- |
| **Shell shims** | PATH-prepended wrapper scripts → Unix socket IPC → gateway | Agent shell commands (`git`, `curl`, `npm`, etc.) |
| **MCP proxy**   | Stdio man-in-the-middle on JSON-RPC `tools/call`           | MCP tool invocations                              |
| **Escalation**  | Inline terminal prompt on `pause` decisions                | Human-in-the-loop confirmation                    |


### Audit Trail

Every action evaluation is logged as append-only JSONL at `~/.blackwall/logs/<session>.jsonl`:

```json
{"ts":"2025-01-15T14:23:01Z","session":"7f3a28","seq":1,"tool":"shell","op":"exec","target":"git status","decision":"allow","latency_us":42}
{"ts":"2025-01-15T14:23:15Z","session":"7f3a28","seq":2,"tool":"shell","op":"exec","target":"sudo rm -rf /","decision":"deny","reason":"command 'sudo' matches deny rule 'sudo'","rule":"permissions.shell.deny","latency_us":18}
```

## Install

```bash
# From source (requires Rust toolchain)
cargo install --path crates/blackwall-cli

# Homebrew (macOS/Linux)
brew tap sambam60/blackwall && brew install blackwall
```

## Protocol

Blackwall implements an open protocol for AI agent containment. The full specification is at `[spec/PROTOCOL.md](spec/PROTOCOL.md)`. Implementations in other languages are encouraged.

## License

MIT