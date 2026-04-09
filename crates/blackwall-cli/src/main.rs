use blackwall_core::audit::AuditLog;
use blackwall_core::engine::PolicyEngine;
use blackwall_core::policy::Policy;
use blackwall_gateway::detect::Environment;
use blackwall_gateway::gateway::BlackwallGateway;
use blackwall_gateway::ipc::IpcServer;
use blackwall_gateway::mcp_proxy::McpProxy;
use blackwall_gateway::shell_shim::ShimDir;
use clap::{Parser, Subcommand};
use std::io::{BufRead, BufReader, Write};
use std::path::PathBuf;
use std::sync::{Arc, Mutex};

#[derive(Parser)]
#[command(
    name = "blackwall",
    version,
    about = "A deterministic execution firewall for AI agents"
)]
struct Cli {
    #[command(subcommand)]
    command: Option<Commands>,

    /// Policy: default, strict, permissive, or path to .yaml
    #[arg(short, long, global = true)]
    policy: Option<String>,

    /// Workspace directory to protect
    #[arg(short, long, global = true)]
    workspace: Option<PathBuf>,
}

#[derive(Subcommand)]
enum Commands {
    /// Stop the running Blackwall gateway
    Off,

    /// Show current session status
    Status,

    /// View the audit log
    Logs {
        /// Follow the log in real time
        #[arg(short, long)]
        follow: bool,

        /// Filter by session ID
        #[arg(short, long)]
        session: Option<String>,
    },

    /// Execute a command under Blackwall protection
    Exec {
        /// Command and arguments to run
        #[arg(trailing_var_arg = true, allow_hyphen_values = true)]
        command: Vec<String>,
    },

    /// Proxy an MCP server with Blackwall interception
    #[command(name = "proxy-mcp")]
    ProxyMcp {
        /// MCP server command and arguments
        #[arg(trailing_var_arg = true, allow_hyphen_values = true)]
        command: Vec<String>,
    },

    /// Evaluate a single action via IPC (used internally by shell shims)
    #[command(name = "eval-action", hide = true)]
    EvalAction {
        /// Path to the gateway's Unix socket
        #[arg(long)]
        socket: PathBuf,

        /// Absolute path to the real binary
        #[arg(long)]
        real: String,

        /// Arguments to pass to the real binary
        #[arg(trailing_var_arg = true, allow_hyphen_values = true)]
        args: Vec<String>,
    },
}

fn main() {
    let cli = Cli::parse();

    match cli.command {
        None => start(cli.policy, cli.workspace),
        Some(Commands::Off) => off(),
        Some(Commands::Status) => status(),
        Some(Commands::Logs { follow, session }) => logs(follow, session),
        Some(Commands::Exec { command }) => exec(command, cli.policy, cli.workspace),
        Some(Commands::ProxyMcp { command }) => proxy_mcp(command, cli.policy, cli.workspace),
        Some(Commands::EvalAction { socket, real, args }) => eval_action(socket, real, args),
    }
}

// ── start ──────────────────────────────────────────────────────────────

fn start(policy_name: Option<String>, workspace_arg: Option<PathBuf>) {
    let workspace = workspace_arg
        .unwrap_or_else(|| std::env::current_dir().expect("cannot determine working directory"));

    let policy = load_policy(policy_name.as_deref(), &workspace);
    let env = Environment::detect(&workspace);
    let session_id = uuid::Uuid::new_v4().to_string()[..8].to_string();

    let log_dir = blackwall_dir().join("logs");
    let audit = AuditLog::new(&log_dir, &session_id).expect("cannot create audit log");
    let engine =
        PolicyEngine::new(policy.clone(), workspace.clone()).expect("cannot initialize engine");
    let gateway = Arc::new(Mutex::new(BlackwallGateway::new(engine, audit)));

    let ipc = IpcServer::bind(&session_id).expect("cannot start IPC server");
    let socket_path = ipc.socket_path().to_path_buf();

    let blackwall_bin = std::env::current_exe().expect("cannot determine blackwall binary path");
    let shim_dir =
        ShimDir::create(&session_id, &socket_path, &blackwall_bin).expect("cannot create shims");
    let shim_path_display = shim_dir.path().display().to_string();
    let shim_path_cleanup = shim_dir.path().to_path_buf();

    let gw_ipc = Arc::clone(&gateway);
    let sid = session_id.clone();
    std::thread::spawn(move || ipc.run(gw_ipc, sid));

    print_banner(&policy, &session_id, &workspace, &env);
    eprintln!(
        "  \x1b[36mshell shims:\x1b[0m  export PATH=\"{}:$PATH\"",
        shim_path_display
    );
    eprintln!("  \x1b[36mmcp proxy:\x1b[0m    blackwall proxy-mcp -- <server-command>");
    eprintln!();
    eprintln!("  \x1b[2mgateway active — press ctrl+c to stop\x1b[0m");
    eprintln!();

    let socket_cleanup = socket_path.clone();
    let (tx, rx) = std::sync::mpsc::channel();
    ctrlc::set_handler(move || {
        let _ = std::fs::remove_file(&socket_cleanup);
        let _ = std::fs::remove_dir_all(&shim_path_cleanup);
        tx.send(()).ok();
    })
    .expect("cannot set ctrl+c handler");

    rx.recv().ok();
    eprintln!();
    eprintln!("  \x1b[2m■ blackwall stopped\x1b[0m");
}

