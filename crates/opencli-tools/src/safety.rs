use std::io::IsTerminal;
use std::path::{Path, PathBuf};

use anyhow::{Result, bail};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum CommandRisk {
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

pub fn approve_shell(
    command: &str,
    approval_mode: &str,
    non_interactive_approval: &str,
) -> Result<()> {
    let risk = classify_command(command);

    if risk == CommandRisk::Dangerous {
        bail!("blocked dangerous command: {command}");
    }

    if std::io::stdin().is_terminal() && std::io::stdout().is_terminal() {
        return approve_interactive(command, approval_mode, risk);
    }

    approve_non_interactive(command, approval_mode, non_interactive_approval, risk)
}

fn classify_command(command: &str) -> CommandRisk {
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

fn approve_interactive(command: &str, approval_mode: &str, risk: CommandRisk) -> Result<()> {
    match approval_mode {
        "never-ask" => Ok(()),
        "on-write" if risk == CommandRisk::Read => Ok(()),
        "always-ask" | "on-write" => prompt_for_approval(command),
        other => bail!("unsupported approvalMode: {other}"),
    }
}

fn approve_non_interactive(
    command: &str,
    approval_mode: &str,
    non_interactive_approval: &str,
    risk: CommandRisk,
) -> Result<()> {
    if approval_mode == "never-ask" {
        return Ok(());
    }

    if approval_mode == "on-write" && risk == CommandRisk::Read {
        return Ok(());
    }

    match non_interactive_approval {
        "deny" => bail!("command requires approval in non-interactive mode: {command}"),
        "allow-read-only" if risk == CommandRisk::Read => Ok(()),
        "allow-all" => Ok(()),
        other => bail!("unsupported nonInteractiveApproval: {other}"),
    }
}

fn prompt_for_approval(command: &str) -> Result<()> {
    use std::io::{self, Write};

    print!("Approve shell command? {command} [y/N] ");
    io::stdout().flush()?;
    let mut input = String::new();
    io::stdin().read_line(&mut input)?;
    let answer = input.trim().to_lowercase();

    if answer == "y" || answer == "yes" {
        Ok(())
    } else {
        bail!("command rejected by user: {command}")
    }
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

    /// 测试 `normalize_path` 路径归一化辅助函数。
    /// 验证其是否能正确剥离相对路径中的 `.` 与正确向上回溯 `..`。
    #[test]
    fn test_normalize_path() {
        // 测试常规路径
        let path1 = Path::new("/a/b/c");
        assert_eq!(normalize_path(path1), PathBuf::from("/a/b/c"));

        // 测试包含当前目录 `.` 的路径
        let path2 = Path::new("/a/./b/./c");
        assert_eq!(normalize_path(path2), PathBuf::from("/a/b/c"));

        // 测试包含父目录回溯 `..` 的路径
        let path3 = Path::new("/a/b/../c");
        assert_eq!(normalize_path(path3), PathBuf::from("/a/c"));

        // 测试深度回溯导致向上溢出
        let path4 = Path::new("/a/../../b");
        assert_eq!(normalize_path(path4), PathBuf::from("/b"));
    }

    /// 测试 `classify_command` 命令风险分级逻辑。
    /// 确保危险指令、写操作指令与读操作指令能被系统精准区分。
    #[test]
    fn test_classify_command() {
        // 危险指令 (Dangerous)
        assert_eq!(
            classify_command("sudo systemctl restart nginx"),
            CommandRisk::Dangerous
        );
        assert_eq!(classify_command("rm -rf /usr/bin"), CommandRisk::Dangerous);
        assert_eq!(
            classify_command("cat file | dd of=/dev/sdb"),
            CommandRisk::Dangerous
        );

        // 写操作指令 (Write)
        assert_eq!(classify_command("mkdir -p src/tests"), CommandRisk::Write);
        assert_eq!(classify_command("rm temp.txt"), CommandRisk::Write);
        assert_eq!(
            classify_command("echo 'hello' > config.json"),
            CommandRisk::Write
        );
        assert_eq!(
            classify_command("git commit -m 'feat: add tests'"),
            CommandRisk::Write
        );

        // 纯读操作指令 (Read)
        assert_eq!(classify_command("cat README.md"), CommandRisk::Read);
        assert_eq!(classify_command("ls -la crates/"), CommandRisk::Read);
        assert_eq!(classify_command("cargo check"), CommandRisk::Read);
    }

    /// 测试 `resolve_workspace_path` 安全沙箱越界校验。
    /// 验证工作区内的合法相对路径是否可以被成功解析，以及跨越沙箱根目录的越界路径是否会被有效拦截。
    #[test]
    fn test_resolve_workspace_path() {
        // 使用当前目录作为真实的 canonical 工作区根目录，以避免操作系统硬编码引起的测试失败
        let current_dir = std::env::current_dir().expect("获取当前目录失败");
        let workspace_root = current_dir.to_str().expect("转换当前路径为字符串失败");

        // 场景1：解析合法的、位于工作区内部的子目录路径
        let target_valid = "crates/opencli-tools";
        let resolved = resolve_workspace_path(workspace_root, target_valid);
        assert!(resolved.is_ok());
        let expected_path = current_dir.join(target_valid);
        assert_eq!(resolved.unwrap(), expected_path);

        // 场景2：解析跨越沙箱根目录向外越界的路径 (利用 `..` 超出边界)
        let target_invalid = "../../../etc/passwd";
        let resolved_err = resolve_workspace_path(workspace_root, target_invalid);
        assert!(resolved_err.is_err());
        assert!(
            resolved_err
                .unwrap_err()
                .to_string()
                .contains("path is outside workspace")
        );
    }
}
