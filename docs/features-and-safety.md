# 能力与安全边界

## 内置能力

`opencli` 不只是把 prompt 发给模型，它还支持在本地工作区内执行受控工具调用。

当前内置工具包括：

- `read_file`
- `list_dir`
- `search_files`
- `run_shell`
- `delegate_agent`
- `delegate_agents`

## 安全边界

### 工作区限制

文件系统访问和 shell 执行都应受 `workspaceRoot` 约束，避免越界访问本地环境。

### 审批策略

shell 命令不是无条件执行的。系统会根据 `approvalMode` 和 `nonInteractiveApproval` 判断是否允许执行。

### 危险命令拦截

即使审批策略允许，明显危险的命令也会被阻止，例如：

- `sudo`
- `rm -rf /`
- `mkfs`
- `shutdown`
- `reboot`

### 超时控制

shell 命令超过 `shellTimeoutMs` 会被终止，请求本身则受 `requestTimeoutMs` 控制。

## 审计能力

所有工具执行过程都可以写入 `auditLogPath`，用于：

- 回看某次工具调用做了什么
- 排查失败任务的上下文
- 统计常用工具和事件类型
- 导出审计数据做进一步分析

常用命令：

```bash
opencli audit list --limit 50
opencli audit tail --lines 20 --follow
opencli audit export --output /tmp/audit.json
```

## A2A 的边界

本项目支持本地 agent-to-agent delegation，但其行为同样受配置约束：

- `a2aEnabled`
- `a2aMaxDepth`
- `a2aMaxConcurrency`
- `allowedToolKinds`
- `allowedTools`

这意味着 A2A 不是无限制的多层自主执行，而是一个在本地可控、可追踪的 delegation 机制。
