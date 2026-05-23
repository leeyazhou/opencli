<template>
  <!-- 活动侧边栏：通过 width 动画折叠，保证折叠后不占据任何布局空间 -->
  <aside
    class="sidebar flex flex-col relative border-r border-black/5 dark:border-white/5 bg-sidebar select-none group h-full z-40 overflow-hidden"
    :style="{ width: sidebarOpen ? sidebarWidth + 'px' : '0px' }"
  >
    <!-- 侧边栏主面板内容区 -->
    <div class="flex flex-col overflow-hidden flex-1 min-h-0">
      <!-- 顶部的 Primary Action: 新建会话 -->
      <div class="p-3 border-b border-white/5 flex gap-2 flex-shrink-0">
        <button 
          @click="requestNewSession"
          class="flex-1 flex items-center justify-center gap-2 px-3 py-2 rounded-lg bg-primary hover:bg-primary-hover text-white text-xs font-semibold shadow-lg hover:shadow-primary/20 transition-all cursor-pointer"
        >
          <PlusIcon class="w-4 h-4" />
          <span>{{ t('sidebar.new_session') }}</span>
        </button>
      </div>

      <!-- 面板：历史会话列表 -->
      <div class="flex-1 flex flex-col overflow-hidden min-h-0">
        <!-- 会话列表标题 -->
        <div class="px-4 py-2.5 text-[10px] font-mono font-semibold text-slate-500 uppercase tracking-wider border-b border-black/5 dark:border-white/5 flex items-center justify-between">
          <span>{{ t('sidebar.recent') }}</span>
          <span class="px-1.5 py-0.5 rounded bg-black/5 dark:bg-white/5 text-[9px] text-slate-500 dark:text-slate-400 font-mono">{{ sessions.length }}</span>
        </div>

        <!-- 会话 Item 滚动区 -->
        <div class="flex-1 overflow-y-auto py-2 flex flex-col gap-1 custom-scrollbar">
          <!-- 单个会话项 -->
          <div 
            v-for="s in sessions" 
            :key="s.id"
            @click="switchSession(s.id)"
            class="group/item flex items-center justify-between px-3 py-2 mx-2 rounded-lg cursor-pointer transition-all border border-transparent"
            :class="s.id === currentSessionId 
              ? 'bg-black/5 dark:bg-white/10 border-black/10 dark:border-white/10 shadow-sm' 
              : 'hover:bg-black/5 dark:hover:bg-white/5 hover:border-black/5 dark:hover:border-white/5'"
          >
            <div class="flex items-center gap-2.5 overflow-hidden flex-1">
              <MessageSquareIcon 
                class="w-3.5 h-3.5 flex-shrink-0"
                :class="s.id === currentSessionId ? 'text-primary' : 'text-slate-500 group-hover/item:text-slate-700 dark:group-hover/item:text-slate-300'"
              />
              <div class="flex flex-col overflow-hidden">
                <span 
                  class="text-xs truncate max-w-[130px] font-mono"
                  :class="s.id === currentSessionId ? 'text-slate-900 dark:text-slate-200 font-medium' : 'text-slate-600 dark:text-slate-400 group-hover/item:text-slate-800 dark:group-hover/item:text-slate-300'"
                >
                  {{ s.title || 'Untitled' }}
                </span>
                <span class="text-[9px] font-mono text-slate-500 dark:text-slate-400/50 group-hover/item:text-slate-600 dark:group-hover/item:text-slate-400">
                  {{ s.messageCount || 0 }} msgs · {{ shortId(s.id) }}
                </span>
              </div>
            </div>

            <!-- 删除按钮 -->
            <button 
              @click.stop="deleteSession(s.id)"
              class="opacity-0 group-hover/item:opacity-100 p-1 hover:bg-rose-500/20 hover:text-rose-400 rounded text-slate-500 transition-all cursor-pointer"
              title="Delete Session"
            >
              <Trash2Icon class="w-3.5 h-3.5" />
            </button>
          </div>

          <!-- 缺省态 -->
          <div v-if="sessions.length === 0" class="text-center py-8 text-xs font-mono text-slate-500">
            {{ t('sidebar.no_sessions') }}
          </div>
        </div>
      </div>

      <!-- 底部工作区路径与设置入口 -->
      <div class="p-3 border-t border-black/5 dark:border-white/5 bg-black/5 dark:bg-black/10 flex flex-col gap-2 flex-shrink-0">
        <div class="flex flex-col gap-1">
          <span class="text-[9px] font-mono font-semibold uppercase tracking-wider text-slate-500">{{ t('sidebar.cwd_label') }}</span>
          <div class="px-2 py-1 rounded bg-black/5 dark:bg-black/30 border border-black/5 dark:border-white/5 text-[10px] font-mono text-slate-500 dark:text-slate-400 break-all select-all">
            {{ projectCwd }}
          </div>
        </div>

        <button 
          @click="openSettings"
          class="flex items-center justify-center gap-2 px-3 py-1.5 rounded bg-black/5 dark:bg-white/5 border border-black/5 dark:border-white/10 hover:bg-black/10 dark:hover:bg-white/10 text-xs font-mono text-slate-600 dark:text-slate-300 hover:text-slate-900 dark:hover:text-white transition-all cursor-pointer"
        >
          <SettingsIcon class="w-3.5 h-3.5" />
          <span>{{ t('settings.title') }}</span>
        </button>
      </div>
    </div>

    <!-- 侧边栏 Resize 物理拉伸手柄 -->
    <div 
      @mousedown="startResize"
      class="absolute top-0 right-0 w-[4px] h-full cursor-col-resize hover:bg-primary/50 transition-colors z-50 active:bg-primary"
      id="sidebarResizeHandle"
    ></div>
  </aside>
