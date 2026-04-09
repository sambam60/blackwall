# Blackwall Protocol Specification v0.1

**A deterministic execution firewall for AI agents.**

This document defines the Blackwall Protocol — an open standard for constraining, monitoring, and sandboxing AI agents when they are given tool access.

## 1. Overview

The protocol operates at the **action boundary** — the moment between an agent's intent and its effect on the world. It defines how a gateway intercepts tool invocations, evaluates them against a policy, and decides: allow, deny, escalate to a human, or allow-but-log.

The gateway is **deterministic, not AI**. It cannot be prompt-injected. It cannot be socially engineered. Its behavior is fully auditable and reproducible.

### Design Principles

- **Default-deny for dangerous operations.** Agents operate freely within their workspace but must be explicitly granted access to anything outside it.
- **Deterministic evaluation.** Policy checks are rule-based, fast, and reproducible. No inference in the enforcement path.
- **Transport-agnostic.** The protocol defines the semantics. MCP, function calling, and shell execution are adapter implementations.
- **Temporal pattern detection.** Danger emerges from sequences of individually-innocuous actions. Policies match over sliding windows, not single actions.
- **Human-in-the-loop escalation.** When the gateway isn't confident, it doesn't just block — it creates a structured handoff with context.

## 2. Action Envelope

Every tool invocation is wrapped in an action envelope before reaching the gateway.

### Schema

```json
{
  "id": "string (UUID v4)",
  "timestamp": "string (ISO 8601)",
  "session_id": "string",
  "sequence": "integer (monotonic)",
  "action": {
    "tool": "filesystem | shell | network | process | mcp",
    "operation": "string (read, write, exec, request, etc.)",
    "target": "string (path, command, domain, etc.)",
    "parameters": "object (optional, tool-specific)"
  }
}
```

### Tool Categories


| Category     | Operations                | Target                   |
| ------------ | ------------------------- | ------------------------ |
| `filesystem` | `read`, `write`, `delete` | File path                |
| `shell`      | `exec`                    | Full command string      |
| `network`    | `request`                 | Domain or URL            |
| `process`    | `read`, `escalate`        | Process path or resource |
| `mcp`        | `call`                    | Tool name                |


## 3. Policy Schema

Policies are expressed in YAML and define what an agent can and cannot do.

### Structure

```yaml
version: "blackwall/policy-1.0"
name: "string"
description: "string"

labels:
  label_name:
    - "glob pattern"

permissions:
  filesystem:
    read: { allow: [], deny: [], confirm: [] }
    write: { allow: [], deny: [], confirm: [] }
  shell:
    allow: []
    deny: []
    confirm: []
    deny_patterns: []
  network:
    allow: []
    deny: []
    confirm_new: boolean
  process:
    deny: []
    deny_escalation: boolean

patterns:
  - name: "string"
    description: "string"
    sequence:
      - action: "tool.operation"
        path: "glob (optional)"
        pattern: "wildcard (optional)"
        domain: "string (optional)"
    within: integer (action window size)
    risk: integer (0-100)
    on_match: halt | pause | degrade | isolate

scoring:
  window: integer
  thresholds:
    pause: integer
    halt: integer
  weights:
    "category.label": integer

circuit_breakers:
  max_denials_before_halt: integer
  max_actions_per_minute: integer
  max_actions_per_session: integer

escalation:
  method: "inline | webhook | slack"
  timeout_seconds: integer
  default_on_timeout: "allow | deny"
```

### Evaluation Order

For each action, the gateway evaluates in this order:

1. **Circuit breakers** — is the session already halted or rate-limited?
2. **Permissions** — deny rules checked first, then confirm rules, then allow rules. Default deny.
3. **Patterns** — does this action complete a known dangerous sequence within the sliding window?
4. **Risk scoring** — does the cumulative risk score exceed a threshold?

### Variable Resolution

The `${WORKSPACE}` variable in policy files is resolved to the actual workspace path at load time.

### Resource Labels

Labels classify targets by sensitivity:

```yaml
labels:
  sensitive: ["/proc/**", "/sys/**"]
  secret: ["**/.env", "**/*.key"]
  normal: ["${WORKSPACE}/**"]
```

Scoring weights can reference labels: `"filesystem.read.secret": 50` assigns a risk score of 50 when a file matching the `secret` label is read.

## 4. Decisions

The gateway produces one of four decisions:


| Decision | Effect                                                       |
| -------- | ------------------------------------------------------------ |
| `allow`  | Action proceeds.                                             |
| `deny`   | Action is blocked. Agent receives an error.                  |
| `pause`  | Action is held. Human is presented with context and options. |
| `log`    | Action proceeds but is flagged in the audit trail.           |


## 5. Circuit Breaker Signals

When a circuit breaker trips, the gateway emits one of:


| Signal    | Meaning                                                    |
| --------- | ---------------------------------------------------------- |
| `HALT`    | Immediate stop. No further actions. Human review required. |
| `PAUSE`   | Pause execution. Present context to human for decision.    |
| `DEGRADE` | Revoke specific permissions, continue with reduced scope.  |
| `ISOLATE` | Cut network access, restrict to read-only.                 |


Triggers:

- Denial count exceeds threshold
- Action rate exceeds window limit
- Anti-pattern match with severity >= critical
- Cumulative risk score exceeds threshold

## 6. Audit Trail Format

Append-only JSONL. One entry per action evaluation.

```json
{
  "ts": "2026-04-08T14:23:01Z",
  "session": "7f3a28",
  "seq": 1,
  "tool": "filesystem",
  "op": "read",
  "target": "src/main.rs",
  "decision": "allow",
  "rule": "permissions.filesystem.read.allow",
  "latency_us": 42
}
```

Fields `reason`, `rule`, `pattern`, and `risk_score` are included when applicable.

## 7. Escalation Handoff

When the gateway produces a `pause` decision, it creates a structured handoff:

- **what_happened**: what action the agent attempted
- **why_flagged**: which policy rule or pattern triggered the pause
- **risk_score**: cumulative risk at the time of the pause

The human responds with one of: allow, deny, or terminate.

## 8. Temporal Patterns

Patterns define sequences of actions that, when observed within a sliding window, indicate dangerous behavior. Patterns are matched using a simple forward scan over the action history.

### Canonical Anti-Patterns

Based on documented frontier model behavior:

1. **credential_harvesting** — reading /proc then environment data
2. **sandbox_probing** — reading process info, network config, then making external requests
3. **self_modification** — reading then writing to MCP or tool configuration
4. **public_exfiltration** — reading workspace files then sending to external services
5. **history_tampering** — executing git commands then writing to .git internals

## 9. Adapter Requirements

An adapter integrates the Blackwall gateway with a specific agent framework. Adapters must:

1. Intercept tool invocations before execution
2. Construct an `ActionEnvelope` from the invocation
3. Submit the envelope to the gateway for evaluation
4. If `allow` or `log`: forward the invocation to the tool
5. If `deny`: return an error to the agent
6. If `pause`: present the escalation to the human, await response

### Planned Adapters

- **MCP**: proxy between agent and MCP servers
- **Shell**: shim around command execution
- **OpenAI function calling**: middleware layer
- **LangChain/LangGraph**: chain interceptors

## 10. Principal Hierarchy

When multiple policies apply, the more restrictive rule wins at each level:

1. **Platform** (Anthropic, OpenAI, etc.) — highest authority
2. **Operator** (enterprise deploying the agent)
3. **User** (end user)

Deny at any level is final. Allow must be granted at all levels.

---

*Blackwall Protocol is an open standard. Implementations are encouraged.*