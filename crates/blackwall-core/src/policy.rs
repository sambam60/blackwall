use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::Path;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Policy {
    pub version: String,
    pub name: String,
    pub description: String,
    #[serde(default)]
    pub labels: HashMap<String, Vec<String>>,
    pub permissions: Permissions,
    #[serde(default)]
    pub patterns: Vec<Pattern>,
    #[serde(default)]
    pub scoring: ScoringConfig,
    #[serde(default)]
    pub circuit_breakers: CircuitBreakerConfig,
    #[serde(default)]
    pub escalation: EscalationConfig,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Permissions {
    #[serde(default)]
    pub filesystem: FilesystemPermissions,
    #[serde(default)]
    pub shell: ShellPermissions,
    #[serde(default)]
    pub network: NetworkPermissions,
    #[serde(default)]
    pub process: ProcessPermissions,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct FilesystemPermissions {
    #[serde(default)]
    pub read: PermissionRule,
    #[serde(default)]
    pub write: PermissionRule,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct PermissionRule {
    #[serde(default)]
    pub allow: Vec<String>,
    #[serde(default)]
    pub deny: Vec<String>,
    #[serde(default)]
    pub confirm: Vec<String>,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ShellPermissions {
    #[serde(default)]
    pub allow: Vec<String>,
    #[serde(default)]
    pub deny: Vec<String>,
    #[serde(default)]
    pub confirm: Vec<String>,
    #[serde(default)]
    pub deny_patterns: Vec<String>,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct NetworkPermissions {
    #[serde(default)]
    pub allow: Vec<String>,
    #[serde(default)]
    pub deny: Vec<String>,
    #[serde(default)]
    pub confirm_new: bool,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ProcessPermissions {
    #[serde(default)]
    pub deny: Vec<String>,
    #[serde(default)]
    pub deny_escalation: bool,
}

// --- Patterns (temporal anti-pattern detection) ---

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Pattern {
    pub name: String,
    #[serde(default)]
    pub description: String,
    pub sequence: Vec<PatternStep>,
    #[serde(default = "default_within")]
    pub within: usize,
    #[serde(default)]
    pub risk: u32,
    #[serde(default)]
    pub on_match: CircuitAction,
}

fn default_within() -> usize {
    10
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PatternStep {
    pub action: String,
    #[serde(default)]
    pub path: Option<String>,
    #[serde(default)]
    pub pattern: Option<String>,
    #[serde(default)]
    pub domain: Option<String>,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum CircuitAction {
    #[default]
    Pause,
    Halt,
    Degrade,
    Isolate,
}

// --- Scoring ---

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScoringConfig {
    #[serde(default = "default_window")]
    pub window: usize,
    #[serde(default)]
    pub thresholds: ScoringThresholds,
    #[serde(default)]
    pub weights: HashMap<String, u32>,
}

fn default_window() -> usize {
    50
}

impl Default for ScoringConfig {
    fn default() -> Self {
        Self {
            window: 50,
            thresholds: ScoringThresholds::default(),
            weights: HashMap::new(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScoringThresholds {
    #[serde(default = "default_pause_threshold")]
    pub pause: u32,
    #[serde(default = "default_halt_threshold")]
    pub halt: u32,
}

fn default_pause_threshold() -> u32 {
    150
}
fn default_halt_threshold() -> u32 {
    250
}

impl Default for ScoringThresholds {
    fn default() -> Self {
        Self {
            pause: 150,
            halt: 250,
        }
    }
}

// --- Circuit breakers ---

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CircuitBreakerConfig {
    #[serde(default = "default_max_denials")]
    pub max_denials_before_halt: u32,
    #[serde(default = "default_max_per_minute")]
    pub max_actions_per_minute: u32,
    #[serde(default = "default_max_per_session")]
    pub max_actions_per_session: u64,
}

fn default_max_denials() -> u32 {
    5
}
fn default_max_per_minute() -> u32 {
    60
}
fn default_max_per_session() -> u64 {
    5000
}

impl Default for CircuitBreakerConfig {
    fn default() -> Self {
        Self {
            max_denials_before_halt: 5,
            max_actions_per_minute: 60,
            max_actions_per_session: 5000,
        }
    }
}

// --- Escalation ---

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EscalationConfig {
    #[serde(default = "default_method")]
    pub method: String,
    #[serde(default = "default_timeout")]
    pub timeout_seconds: u64,
    #[serde(default = "default_on_timeout")]
    pub default_on_timeout: String,
}

fn default_method() -> String {
    "inline".into()
}
fn default_timeout() -> u64 {
    300
}
fn default_on_timeout() -> String {
    "deny".into()
}

impl Default for EscalationConfig {
    fn default() -> Self {
        Self {
            method: "inline".into(),
            timeout_seconds: 300,
            default_on_timeout: "deny".into(),
        }
    }
}

// --- Loading and resolution ---

impl Policy {
    pub fn default_policy(workspace: &Path) -> Self {
        let yaml = include_str!("../../../policies/default.yaml");
        let mut policy: Policy =
            serde_yaml::from_str(yaml).expect("embedded default policy is invalid");
        policy.resolve_workspace(workspace);
        policy
    }

    pub fn strict_policy(workspace: &Path) -> Self {
        let yaml = include_str!("../../../policies/strict.yaml");
        let mut policy: Policy =
            serde_yaml::from_str(yaml).expect("embedded strict policy is invalid");
        policy.resolve_workspace(workspace);
        policy
    }

    pub fn permissive_policy(workspace: &Path) -> Self {
        let yaml = include_str!("../../../policies/permissive.yaml");
        let mut policy: Policy =
            serde_yaml::from_str(yaml).expect("embedded permissive policy is invalid");
        policy.resolve_workspace(workspace);
        policy
    }

    pub fn load(path: &Path) -> Result<Self, crate::BlackwallError> {
        let content = std::fs::read_to_string(path)?;
        let policy: Policy = serde_yaml::from_str(&content)?;
        Ok(policy)
    }

    pub fn resolve_workspace(&mut self, workspace: &Path) {
        let ws = workspace.to_string_lossy();
        let replace = |s: &mut String| {
            *s = s.replace("${WORKSPACE}", &ws);
        };

        for s in &mut self.permissions.filesystem.read.allow {
            replace(s);
        }
        for s in &mut self.permissions.filesystem.read.deny {
            replace(s);
        }
        for s in &mut self.permissions.filesystem.read.confirm {
            replace(s);
        }
        for s in &mut self.permissions.filesystem.write.allow {
            replace(s);
        }
        for s in &mut self.permissions.filesystem.write.deny {
            replace(s);
        }
        for s in &mut self.permissions.filesystem.write.confirm {
            replace(s);
        }
        for pattern in &mut self.patterns {
            for step in &mut pattern.sequence {
                if let Some(ref mut p) = step.path {
                    replace(p);
                }
            }
        }
        for globs in self.labels.values_mut() {
            for s in globs {
                replace(s);
            }
        }
    }
}
