<template>
  <Transition name="fade">
    <!-- 全局半透明遮罩背景 -->
    <div
      v-if="isOpen"
      ref="overlayRef"
      class="fixed inset-0 z-50 flex items-start justify-center pt-[15vh] px-4 bg-slate-950/60 backdrop-blur-sm"
      @click="handleOverlayClick"
    >
      <Transition name="scale">
        <!-- 命令面板主体 -->
        <div
          class="w-full max-w-xl overflow-hidden border rounded-xl shadow-2xl bg-slate-900/80 border-slate-700/50 backdrop-blur-xl shadow-indigo-500/5"
        >
          <!-- 输入搜索区域 -->
          <div class="flex items-center px-4 py-3 border-b border-slate-700/40">
            <svg
              class="w-5 h-5 mr-3 text-slate-400"
              xmlns="http://www.w3.org/2000/svg"
              fill="none"
              viewBox="0 0 24 24"
              stroke="currentColor"
            >
              <path
                stroke-linecap="round"
                stroke-linejoin="round"
                stroke-width="2"
                d="M21 21l-6-6m2-5a7 7 0 11-14 0 7 7 0 0114 0z"
              />
            </svg>
            <input
              ref="inputRef"
              v-model="searchQuery"
              type="text"
              class="w-full text-base bg-transparent border-none outline-none text-slate-100 placeholder-slate-400 focus:ring-0"
              :placeholder="t('palette.placeholder') || 'Type a command or search...'"
              @keydown="handleKeyDown"
            />
          </div>

          <!-- 命令列表区域 -->
          <div
            ref="listRef"
            class="max-h-[300px] overflow-y-auto p-2 space-y-0.5 custom-scrollbar"
          >
            <div
              v-for="(cmd, index) in filteredCommands"
              :key="cmd.actionName"
              :class="[
                'flex items-center justify-between px-3 py-2.5 rounded-lg cursor-pointer transition-all duration-150',
                activeIndex === index
                  ? 'bg-indigo-600/30 border-indigo-500/30 text-indigo-200 shadow-inner'
                  : 'text-slate-300 hover:bg-slate-800/40 hover:text-slate-100'
              ]"
              @click="executeCommand(cmd)"
              @mouseenter="activeIndex = index"
            >
              <div class="flex items-center space-x-3">
                <!-- 图标装饰，可以根据不同命令展示不同的视觉反馈 -->
                <div
                  :class="[
                    'flex items-center justify-center w-6 h-6 rounded-md',
                    activeIndex === index ? 'bg-indigo-500/20 text-indigo-400' : 'bg-slate-800 text-slate-400'
                  ]"
                >
                  <component :is="cmd.icon" class="w-3.5 h-3.5" />
                </div>
                <!-- 命令名称 (支持高亮匹配) -->
                <span class="text-sm font-medium" v-html="highlightText(cmd.label, searchQuery)" />
              </div>
              
              <!-- 快捷键徽章 -->
              <span
                v-if="cmd.key"
                class="px-1.5 py-0.5 text-xs font-mono rounded bg-slate-800 text-slate-400 border border-slate-700/30 group-hover:border-slate-600/50"
              >
                {{ cmd.key }}
              </span>
            </div>

            <!-- 空状态 -->
            <div
              v-if="filteredCommands.length === 0"
              class="py-8 text-center text-slate-500 text-sm"
            >
              {{ t('palette.no_results') || '未找到匹配的命令' }}
            </div>
          </div>
        </div>
      </Transition>
    </div>
  </Transition>
</template>

<script setup lang="ts">
import { ref, computed, watch, inject, onMounted, onUnmounted, nextTick } from "vue";
import { useI18n } from "../../hooks/useI18n";
import type { StateService } from "../../services/StateService";
import { 
  PlusCircle, 
  Sidebar, 
  Terminal, 
  Keyboard, 
  Trash2 
} from "lucide-vue-next";

// 注入 i18n
const { t } = useI18n();

// 注入全局 state
const state = inject<StateService>("stateService")!;

const isOpen = ref(false);
const searchQuery = ref("");
const activeIndex = ref(0);

const overlayRef = ref<HTMLElement | null>(null);
const inputRef = ref<HTMLInputElement | null>(null);
const listRef = ref<HTMLElement | null>(null);

// 监听 State 中 activePanel 的状态来决定是否开启命令面板
onMounted(() => {
  isOpen.value = state.get("activePanel") === "palette";
  
  state.subscribe("activePanel", (val) => {
    isOpen.value = val === "palette";
  });
});

// 当打开面板时，重置并聚焦输入框
watch(isOpen, async (newVal) => {
  if (newVal) {
    searchQuery.value = "";
    activeIndex.value = 0;
    await nextTick();
    if (inputRef.value) {
      inputRef.value.focus();
    }
  }
});

