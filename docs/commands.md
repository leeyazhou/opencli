# 命令参考

## 基础用法

直接传 prompt：

```bash
ai-cli "Explain this repository"
```

如果在本地源码中运行，也可以使用：

```bash
cargo run -- "Explain this repository"
```

## 交互命令

### `chat`

启动交互式聊天模式。

```bash
ai-cli chat
```

### `tui`

启动终端 UI 模式。该功能由 `tui` feature 提供，当前默认启用。

```bash
ai-cli tui
```

## 上下文运行

### `run`

为 prompt 注入指定文件或目录内容。

```bash
ai-cli run --file src/main.rs "Explain this file"
ai-cli run --dir src "Summarize this codebase"
```

## 模型与补全

### `models`

列出可用模型。

```bash
ai-cli models
```

### `completions`

生成 shell 补全脚本。

```bash
ai-cli completions bash
ai-cli completions zsh
ai-cli completions fish
```

## 配置命令

### `config init`

创建默认配置文件。

### `config show`

输出当前生效配置。

### `config doctor`

检查 API key、base URL、workspace 等基础配置问题。

```bash
ai-cli config doctor
```

## 会话命令

### `session list`

列出本地会话。

### `session resume <id>`

恢复指定会话。

### `session delete <id>`

删除指定会话。

### `session rename <id> <title>`

重命名会话标题。

## 审计命令

### `audit list`

查看最近的审计记录。

```bash
ai-cli audit list --limit 20
```

### `audit tail`

实时追踪审计日志。

```bash
ai-cli audit tail --lines 20 --follow
```

### `audit stats`

查看审计统计。

### `audit graph`

查看审计图结构输出。

### `audit export`

导出审计数据。

```bash
ai-cli audit export --output /tmp/audit.json
```

### `audit clear`

清空审计日志。

## A2A 命令

### `a2a`

把任务分配给一个本地子 agent。

```bash
ai-cli a2a --role researcher "Summarize the project structure"
```

### `a2a-batch`

从任务文件中并发执行多个 delegation 任务。

```bash
ai-cli a2a-batch --file tasks.json --concurrency 3
```
