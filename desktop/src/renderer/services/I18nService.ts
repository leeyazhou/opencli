/**
 * @file I18nService.ts
 * @description 客户端轻量级多语言翻译服务，支持中英文即时无缝切换及动态词条插值
 * @author Antigravity
 */

import { StateService } from "./StateService";

export type Locale = "zh" | "en";

export interface TranslationDict {
  [key: string]: {
    zh: string;
    en: string;
  };
}

/**
 * @class I18nService
 * @description 多语言翻译单例服务，通过订阅全局 language 状态来实现文案的动态刷新
 */
export class I18nService {
  private static instance: I18nService | null = null;
  private stateService: StateService;

  // 全局词条对照字典
  private dictionary: TranslationDict = {
    // Titlebar
    "titlebar.agent.starting": { zh: "正在启动后端 ACP 舱...", en: "Starting backend agent..." },
    "titlebar.agent.connecting": { zh: "建立通道连接中...", en: "Connecting..." },
    "titlebar.agent.connected": { zh: "联机就绪", en: "Online Connected" },
    "titlebar.agent.offline": { zh: "未联机", en: "Offline" },
    "titlebar.button.terminal": { zh: "切换系统日志终端 (Ctrl+`)", en: "Toggle Trace Terminal (Ctrl+`)" },
    "titlebar.button.context": { zh: "切换右侧上下文面板", en: "Toggle Context Pane" },
    // Sidebar
    "sidebar.new_session": { zh: "新建会话", en: "New Session" },
    "sidebar.sessions_title": { zh: "会话历史", en: "Sessions" },
    "sidebar.no_sessions": { zh: "暂无历史会话", en: "No Sessions" },
    "sidebar.cwd_label": { zh: "当前工作区路径 (CWD)", en: "Workspace Path (CWD)" },
    "sidebar.model_label": { zh: "底层推理模型", en: "Inference Model" },
    // Chat
    "chat.input.placeholder": { zh: "向 OpenCLI 提问，输入 Cmd+L 快速聚焦输入舱...", en: "Ask OpenCLI anything, Cmd+L to focus input..." },
    "chat.welcome.title": { zh: "OpenCLI 极客桌面太空舱", en: "OpenCLI Intelligent Terminal Deck" },
    "chat.welcome.subtitle": { zh: "基于 Rust 异步高并发核心与沙箱安全机制，为您提供全自动化的代码调试、文件扫描及终端操作自动化代理。", en: "Empowered by Rust async runtime and strict path sandbox, providing full-automatic workspace scanning, CLI command execution and code fixing." },
    "chat.welcome.feature1": { zh: "🚀 自动执行 Shell 构建、安装与诊断", en: "🚀 Auto executing shell build, install and diagnosis" },
    "chat.welcome.feature2": { zh: "🔍 智能 Workspace 级语义代码检索", en: "🔍 Semantic workspace codebase search" },
    "chat.welcome.feature3": { zh: "🛡️ 全程沙箱路径检查与命令审批拦截", en: "🛡️ Sandboxed filesystem checks & shell approvals" },
    "chat.button.send": { zh: "发送", en: "Send" },
    "chat.system.connecting_fail": { zh: "未能与 ACP 后端代理建立通信，请确保 opencli 程序编译正常，并可顺利执行。", en: "Failed to communicate with ACP backend. Ensure opencli is compiled and executable." },
    // Context
    "context.tabs.overview": { zh: "环境概览", en: "Overview" },
    "context.tabs.optimization": { zh: "优化分析", en: "Optimization Analysis" },
    "context.tabs.plan": { zh: "实施计划", en: "Implementation Plan" },
    "context.tabs.env": { zh: "工作环境", en: "Workspace" },
    "context.tabs.changes": { zh: "文件变更", en: "Changes" },
    "context.tabs.tasks": { zh: "异步任务", en: "Tasks" },
    "context.section.changed_files": { zh: "已变更文件", en: "Changed Files" },
    "context.section.artifacts": { zh: "持久化产物", en: "Artifacts" },
    "context.section.subagents": { zh: "活跃子代理 (Subagents)", en: "Active Subagents" },
    "context.section.bg_tasks": { zh: "后台运行任务", en: "Background Tasks" },
    // Terminal
    "terminal.title": { zh: "系统执行日志控制台 (Trace Drawer)", en: "System Trace Log Drawer" },
    "terminal.welcome": { zh: "系统终端面板已连接。等待任务执行...", en: "System Terminal interface connected. Awaiting tasks..." },
    "terminal.cleared": { zh: "控制台已清空。", en: "Console cleared." },
    "terminal.clear_tooltip": { zh: "清空控制台 (Ctrl+L)", en: "Clear Console (Ctrl+L)" },
    "terminal.close_tooltip": { zh: "收起终端面板", en: "Close Shell Panel" },
    // Command Palette
    "palette.placeholder": { zh: "输入指令或进行快捷检索...", en: "Type command or quick search..." },
    "palette.cmd.new_session": { zh: "新建会话 (New Session)", en: "New Session" },
    "palette.cmd.toggle_sidebar": { zh: "折叠/展开左侧边栏 (Toggle Sidebar)", en: "Toggle Sidebar" },
    "palette.cmd.toggle_terminal": { zh: "折叠/展开执行终端 (Toggle Terminal)", en: "Toggle Trace Drawer" },
    "palette.cmd.focus_input": { zh: "聚焦 Prompt 输入舱 (Focus Input)", en: "Focus Prompt Input" },
    "palette.cmd.clear_chat": { zh: "清空当前聊天会话 (Clear Current Chat)", en: "Clear Current Chat Screen" },
    // Settings
    "settings.title": { zh: "设置", en: "System Settings" },
    "settings.language": { zh: "应用界面语言", en: "UI Language" },
    "settings.theme": { zh: "主题色彩倾向", en: "UI Theme Preference" },
    "settings.close": { zh: "保存并关闭", en: "Save & Close" },
  };

  private constructor() {
    this.stateService = StateService.getInstance();
  }

  /**
   * 获取单例翻译服务实例
   */
  public static getInstance(): I18nService {
    if (!I18nService.instance) {
      I18nService.instance = new I18nService();
    }
    return I18nService.instance;
  }

  /**
   * 翻译指定词条的 Key，并支持大括号变量插值
   * @param key 词条键
   * @param variables 插值变量，如 {name}
   */
  public t(key: string, variables?: Record<string, string>): string {
    const locale = (this.stateService.get("language") as Locale) || "zh";
    const term = this.dictionary[key];
    if (!term) return key;

    let text = term[locale] || term["zh"] || key;

    if (variables) {
      Object.entries(variables).forEach(([k, v]) => {
        text = text.replace(new RegExp(`{${k}}`, "g"), v);
      });
    }

    return text;
  }
}
