# 配置说明

## 配置文件位置

默认配置文件：`~/.config/ai-cli/config.json`

可以先执行下面的命令生成模板：

```bash
ai-cli config init
```

## 常用字段

### 模型相关

- `provider`: 当前使用的 provider，例如 `openai-compatible` 或 `anthropic`
- `baseUrl`: provider 接口地址
- `apiKey`: 访问 API 所需凭据
- `model`: 默认模型名
- `temperature`: 生成温度
- `maxTokens`: 最大输出 token 数
- `requestTimeoutMs`: 请求超时

### shell 与审批相关

- `approvalMode`: 交互环境下的 shell 审批策略
- `nonInteractiveApproval`: 非交互环境下的 shell 策略
- `shellTimeoutMs`: shell 命令超时
- `workspaceRoot`: shell 与文件访问的工作区根目录

### 持久化相关

- `sessionDir`: 会话保存目录
- `auditLogPath`: 审计日志路径

### A2A 相关

- `a2aEnabled`: 是否启用 agent-to-agent delegation
- `a2aMaxDepth`: delegation 最大深度
- `a2aMaxConcurrency`: 最大并发子 agent 数

### 工具权限相关

- `allowedToolKinds`: 允许的工具类别
- `allowedTools`: 允许的具体工具名

## `approvalMode`

支持以下值：

- `on-write`: 读类 shell 直接执行，写类 shell 需要审批
- `always-ask`: 所有 shell 都需要审批
- `never-ask`: 不主动审批，但危险命令仍然会被拦截

## `nonInteractiveApproval`

支持以下值：

- `deny`
- `allow-read-only`
- `allow-all`

## `allowedToolKinds`

当前支持的工具类别：

- `filesystem-read`
- `filesystem-search`
- `shell`
- `agent`

当 `allowedTools` 为空数组时，表示在类别允许的前提下，不额外限制具体工具名。

## 一个完整示例

```json
{
  "provider": "openai-compatible",
  "baseUrl": "https://api.openai.com/v1",
  "apiKey": "",
  "model": "gpt-4.1",
  "anthropicVersion": "2023-06-01",
  "temperature": 0.2,
  "maxTokens": 4096,
  "approvalMode": "on-write",
  "nonInteractiveApproval": "deny",
  "workspaceRoot": ".",
  "sessionDir": "~/.config/ai-cli/sessions",
  "auditLogPath": "~/.config/ai-cli/audit.jsonl",
  "requestTimeoutMs": 120000,
  "shellTimeoutMs": 120000,
  "agentMaxSteps": 8,
  "a2aEnabled": true,
  "a2aMaxDepth": 2,
  "a2aMaxConcurrency": 4,
  "allowedToolKinds": [
    "filesystem-read",
    "filesystem-search",
    "shell",
    "agent"
  ],
  "allowedTools": []
}
```
