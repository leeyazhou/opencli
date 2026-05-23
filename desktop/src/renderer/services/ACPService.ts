/**
 * @file ACPService.ts
 * @description 核心通信服务层，负责与底层的 Electron IPC Bridge (window.opencli) 进行所有 RPC 通信，并对外提供流式事件与状态分发
 * @author Antigravity
 */

import { marked } from "marked";
import DOMPurify from "dompurify";
import type { ACPSession, ACPInitializeResponse, ACPAgentStatus, ACPNotificationPayload, ACPLogPayload } from "../types/acp";

// 配置 marked 的默认解析选项
marked.setOptions({
  breaks: true,
  gfm: true,
});

export interface OpenCLIApi {
  initialize: () => Promise<ACPInitializeResponse>;
  newSession: () => Promise<ACPSession>;
  prompt: (sessionId: string, prompt: string, model: string) => Promise<void>;
  listSessions: () => Promise<import("../types/acp").ACPSessionListResponse>;
  loadSession: (id: string) => Promise<any>;
  deleteSession: (id: string) => Promise<void>;
  startAgent: () => Promise<boolean>;
  agentStatus: () => Promise<ACPAgentStatus>;
  storeGet: (key: string) => Promise<any>;
  storeSet: (key: string, value: any) => Promise<boolean>;
  onNotification: (callback: (data: ACPNotificationPayload) => void) => () => void;
  onLog: (callback: (data: ACPLogPayload) => void) => () => void;
  onRaw: (callback: (data: any) => void) => () => void;
}


// 确保 TypeScript 知道 window 上的 opencli 属性
declare global {
  interface Window {
    opencli: OpenCLIApi;
  }
}

export type ACPNotificationCallback = (data: ACPNotificationPayload) => void;
export type ACPLogCallback = (data: ACPLogPayload) => void;

/**
 * @class ACPService
 * @description ACP 代理通信服务，采用单例模式管理 Electron 渲染进程与 Rust 后端进程的 IPC 交互
 */
export class ACPService {
  private static instance: ACPService | null = null;

  private notificationCallbacks: Set<ACPNotificationCallback> = new Set();
  private logCallbacks: Set<ACPLogCallback> = new Set();
  private isListening = false;

  private constructor() {
    this.setupListeners();
  }

  /**
   * 获取 ACP 服务的唯一单例实例
   */
  public static getInstance(): ACPService {
    if (!ACPService.instance) {
      ACPService.instance = new ACPService();
    }
    return ACPService.instance;
  }

  /**
   * 注册底层 IPC 监听器，接收来自主进程的分发事件
   */
  private setupListeners(): void {
    if (this.isListening) return;

    if (window.opencli) {
      // 监听后端通知（比如流式消息 Chunk、Thought 思考块、Tool 调用状态）
      window.opencli.onNotification((data: ACPNotificationPayload) => {
        this.notificationCallbacks.forEach((cb) => {
          try {
            cb(data);
          } catch (err) {
            console.error("执行 Notification 回调出错:", err);
          }
        });
      });

      // 监听后端日志
      window.opencli.onLog((data: ACPLogPayload) => {
        console.log(`[agent:${data.level}]`, data.text);
        this.logCallbacks.forEach((cb) => {
          try {
            cb(data);
          } catch (err) {
            console.error("执行 Log 回调出错:", err);
          }
        });
      });

      // 监听原始数据包并打印
      window.opencli.onRaw((data: any) => {
        console.log("[agent:raw]", data);
      });

      this.isListening = true;
    } else {
      console.error("window.opencli 桥接不可用，请确认 preload 加载正确。");
    }
  }

  /**
   * 注册 session 状态通知监听器
   * @param callback 接收通知的回调函数
   * @returns 取消监听的注销函数
   */
  public subscribeNotification(callback: ACPNotificationCallback): () => void {
    this.notificationCallbacks.add(callback);
    return () => {
      this.notificationCallbacks.delete(callback);
    };
  }

  /**
   * 注册日志通知监听器
   * @param callback 接收日志的回调函数
   * @returns 取消监听的注销函数
   */
  public subscribeLog(callback: ACPLogCallback): () => void {
    this.logCallbacks.add(callback);
    return () => {
      this.logCallbacks.delete(callback);
    };
  }

  /**
   * 初始化 ACP 后端代理
   */
  public async initializeAgent(): Promise<ACPInitializeResponse> {
    return await window.opencli.initialize();
  }

  /**
   * 新建会话
   */
  public async newSession(): Promise<ACPSession> {
    return await window.opencli.newSession();
  }

  /**
   * 发送 Prompt 请求
   * @param sessionId 会话ID
   * @param prompt 用户输入的 prompt 文本
   * @param model 选中的模型名称
   */
  public async prompt(sessionId: string, prompt: string, model: string): Promise<void> {
    return await window.opencli.prompt(sessionId, prompt, model);
  }

  /**
   * 获取所有本地持久化会话列表
   */
  public async listSessions(): Promise<import("../types/acp").ACPSessionListResponse> {
    return await window.opencli.listSessions();
  }

  /**
   * 加载指定 ID 的会话
   * @param sessionId 会话ID
   */
  public async loadSession(sessionId: string): Promise<any> {
    return await window.opencli.loadSession(sessionId);
  }

  /**
   * 删除指定 ID 的会话
   * @param sessionId 会话ID
   */
  public async deleteSession(sessionId: string): Promise<void> {
    return await window.opencli.deleteSession(sessionId);
  }

  /**
   * 启动 ACP 后端代理进程
   */
  public async startAgent(): Promise<boolean> {
    return await window.opencli.startAgent();
  }

  /**
   * 查询后端代理当前的存活状态
   */
  public async getAgentStatus(): Promise<ACPAgentStatus> {
    return await window.opencli.agentStatus();
  }

  /**
   * 从配置存储库中获取指定 Key 的配置
   */
  public async storeGet(key: string): Promise<any> {
    return await window.opencli.storeGet(key);
  }

  /**
   * 将配置存入本地持久化配置库
   */
  public async storeSet(key: string, value: any): Promise<boolean> {
    return await window.opencli.storeSet(key, value);
  }

  /**
   * 调用本地的渲染器将 Markdown 转换为带语法高亮的 HTML
   */
  public renderMarkdown(text: string): string {
    if (!text) return "";
    try {
      // 编译 Markdown 为 HTML 字符串
      const rawHtml = marked.parse(text) as string;
      // 对生成的 HTML 进行 XSS 安全过滤，防范注入漏洞
      return DOMPurify.sanitize(rawHtml, {
        ALLOWED_TAGS: [
          "p", "br", "strong", "em", "u", "s", "del",
          "h1", "h2", "h3", "h4", "h5", "h6",
          "ul", "ol", "li",
          "blockquote",
          "pre", "code",
          "a", "img",
          "table", "thead", "tbody", "tr", "th", "td",
          "hr",
          "span", "div",
        ],
        ALLOWED_ATTR: ["href", "src", "alt", "class", "target", "rel"],
        ALLOW_DATA_ATTR: false,
      });
    } catch (err) {
      console.error("Markdown 渲染出错，执行 escapeHtml 降级处理:", err);
      return this.escapeHtml(text);
    }
  }

  /**
   * 字符实体转义，用于防止 XSS 攻击的备用降级逻辑
   */
  private escapeHtml(text: string): string {
    return text
      .replace(/&/g, "&amp;")
      .replace(/</g, "&lt;")
      .replace(/>/g, "&gt;");
  }
}

