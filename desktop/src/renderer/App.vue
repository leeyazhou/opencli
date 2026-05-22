<template>
  <!-- 极客科技感环境光背景 -->
  <div class="bg-layer bg-grid"></div>
  <div class="bg-layer bg-gradient"></div>
  <ParticlesComponent />

  <!-- App shell (Antigravity 2.0 经典三栏毛玻璃布局) -->
  <div class="app-shell flex flex-col h-screen overflow-hidden relative select-none">
    <!-- 顶部完美对齐菜单标题栏 -->
    <TitlebarComponent />

    <!-- 底部主体区 -->
    <div class="app-body flex flex-1 overflow-hidden relative">
      <!-- 左侧边栏 -->
      <SidebarComponent />

      <!-- 中间主聊天区 -->
      <ChatComponent />

      <!-- 右侧上下文面板 -->
      <ContextComponent />
    </div>

    <!-- 嵌入式磨砂日志终端抽屉 -->
    <TerminalComponent />

    <!-- 全局快捷命令调色盘 -->
    <CommandPaletteComponent />
  </div>
</template>

<script setup lang="ts">
import { onMounted, onUnmounted } from "vue";
import { ACPService } from "./services/ACPService";
import { StateService } from "./services/StateService";
import { I18nService } from "./services/I18nService";

// 优雅地从各组件包统一入口导入 Vue 3 组件
import { TitlebarComponent } from "./components/titlebar";
import { SidebarComponent } from "./components/sidebar";
import { ChatComponent } from "./components/chat";
import { ContextComponent } from "./components/context";
import { TerminalComponent } from "./components/terminal";
import { CommandPaletteComponent } from "./components/command-palette";
import { ParticlesComponent } from "./components/particles";

// 直接获取底层单例服务，避免根组件 inject 为 undefined 的 Vue 限制
const acp = ACPService.getInstance();
const state = StateService.getInstance();
const i18n = I18nService.getInstance();


// 绑定全局状态事件总线
const bindStateEventBus = () => {
  // 监听并执行重新初始化 Agent 进程
  state.on("requestInitializeAgent", async () => {
    await initializeAgent();
  });

  // 全局系统级快捷键监听
  document.addEventListener("keydown", handleGlobalHotkeys);
};

const handleGlobalHotkeys = (e: KeyboardEvent) => {
  const mod = e.ctrlKey || e.metaKey;

  // Cmd+L / Ctrl+L 快速聚焦输入舱
  if (mod && e.key === "l") {
    e.preventDefault();
    state.emit("focusInputHotKey");
  }

  // Cmd+N / Ctrl+N 新建会话
  if (mod && e.key === "n") {
    e.preventDefault();
    state.emit("requestNewSession");
  }

  // Cmd+K / Ctrl+K 唤醒全局快捷命令面板
  if (mod && e.key === "k") {
    e.preventDefault();
    const currentPaletteOpen = state.get("activePanel") === "palette";
    state.set("activePanel", currentPaletteOpen ? "" : "palette");
  }
};

/**
 * 从本地持久化存储中读取并物理同步用户偏好设定
 */
const restorePreferences = async () => {
  try {
    // 1. 同步恢复推理模型
    const savedModel = await acp.storeGet("model");
    if (savedModel) {
      state.set("selectedModel", savedModel);
    }

    // 2. 同步恢复 UI 主题色彩
    const savedTheme = await acp.storeGet("theme");
    const activeTheme = savedTheme || "dark";
    state.set("selectedTheme", activeTheme);
    document.documentElement.setAttribute("data-theme", activeTheme);

    // 3. 同步恢复左侧栏折叠状态
    const savedSidebarOpen = await acp.storeGet("sidebarOpen");
    if (savedSidebarOpen !== undefined && savedSidebarOpen !== null) {
      state.set("sidebarOpen", savedSidebarOpen);
    }

    // 4. 同步恢复语言偏好设定 (默认 zh 中文)
    const savedLanguage = await acp.storeGet("language") as "zh" | "en" | undefined;
    const activeLang = savedLanguage || "zh";
    state.set("language", activeLang);
  } catch (err) {
    console.warn("读取本地配置偏好设定失败，这在初次启动时是正常的:", err);
  }
};

/**
 * 启动连接并连线 Rust ACP 后端子进程
 */
const connectAgent = async () => {
  try {
    const status = await acp.getAgentStatus();
    if (status.running) {
      state.set("agentRunning", true);
      await initializeAgent();
    } else {
      state.emit("requestUpdateAgentStatus", { status: "offline", label: i18n.t("titlebar.agent.starting") });
      await acp.startAgent();
      
      // 预留 500ms 的管道握手冷启动时间
      setTimeout(async () => {
        await initializeAgent();
      }, 500);
    }
  } catch (err) {
    console.error("Agent 引导启动流程异常，尝试强制拉起并降级建立连接:", err);
    try {
      await acp.startAgent();
      setTimeout(async () => {
        await initializeAgent();
      }, 800);
    } catch (innerErr) {
      state.emit("requestUpdateAgentStatus", { status: "error", label: "offline" });
      state.set("agentRunning", false);
      state.emit("addSystemMessage", i18n.t("chat.system.connecting_fail"));
    }
  }
};

