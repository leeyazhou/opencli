/**
 * @file StateService.ts
 * @description 客户端渲染层全局状态管理服务，基于订阅-发布模式解耦组件间的通信与状态共享
 * @author Antigravity
 */

export interface AppState {
  sessionId: string | null;
  initialized: boolean;
  agentRunning: boolean;
  streamingContent: string;
  messages: any[];
  selectedModel: string;
  selectedTheme: string;
  sidebarOpen: boolean;
  activePanel: string;
  contextPaneOpen: boolean;
  terminalPaneOpen: boolean;
  language: "zh" | "en";
}

export type StateListener<T> = (value: T) => void;

/**
 * @class StateService
 * @description 渲染进程中央状态总线，用于管理 UI 状态并通知感兴趣的视图组件进行自我重绘
 */
export class StateService {
  private static instance: StateService | null = null;

  // 默认全局状态
  private state: AppState = {
    sessionId: null,
    initialized: false,
    agentRunning: false,
    streamingContent: "",
    messages: [],
    selectedModel: "Gemini 3.5 Flash (High)",
    selectedTheme: "dark",
    sidebarOpen: true,
    activePanel: "sessions",
    contextPaneOpen: true,
    terminalPaneOpen: false, // 嵌入式 Terminal 抽屉默认收起
    language: "zh",
  };

  // 状态属性监听器映射表
  private listeners: { [K in keyof AppState]?: Set<StateListener<AppState[K]>> } = {};

  // 通用自定义事件监听器，用于非常规状态事件（例如：新消息送达、清空会话、触发外部聚焦等）
  private customEventListeners: { [event: string]: Set<StateListener<any>> } = {};

  private constructor() {}

  /**
   * 获取全局唯一的状态服务实例
   */
  public static getInstance(): StateService {
    if (!StateService.instance) {
      StateService.instance = new StateService();
    }
    return StateService.instance;
  }

  /**
   * 获取当前全部的状态快照
   */
  public getState(): Readonly<AppState> {
    return this.state;
  }

  /**
   * 获取指定状态属性的值
   * @param key 状态属性名称
   */
  public get<K extends keyof AppState>(key: K): AppState[K] {
    return this.state[key];
  }

  /**
   * 更新指定状态属性，并通知所有的属性监听器
   * @param key 状态属性名称
   * @param value 新的状态值
   */
  public set<K extends keyof AppState>(key: K, value: AppState[K]): void {
    if (this.state[key] === value) return; // 状态无变化则跳过通知

    this.state[key] = value;
    const keyListeners = this.listeners[key];
    if (keyListeners) {
      keyListeners.forEach((cb) => {
        try {
          cb(value);
        } catch (err) {
          console.error(`属性 [${key}] 状态更新回调执行异常:`, err);
        }
      });
    }
  }

  /**
   * 订阅指定状态属性的变化
   * @param key 状态属性名称
   * @param callback 属性变化时的回调函数
   * @returns 取消订阅的注销函数
   */
  public subscribe<K extends keyof AppState>(key: K, callback: StateListener<AppState[K]>): () => void {
    if (!this.listeners[key]) {
      this.listeners[key] = new Set();
    }
    this.listeners[key]!.add(callback);

    // 订阅时立即使用当前值回调一次，便于组件初始化同步
    try {
      callback(this.state[key]);
    } catch (e) {
      console.error(`初始订阅 [${key}] 回调出错:`, e);
    }

    return () => {
      this.listeners[key]?.delete(callback);
    };
  }

  /**
   * 触发自定义的全局业务事件
   * @param event 事件名称
   * @param payload 事件载荷
   */
  public emit(event: string, payload?: any): void {
    const list = this.customEventListeners[event];
    if (list) {
      list.forEach((cb) => {
        try {
          cb(payload);
        } catch (err) {
          console.error(`自定义事件 [${event}] 执行异常:`, err);
        }
      });
    }
  }

  /**
   * 监听自定义的全局业务事件
   * @param event 事件名称
   * @param callback 事件触发时的回调函数
   * @returns 取消监听的注销函数
   */
  public on(event: string, callback: StateListener<any>): () => void {
    if (!this.customEventListeners[event]) {
      this.customEventListeners[event] = new Set();
    }
    this.customEventListeners[event].add(callback);
    return () => {
      this.customEventListeners[event]?.delete(callback);
    };
  }
}