// ── exec ───────────────────────────────────────────────────────────────

fn exec(command: Vec<String>, policy_name: Option<String>, workspace_arg: Option<PathBuf>) {
    if command.is_empty() {
        eprintln!("  \x1b[31merror:\x1b[0m no command specified");
        eprintln!("  usage: blackwall exec -- <command>");
        std::process::exit(1);
    }

    let workspace = workspace_arg
        .unwrap_or_else(|| std::env::current_dir().expect("cannot determine working directory"));
    let policy = load_policy(policy_name.as_deref(), &workspace);
    let session_id = uuid::Uuid::new_v4().to_string()[..8].to_string();

    let log_dir = blackwall_dir().join("logs");
    let audit = AuditLog::new(&log_dir, &session_id).expect("cannot create audit log");
    let engine =
        PolicyEngine::new(policy.clone(), workspace.clone()).expect("cannot initialize engine");
    let gateway = Arc::new(Mutex::new(BlackwallGateway::new(engine, audit)));

    let ipc = IpcServer::bind(&session_id).expect("cannot start IPC server");
    let socket_path = ipc.socket_path().to_path_buf();

    let blackwall_bin = std::env::current_exe().expect("cannot determine blackwall binary path");
    let shim_dir =
        ShimDir::create(&session_id, &socket_path, &blackwall_bin).expect("cannot create shims");
    let modified_path = shim_dir.prepend_to_path();
    let orig_path = std::env::var("PATH").unwrap_or_default();
    let shim_path_cleanup = shim_dir.path().to_path_buf();

    let gw_ipc = Arc::clone(&gateway);
    let sid = session_id.clone();
    std::thread::spawn(move || ipc.run(gw_ipc, sid));

    eprintln!(
        "  \x1b[1;97m■ blackwall exec\x1b[0m policy: {} | session: {} | wrapping: {}",
        policy.name,
        session_id,
        command.join(" ")
    );
    eprintln!("  log: ~/.blackwall/logs/{}.jsonl", session_id);
    eprintln!();

    // Ignore SIGINT in parent — the child receives it from the terminal's
    // process group and handles it (or dies), then .status() returns.
    ctrlc::set_handler(|| {}).expect("cannot set signal handler");

    let status = std::process::Command::new(&command[0])
        .args(&command[1..])
        .env("PATH", &modified_path)
        .env("BLACKWALL_SESSION", &session_id)
        .env("BLACKWALL_SOCKET", socket_path.to_str().unwrap_or(""))
        .env("BLACKWALL_ORIG_PATH", &orig_path)
        .status();

    let _ = std::fs::remove_file(&socket_path);
    let _ = std::fs::remove_dir_all(&shim_path_cleanup);

    match status {
        Ok(s) => {
            eprintln!();
            eprintln!(
                "  \x1b[2m■ blackwall exec finished (exit {})\x1b[0m",
                s.code().unwrap_or(-1)
            );
            std::process::exit(s.code().unwrap_or(1));
        }
        Err(e) => {
            eprintln!("  \x1b[31merror:\x1b[0m cannot execute '{}': {}", command[0], e);
            std::process::exit(1);
        }
    }
}

// ── proxy-mcp ──────────────────────────────────────────────────────────

