<template>
  <!-- 右侧上下文面板：通过 width 动画折叠 -->
  <aside
    class="context-pane flex flex-col w-[300px] border-l border-white/5 bg-[#0a0a14]/85 backdrop-blur-md select-none h-full z-40 relative overflow-hidden context-transition"
    :style="{ width: contextPaneOpen ? '300px' : '0px' }"
  >
    <!-- 三档高拟真极客 Tab 头部 -->
    <div class="flex border-b border-white/5 bg-black/10">
      <button 
        v-for="tab in tabs" 
        :key="tab.id"
        @click="activeTab = tab.id"
        class="flex-1 py-2 text-[10px] font-mono font-semibold tracking-wider transition-all relative border-r border-white/5 last:border-r-0 cursor-pointer"
        :class="activeTab === tab.id ? 'text-primary' : 'text-slate-500 hover:text-slate-300'"
      >
        <span>{{ t(tab.i18nKey) }}</span>
        <!-- 活跃下划线 -->
        <span 
          v-if="activeTab === tab.id" 
          class="absolute bottom-0 left-0 w-full h-[1.5px] bg-primary animate-in fade-in duration-200"
        ></span>
      </button>
    </div>

    <!-- 面板内容滚动区 -->
    <div class="flex-1 overflow-y-auto p-4 flex flex-col gap-4 custom-scrollbar">
      
      <!-- TAB 1: 概览 (Overview) -->
      <div v-if="activeTab === 'overview'" class="flex flex-col gap-4 animate-in fade-in duration-200">
        <!-- 风琴面板 1: Changed Files -->
        <div class="rounded-lg border border-white/5 bg-black/20 overflow-hidden">
          <button 
            @click="collapsedSections.changedFiles = !collapsedSections.changedFiles"
            class="w-full flex items-center justify-between px-3 py-2 bg-white/5 hover:bg-white/10 text-xs font-mono font-semibold text-slate-300 transition-colors cursor-pointer"
          >
            <span class="flex items-center gap-1.5">
              <FolderSyncIcon class="w-3.5 h-3.5 text-primary" />
              {{ t('context.section.changed_files') }}
            </span>
            <span class="flex items-center gap-1.5">
              <span class="text-[9px] px-1.5 py-0.5 rounded bg-white/10 text-slate-400">1</span>
              <ChevronDownIcon 
                class="w-3.5 h-3.5 opacity-60 transition-transform" 
                :class="{ '-rotate-90': collapsedSections.changedFiles }" 
              />
            </span>
          </button>

          <div v-show="!collapsedSections.changedFiles" class="p-2 flex flex-col gap-1 text-[11px] font-mono text-slate-400 border-t border-white/5 animate-in slide-in-from-top-1 duration-150">
            <div class="flex items-center justify-between p-1.5 rounded hover:bg-white/5 transition-colors cursor-pointer">
              <span class="truncate flex items-center gap-1.5">
                <FileCodeIcon class="w-3.5 h-3.5 text-amber-400" />
                desktop/src/renderer/App.vue
              </span>
              <span class="text-[9px] text-amber-500 uppercase font-semibold">modify</span>
            </div>
          </div>
        </div>

        <!-- 风琴面板 2: Artifacts -->
        <div class="rounded-lg border border-white/5 bg-black/20 overflow-hidden">
          <button 
            @click="collapsedSections.artifacts = !collapsedSections.artifacts"
            class="w-full flex items-center justify-between px-3 py-2 bg-white/5 hover:bg-white/10 text-xs font-mono font-semibold text-slate-300 transition-colors cursor-pointer"
          >
            <span class="flex items-center gap-1.5">
              <ShieldAlertIcon class="w-3.5 h-3.5 text-primary" />
              {{ t('context.section.artifacts') }}
            </span>
            <span class="flex items-center gap-1.5">
              <span class="text-[9px] px-1.5 py-0.5 rounded bg-white/10 text-slate-400">2</span>
              <ChevronDownIcon 
                class="w-3.5 h-3.5 opacity-60 transition-transform" 
                :class="{ '-rotate-90': collapsedSections.artifacts }" 
              />
            </span>
          </button>

          <div v-show="!collapsedSections.artifacts" class="p-2 flex flex-col gap-1 text-[11px] font-mono text-slate-400 border-t border-white/5 animate-in slide-in-from-top-1 duration-150">
            <div class="flex items-center justify-between p-1.5 rounded hover:bg-white/5 transition-colors cursor-pointer">
              <span class="truncate flex items-center gap-1.5">
                <FileTextIcon class="w-3.5 h-3.5 text-primary/80" />
                implementation_plan.md
              </span>
              <span class="text-[9px] text-emerald-500 font-semibold uppercase">ready</span>
            </div>
            <div class="flex items-center justify-between p-1.5 rounded hover:bg-white/5 transition-colors cursor-pointer">
              <span class="truncate flex items-center gap-1.5">
                <FileTextIcon class="w-3.5 h-3.5 text-primary/80" />
                task.md
              </span>
              <span class="text-[9px] text-emerald-500 font-semibold uppercase">ready</span>
            </div>
          </div>
        </div>

        <!-- 扁平面板 3: Subagents (活跃子代理) -->
        <div class="rounded-lg border border-white/5 bg-black/20 overflow-hidden">
          <div class="flex items-center justify-between px-3 py-2 bg-white/5 text-xs font-mono font-semibold text-slate-300 border-b border-white/5">
            <span class="flex items-center gap-1.5">
              <UsersIcon class="w-3.5 h-3.5 text-primary" />
              {{ t('context.section.subagents') }}
            </span>
            <span class="text-[9px] px-1.5 py-0.5 rounded bg-white/10 text-slate-400">0</span>
          </div>
          <div class="p-4 text-center text-[10px] font-mono text-slate-600">
            No active subagents running
          </div>
        </div>

        <!-- 扁平面板 4: Background Tasks (后台异步任务) -->
        <div class="rounded-lg border border-white/5 bg-black/20 overflow-hidden">
          <div class="flex items-center justify-between px-3 py-2 bg-white/5 text-xs font-mono font-semibold text-slate-300 border-b border-white/5">
            <span class="flex items-center gap-1.5">
              <PlayIcon class="w-3.5 h-3.5 text-primary" />
              {{ t('context.section.bg_tasks') }}
            </span>
            <span class="text-[9px] px-1.5 py-0.5 rounded bg-white/10 text-slate-400">0</span>
          </div>
          <div class="p-4 text-center text-[10px] font-mono text-slate-600">
            No background tasks executing
          </div>
        </div>
      </div>

      <!-- TAB 2: 优化分析 (Optimization) -->
      <div v-else-if="activeTab === 'optimization'" class="flex flex-col gap-3 animate-in fade-in duration-200">
        <h4 class="text-xs font-mono font-semibold text-slate-300">{{ t('context.tabs.optimization') }}</h4>
        <p class="text-[11px] font-mono text-slate-400 leading-relaxed bg-black/20 p-3 rounded-lg border border-white/5">
          Vue 3 渲染进程响应式状态已建立，多语言热刷新机制微秒级可用。所有 UI 组件均已模块化物理包隔离封装。
        </p>
      </div>

      <!-- TAB 3: 实施计划 (Plan) -->
      <div v-else-if="activeTab === 'plan'" class="flex flex-col gap-3 animate-in fade-in duration-200">
        <h4 class="text-xs font-mono font-semibold text-slate-300">{{ t('context.tabs.plan') }}</h4>
        <div class="flex flex-col gap-2">
          <div class="p-2.5 rounded-lg border border-primary/20 bg-primary/5 flex flex-col gap-1">
            <span class="text-[11px] font-mono font-semibold text-primary">Vue 3 + Tailwind 重塑</span>
            <span class="text-[10px] font-mono text-slate-400">正在重写 Vue 3 的 7 大组件包，并建立 app.ts 主装配</span>
          </div>
        </div>
      </div>
    </div>
  </aside>