</template>

<script setup lang="ts">
import { ref, inject, onMounted, onUnmounted } from "vue";
import { useI18n } from "../../hooks/useI18n";
import type { StateService } from "../../services/StateService";
import type { ACPService } from "../../services/ACPService";
import { 
  PlusIcon, 
  MessageSquareIcon, 
  Trash2Icon, 
  SettingsIcon, 
  XIcon,
  XSquareIcon
} from "lucide-vue-next";

// 注入核心服务
const state = inject<StateService>("stateService")!;
const acp = inject<ACPService>("acpService")!;

// 订阅响应式翻译与语言
const { t, currentLang } = useI18n();

// 侧栏宽度与状态
const sidebarWidth = ref(240);
const sidebarOpen = ref(true);
const sessions = ref<any[]>([]);
const currentSessionId = ref<string | null>(null);
const projectCwd = ref("/");
const currentTheme = ref("");

// 新建会话
const requestNewSession = () => {
  state.emit("requestNewSession", {});
};

// 切换会话
const switchSession = (id: string) => {
  if (id !== currentSessionId.value) {
    state.emit("requestSwitchSession", id);
  }
};

// 删除会话
const deleteSession = (id: string) => {
  state.emit("requestDeleteSession", id);
};

// 触发全局设置面板
const openSettings = () => {
  state.emit("requestOpenSettings");
};

// 侧边栏拖拽 Resize 机制
const startResize = (e: MouseEvent) => {
  e.preventDefault();
  const startX = e.clientX;
  const startWidth = sidebarWidth.value;

  const onMouseMove = (moveEvt: MouseEvent) => {
    const diff = moveEvt.clientX - startX;
    // 限制宽度在 180px - 450px 之间
    sidebarWidth.value = Math.max(180, Math.min(450, startWidth + diff));
  };

  const onMouseUp = async () => {
    document.removeEventListener("mousemove", onMouseMove);
    document.removeEventListener("mouseup", onMouseUp);
    // 物理保存拉伸宽度
    await acp.storeSet("sidebarWidth", sidebarWidth.value);
  };

  document.addEventListener("mousemove", onMouseMove);
  document.addEventListener("mouseup", onMouseUp);
};

// 短化 ID
const shortId = (id: string) => {
  return id ? id.slice(0, 8) : "--";
};

// 拉取会话列表
const loadSessionList = async () => {
  try {
    const result = await acp.listSessions();
    sessions.value = result.sessions || [];
  } catch (err) {
    console.error("Sidebar 刷新会话历史失败:", err);
  }
};

let unsubscribers: (() => void)[] = [];

onMounted(async () => {
  // 只有当全局已连接初始化好时，挂载阶段才预载历史会话列表，否则静默等待 Agent 连接就绪事件广播
  if (state.get("initialized")) {
    await loadSessionList();
  }

  // 1. 同步物理拉伸宽度偏好
  const savedWidth = await acp.storeGet("sidebarWidth");
  if (savedWidth) {
    sidebarWidth.value = savedWidth;
  }

  // 2. 状态总线订阅与自动绑定
  unsubscribers.push(
    state.subscribe("sidebarOpen", (open) => {
      sidebarOpen.value = open;
    })
  );

  unsubscribers.push(
    state.subscribe("sessionId", (id) => {
      currentSessionId.value = id;
      loadSessionList(); // 自动拉取刷新列表状态
    })
  );

  unsubscribers.push(
    state.subscribe("selectedTheme", (theme) => {
      currentTheme.value = theme;
    })
  );

  // 监听重新刷新列表与 CWD 的广播
  unsubscribers.push(
    state.on("requestRefreshSessionList", async () => {
      await loadSessionList();
    })
  );

  unsubscribers.push(
    state.on("updateProjectCwd", (cwd: string) => {
      projectCwd.value = cwd;
    })
  );
});

onUnmounted(() => {
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
.bg-primary-hover:hover {
  background-color: #e62e2e;
}
.sidebar {
  transition: width 0.25s cubic-bezier(0.4, 0, 0.2, 1);
}

</style>
