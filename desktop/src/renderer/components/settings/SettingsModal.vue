<template>
  <div v-if="settingsVisible"
       class="fixed inset-0 bg-black/40 z-[999] flex items-center justify-center transition-all duration-300 animate-in fade-in select-none">
    
    <!-- 弹窗容器：最大化双栏布局，白灰质感 -->
    <div class="w-[85vw] max-w-6xl h-[80vh] min-h-[600px] flex rounded-xl border border-panelBorder bg-panel shadow-2xl overflow-hidden animate-in scale-in duration-200">
      
      <!-- 左侧分类导航 -->
      <SettingsSidebar v-model:activeTab="activeTab" />

      <!-- 右侧内容区 -->
      <main class="flex-1 flex flex-col relative bg-panel">
        <!-- 顶部控制栏 (关闭按钮) -->
        <div class="absolute top-4 right-4 z-10">
          <button @click="close" class="p-1.5 hover:bg-black/5 dark:hover:bg-transparent dark:bg-white/10 rounded-md text-slate-500 dark:text-muted-foreground hover:text-slate-800 dark:hover:text-white transition-colors cursor-pointer">
            <XIcon class="w-5 h-5" />
          </button>
        </div>

        <!-- 滚动内容 -->
        <div class="flex-1 overflow-y-auto p-10 custom-scrollbar relative">
          <div class="max-w-2xl">
            <!-- 标题 -->
            <div class="mb-8">
              <h2 class="text-2xl font-semibold text-slate-900 dark:text-white mb-1">{{ activeTab }}</h2>
              <p class="text-sm text-slate-500 dark:text-muted-foreground">
                <template v-if="activeTab === 'Appearance'">Configure the agent's visual theme and display preferences.</template>
                <template v-else>Configurations for {{ activeTab }}.</template>
              </p>
            </div>

            <!-- 动态 Tab 路由内容 -->
            <component 
              :is="tabComponents[activeTab]" 
              v-if="tabComponents[activeTab]" 
              class="w-full"
            />
            
            <div v-else class="flex items-center justify-center h-40 border border-dashed border-black/10 dark:border-white/10 rounded-xl bg-black/5 dark:bg-transparent dark:bg-white/5 text-muted-foreground">
              Configurations for {{ activeTab }} will be implemented here.
            </div>

          </div>
        </div>
      </main>

    </div>
  </div>
</template>

<script setup lang="ts">
import { ref, inject, onMounted, onUnmounted, watch, computed } from 'vue';
import { XIcon } from 'lucide-vue-next';
import type { StateService } from '../../services/StateService';

// 导入拆分的子组件
import SettingsSidebar from './SettingsSidebar.vue';
import AppearanceTab from './tabs/AppearanceTab.vue';
import ModelsTab from './tabs/ModelsTab.vue';
import AccountTab from './tabs/AccountTab.vue';
import GeneralTab from './tabs/GeneralTab.vue';

const state = inject<StateService>('stateService')!;

// Props & Modal Visibility
const props = defineProps<{ modelValue: boolean }>();
const emit = defineEmits(['update:modelValue']);
const settingsVisible = ref(props.modelValue);

watch(() => props.modelValue, (v) => (settingsVisible.value = v));
watch(settingsVisible, (v) => emit('update:modelValue', v));

const close = () => {
  settingsVisible.value = false;
  state.emit('requestCloseSettings');
};

// UI States
const activeTab = ref('Appearance');

// 动态组件映射
const tabComponents: Record<string, any> = {
  'Appearance': AppearanceTab,
  'Models': ModelsTab,
  'Account': AccountTab,
  'App': GeneralTab,
};

// Global Event Listeners
onMounted(() => {
  const openHandler = () => (settingsVisible.value = true);
  const closeHandler = () => (settingsVisible.value = false);
  state.on('requestOpenSettings', openHandler);
  state.on('requestCloseSettings', closeHandler);

  onUnmounted(() => {
    state.off('requestOpenSettings', openHandler);
    state.off('requestCloseSettings', closeHandler);
  });
});
</script>

<style scoped>
.custom-scrollbar::-webkit-scrollbar {
  width: 6px;
}
.custom-scrollbar::-webkit-scrollbar-track {
  background: transparent;
}
.custom-scrollbar::-webkit-scrollbar-thumb {
  background: rgba(150, 150, 150, 0.2);
  border-radius: 99px;
}
.custom-scrollbar::-webkit-scrollbar-thumb:hover {
  background: rgba(150, 150, 150, 0.4);
}
</style>