/**
 * 带重试保护的异步方法包裹器 (指数退避算法)，针对外部代理程序建连的冷启动耗时进行安全自适应防崩溃保护
 */
const withRetry = async <T>(fn: () => Promise<T>, maxRetries = 5, initialDelay = 150): Promise<T> => {
  let attempt = 0;
  while (attempt < maxRetries) {
    try {
      return await fn();
    } catch (err: any) {
      attempt++;
      const errMsg = err && err.message ? String(err.message) : "";
      const isAgentNotRunning = errMsg.includes("Agent not running") || errMsg.includes("remote method");
      if (isAgentNotRunning && attempt < maxRetries) {
        const delay = initialDelay * Math.pow(2, attempt - 1);
        console.warn(`[Agent Connection] 后端代理暂未就绪 (重试尝试 ${attempt}/${maxRetries}), 将在 ${delay}ms 后自动重试...`);
        await new Promise((resolve) => setTimeout(resolve, delay));
      } else {
        throw err; // 其他实质性异常或已达重试上限则抛出
      }
    }
  }
  throw new Error("连接后端代理超时，重试上限已满");
};

/**
 * 执行 ACP 握手初始化连接
 */
const initializeAgent = async () => {
  try {
    state.emit("requestUpdateAgentStatus", { status: "loading", label: i18n.t("titlebar.agent.connecting") });
    
    // 1. 进行握手与协议初始化（包含指数退避安全退路保护）
    const initResult = await withRetry(() => acp.initializeAgent());
    console.log("ACP 协议初始化握手成功返回:", initResult);

    // 2. 优先恢复最近的历史会话，无历史时才创建新会话，避免每次启动都新建会话
    let sessionId: string | null = null;
    let sessionCwd = "/";
    try {
      const listResult = await acp.listSessions();
      const sessions: any[] = listResult.sessions || [];
      if (sessions.length > 0) {
        // 取最近一条（列表已按时间倒序排列）
        const latest = sessions[0];
        const loadResult = await acp.loadSession(latest.id);
        sessionId = latest.id;
        sessionCwd = loadResult.cwd || latest.cwd || "/";
        console.log("恢复最近历史会话:", sessionId);
      } else {
        // 无历史会话，新建一个
        const sessionResult = await acp.newSession();
        sessionId = sessionResult.sessionId || sessionResult.session_id;
        sessionCwd = sessionResult.cwd || "/";
        console.log("无历史会话，已新建:", sessionId);
      }
    } catch {
      // 拉取列表失败时降级新建
      const sessionResult = await acp.newSession();
      sessionId = sessionResult.sessionId || sessionResult.session_id;
      sessionCwd = sessionResult.cwd || "/";
    }

    // 3. 将状态流刷新，同步前端
    state.set("sessionId", sessionId);
    state.set("initialized", true);
    state.set("agentRunning", true);
    state.emit("requestUpdateAgentStatus", { status: "online", label: i18n.t("titlebar.agent.connected") });

    // 4. 通知侧边栏重新刷新历史会话列表，更新 CWD
    state.emit("requestRefreshSessionList", {});
    state.emit("updateProjectCwd", sessionCwd);

  } catch (err) {
    console.error("Agent 握手初始化失败:", err);
    state.emit("requestUpdateAgentStatus", { status: "error", label: "offline" });
    state.set("agentRunning", false);
    state.emit("addSystemMessage", i18n.t("chat.system.connecting_fail"));
  }
};

onMounted(async () => {
  bindStateEventBus();
  await restorePreferences();
  await connectAgent();
});

onUnmounted(() => {
  document.removeEventListener("keydown", handleGlobalHotkeys);
});
</script>

<style>
/* 经典磨砂玻璃与三栏定位 */
.bg-layer {
  position: absolute;
  top: 0;
  left: 0;
  width: 100%;
  height: 100%;
  z-index: 0;
  pointer-events: none;
}
.bg-grid {
  background-image: linear-gradient(rgba(255, 255, 255, 0.03) 1px, transparent 1px),
                    linear-gradient(90deg, rgba(255, 255, 255, 0.03) 1px, transparent 1px);
  background-size: 20px 20px;
}
.bg-gradient {
  background: radial-gradient(circle at 50% 50%, rgba(99, 102, 241, 0.05), transparent 80%),
              radial-gradient(circle at 10% 20%, rgba(255, 51, 51, 0.03), transparent 50%);
}
.app-shell {
  z-index: 10;
}
</style>
