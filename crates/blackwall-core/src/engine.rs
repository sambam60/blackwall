use crate::circuit::CircuitBreaker;
use crate::envelope::{Action, ActionEnvelope, Decision, EscalationContext, ToolCategory};
use crate::pattern::{simple_wildcard_match, PatternMatcher};
use crate::policy::Policy;
use crate::scoring::RiskScorer;
use crate::BlackwallError;
use globset::{Glob, GlobMatcher};
use std::path::PathBuf;

struct CompiledGlob {
    pattern: String,
    matcher: GlobMatcher,
}

struct FsRules {
    allow: Vec<CompiledGlob>,
    deny: Vec<CompiledGlob>,
    confirm: Vec<CompiledGlob>,
}

pub struct PolicyEngine {
    policy: Policy,
    _workspace: PathBuf,
    fs_read: FsRules,
    fs_write: FsRules,
    proc_deny: Vec<CompiledGlob>,
    pattern_matcher: PatternMatcher,
    risk_scorer: RiskScorer,
    circuit_breaker: CircuitBreaker,
    action_count: u64,
}

fn compile_globs(patterns: &[String]) -> Result<Vec<CompiledGlob>, BlackwallError> {
    patterns
        .iter()
        .map(|p| {
            let glob = Glob::new(p).map_err(|e| BlackwallError::InvalidGlob {
                pattern: p.clone(),
                source: e,
            })?;
            Ok(CompiledGlob {
                pattern: p.clone(),
                matcher: glob.compile_matcher(),
            })
        })
        .collect()
}

fn matches_any<'a>(rules: &'a [CompiledGlob], target: &str) -> Option<&'a str> {
    rules
        .iter()
        .find(|g| g.matcher.is_match(target))
        .map(|g| g.pattern.as_str())
}

impl PolicyEngine {
    pub fn new(policy: Policy, workspace: PathBuf) -> Result<Self, BlackwallError> {
        let fs_read = FsRules {
            allow: compile_globs(&policy.permissions.filesystem.read.allow)?,
            deny: compile_globs(&policy.permissions.filesystem.read.deny)?,
            confirm: compile_globs(&policy.permissions.filesystem.read.confirm)?,
        };
        let fs_write = FsRules {
            allow: compile_globs(&policy.permissions.filesystem.write.allow)?,
            deny: compile_globs(&policy.permissions.filesystem.write.deny)?,
            confirm: compile_globs(&policy.permissions.filesystem.write.confirm)?,
        };
        let proc_deny = compile_globs(&policy.permissions.process.deny)?;

        let pattern_matcher = PatternMatcher::new(&policy.patterns);
        let risk_scorer = RiskScorer::new(policy.scoring.clone(), &policy.labels);
        let circuit_breaker = CircuitBreaker::new(policy.circuit_breakers.clone());

        Ok(Self {
            policy,
            _workspace: workspace,
            fs_read,
            fs_write,
            proc_deny,
            pattern_matcher,
            risk_scorer,
            circuit_breaker,
            action_count: 0,
        })
    }

    pub fn evaluate(&mut self, envelope: &ActionEnvelope) -> Decision {
        self.action_count += 1;

        if let Some(d) = self.circuit_breaker.check() {
            return d;
        }

        self.circuit_breaker.record_action();

        let permission_decision = self.check_permissions(&envelope.action);
        if let Decision::Deny { .. } = &permission_decision {
            self.circuit_breaker.record_denial();
            return permission_decision;
        }

        if let Some(pm) = self
            .pattern_matcher
            .record_and_check(&envelope.action, self.action_count)
        {
            if matches!(pm, Decision::Deny { .. }) {
                self.circuit_breaker.trip();
            }
            return pm;
        }

        let score = self.risk_scorer.score_action(&envelope.action);
        if let Some(d) = self.risk_scorer.check_thresholds(score) {
            return d;
        }

        permission_decision
    }

    pub fn action_count(&self) -> u64 {
        self.action_count
    }

    pub fn risk_score(&self) -> u32 {
        self.risk_scorer.cumulative_score()
    }

