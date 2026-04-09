use blackwall_core::envelope::{Action, ActionEnvelope, Decision};
use std::io::{self, BufRead, BufReader, Write};
use std::os::unix::net::{UnixListener, UnixStream};
use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex};

use crate::escalation;
use crate::gateway::BlackwallGateway;

pub struct IpcServer {
    listener: UnixListener,
    socket_path: PathBuf,
}

impl IpcServer {
    pub fn bind(session_id: &str) -> io::Result<Self> {
        let run_dir = dirs::home_dir()
            .expect("cannot find home directory")
            .join(".blackwall/run");
        std::fs::create_dir_all(&run_dir)?;

        let socket_path = run_dir.join(format!("{}.sock", session_id));
        let _ = std::fs::remove_file(&socket_path);
        let listener = UnixListener::bind(&socket_path)?;

        Ok(Self {
            listener,
            socket_path,
        })
    }

    pub fn socket_path(&self) -> &Path {
        &self.socket_path
    }

    /// Accept connections forever. Each connection is handled in its own thread.
    /// Blocks the calling thread.
    pub fn run(self, gateway: Arc<Mutex<BlackwallGateway>>, session_id: String) {
        for stream in self.listener.incoming() {
            match stream {
                Ok(stream) => {
                    let gw = Arc::clone(&gateway);
                    let sid = session_id.clone();
                    std::thread::spawn(move || handle_connection(stream, gw, &sid));
                }
                Err(_) => break,
            }
        }
    }
}

fn handle_connection(
    stream: UnixStream,
    gateway: Arc<Mutex<BlackwallGateway>>,
    session_id: &str,
) {
    let reader = BufReader::new(&stream);
    let mut writer = &stream;

    for line in reader.lines() {
        let line = match line {
            Ok(l) if !l.trim().is_empty() => l,
            Ok(_) => continue,
            Err(_) => break,
        };

        let msg: serde_json::Value = match serde_json::from_str(&line) {
            Ok(v) => v,
            Err(e) => {
                let resp = serde_json::json!({
                    "decision": "deny",
                    "reason": format!("invalid request: {}", e)
                });
                let _ = writeln!(writer, "{}", resp);
                continue;
            }
        };

        // The request may include an "interactive" flag alongside the
        // action fields. Extract it before deserializing the Action.
        let interactive = msg
            .get("interactive")
            .and_then(|v| v.as_bool())
            .unwrap_or(false);

        let action: Action = match serde_json::from_value(msg) {
            Ok(a) => a,
            Err(e) => {
                let resp = serde_json::json!({
                    "decision": "deny",
                    "reason": format!("invalid action: {}", e)
                });
                let _ = writeln!(writer, "{}", resp);
                continue;
            }
        };

        let decision = {
            let mut gw = gateway.lock().unwrap();
            let seq = gw.action_count() + 1;
            let envelope = ActionEnvelope::new(session_id, seq, action);
            let raw = gw.process(&envelope);
            escalation::resolve_pause(raw, interactive)
        };

        let resp = match &decision {
            Decision::Allow => serde_json::json!({"decision": "allow"}),
            Decision::Deny { reason, rule } => {
                serde_json::json!({"decision": "deny", "reason": reason, "rule": rule})
            }
            Decision::Log { reason } => {
                serde_json::json!({"decision": "allow", "note": reason})
            }
            Decision::Pause { .. } => unreachable!(),
        };

        let _ = writeln!(writer, "{}", resp);
    }
}

impl Drop for IpcServer {
    fn drop(&mut self) {
        let _ = std::fs::remove_file(&self.socket_path);
    }
}
