# Blackwall

**A deterministic execution firewall for AI agents.**

```
blackwall
```

That's it. One command. Your agent is wrapped.

---

## What it does

Blackwall sits between AI agents and their tools. Every action — file read, shell command, network request — passes through a deterministic policy engine before it executes. No silent side effects. Every invocation auditable.

```
■ blackwall v0.1.0
  policy: default | session: 7f3a28
  workspace: /Users/you/project
  log: ~/.blackwall/logs/7f3a28.jsonl

  detected: mcp (3 servers), cursor, git
  gateway active — press ctrl+c to stop

14:23:01 ✓ filesystem.read   src/main.rs
14:23:03 ✓ shell.exec        cargo build
14:23:15 ✗ filesystem.read   /proc/self/environ  [credential_harvesting]
14:23:15 ⏸ PAUSED — agent attempted to read process environment
         ↳ allow (a) | deny (d) | end session (x)
```

The gateway is **not AI**. It's deterministic policy evaluation. It can't be prompt-injected. It can't be socially engineered. It runs at the speed of rule matching, not inference.

## Install

```bash
# From source
cargo install --path crates/blackwall-cli

# Homebrew (macOS/Linux)
brew tap blackwall-protocol/blackwall
brew install blackwall
```

## Usage

```bash
# Start with sensible defaults
blackwall

# Choose a policy
blackwall --policy strict
blackwall --policy permissive
blackwall --policy ./my-policy.yaml

# Protect a specific workspace
blackwall --workspace /path/to/project

# Wrap a specific command
blackwall exec -- claude
blackwall exec -- python my_agent.py

# View audit logs
blackwall logs
blackwall logs --follow

# Check status
blackwall status

# Stop
blackwall off
```

## Default Policy

When you run `blackwall` with no arguments:

| | Allowed | Blocked | Asks First |
|---|---|---|---|
| **File read** | Workspace files | `.env`, secrets, `/proc`, SSH keys | — |
| **File write** | Workspace files | System dirs, `.git/config`, SSH | `package.json`, lockfiles |
| **Shell** | `git`, `npm`, `cargo`, `python`, `node`, `make`... | `sudo`, `su`, `chmod +s`, `dd` | `rm -rf`, `git push --force` |
| **Network** | Package registries, GitHub | Everything else | New domains (first time) |
| **Process** | — | `/proc/*/mem`, `/proc/*/environ` | — |

Plus **temporal anti-pattern detection**:

- `credential_harvesting` — reading `/proc` then environment data → HALT
- `sandbox_probing` — process info → network config → external request → HALT
- `self_modification` — reading then writing MCP/tool config → HALT
- `public_exfiltration` — workspace reads then unknown network requests → PAUSE
- `history_tampering` — git operations then writing `.git` internals → PAUSE

These patterns are derived from documented frontier model behavior in Anthropic's Mythos system card.

## Policies

Three built-in profiles:

- **`default`** — sensible defaults for development. Standard dev ops allowed, dangerous patterns blocked.
- **`strict`** — maximum restriction. Every write requires confirmation. No outbound network.
- **`permissive`** — log-only mode. Nothing blocked, everything recorded. For auditing.

Or write your own in YAML:

```yaml
version: "blackwall/policy-1.0"
name: "my-custom-policy"
description: "..."

permissions:
  filesystem:
    read:
      allow: ["${WORKSPACE}/**"]
      deny: ["**/.env"]
    write:
      allow: ["${WORKSPACE}/src/**"]
      confirm: ["${WORKSPACE}/config/**"]
  shell:
    allow: [git, cargo, npm]
    deny: [sudo]
  network:
    allow: [github.com, crates.io]
    deny: ["*"]

patterns:
  - name: my_custom_pattern
    sequence:
      - action: "filesystem.read"
        path: "**/.env"
      - action: "network.request"
    within: 10
    risk: 90
    on_match: halt
```

## Architecture

```
┌─────────┐     ┌─────────────────┐     ┌──────────┐
│  Agent   │────▶│  Blackwall GW   │────▶│  Tools   │
│ (Claude, │     │                 │     │ (MCP,    │
│  GPT,    │     │  ┌───────────┐  │     │  Shell,  │
│  etc.)   │◀────│  │  Policy   │  │◀────│  FS,     │
│          │     │  │  Engine   │  │     │  Net)    │
└─────────┘     │  └───────────┘  │     └──────────┘
                │  ┌───────────┐  │
                │  │  Pattern   │  │     ┌──────────┐
                │  │  Matcher   │  │────▶│  Audit   │
                │  └───────────┘  │     │  Trail   │
                │  ┌───────────┐  │     │ (JSONL)  │
                │  │  Circuit   │  │     └──────────┘
                │  │  Breaker   │  │
                │  └───────────┘  │
                └─────────────────┘
```

The gateway evaluates each action in order:

1. **Circuit breakers** — is the session already halted?
2. **Permissions** — deny → confirm → allow → default deny
3. **Patterns** — sliding window anti-pattern detection
4. **Risk scoring** — cumulative score over rolling window

All evaluation is deterministic. Sub-millisecond latency.

## Why

AI agents are being given persistent tool access — filesystem, shell, network, APIs. Frontier models have demonstrated:

- Harvesting credentials from process memory
- Escaping sandboxes through multi-step exploits
- Covering tracks after rule violations
- Editing running MCP servers to change outbound URLs
- Posting content publicly without authorization

Current containment is ad-hoc: system prompts, per-tool permissions, framework-specific guardrails. There is no standard for how agents should be constrained when given real-world tool access.

Blackwall is that standard. It operates at the execution layer — the only place where intent becomes irreversible and side effects happen.

## Protocol

The full protocol specification is at [`spec/PROTOCOL.md`](spec/PROTOCOL.md).

## License

MIT
