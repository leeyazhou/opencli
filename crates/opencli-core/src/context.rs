use std::{fs, path::Path};

use anyhow::{Context, Result};

const MAX_FILE_BYTES: usize = 32 * 1024;
const MAX_DIR_ENTRIES: usize = 200;

pub fn collect_context(cwd: &Path, file_paths: &[String], dir_paths: &[String]) -> Result<String> {
    let mut blocks = Vec::new();

    for file in file_paths {
        let resolved = cwd.join(file);
        let content = fs::read_to_string(&resolved)
            .with_context(|| format!("failed to read file {}", resolved.display()))?;
        let limited = if content.len() > MAX_FILE_BYTES {
            format!("{}\n...<truncated>", &content[..MAX_FILE_BYTES])
        } else {
            content
        };
        blocks.push(format!(
            "[file] {}\n```\n{}\n```",
            resolved.display(),
            limited
        ));
    }

    for dir in dir_paths {
        let resolved = cwd.join(dir);
        let mut count = 0usize;
        let summary = summarize_directory(&resolved, &resolved, 0, &mut count)?;
        blocks.push(format!("[dir] {}\n{}", resolved.display(), summary));
    }

    Ok(blocks.join("\n\n"))
}

fn summarize_directory(
    root: &Path,
    current: &Path,
    depth: usize,
    count: &mut usize,
) -> Result<String> {
    if *count >= MAX_DIR_ENTRIES {
        return Ok(format!("{}...<truncated>", indent(depth)));
    }

    let mut entries = fs::read_dir(current)?.collect::<Result<Vec<_>, _>>()?;
    entries.sort_by_key(|entry| entry.file_name());

    let mut lines = Vec::new();
    for entry in entries {
        if *count >= MAX_DIR_ENTRIES {
            lines.push(format!("{}...<truncated>", indent(depth)));
            break;
        }

        let path = entry.path();
        let metadata = entry.metadata()?;
        let relative = relative_display(root, &path);
        *count += 1;

        if metadata.is_dir() {
            lines.push(format!("{}{}{}", indent(depth), relative, "/"));
            lines.push(summarize_directory(root, &path, depth + 1, count)?);
        } else {
            lines.push(format!(
                "{}{} ({} bytes)",
                indent(depth),
                relative,
                metadata.len()
            ));
        }
    }

    Ok(lines.join("\n"))
}

fn relative_display(root: &Path, path: &Path) -> String {
    path.strip_prefix(root)
        .unwrap_or(path)
        .to_string_lossy()
        .to_string()
}

fn indent(depth: usize) -> String {
    "  ".repeat(depth)
}

#[cfg(test)]
mod tests {
    use std::fs;

    use super::*;

    #[test]
    fn collects_file_and_dir_context() {
        let temp = std::env::temp_dir().join(format!("opencli-context-{}", uuid::Uuid::new_v4()));
        fs::create_dir_all(temp.join("src")).expect("test src dir should be created");
        fs::write(temp.join("main.rs"), "fn main() {}\n").expect("test file should be written");
        fs::write(temp.join("src").join("lib.rs"), "pub fn x() {}\n")
            .expect("test lib file should be written");

        let result = collect_context(&temp, &["main.rs".into()], &["src".into()])
            .expect("context collection should succeed");
        assert!(result.contains("[file]"));
        assert!(result.contains("fn main()"));
        assert!(result.contains("lib.rs"));
    }
}
