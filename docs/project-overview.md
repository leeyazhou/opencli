# 项目概览

## 项目是什么

`opencli` 是一个代码导向的 AI 命令行工具。它的目标不是只做单轮问答，而是在终端环境里提供一套可控的工作流：模型回答、上下文注入、工具调用、审计记录、会话恢复，以及多 agent 协作。

## 核心能力

- 单次提示词执行
- `chat` 交互式会话
- `tui` 终端 UI 模式
- `run --file/--dir` 按文件或目录注入上下文
- 本地 session 保存、列出、恢复、删除、重命名
- 审计日志查询、导出、清理
- 本地 A2A 与并发 A2A batch delegation

## 支持的模型提供方

- `openai-compatible`
- `anthropic`

这意味着项目既可以连官方兼容 OpenAI API 的服务，也可以接入 Anthropic 风格接口。

## 内置工具

模型可以根据当前策略调用以下工具：

- `read_file`
- `list_dir`
- `search_files`
- `run_shell`
- `delegate_agent`
- `delegate_agents`

## 为什么这个项目值得单独写文档

相较于纯聊天 CLI，`opencli` 有几个更偏工程化的特点：

- workspace crate 边界明确，便于扩展 provider、工具、输出方式和审批策略
- 本地文件系统与 shell 操作受到 workspace 和策略限制
- 所有工具调用都可以进入审计日志，便于追踪与排障
- A2A 能力让一个任务可以拆分给多个子 agent 处理

## 代码结构概览

项目围绕以下 workspace crates 组织：

- `opencli`: CLI 入口、参数解析、shell completions
- `opencli-core`: 应用编排、运行时装配、agent 循环、TUI、A2A
- `opencli-provider`: provider 抽象、消息模型、provider 实现与 provider 选择
- `opencli-tools`: 通用工具、工具注册、shell/path 安全策略
- `opencli-config`: 配置加载、默认值、环境变量覆盖
- `opencli-audit`: 审计日志读写与查询
- `opencli-session`: 会话持久化
- `opencli-output`: 终端输出与 markdown 渲染

更详细的分层说明见 [架构设计](./architecture.md)。