    fn check_permissions(&self, action: &Action) -> Decision {
        match action.tool {
            ToolCategory::Filesystem => self.check_filesystem(action),
            ToolCategory::Shell => self.check_shell(action),
            ToolCategory::Network => self.check_network(action),
            ToolCategory::Process => self.check_process(action),
            ToolCategory::Mcp => self.check_mcp(action),
        }
    }

    fn check_filesystem(&self, action: &Action) -> Decision {
        let rules = if action.operation == "read" {
            &self.fs_read
        } else {
            &self.fs_write
        };

        if let Some(pattern) = matches_any(&rules.deny, &action.target) {
            return Decision::Deny {
                reason: format!("'{}' matches deny rule '{}'", action.target, pattern),
                rule: format!("permissions.filesystem.{}.deny", action.operation),
            };
        }

        if let Some(pattern) = matches_any(&rules.confirm, &action.target) {
            return Decision::Pause {
                reason: format!("'{}' requires confirmation", action.target),
                context: EscalationContext {
                    what_happened: format!(
                        "{}.{} {}",
                        action.tool, action.operation, action.target
                    ),
                    why_flagged: format!("matches confirmation rule '{}'", pattern),
                    risk_score: 0,
                },
            };
        }

        if matches_any(&rules.allow, &action.target).is_some() {
            return Decision::Allow;
        }

        Decision::Deny {
            reason: format!("no allow rule matches '{}'", action.target),
            rule: format!("permissions.filesystem.{}.default_deny", action.operation),
        }
    }

    fn check_shell(&self, action: &Action) -> Decision {
        let command = &action.target;
        let program = command.split_whitespace().next().unwrap_or("");

        for denied in &self.policy.permissions.shell.deny {
            if command_rule_matches(denied, program, command) {
                return Decision::Deny {
                    reason: format!("command '{}' matches deny rule '{}'", program, denied),
                    rule: "permissions.shell.deny".into(),
                };
            }
        }

        for pattern in &self.policy.permissions.shell.deny_patterns {
            let parts: Vec<&str> = pattern.split('*').collect();
            if parts.len() > 1 {
                let mut pos = 0;
                let mut matched = true;
                for (i, seg) in parts.iter().enumerate() {
                    if seg.is_empty() {
                        continue;
                    }
                    match command[pos..].find(seg) {
                        Some(found) => {
                            if i == 0 && found != 0 {
                                matched = false;
                                break;
                            }
                            pos += found + seg.len();
                        }
                        None => {
                            matched = false;
                            break;
                        }
                    }
                }
                if matched {
                    return Decision::Deny {
                        reason: format!("command matches deny pattern '{}'", pattern),
                        rule: "permissions.shell.deny_patterns".into(),
                    };
                }
            }
        }

        for confirm_cmd in &self.policy.permissions.shell.confirm {
            if command_rule_matches(confirm_cmd, program, command) {
                return Decision::Pause {
                    reason: format!("'{}' requires confirmation", command),
                    context: EscalationContext {
                        what_happened: format!("shell.exec {}", command),
                        why_flagged: format!("matches confirmation rule '{}'", confirm_cmd),
                        risk_score: 0,
                    },
                };
            }
        }

        for allowed in &self.policy.permissions.shell.allow {
            if program == allowed || simple_wildcard_match(allowed, program) {
                return Decision::Allow;
            }
        }

        Decision::Deny {
            reason: format!("program '{}' not in allow list", program),
            rule: "permissions.shell.default_deny".into(),
        }
    }

    fn check_network(&self, action: &Action) -> Decision {
        let domain = &action.target;

        for allowed in &self.policy.permissions.network.allow {
            if domain == allowed || domain.ends_with(&format!(".{}", allowed)) {
                return Decision::Allow;
            }
        }

        for denied in &self.policy.permissions.network.deny {
            if denied == "*" || domain == denied {
                return Decision::Deny {
                    reason: format!("domain '{}' not in network allow list", domain),
                    rule: "permissions.network.default_deny".into(),
                };
            }
        }

        if self.policy.permissions.network.confirm_new {
            return Decision::Pause {
                reason: format!("first request to unknown domain '{}'", domain),
                context: EscalationContext {
                    what_happened: format!("network.request {}", domain),
                    why_flagged: "domain not in allow list, confirm_new enabled".into(),
                    risk_score: 0,
                },
            };
        }

        Decision::Allow
    }

