use std::path::{Path, PathBuf};

use anyhow::{Result, bail};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CommandRisk {
    Read,
    Write,
    Dangerous,
}

pub fn resolve_workspace_path(workspace_root: &str, target_path: &str) -> Result<PathBuf> {
    let root = Path::new(workspace_root).canonicalize()?;
    let candidate = root.join(target_path);
    let normalized = normalize_path(&candidate);

    if normalized.starts_with(&root) {
        Ok(normalized)
    } else {
        bail!("path is outside workspace: {target_path}")
    }
}

pub fn classify_command(command: &str) -> CommandRisk {
    let lowered = command.to_lowercase();

    for pattern in ["sudo ", "rm -rf /", "mkfs", " dd ", "shutdown", "reboot"] {
        if lowered.contains(pattern) {
            return CommandRisk::Dangerous;
        }
    }

    for pattern in [
        "rm ",
        "mv ",
        "cp ",
        "mkdir ",
        "touch ",
        "npm install",
        "pnpm add",
        "cargo add",
        "git add",
        "git commit",
        "git restore",
        "git clean",
        ">",
        "tee ",
    ] {
        if lowered.contains(pattern) {
            return CommandRisk::Write;
        }
    }

    CommandRisk::Read
}

fn normalize_path(path: &Path) -> PathBuf {
    let mut normalized = PathBuf::new();

    for component in path.components() {
        match component {
            std::path::Component::ParentDir => {
                normalized.pop();
            }
            std::path::Component::CurDir => {}
            other => normalized.push(other.as_os_str()),
        }
    }

    normalized
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn classifies_risk_levels() {
        assert_eq!(classify_command("git status"), CommandRisk::Read);
        assert_eq!(classify_command("cargo add anyhow"), CommandRisk::Write);
        assert_eq!(
            classify_command("sudo rm -rf /tmp/x"),
            CommandRisk::Dangerous
        );
    }
}
