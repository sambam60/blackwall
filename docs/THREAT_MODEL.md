# Blackwall Threat Model

Blackwall's mission is to protect users from rogue or compromised AI agents by enforcing deterministic policy at tool-execution boundaries.

## What Blackwall Defends

Blackwall is designed for agents that can call tools, run commands, use MCP servers, or delegate work to subprocesses. It assumes the model may be mistaken, manipulated by prompt injection, or incentivized to complete a task by taking actions the user did not intend.

Blackwall defends against:

- Destructive shell commands such as privilege escalation, filesystem wipes, forced history rewrites, and dangerous database operations.
- Secret access through common file paths, MCP tool arguments, or process inspection targets.
- Tool self-modification, such as reading then editing MCP or agent configuration.
- Public exfiltration patterns, such as reading workspace data and then contacting an unapproved network destination.
- Silent side effects from MCP tools whose names or arguments imply writes, sends, deletes, deployments, shell execution, or code patching.

## What Blackwall Does Not Claim

Blackwall is not a kernel sandbox, hypervisor, container runtime, EDR, or malware detector.

The shell adapter is PATH-shim based. It catches protected commands launched by name from a Blackwall-wrapped environment. It does not catch:

- Absolute binary paths such as `/bin/rm` unless they are reached through a shimmed shell command that Blackwall can classify.
- Direct syscalls from an already-allowed process.
- Native filesystem edits made by an agent host outside Blackwall's adapters.
- Network requests made inside an already-allowed process unless that process exposes the request as a Blackwall action.

The MCP adapter governs JSON-RPC `tools/call` requests for stdio MCP servers wrapped with `blackwall proxy-mcp`. It does not govern remote HTTP MCP servers unless they are placed behind a Blackwall-aware proxy.

## Required Deployment Posture

Blackwall should be used as one layer in a defense-in-depth stack:

- Keep the agent's native sandbox enabled.
- Use the agent's native approval policy for operations outside its sandbox.
- Run the agent through `blackwall exec -- <agent>` when the agent launches subprocesses.
- Wrap stdio MCP servers with `blackwall proxy-mcp -- <server>`.
- Use `strict` for untrusted repositories or high-value environments.
- Treat `permissive` as audit-only, not protection.

## Design Standard

Every adapter should follow this rule: if Blackwall cannot reliably classify an action, it should either deny it or require confirmation, depending on policy. Unknown side effects are safer to stop than to silently permit.
