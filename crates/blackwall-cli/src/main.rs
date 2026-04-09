use blackwall_core::audit::AuditLog;
use blackwall_core::engine::PolicyEngine;
use blackwall_core::policy::Policy;
use blackwall_gateway::detect::Environment;
use blackwall_gateway::gateway::BlackwallGateway;
use clap::{Parser, Subcommand};
use std::path::PathBuf;

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
}

fn main() {
    let cli = Cli::parse();

    match cli.command {
        None => start(cli.policy, cli.workspace),
        Some(Commands::Off) => off(),
        Some(Commands::Status) => status(),
        Some(Commands::Logs { follow, session }) => logs(follow, session),
        Some(Commands::Exec { command }) => exec(command, cli.policy, cli.workspace),
    }
}

fn start(policy_name: Option<String>, workspace_arg: Option<PathBuf>) {
    let workspace = workspace_arg
        .unwrap_or_else(|| std::env::current_dir().expect("cannot determine working directory"));

    let policy = load_policy(policy_name.as_deref(), &workspace);
    let env = Environment::detect(&workspace);
    let session_id = &uuid::Uuid::new_v4().to_string()[..8];

    let log_dir = blackwall_dir().join("logs");
    let audit = AuditLog::new(&log_dir, session_id).expect("cannot create audit log");
    let engine =
        PolicyEngine::new(policy.clone(), workspace.clone()).expect("cannot initialize engine");
    let _gateway = BlackwallGateway::new(engine, audit);

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
    eprintln!("  policy: \x1b[1m{}\x1b[0m | session: {}", policy.name, session_id);
    eprintln!("  workspace: {}", workspace.display());
    eprintln!("  log: ~/.blackwall/logs/{}.jsonl", session_id);
    eprintln!();
    eprintln!("  {}", env.summary());
    eprintln!();
    eprintln!("  \x1b[2mgateway active — press ctrl+c to stop\x1b[0m");
    eprintln!();

    let (tx, rx) = std::sync::mpsc::channel();
    ctrlc::set_handler(move || {
        tx.send(()).ok();
    })
    .expect("cannot set ctrl+c handler");

    rx.recv().ok();
    eprintln!();
    eprintln!("  \x1b[2m■ blackwall stopped\x1b[0m");
}

fn off() {
    eprintln!("  \x1b[2m■ blackwall stopped\x1b[0m");
}

fn status() {
    let log_dir = blackwall_dir().join("logs");
    if !log_dir.exists() {
        eprintln!("  \x1b[2mno active session\x1b[0m");
        return;
    }

    let mut entries: Vec<_> = std::fs::read_dir(&log_dir)
        .into_iter()
        .flatten()
        .filter_map(|e| e.ok())
        .filter(|e| e.path().extension().map(|x| x == "jsonl").unwrap_or(false))
        .collect();

    entries.sort_by_key(|e| std::cmp::Reverse(e.metadata().ok().and_then(|m| m.modified().ok())));

    if entries.is_empty() {
        eprintln!("  \x1b[2mno sessions found\x1b[0m");
        return;
    }

    eprintln!("  \x1b[1msessions:\x1b[0m");
    for entry in entries.iter().take(10) {
        let name = entry.file_name();
        let size = entry.metadata().map(|m| m.len()).unwrap_or(0);
        eprintln!("    {} ({} bytes)", name.to_string_lossy(), size);
    }
}

fn logs(follow: bool, session: Option<String>) {
    if follow {
        eprintln!("  \x1b[33m--follow is not yet implemented\x1b[0m");
        std::process::exit(1);
    }

    let log_dir = blackwall_dir().join("logs");
    if !log_dir.exists() {
        eprintln!("  \x1b[2mno logs found\x1b[0m");
        return;
    }

    let target = if let Some(ref id) = session {
        log_dir.join(format!("{}.jsonl", id))
    } else {
        let mut entries: Vec<_> = std::fs::read_dir(&log_dir)
            .into_iter()
            .flatten()
            .filter_map(|e| e.ok())
            .filter(|e| e.path().extension().map(|x| x == "jsonl").unwrap_or(false))
            .collect();
        entries.sort_by_key(|e| {
            std::cmp::Reverse(e.metadata().ok().and_then(|m| m.modified().ok()))
        });
        match entries.first() {
            Some(e) => e.path(),
            None => {
                eprintln!("  \x1b[2mno logs found\x1b[0m");
                return;
            }
        }
    };

    if !target.exists() {
        eprintln!("  \x1b[2mlog file not found\x1b[0m");
        return;
    }

    match std::fs::read_to_string(&target) {
        Ok(content) => {
            for line in content.lines() {
                println!("{}", line);
            }
        }
        Err(e) => eprintln!("  \x1b[31merror reading log: {}\x1b[0m", e),
    }
}

fn exec(command: Vec<String>, policy_name: Option<String>, workspace_arg: Option<PathBuf>) {
    if command.is_empty() {
        eprintln!("  \x1b[31merror:\x1b[0m no command specified");
        eprintln!("  usage: blackwall exec -- <command>");
        std::process::exit(1);
    }

    let workspace = workspace_arg
        .unwrap_or_else(|| std::env::current_dir().expect("cannot determine working directory"));
    let policy = load_policy(policy_name.as_deref(), &workspace);

    eprintln!(
        "  \x1b[1;97m■ blackwall exec\x1b[0m policy: {} | wrapping: {}",
        policy.name,
        command.join(" ")
    );
    eprintln!("  \x1b[2mexec mode not yet implemented — run `blackwall` for gateway mode\x1b[0m");
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
