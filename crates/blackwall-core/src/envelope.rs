use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ActionEnvelope {
    pub id: String,
    pub timestamp: DateTime<Utc>,
    pub session_id: String,
    pub sequence: u64,
    pub action: Action,
}

impl ActionEnvelope {
    pub fn new(session_id: &str, sequence: u64, action: Action) -> Self {
        Self {
            id: uuid::Uuid::new_v4().to_string(),
            timestamp: Utc::now(),
            session_id: session_id.to_string(),
            sequence,
            action,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Action {
    pub tool: ToolCategory,
    pub operation: String,
    pub target: String,
    #[serde(default)]
    pub parameters: serde_json::Value,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
#[serde(rename_all = "lowercase")]
pub enum ToolCategory {
    Filesystem,
    Shell,
    Network,
    Process,
    Mcp,
}

impl std::fmt::Display for ToolCategory {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Filesystem => write!(f, "filesystem"),
            Self::Shell => write!(f, "shell"),
            Self::Network => write!(f, "network"),
            Self::Process => write!(f, "process"),
            Self::Mcp => write!(f, "mcp"),
        }
    }
}

#[derive(Debug, Clone)]
pub enum Decision {
    Allow,
    Deny { reason: String, rule: String },
    Pause { reason: String, context: EscalationContext },
    Log { reason: String },
}

impl Decision {
    pub fn is_allowed(&self) -> bool {
        matches!(self, Decision::Allow | Decision::Log { .. })
    }

    pub fn label(&self) -> &str {
        match self {
            Decision::Allow => "allow",
            Decision::Deny { .. } => "deny",
            Decision::Pause { .. } => "pause",
            Decision::Log { .. } => "log",
        }
    }

    pub fn reason(&self) -> Option<&str> {
        match self {
            Decision::Allow => None,
            Decision::Deny { reason, .. } => Some(reason),
            Decision::Pause { reason, .. } => Some(reason),
            Decision::Log { reason } => Some(reason),
        }
    }

    pub fn rule(&self) -> Option<&str> {
        match self {
            Decision::Deny { rule, .. } => Some(rule),
            _ => None,
        }
    }
}

#[derive(Debug, Clone)]
pub struct EscalationContext {
    pub what_happened: String,
    pub why_flagged: String,
    pub risk_score: u32,
}
