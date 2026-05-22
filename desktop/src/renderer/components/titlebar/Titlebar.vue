<template>
  <header class="titlebar drag-area flex items-center h-[38px] border-b border-white/5 bg-[#0a0a14]/65 backdrop-blur-md select-none relative z-50">

    <!-- 左侧：macOS 交通灯占位区 + 左侧栏折叠按钮 + 前进后退 -->
    <div class="flex items-center gap-1 pl-[80px] flex-shrink-0 no-drag-area">
      <!-- 左侧栏折叠开关（紧跟交通灯右侧） -->
      <button
        @click="toggleSidebar"
        class="p-1 hover:bg-white/5 rounded text-slate-400 hover:text-white transition-all"
        :class="{ 'text-primary bg-primary/10 border border-primary/20': sidebarOpen }"
        :title="t('palette.cmd.toggle_sidebar')"
      >
        <SidebarIcon class="w-4 h-4" />
      </button>

      <!-- 历史导航 前进 / 后退 -->
      <button
        @click="navigateBack"
        :disabled="backStack.length === 0"
        class="p-1 text-slate-400 hover:text-white disabled:opacity-20 disabled:hover:text-slate-400 transition-colors"
        :title="t('titlebar.back') || 'Back'"
      >
        <ChevronLeftIcon class="w-4 h-4 stroke-[2.5]" />
      </button>
      <button
        @click="navigateForward"
        :disabled="forwardStack.length === 0"
        class="p-1 text-slate-400 hover:text-white disabled:opacity-20 disabled:hover:text-slate-400 transition-colors"
        :title="t('titlebar.forward') || 'Forward'"
      >
        <ChevronRightIcon class="w-4 h-4 stroke-[2.5]" />
      </button>
    </div>

    <!-- 中间：可拖拽区域 + 会话标题（绝对居中） -->
    <div class="flex-1 drag-area h-full"></div>
    <div class="absolute left-1/2 -translate-x-1/2 text-xs font-mono font-semibold tracking-wider text-slate-300 flex items-center gap-1.5 pointer-events-none">
      <span class="px-1.5 py-0.5 rounded bg-white/5 border border-white/10 text-slate-400 text-[10px]">opencli</span>
      <span class="text-slate-500">/</span>
      <span class="max-w-[200px] truncate text-slate-200">{{ currentSessionTitle }}</span>
    </div>

    <!-- 右侧：状态灯 + 日志抽屉 + 右侧面板开关 -->
    <div class="flex items-center gap-2 pr-4 flex-shrink-0 no-drag-area">

      <!-- 在线状态指示灯 -->
      <div
        class="flex items-center gap-1.5 px-2.5 py-0.5 rounded-full border text-[10px] font-mono transition-all"
        :class="statusClasses[agentStatus]"
      >
        <span class="w-1.5 h-1.5 rounded-full animate-pulse" :class="statusDotClasses[agentStatus]"></span>
        {{ agentStatusLabel }}
      </div>

      <!-- 日志抽屉开关 -->
      <button
        @click="toggleTerminal"
        class="p-1 hover:bg-white/5 rounded text-slate-400 hover:text-white transition-all"
        :class="{ 'text-primary bg-primary/10 border border-primary/20': terminalPaneOpen }"
        :title="t('titlebar.button.terminal')"
      >
        <TerminalIcon class="w-4 h-4" />
      </button>

      <!-- 右侧上下文面板开关 -->
      <button
        @click="toggleContext"
        class="p-1 hover:bg-white/5 rounded text-slate-400 hover:text-white transition-all"
        :class="{ 'text-primary bg-primary/10 border border-primary/20': contextPaneOpen }"
        :title="t('titlebar.button.context')"
      >
        <LayoutDashboardIcon class="w-4 h-4" />
      </button>
    </div>
  </header>
</template>

<script setup lang="ts">
import { ref, inject, onMounted, onUnmounted } from "vue";
import { useI18n } from "../../hooks/useI18n";
import type { StateService } from "../../services/StateService";
import type { ACPService } from "../../services/ACPService";
import {
  ChevronLeftIcon,
  ChevronRightIcon,
  SidebarIcon,
  TerminalIcon,
  LayoutDashboardIcon
} from "lucide-vue-next";