</template>

<script setup lang="ts">
import { ref, inject, onMounted, onUnmounted } from "vue";
import { useI18n } from "../../hooks/useI18n";
import type { StateService } from "../../services/StateService";
import { 
  FolderSyncIcon, 
  ChevronDownIcon, 
  FileCodeIcon,
  ShieldAlertIcon,
  FileTextIcon,
  UsersIcon,
  PlayIcon
} from "lucide-vue-next";

// 注入服务
const state = inject<StateService>("stateService")!;

// 订阅响应式翻译
const { t } = useI18n();

// 响应式属性
const contextPaneOpen = ref(true);
const activeTab = ref("overview");

// 选项卡配置
const tabs = [
  { id: "overview", i18nKey: "context.tabs.overview" },
  { id: "optimization", i18nKey: "context.tabs.optimization" },
  { id: "plan", i18nKey: "context.tabs.plan" }
];

// 风琴面板折叠状态
const collapsedSections = ref({
  changedFiles: false,
  artifacts: false
});

let unsubscribe: (() => void) | null = null;

onMounted(() => {
  // 订阅 context 面板折叠状态
  unsubscribe = state.subscribe("contextPaneOpen", (open) => {
    contextPaneOpen.value = open;
  });
});

onUnmounted(() => {
  if (unsubscribe) {
    unsubscribe();
  }
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
.context-transition {
  transition: width 0.25s cubic-bezier(0.4, 0, 0.2, 1);
}
</style>
