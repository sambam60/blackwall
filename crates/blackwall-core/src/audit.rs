use crate::envelope::{ActionEnvelope, Decision};
use serde::Serialize;
use std::fs::{File, OpenOptions};
use std::io::{self, BufWriter, Write};
use std::path::Path;

#[derive(Serialize)]
pub struct AuditEntry {
    pub ts: String,
    pub session: String,
    pub seq: u64,
    pub tool: String,
    pub op: String,
    pub target: String,
    pub decision: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub reason: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub rule: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub pattern: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub risk_score: Option<u32>,
    pub latency_us: u64,
}

impl AuditEntry {
    pub fn from_evaluation(
        envelope: &ActionEnvelope,
        decision: &Decision,
        latency_us: u64,
    ) -> Self {
        Self {
            ts: envelope.timestamp.to_rfc3339(),
            session: envelope.session_id.clone(),
            seq: envelope.sequence,
            tool: envelope.action.tool.to_string(),
            op: envelope.action.operation.clone(),
            target: envelope.action.target.clone(),
            decision: decision.label().to_string(),
            reason: decision.reason().map(String::from),
            rule: decision.rule().map(String::from),
            pattern: None,
            risk_score: None,
            latency_us,
        }
    }

    pub fn display_symbol(&self) -> &str {
        match self.decision.as_str() {
            "allow" => "\x1b[32m✓\x1b[0m",
            "deny" => "\x1b[31m✗\x1b[0m",
            "pause" => "\x1b[33m⏸\x1b[0m",
            "log" => "\x1b[2m·\x1b[0m",
            _ => "?",
        }
    }

    pub fn display_line(&self) -> String {
        let ts = &self.ts[11..19]; // HH:MM:SS
        let extra = match self.decision.as_str() {
            "deny" => self
                .rule
                .as_deref()
                .map(|r| format!("  \x1b[31m[{}]\x1b[0m", r))
                .unwrap_or_default(),
            "pause" => self
                .reason
                .as_deref()
                .map(|r| format!("  \x1b[33m[{}]\x1b[0m", r))
                .unwrap_or_default(),
            _ => String::new(),
        };
        format!(
            "{} {} {:<18} {}{}",
            ts,
            self.display_symbol(),
            format!("{}.{}", self.tool, self.op),
            self.target,
            extra
        )
    }
}

pub struct AuditLog {
    writer: BufWriter<File>,
    session_id: String,
}

impl AuditLog {
    pub fn new(log_dir: &Path, session_id: &str) -> io::Result<Self> {
        std::fs::create_dir_all(log_dir)?;
        let path = log_dir.join(format!("{}.jsonl", session_id));
        let file = OpenOptions::new()
            .create(true)
            .append(true)
            .open(path)?;
        Ok(Self {
            writer: BufWriter::new(file),
            session_id: session_id.to_string(),
        })
    }

    pub fn log(&mut self, entry: &AuditEntry) -> io::Result<()> {
        let json = serde_json::to_string(entry).map_err(|e| io::Error::other(e.to_string()))?;
        writeln!(self.writer, "{}", json)?;
        self.writer.flush()
    }

    pub fn session_id(&self) -> &str {
        &self.session_id
    }
}
