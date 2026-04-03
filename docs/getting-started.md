# 快速开始

## 环境要求

- Rust 工具链
- 可访问的模型 API
- 已配置好的 `apiKey`、`baseUrl` 和 `model`

## 构建项目

```bash
cargo build
```

## 初始化配置

```bash
cargo run -- config init
```

默认配置文件路径：`~/.config/ai-cli/config.json`

最小可用示例：

```json
{
  "provider": "openai-compatible",
  "baseUrl": "https://api.openai.com/v1",
  "apiKey": "<your-api-key>",
  "model": "gpt-4.1",
  "approvalMode": "on-write",
  "nonInteractiveApproval": "deny",
  "workspaceRoot": ".",
  "allowedToolKinds": [
    "filesystem-read",
    "filesystem-search",
    "shell",
    "agent"
  ],
  "allowedTools": []
}
```

## 第一次运行

单轮执行：

```bash
cargo run -- "Explain this repository"
```

聊天模式：

```bash
cargo run -- chat
```

带上下文运行：

```bash
cargo run -- run --file src/main.rs "Explain this file"
cargo run -- run --dir src "Summarize this codebase"
```

## 常见本地安装方式

Unix-like 环境：

```bash
./scripts/install.sh
```

Windows PowerShell：

```powershell
./scripts/install.ps1
```

## 建议阅读顺序

1. 先看 [项目概览](./project-overview.md)
2. 再看 [配置说明](./configuration.md)
3. 最后按需查 [命令参考](./commands.md)