// 注入核心服务
const state = inject<StateService>("stateService")!;
const acp = inject<ACPService>("acpService")!;

// 订阅响应式 i18n
const { t } = useI18n();

// 响应式核心状态
const currentSessionTitle = ref("New Session");
const sidebarOpen = ref(true);
const contextPaneOpen = ref(true);
const terminalPaneOpen = ref(false);

const agentStatus = ref<"online" | "offline" | "loading" | "error">("offline");
const agentStatusLabel = ref("Offline");

// 在线状态指示灯 CSS 映射
const statusClasses = {
  online:  "bg-emerald-500/10 border-emerald-500/20 text-emerald-400",
  loading: "bg-amber-500/10 border-amber-500/20 text-amber-400",
  error:   "bg-rose-500/10 border-rose-500/20 text-rose-400",
  offline: "bg-slate-500/10 border-slate-500/20 text-slate-400"
};

const statusDotClasses = {
  online:  "bg-emerald-400",
  loading: "bg-amber-400",
  error:   "bg-rose-400",
  offline: "bg-slate-400"
};

// 历史导航物理栈
const backStack = ref<string[]>([]);
const forwardStack = ref<string[]>([]);
let isNavigating = false;

// 折叠开关
const toggleSidebar = () => {
  sidebarOpen.value = !sidebarOpen.value;
  state.set("sidebarOpen", sidebarOpen.value);
};

const toggleContext = () => {
  contextPaneOpen.value = !contextPaneOpen.value;
  state.set("contextPaneOpen", contextPaneOpen.value);
};

const toggleTerminal = () => {
  terminalPaneOpen.value = !terminalPaneOpen.value;
  state.set("terminalPaneOpen", terminalPaneOpen.value);
};

// 历史导航
const navigateBack = () => {
  if (backStack.value.length === 0) return;
  const cur = state.get("sessionId");
  const prev = backStack.value.pop();
  if (prev) {
    isNavigating = true;
    if (cur) forwardStack.value.push(cur);
    state.emit("requestSwitchSession", prev);
    isNavigating = false;
  }
};

const navigateForward = () => {
  if (forwardStack.value.length === 0) return;
  const cur = state.get("sessionId");
  const next = forwardStack.value.pop();
  if (next) {
    isNavigating = true;
    if (cur) backStack.value.push(cur);
    state.emit("requestSwitchSession", next);
    isNavigating = false;
  }
};

let unsubscribers: (() => void)[] = [];

onMounted(() => {
  unsubscribers.push(
    state.subscribe("sidebarOpen", (v) => { sidebarOpen.value = v; })
  );
  unsubscribers.push(
    state.subscribe("contextPaneOpen", (v) => { contextPaneOpen.value = v; })
  );
  unsubscribers.push(
    state.subscribe("terminalPaneOpen", (v) => { terminalPaneOpen.value = v; })
  );
  unsubscribers.push(
    state.subscribe("agentRunning", (running) => {
      agentStatus.value = running ? "online" : "offline";
      agentStatusLabel.value = running
        ? t("titlebar.agent.connected")
        : t("titlebar.agent.offline");
    })
  );

  state.on("requestUpdateAgentStatus", (data: { status: "online" | "offline" | "loading" | "error"; label: string }) => {
    agentStatus.value = data.status;
    agentStatusLabel.value = data.label;
  });

  unsubscribers.push(
    state.subscribe("sessionId", (sessionId) => {
      // 更新标题
      const s = state.getState() as any;
      const active = s.sessions?.find((x: any) => x.id === sessionId);
      currentSessionTitle.value = active?.title || "New Session";

      if (isNavigating || !sessionId) return;
      const last = backStack.value[backStack.value.length - 1];
      if (last !== sessionId) {
        if (last) backStack.value.push(last);
        forwardStack.value = [];
      }
    })
  );
});

onUnmounted(() => {
  unsubscribers.forEach(un => un());
});
</script>

<style scoped>
.drag-area {
  -webkit-app-region: drag;
}
.no-drag-area {
  -webkit-app-region: no-drag;
}
</style>
