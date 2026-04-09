use std::io;
use std::path::{Path, PathBuf};

const SHIMMED_COMMANDS: &[&str] = &[
    "curl", "wget", "ssh", "scp", "rsync",
    "rm", "chmod", "chown", "chgrp",
    "docker", "kubectl", "podman",
    "pip", "pip3", "npm", "npx", "yarn", "pnpm", "cargo",
    "python", "python3", "node", "ruby", "perl",
    "bash", "sh", "zsh",
    "git",
    "sudo", "su", "doas",
    "nc", "ncat", "socat",
    "dd", "mkfs", "mount", "umount",
    "kill", "pkill", "killall",
    "env", "nohup",
];

pub struct ShimDir {
    path: PathBuf,
}

impl ShimDir {
    /// Create a shim directory with wrapper scripts for common commands.
    /// Each shim calls `blackwall eval-action` which evaluates the command
    /// through the gateway IPC socket, then execs the real binary if allowed.
    pub fn create(
        session_id: &str,
        socket_path: &Path,
        blackwall_bin: &Path,
    ) -> io::Result<Self> {
        let shim_dir = dirs::home_dir()
            .expect("cannot find home directory")
            .join(".blackwall/shims")
            .join(session_id);
        std::fs::create_dir_all(&shim_dir)?;

        let orig_path = std::env::var("PATH").unwrap_or_default();

        for cmd in SHIMMED_COMMANDS {
            if let Some(real_path) = resolve_real_binary(cmd, &orig_path) {
                write_shim(&shim_dir, cmd, &real_path, blackwall_bin, socket_path)?;
            }
        }

        Ok(Self { path: shim_dir })
    }

    pub fn path(&self) -> &Path {
        &self.path
    }

    pub fn prepend_to_path(&self) -> String {
        let orig = std::env::var("PATH").unwrap_or_default();
        format!("{}:{}", self.path.display(), orig)
    }

    pub fn cleanup(&self) {
        let _ = std::fs::remove_dir_all(&self.path);
    }
}

fn resolve_real_binary(name: &str, path_var: &str) -> Option<String> {
    for dir in path_var.split(':') {
        let candidate = Path::new(dir).join(name);
        if candidate.is_file() {
            return Some(candidate.to_string_lossy().to_string());
        }
    }
    None
}

fn write_shim(
    shim_dir: &Path,
    name: &str,
    real_path: &str,
    blackwall_bin: &Path,
    socket_path: &Path,
) -> io::Result<()> {
    let shim_path = shim_dir.join(name);

    // The shebang uses /bin/sh by absolute path — this bypasses PATH
    // so even a shimmed `sh` won't cause infinite recursion.
    let content = format!(
        r#"#!/bin/sh
exec "{blackwall}" eval-action --socket "{socket}" --real "{real}" -- "$@"
"#,
        blackwall = blackwall_bin.display(),
        socket = socket_path.display(),
        real = real_path,
    );

    std::fs::write(&shim_path, content)?;
    set_executable(&shim_path)?;
    Ok(())
}

fn set_executable(path: &Path) -> io::Result<()> {
    use std::os::unix::fs::PermissionsExt;
    let mut perms = std::fs::metadata(path)?.permissions();
    perms.set_mode(0o755);
    std::fs::set_permissions(path, perms)
}