// 动态构建支持的命令行结构
const commands = computed(() => [
  {
    label: t("palette.cmd.new_session") || "新建会话",
    key: "Cmd+N",
    actionName: "new_session",
    icon: PlusCircle,
    action: () => state.emit("requestNewSession"),
  },
  {
    label: t("palette.cmd.toggle_sidebar") || "切换侧边栏",
    key: "Cmd+B",
    actionName: "toggle_sidebar",
    icon: Sidebar,
    action: () => state.emit("toggleSidebarHotKey"),
  },
  {
    label: t("palette.cmd.toggle_terminal") || "开关日志终端",
    key: "",
    actionName: "toggle_terminal",
    icon: Terminal,
    action: () => {
      const open = state.get("terminalPaneOpen");
      state.set("terminalPaneOpen", !open);
    },
  },
  {
    label: t("palette.cmd.focus_input") || "聚焦输入舱",
    key: "Cmd+L",
    actionName: "focus_input",
    icon: Keyboard,
    action: () => state.emit("focusInputHotKey"),
  },
  {
    label: t("palette.cmd.clear_chat") || "清空当前聊天屏",
    key: "",
    actionName: "clear_chat",
    icon: Trash2,
    action: () => {
      const chatMsgEl = document.querySelector("#chatMessages");
      if (chatMsgEl) {
        chatMsgEl.innerHTML = "";
        state.emit("requestRefreshSessionList");
      }
      console.log("已清空对话屏");
    },
  },
]);

// 过滤后的指令列表
const filteredCommands = computed(() => {
  const query = searchQuery.value.trim().toLowerCase();
  if (!query) return commands.value;
  return commands.value.filter(
    (c) =>
      c.label.toLowerCase().includes(query) ||
      c.actionName.toLowerCase().includes(query)
  );
});

// 监听过滤列表变化，防止指针索引溢出
watch(filteredCommands, (newFiltered) => {
  if (activeIndex.value >= newFiltered.length) {
    activeIndex.value = Math.max(0, newFiltered.length - 1);
  }
});

// 实体安全转义 HTML 并进行高亮匹配
const escapeHtml = (text: string): string => {
  const div = document.createElement("div");
  div.textContent = text;
  return div.innerHTML;
};

const highlightText = (text: string, query: string): string => {
  if (!query) return escapeHtml(text);
  const matchIdx = text.toLowerCase().indexOf(query.toLowerCase());
  if (matchIdx >= 0) {
    return (
      escapeHtml(text.slice(0, matchIdx)) +
      '<span class="text-indigo-400 font-bold">' +
      escapeHtml(text.slice(matchIdx, matchIdx + query.length)) +
      "</span>" +
      escapeHtml(text.slice(matchIdx + query.length))
    );
  }
  return escapeHtml(text);
};

// 键盘按键管理
const handleKeyDown = (e: KeyboardEvent) => {
  if (e.key === "Escape") {
    closePalette();
  } else if (e.key === "Enter") {
    e.preventDefault();
    const selectedCmd = filteredCommands.value[activeIndex.value];
    if (selectedCmd) {
      executeCommand(selectedCmd);
    }
  } else if (e.key === "ArrowDown") {
    e.preventDefault();
    moveSelection(1);
  } else if (e.key === "ArrowUp") {
    e.preventDefault();
    moveSelection(-1);
  }
};

// 移动光标选择
const moveSelection = (direction: number) => {
  const total = filteredCommands.value.length;
  if (total === 0) return;
  activeIndex.value = (activeIndex.value + direction + total) % total;
  
  // 智能随动滚动
  nextTick(() => {
    if (listRef.value) {
      const items = listRef.value.children;
      const activeEl = items[activeIndex.value] as HTMLElement;
      if (activeEl) {
        activeEl.scrollIntoView({ block: "nearest" });
      }
    }
  });
};

// 执行具体命令
const executeCommand = (cmd: any) => {
  cmd.action();
  closePalette();
};

// 关闭面板
const closePalette = () => {
  state.set("activePanel", "");
};

// 遮罩点击关闭
const handleOverlayClick = (e: MouseEvent) => {
  if (e.target === overlayRef.value) {
    closePalette();
  }
};
</script>

<style scoped>
/* 经典滚动条定制 */
.custom-scrollbar::-webkit-scrollbar {
  width: 5px;
}
.custom-scrollbar::-webkit-scrollbar-track {
  background: transparent;
}
.custom-scrollbar::-webkit-scrollbar-thumb {
  background: rgba(255, 255, 255, 0.1);
  border-radius: 4px;
}
.custom-scrollbar::-webkit-scrollbar-thumb:hover {
  background: rgba(255, 255, 255, 0.2);
}

/* 动效过渡 */
.fade-enter-active,
.fade-leave-active {
  transition: opacity 0.2s ease;
}
.fade-enter-from,
.fade-leave-to {
  opacity: 0;
}

.scale-enter-active,
.scale-leave-active {
  transition: transform 0.2s cubic-bezier(0.16, 1, 0.3, 1), opacity 0.2s ease;
}
.scale-enter-from,
.scale-leave-to {
  transform: scale(0.96);
  opacity: 0;
}
</style>
