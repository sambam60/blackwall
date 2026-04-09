use blackwall_core::envelope::{Action, ActionEnvelope, Decision, ToolCategory};
use std::io::{self, BufRead, BufReader, Write};
use std::process::{Command, Stdio};
use std::sync::{mpsc, Arc, Mutex};

use crate::escalation;
use crate::gateway::BlackwallGateway;

pub struct McpProxy {
    gateway: Arc<Mutex<BlackwallGateway>>,
    session_id: String,
}

impl McpProxy {
    pub fn new(gateway: BlackwallGateway, session_id: String) -> Self {
        Self {
            gateway: Arc::new(Mutex::new(gateway)),
            session_id,
        }
    }

    /// Spawn the real MCP server and sit between the client's stdio and the server's.
    /// Returns the server's exit code.
    pub fn run(
        &self,
        server_cmd: &str,
        server_args: &[String],
        server_env: Vec<(String, String)>,
    ) -> io::Result<i32> {
        let mut child = Command::new(server_cmd)
            .args(server_args)
            .envs(server_env.iter().map(|(k, v)| (k.as_str(), v.as_str())))
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::inherit())
            .spawn()?;

        let child_stdin = child.stdin.take().unwrap();
        let child_stdout = child.stdout.take().unwrap();

        // All output to our stdout goes through this channel so the
        // forward thread and passback thread never race on stdout.
        let (out_tx, out_rx) = mpsc::channel::<String>();

        let writer_handle = std::thread::spawn(move || {
            let mut stdout = io::stdout().lock();
            for line in out_rx {
                let _ = writeln!(stdout, "{}", line);
                let _ = stdout.flush();
            }
        });

        let gw = Arc::clone(&self.gateway);
        let sid = self.session_id.clone();
        let deny_tx = out_tx.clone();
        let forward_handle = std::thread::spawn(move || {
            intercept_forward(io::stdin().lock(), child_stdin, gw, &sid, deny_tx);
        });

        let pass_tx = out_tx;
        let passback_handle = std::thread::spawn(move || {
            let reader = BufReader::new(child_stdout);
            for line in reader.lines().flatten() {
                if pass_tx.send(line).is_err() {
                    break;
                }
            }
        });

        forward_handle.join().ok();
        let status = child.wait()?;
        passback_handle.join().ok();
        drop(writer_handle);

        Ok(status.code().unwrap_or(1))
    }
}

// ── client → server direction with interception ────────────────────────

fn intercept_forward<R: io::Read, W: Write>(
    reader: R,
    mut server_stdin: W,
    gateway: Arc<Mutex<BlackwallGateway>>,
    session_id: &str,
    deny_tx: mpsc::Sender<String>,
) {
    let buf = BufReader::new(reader);

    for line in buf.lines() {
        let line = match line {
            Ok(l) => l,
            Err(_) => break,
        };

        if line.trim().is_empty() {
            let _ = writeln!(server_stdin);
            let _ = server_stdin.flush();
            continue;
        }

        let msg: serde_json::Value = match serde_json::from_str(&line) {
            Ok(v) => v,
            Err(_) => {
                let _ = writeln!(server_stdin, "{}", line);
                let _ = server_stdin.flush();
                continue;
            }
        };

        if !is_tool_call(&msg) {
            let _ = writeln!(server_stdin, "{}", line);
            let _ = server_stdin.flush();
            continue;
        }

        let request_id = msg.get("id").cloned();
        let (tool_name, arguments) = extract_tool_call(&msg);

        let action = Action {
            tool: ToolCategory::Mcp,
            operation: "tool_call".into(),
            target: tool_name.clone(),
            parameters: arguments,
        };

        let decision = {
            let mut gw = gateway.lock().unwrap();
            let seq = gw.action_count() + 1;
            let envelope = ActionEnvelope::new(session_id, seq, action);
            let raw = gw.process(&envelope);
            escalation::resolve_pause(raw, false)
        };

        match decision {
            Decision::Allow | Decision::Log { .. } => {
                let _ = writeln!(server_stdin, "{}", line);
                let _ = server_stdin.flush();
            }
            Decision::Deny { reason, .. } => {
                let err_resp = synthesize_mcp_error(request_id, &tool_name, &reason);
                let _ = deny_tx.send(err_resp);
            }
            Decision::Pause { .. } => unreachable!(),
        }
    }
}

// ── JSON-RPC helpers ───────────────────────────────────────────────────

fn is_tool_call(msg: &serde_json::Value) -> bool {
    msg.get("method")
        .and_then(|m| m.as_str())
        .map(|m| m == "tools/call")
        .unwrap_or(false)
}

fn extract_tool_call(msg: &serde_json::Value) -> (String, serde_json::Value) {
    let params = msg
        .get("params")
        .cloned()
        .unwrap_or(serde_json::Value::Null);
    let name = params
        .get("name")
        .and_then(|n| n.as_str())
        .unwrap_or("unknown")
        .to_string();
    let arguments = params
        .get("arguments")
        .cloned()
        .unwrap_or(serde_json::Value::Null);
    (name, arguments)
}

fn synthesize_mcp_error(
    request_id: Option<serde_json::Value>,
    tool_name: &str,
    reason: &str,
) -> String {
    let id = request_id.unwrap_or(serde_json::Value::Null);
    let response = serde_json::json!({
        "jsonrpc": "2.0",
        "id": id,
        "result": {
            "content": [{
                "type": "text",
                "text": format!("Blocked by Blackwall: tool '{}' denied — {}", tool_name, reason)
            }],
            "isError": true
        }
    });
    serde_json::to_string(&response).unwrap_or_default()
}
