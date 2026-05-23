/**
 * @file acp.d.ts
 * @description ACP 协议 (Agent Communication Protocol) 前后端交互类型定义
 */

export interface ACPSession {
  id?: string;
  sessionId?: string;
  session_id?: string;
  title?: string | null;
  messageCount?: number;
  updatedAt?: string;
  cwd?: string;
}

export interface ACPSessionListResponse {
  sessions: ACPSession[];
}

export interface ACPMessage {
  id?: string;
  type?: 'text' | 'thought' | 'tool_call' | 'tool_result' | 'system';
  content?: string;
  role?: 'user' | 'agent' | 'system';
  [key: string]: any; // fallback for loosely typed properties
}

export interface ACPInitializeResponse {
  version: string;
  capabilities: Record<string, boolean>;
}

export interface ACPAgentStatus {
  running: boolean;
  pid: number | null;
}

export interface ACPNotificationPayload {
  type: string;
  sessionId?: string;
  update?: any;
  meta?: any;
}

export interface ACPLogPayload {
  level: 'debug' | 'info' | 'warn' | 'error';
  text: string;
}