fn proxy_mcp(command: Vec<String>, policy_name: Option<String>, workspace_arg: Option<PathBuf>) {
    if command.is_empty() {
        eprintln!("  \x1b[31merror:\x1b[0m no MCP server command specified");
        eprintln!("  usage: blackwall proxy-mcp -- <server-command> [args...]");
        std::process::exit(1);
    }

    let workspace = workspace_arg
        .unwrap_or_else(|| std::env::current_dir().expect("cannot determine working directory"));
    let policy = load_policy(policy_name.as_deref(), &workspace);
    let session_id = uuid::Uuid::new_v4().to_string()[..8].to_string();

    let log_dir = blackwall_dir().join("logs");
    let audit = AuditLog::new(&log_dir, &session_id).expect("cannot create audit log");
    let engine = PolicyEngine::new(policy, workspace).expect("cannot initialize engine");
    let gateway = BlackwallGateway::new(engine, audit);

    eprintln!(
        "  \x1b[1;97m■ blackwall mcp proxy\x1b[0m session: {} | server: {}",
        session_id,
        command.join(" ")
    );

    let proxy = McpProxy::new(gateway, session_id);
    let server_args: Vec<String> = command[1..].to_vec();

    match proxy.run(&command[0], &server_args, vec![]) {
        Ok(code) => std::process::exit(code),
        Err(e) => {
            eprintln!("  \x1b[31merror:\x1b[0m MCP server failed: {}", e);
            std::process::exit(1);
        }
    }
}

// ── eval-action (shell shim helper) ────────────────────────────────────

fn eval_action(socket_path: PathBuf, real_binary: String, args: Vec<String>) {
    use std::os::unix::net::UnixStream;
    use std::os::unix::process::CommandExt;

    // Use the basename for policy matching — policy rules reference short
    // names like "curl", not full paths like "/usr/bin/curl"
    let program_name = std::path::Path::new(&real_binary)
        .file_name()
        .and_then(|n| n.to_str())
        .unwrap_or(&real_binary);

    let command_str = if args.is_empty() {
        program_name.to_string()
    } else {
        format!("{} {}", program_name, args.join(" "))
    };

    let action = serde_json::json!({
        "tool": "shell",
        "operation": "exec",
        "target": command_str,
        "parameters": {}
    });

    let mut stream = match UnixStream::connect(&socket_path) {
        Ok(s) => s,
        Err(_) => {
            eprintln!("  \x1b[31m✗ blackwall: gateway unreachable\x1b[0m");
            std::process::exit(126);
        }
    };

    if writeln!(stream, "{}", action).is_err() || stream.flush().is_err() {
        eprintln!("  \x1b[31m✗ blackwall: cannot communicate with gateway\x1b[0m");
        std::process::exit(126);
    }

    let mut reader = BufReader::new(&stream);
    let mut response = String::new();
    if reader.read_line(&mut response).is_err() {
        eprintln!("  \x1b[31m✗ blackwall: no response from gateway\x1b[0m");
        std::process::exit(126);
    }

    let resp: serde_json::Value = match serde_json::from_str(response.trim()) {
        Ok(v) => v,
        Err(_) => {
            eprintln!("  \x1b[31m✗ blackwall: invalid gateway response\x1b[0m");
            std::process::exit(126);
        }
    };

    let decision = resp
        .get("decision")
        .and_then(|d| d.as_str())
        .unwrap_or("deny");

    if decision == "allow" {
        let err = std::process::Command::new(&real_binary).args(&args).exec();
        eprintln!("  \x1b[31m✗ blackwall: exec failed: {}\x1b[0m", err);
        std::process::exit(126);
    }

    let reason = resp
        .get("reason")
        .and_then(|r| r.as_str())
        .unwrap_or("blocked by policy");
    eprintln!("  \x1b[31m✗ blackwall: {}\x1b[0m", reason);
    std::process::exit(126);
}

// ── logs ───────────────────────────────────────────────────────────────

fn logs(follow: bool, session: Option<String>) {
    let log_dir = blackwall_dir().join("logs");
    if !log_dir.exists() {
        eprintln!("  \x1b[2mno logs found\x1b[0m");
        return;
    }

    let target = match find_log_file(&log_dir, session.as_deref()) {
        Some(p) => p,
        None => {
            eprintln!("  \x1b[2mno logs found\x1b[0m");
            return;
        }
    };

    if !target.exists() {
        eprintln!("  \x1b[2mlog file not found\x1b[0m");
        return;
    }

    let file = match std::fs::File::open(&target) {
        Ok(f) => f,
        Err(e) => {
            eprintln!("  \x1b[31merror reading log: {}\x1b[0m", e);
            return;
        }
    };

    let mut reader = BufReader::new(file);

    loop {
        let mut line = String::new();
        match reader.read_line(&mut line) {
            Ok(0) => break,
            Ok(_) => print_log_line(&line),
            Err(_) => break,
        }
    }

    if !follow {
        return;
    }

    eprintln!(
        "  \x1b[2m--- following {} ---\x1b[0m",
        target.file_name().unwrap().to_string_lossy()
    );

    loop {
        let mut line = String::new();
        match reader.read_line(&mut line) {
            Ok(0) => std::thread::sleep(std::time::Duration::from_millis(200)),
            Ok(_) => print_log_line(&line),
            Err(_) => break,
        }
    }
}

