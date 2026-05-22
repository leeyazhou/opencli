<template>
  <div 
    v-show="terminalPaneOpen"
    class="terminal-pane fixed bottom-0 left-0 w-full h-[220px] border-t border-white/5 bg-[#06060c]/90 backdrop-blur-lg select-none z-50 flex flex-col transition-all duration-300 animate-in slide-in-from-bottom"
    id="terminalPane"
  >
    <!-- 终端标题栏 -->
    <div class="flex items-center justify-between px-4 py-1.5 border-b border-white/5 bg-black/30">
      <div class="flex items-center gap-2 text-xs font-mono font-bold tracking-wider text-slate-300">
        <TerminalIcon class="w-3.5 h-3.5 text-primary" />
        <span class="terminal-title">{{ t('terminal.title') }}</span>
      </div>

      <div class="flex items-center gap-1.5 no-drag-area">
        <!-- 一键清空 -->
        <button 
          @click="clearLogs"
          class="p-1 hover:bg-white/5 rounded text-slate-400 hover:text-white transition-colors cursor-pointer"
          :title="t('terminal.clear_tooltip')"
        >
          <Trash2Icon class="w-3.5 h-3.5" />
        </button>

        <!-- 关闭终端抽屉 -->
        <button 
          @click="closeTerminal"
          class="p-1 hover:bg-white/5 rounded text-slate-400 hover:text-white transition-colors cursor-pointer"
          :title="t('terminal.close_tooltip')"
        >
          <XIcon class="w-3.5 h-3.5" />
        </button>
      </div>
    </div>

    <!-- 终端日志内容滚动区 -->
    <div 
      ref="terminalContentRef"
      class="flex-1 overflow-y-auto p-3 font-mono text-[11px] leading-relaxed flex flex-col gap-1.5 custom-scrollbar"
    >
      <div 
        v-for="(log, idx) in logs" 
        :key="idx"
        class="flex flex-wrap items-center gap-1.5"
        :class="logClasses[log.type]"
      >
        <!-- 正常系统日志样式 -->
        <template v-if="log.type === 'system'">
          <span class="text-primary font-semibold">opencli %</span>
          <span class="text-slate-300">{{ log.text }}</span>
        </template>

        <!-- 工具 Trace 样式 -->
        <template v-else-if="log.type === 'tool-trace'">
          <span class="text-primary/80 font-bold flex items-center gap-1">
            <span>⚙️</span> [SYSTEM-TRACE]
          </span>
          <span class="text-slate-300">
            Tool call invoked: 
            <span class="text-primary font-semibold">{{ log.toolName }}</span> 
            with status 
            <span :class="statusClasses[log.status] || 'text-slate-400'">{{ log.status }}</span>
          </span>
        </template>

        <!-- ACP 运行日志样式 -->
        <template v-else>
          <span class="text-slate-500">[{{ log.time }}]</span>
          <span class="px-1.5 py-0.5 rounded text-[9px] font-semibold tracking-wider" :class="badgeClasses[log.type]">
            {{ log.type.toUpperCase() }}
          </span>
          <span class="text-slate-300 break-all select-text">{{ log.text }}</span>
        </template>
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref, inject, onMounted, onUnmounted, nextTick } from "vue";
import { useI18n } from "../../hooks/useI18n";
import type { StateService } from "../../services/StateService";
import type { ACPService } from "../../services/ACPService";
import { 
  TerminalIcon, 
  Trash2Icon, 
  XIcon 
} from "lucide-vue-next";

// 注入服务
const state = inject<StateService>("stateService")!;
const acp = inject<ACPService>("acpService")!;

// 订阅响应式 i18n
const { t } = useI18n();

// 引用与状态
const terminalContentRef = ref<HTMLElement | null>(null);
const terminalPaneOpen = ref(false);

interface LogEntry {
  type: "system" | "tool-trace" | "info" | "debug" | "warn" | "error";
  text?: string;
  time?: string;
  toolName?: string;
  status?: string;
}

const logs = ref<LogEntry[]>([]);

// 日志及状态高亮映射
const logClasses = {
  system: "text-slate-300",
  "tool-trace": "text-slate-300 border-l border-white/5 pl-2 py-0.5",
  info: "text-slate-300",
  debug: "text-slate-400/80",
  warn: "text-amber-400/90",
  error: "text-rose-400/90"
};

const badgeClasses = {
  info: "bg-emerald-500/10 text-emerald-400 border border-emerald-500/20",
  debug: "bg-slate-500/10 text-slate-400 border border-slate-500/20",
  warn: "bg-amber-500/10 text-amber-400 border border-amber-500/20",
  error: "bg-rose-500/10 text-rose-400 border border-rose-500/20"
};

const statusClasses: Record<string, string> = {
  running: "text-amber-400",
  completed: "text-emerald-400",
  failed: "text-rose-400",
  success: "text-emerald-400"
};

// 自动滚动到底部
const scrollToBottom = async () => {
  await nextTick();
  if (terminalContentRef.value) {
    terminalContentRef.value.scrollTop = terminalContentRef.value.scrollHeight;
  }
};

// 一键清空
const clearLogs = () => {
  logs.value = [];
  appendSystemLog(t("terminal.cleared"));
};

const closeTerminal = () => {
  state.set("terminalPaneOpen", false);
};

// 插入系统级日志
const appendSystemLog = (text: string) => {
  logs.value.push({
    type: "system",
    text
  });
  scrollToBottom();
};

const handleGlobalKeys = (e: KeyboardEvent) => {
  if (terminalPaneOpen.value) {
    const mod = e.ctrlKey || e.metaKey;
    // Ctrl+L 清空
    if (mod && e.key === "l") {
      e.preventDefault();
      clearLogs();
    }
  }
};

let unsubscribers: (() => void)[] = [];

onMounted(() => {
  // 插入迎宾欢迎词
  appendSystemLog(t("terminal.welcome"));

  document.addEventListener("keydown", handleGlobalKeys);

  // 1. 订阅全局终端折叠开合状态
  unsubscribers.push(
    state.subscribe("terminalPaneOpen", (open) => {
      terminalPaneOpen.value = open;
      if (open) {
        scrollToBottom();
      }
    })
  );

  // 2. 订阅底层流式运行日志
  acp.subscribeLog((logData) => {
    logs.value.push({
      type: (logData.level?.toLowerCase() || "info") as any,
      text: logData.text,
      time: new Date().toLocaleTimeString()
    });
    scrollToBottom();
  });

  // 3. 订阅工具 Trace 状态通知
  acp.subscribeNotification((data) => {
    const update = data.update || data;
    const kind = update.update || update.kind || "";

    if (kind === "tool_call" || kind === "ToolCall") {
      logs.value.push({
        type: "tool-trace",
        toolName: update.name || update.tool_name || "unknown",
        status: update.status || "running"
      });
      scrollToBottom();
    }
  });
});

onUnmounted(() => {
  document.removeEventListener("keydown", handleGlobalKeys);
  unsubscribers.forEach(un => un());
});
</script>

<style scoped>
.custom-scrollbar::-webkit-scrollbar {
  width: 4px;
}
.custom-scrollbar::-webkit-scrollbar-track {
  background: transparent;
}
.custom-scrollbar::-webkit-scrollbar-thumb {
  background: rgba(255, 255, 255, 0.05);
  border-radius: 99px;
}
.custom-scrollbar::-webkit-scrollbar-thumb:hover {
  background: rgba(255, 255, 255, 0.15);
}
</style>
