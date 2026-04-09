use std::path::Path;

#[derive(Debug)]
pub struct Environment {
    pub has_cursor: bool,
    pub has_mcp: bool,
    pub has_git: bool,
    pub mcp_server_count: usize,
    pub agent_hint: Option<String>,
}

impl Environment {
    pub fn detect(workspace: &Path) -> Self {
        let has_cursor = workspace.join(".cursor").is_dir();
        let has_git = workspace.join(".git").is_dir();

        let (has_mcp, mcp_server_count, agent_hint) = detect_mcp(workspace);

        Self {
            has_cursor,
            has_mcp,
            has_git,
            mcp_server_count,
            agent_hint,
        }
    }

    pub fn summary(&self) -> String {
        let mut parts = Vec::new();

        if self.has_mcp {
            parts.push(format!("mcp ({} servers)", self.mcp_server_count));
        }
        if self.has_cursor {
            parts.push("cursor".into());
        }
        if self.has_git {
            parts.push("git".into());
        }

        if parts.is_empty() {
            "detected: none (standalone mode)".into()
        } else {
            format!("detected: {}", parts.join(", "))
        }
    }
}

fn detect_mcp(workspace: &Path) -> (bool, usize, Option<String>) {
    let mcp_paths = [
        workspace.join(".cursor/mcp.json"),
        workspace.join(".mcp/config.json"),
        workspace.join("mcp.json"),
    ];

    for path in &mcp_paths {
        if path.is_file() {
            if let Ok(content) = std::fs::read_to_string(path) {
                let count = content.matches("\"command\"").count()
                    + content.matches("\"url\"").count();
                let count = count.max(1);
                return (true, count, Some("mcp".into()));
            }
        }
    }

    (false, 0, None)
}