fn find_log_file(log_dir: &std::path::Path, session: Option<&str>) -> Option<PathBuf> {
    if let Some(id) = session {
        return Some(log_dir.join(format!("{}.jsonl", id)));
    }

    let mut entries: Vec<_> = std::fs::read_dir(log_dir)
        .into_iter()
        .flatten()
        .filter_map(|e| e.ok())
        .filter(|e| {
            e.path()
                .extension()
                .map(|x| x == "jsonl")
                .unwrap_or(false)
        })
        .collect();

    entries.sort_by_key(|e| std::cmp::Reverse(e.metadata().ok().and_then(|m| m.modified().ok())));

    entries.first().map(|e| e.path())
}

fn print_log_line(line: &str) {
    let trimmed = line.trim();
    if trimmed.is_empty() {
        return;
    }

    if let Ok(entry) = serde_json::from_str::<serde_json::Value>(trimmed) {
        let ts = entry.get("ts").and_then(|t| t.as_str()).unwrap_or("?");
        let ts_short = if ts.len() >= 19 { &ts[11..19] } else { ts };
        let decision = entry
            .get("decision")
            .and_then(|d| d.as_str())
            .unwrap_or("?");
        let tool = entry.get("tool").and_then(|t| t.as_str()).unwrap_or("?");
        let op = entry.get("op").and_then(|o| o.as_str()).unwrap_or("?");
        let target = entry
            .get("target")
            .and_then(|t| t.as_str())
            .unwrap_or("?");

        let symbol = match decision {
            "allow" => "\x1b[32m✓\x1b[0m",
            "deny" => "\x1b[31m✗\x1b[0m",
            "pause" => "\x1b[33m⏸\x1b[0m",
            "log" => "\x1b[2m·\x1b[0m",
            _ => "?",
        };

        let extra = match decision {
            "deny" => entry
                .get("rule")
                .and_then(|r| r.as_str())
                .map(|r| format!("  \x1b[31m[{}]\x1b[0m", r))
                .unwrap_or_default(),
            "pause" => entry
                .get("reason")
                .and_then(|r| r.as_str())
                .map(|r| format!("  \x1b[33m[{}]\x1b[0m", r))
                .unwrap_or_default(),
            _ => String::new(),
        };

        eprintln!(
            "{} {} {:<18} {}{}",
            ts_short,
            symbol,
            format!("{}.{}", tool, op),
            target,
            extra
        );
    } else {
        eprint!("{}", line);
    }
}

// ── off / status ───────────────────────────────────────────────────────

fn off() {
    let run_dir = blackwall_dir().join("run");
    if !run_dir.exists() {
        eprintln!("  \x1b[2m■ no active sessions\x1b[0m");
        return;
    }

    let sockets: Vec<_> = std::fs::read_dir(&run_dir)
        .into_iter()
        .flatten()
        .filter_map(|e| e.ok())
        .filter(|e| {
            e.path()
                .extension()
                .map(|x| x == "sock")
                .unwrap_or(false)
        })
        .collect();

    if sockets.is_empty() {
        eprintln!("  \x1b[2m■ no active sessions\x1b[0m");
        return;
    }

    for sock in &sockets {
        let _ = std::fs::remove_file(sock.path());
        let session_id = sock
            .path()
            .file_stem()
            .unwrap()
            .to_string_lossy()
            .to_string();
        let shim_dir = dirs::home_dir()
            .unwrap()
            .join(".blackwall/shims")
            .join(&session_id);
        let _ = std::fs::remove_dir_all(&shim_dir);
        eprintln!("  \x1b[2m■ stopped session {}\x1b[0m", session_id);
    }
}

