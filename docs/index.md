# opencli 文档

`opencli` 是一个面向代码工作的 Rust CLI 工具，提供多模型接入、交互式对话、工具调用、审计日志、会话持久化，以及本地 agent-to-agent 协作能力。

这个 `docs/` 目录适合直接作为 GitHub Pages 的内容来源。

## 文档导航

- [项目概览](./project-overview.md)
- [快速开始](./getting-started.md)
- [命令参考](./commands.md)
- [配置说明](./configuration.md)
- [能力与安全边界](./features-and-safety.md)
- [架构设计](./architecture.md)

## 适用场景

- 在终端里直接向模型提问代码问题
- 对当前仓库做文件读取、搜索和命令执行
- 通过会话功能保存和恢复上下文
- 通过审计日志追踪工具调用过程
- 通过 A2A 模式把任务分发给本地子 agent

## 项目特点

- 使用 Rust 实现，命令行启动快，依赖边界清晰
- 支持 `openai-compatible` 和 `anthropic` 两类 provider
- 支持流式输出、聊天模式和基础 TUI 模式
- 内置文件读取、目录遍历、全文搜索、shell 执行等工具
- 对 shell 操作提供审批策略和危险命令拦截

如果你是第一次接触这个项目，建议从 [项目概览](./project-overview.md) 和 [快速开始](./getting-started.md) 开始阅读。