    fn check_process(&self, action: &Action) -> Decision {
        if self.policy.permissions.process.deny_escalation && action.operation == "escalate" {
            return Decision::Deny {
                reason: "privilege escalation denied".into(),
                rule: "permissions.process.deny_escalation".into(),
            };
        }

        if let Some(pattern) = matches_any(&self.proc_deny, &action.target) {
            return Decision::Deny {
                reason: format!(
                    "'{}' matches process deny rule '{}'",
                    action.target, pattern
                ),
                rule: "permissions.process.deny".into(),
            };
        }

        Decision::Allow
    }

    fn check_mcp(&self, action: &Action) -> Decision {
        let args = serde_json::to_string(&action.parameters).unwrap_or_default();
        let combined = if args.is_empty() || args == "null" {
            action.target.clone()
        } else {
            format!("{} {}", action.target, args)
        };

        if json_has_sensitive_key(&action.parameters) {
            return Decision::Deny {
                reason: format!(
                    "MCP tool '{}' includes sensitive argument keys",
                    action.target
                ),
                rule: "permissions.mcp.sensitive_arguments".into(),
            };
        }

        for denied in &self.policy.permissions.mcp.deny {
            if mcp_rule_matches(denied, &action.target, &args, &combined) {
                return Decision::Deny {
                    reason: format!(
                        "MCP tool '{}' matches deny rule '{}'",
                        action.target, denied
                    ),
                    rule: "permissions.mcp.deny".into(),
                };
            }
        }

        for confirm in &self.policy.permissions.mcp.confirm {
            if mcp_rule_matches(confirm, &action.target, &args, &combined) {
                return Decision::Pause {
                    reason: format!("MCP tool '{}' requires confirmation", action.target),
                    context: EscalationContext {
                        what_happened: format!("mcp.tool_call {}", action.target),
                        why_flagged: format!("matches confirmation rule '{}'", confirm),
                        risk_score: 0,
                    },
                };
            }
        }

        for allowed in &self.policy.permissions.mcp.allow {
            if mcp_rule_matches(allowed, &action.target, &args, &combined) {
                return Decision::Allow;
            }
        }

        match self.policy.permissions.mcp.default.as_str() {
            "allow" => Decision::Allow,
            "log" => Decision::Log {
                reason: "MCP default log".into(),
            },
            "deny" => Decision::Deny {
                reason: format!("MCP tool '{}' not in allow list", action.target),
                rule: "permissions.mcp.default_deny".into(),
            },
            _ => Decision::Pause {
                reason: format!("MCP tool '{}' is not explicitly allowed", action.target),
                context: EscalationContext {
                    what_happened: format!("mcp.tool_call {}", action.target),
                    why_flagged: "MCP default requires confirmation".into(),
                    risk_score: 0,
                },
            },
        }
    }
}

fn command_rule_matches(rule: &str, program: &str, command: &str) -> bool {
    rule == "*"
        || program == rule
        || simple_wildcard_match(rule, program)
        || command.contains(rule)
        || simple_wildcard_match(rule, command)
}

fn mcp_rule_matches(rule: &str, tool_name: &str, args: &str, combined: &str) -> bool {
    rule == "*"
        || tool_name == rule
        || simple_wildcard_match(rule, tool_name)
        || args.contains(rule)
        || simple_wildcard_match(rule, args)
        || combined.contains(rule)
        || simple_wildcard_match(rule, combined)
}

fn json_has_sensitive_key(value: &serde_json::Value) -> bool {
    match value {
        serde_json::Value::Object(map) => map.iter().any(|(key, value)| {
            let lower = key.to_ascii_lowercase();
            is_sensitive_key(&lower) || json_has_sensitive_key(value)
        }),
        serde_json::Value::Array(items) => items.iter().any(json_has_sensitive_key),
        _ => false,
    }
}

fn is_sensitive_key(key: &str) -> bool {
    key.contains("password")
        || key.contains("passwd")
        || key.contains("secret")
        || key.contains("credential")
        || key.contains("api_key")
        || key.contains("apikey")
        || key.contains("token")
        || key.contains("private_key")
}