fn status() {
    let run_dir = blackwall_dir().join("run");
    let log_dir = blackwall_dir().join("logs");

    let active: Vec<_> = if run_dir.exists() {
        std::fs::read_dir(&run_dir)
            .into_iter()
            .flatten()
            .filter_map(|e| e.ok())
            .filter(|e| {
                e.path()
                    .extension()
                    .map(|x| x == "sock")
                    .unwrap_or(false)
            })
            .collect()
    } else {
        vec![]
    };

    if !active.is_empty() {
        eprintln!("  \x1b[1mactive sessions:\x1b[0m");
        for entry in &active {
            let name = entry.file_name();
            let session_id = name.to_string_lossy().replace(".sock", "");
            eprintln!("    \x1b[32m●\x1b[0m {}", session_id);
        }
        eprintln!();
    }

    if !log_dir.exists() {
        if active.is_empty() {
            eprintln!("  \x1b[2mno active sessions\x1b[0m");
        }
        return;
    }

    let mut entries: Vec<_> = std::fs::read_dir(&log_dir)
        .into_iter()
        .flatten()
        .filter_map(|e| e.ok())
        .filter(|e| {
            e.path()
                .extension()
                .map(|x| x == "jsonl")
                .unwrap_or(false)
        })
        .collect();

    entries.sort_by_key(|e| std::cmp::Reverse(e.metadata().ok().and_then(|m| m.modified().ok())));

    if entries.is_empty() && active.is_empty() {
        eprintln!("  \x1b[2mno sessions found\x1b[0m");
        return;
    }

    eprintln!("  \x1b[1mrecent logs:\x1b[0m");
    for entry in entries.iter().take(10) {
        let name = entry.file_name();
        let size = entry.metadata().map(|m| m.len()).unwrap_or(0);
        let session_id = name.to_string_lossy().replace(".jsonl", "");
        let is_active = active
            .iter()
            .any(|a| a.file_name().to_string_lossy().replace(".sock", "") == session_id);
        let indicator = if is_active {
            "\x1b[32m●\x1b[0m"
        } else {
            "\x1b[2m○\x1b[0m"
        };
        eprintln!(
            "    {} {} ({} bytes)",
            indicator,
            name.to_string_lossy(),
            size
        );
    }
}

// ── helpers ────────────────────────────────────────────────────────────

fn print_banner(
    policy: &Policy,
    session_id: &str,
    workspace: &std::path::Path,
    env: &Environment,
) {
    eprintln!();
    eprintln!("\x1b[2m  :::::::::  :::            :::      ::::::::  :::    ::: :::       :::     :::     :::        :::        ");
    eprintln!("  :+:    :+: :+:          :+: :+:   :+:    :+: :+:   :+:  :+:       :+:   :+: :+:   :+:        :+:        ");
    eprintln!("  +:+    +:+ +:+         +:+   +:+  +:+        +:+  +:+   +:+       +:+  +:+   +:+  +:+        +:+        ");
    eprintln!("  +#++:++#+  +#+        +#++:++#++: +#+        +#++:++    +#+  +:+  +#+ +#++:++#++: +#+        +#+        ");
    eprintln!("  +#+    +#+ +#+        +#+     +#+ +#+        +#+  +#+   +#+ +#+#+ +#+ +#+     +#+ +#+        +#+        ");
    eprintln!("  #+#    #+# #+#        #+#     #+# #+#    #+# #+#   #+#   #+#+# #+#+#  #+#     #+# #+#        #+#        ");
    eprintln!("  #########  ########## ###     ###  ########  ###    ###   ###   ###   ###     ### ########## ##########\x1b[0m");
    eprintln!();
    eprintln!(
        "  \x1b[1;97m■ blackwall\x1b[0m v{}",
        env!("CARGO_PKG_VERSION")
    );
    eprintln!(
        "  policy: \x1b[1m{}\x1b[0m | session: {}",
        policy.name, session_id
    );
    eprintln!("  workspace: {}", workspace.display());
    eprintln!("  log: ~/.blackwall/logs/{}.jsonl", session_id);
    eprintln!();
    eprintln!("  {}", env.summary());
    eprintln!();
}

fn load_policy(name: Option<&str>, workspace: &PathBuf) -> Policy {
    match name {
        None | Some("default") => Policy::default_policy(workspace),
        Some("strict") => Policy::strict_policy(workspace),
        Some("permissive") => Policy::permissive_policy(workspace),
        Some(path) => {
            let mut policy = Policy::load(std::path::Path::new(path)).unwrap_or_else(|e| {
                eprintln!("  \x1b[31merror:\x1b[0m cannot load policy '{}': {}", path, e);
                std::process::exit(1);
            });
            policy.resolve_workspace(workspace);
            policy
        }
    }
}

fn blackwall_dir() -> PathBuf {
    dirs::home_dir()
        .expect("cannot find home directory")
        .join(".blackwall")
}
