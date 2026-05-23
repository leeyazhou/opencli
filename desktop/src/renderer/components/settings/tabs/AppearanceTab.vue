<template>
  <div>
    <!-- Chat Settings Section -->
    <div class="mb-8">
      <h3 class="text-sm font-semibold text-slate-800 dark:text-foreground mb-3">Chat Settings</h3>
      <div class="rounded-xl border border-black/5 dark:border-white/10 bg-transparent dark:bg-white/50 dark:bg-transparent dark:bg-white/5 p-4 flex items-center justify-between">
        <div class="flex flex-col">
          <span class="text-sm font-medium text-slate-800 dark:text-foreground">Verbose agent chat</span>
          <span class="text-xs text-slate-500 dark:text-muted-foreground mt-0.5">Display and preserve intermediate thinking steps</span>
        </div>
        <!-- Switch Toggle -->
        <BaseSwitch v-model="verboseChat" />
      </div>
    </div>

    <!-- Appearance Section -->
    <div class="mb-8">
      <h3 class="text-sm font-semibold text-slate-800 dark:text-foreground mb-3">Appearance</h3>
      <div class="rounded-xl border border-black/5 dark:border-white/10 bg-transparent dark:bg-white/50 dark:bg-transparent dark:bg-white/5 p-4 flex items-center justify-between">
        <div class="flex flex-col">
          <span class="text-sm font-medium text-slate-800 dark:text-foreground">Appearance</span>
          <span class="text-xs text-slate-500 dark:text-muted-foreground mt-0.5">Select light, dark, or inherit system settings.</span>
        </div>
        <!-- Select Dropdown UI -->
        <div class="w-40">
          <BaseSelect 
            v-model="currentTheme" 
            :options="[
              { label: 'Light', value: 'light' },
              { label: 'Dark', value: 'dark' },
              { label: 'System', value: 'system' }
            ]" 
          />
        </div>
      </div>
    </div>

    <!-- Light Theme Config -->
    <div class="mb-8">
      <h3 class="text-sm font-semibold text-slate-800 dark:text-foreground mb-3">Light Theme</h3>
      <div class="rounded-xl border border-black/5 dark:border-white/10 bg-transparent dark:bg-white/50 dark:bg-transparent dark:bg-white/5 overflow-hidden">
        
        <div class="flex items-center justify-between p-3.5 border-b border-black/5 dark:border-border">
          <span class="text-sm text-slate-800 dark:text-foreground">Preset</span>
          <div class="w-40">
            <BaseSelect 
              modelValue="default"
              :options="[{ label: 'Default Light', value: 'default' }]"
            />
          </div>
        </div>

        <div class="flex items-center justify-between p-3.5 border-b border-black/5 dark:border-border">
          <span class="text-sm text-slate-800 dark:text-foreground">Background</span>
          <div class="flex items-center gap-2 bg-white dark:bg-[#222] border border-black/10 dark:border-white/10 rounded-md px-2 py-1 min-w-[120px]">
            <div class="w-4 h-4 rounded-sm border border-black/10 shadow-sm" style="background-color: #EEEEEE;"></div>
            <span class="text-xs font-mono text-slate-600 dark:text-muted-foreground uppercase">#EEEEEE</span>
          </div>
        </div>

        <div class="flex items-center justify-between p-3.5 border-b border-black/5 dark:border-border">
          <span class="text-sm text-slate-800 dark:text-foreground">Foreground</span>
          <div class="flex items-center gap-2 bg-white dark:bg-[#222] border border-black/10 dark:border-white/10 rounded-md px-2 py-1 min-w-[120px]">
            <div class="w-4 h-4 rounded-sm border border-black/10 shadow-sm" style="background-color: #101010;"></div>
            <span class="text-xs font-mono text-slate-600 dark:text-muted-foreground uppercase">#101010</span>
          </div>
        </div>

        <div class="flex items-center justify-between p-3.5">
          <span class="text-sm text-slate-800 dark:text-foreground">Accent</span>
          <div class="flex items-center gap-2 bg-white dark:bg-[#222] border border-black/10 dark:border-white/10 rounded-md px-2 py-1 min-w-[120px]">
            <div class="w-4 h-4 rounded-sm border border-black/10 shadow-sm" style="background-color: #007ACC;"></div>
            <span class="text-xs font-mono text-slate-600 dark:text-muted-foreground uppercase">#007ACC</span>
          </div>
        </div>

      </div>
    </div>

    <!-- Dark Theme Config -->
    <div class="mb-8">
      <h3 class="text-sm font-semibold text-slate-800 dark:text-foreground mb-3">Dark Theme</h3>
      <div class="rounded-xl border border-black/5 dark:border-white/10 bg-transparent dark:bg-white/50 dark:bg-transparent dark:bg-white/5 overflow-hidden">
        
        <div class="flex items-center justify-between p-3.5 border-b border-black/5 dark:border-border">
          <span class="text-sm text-slate-800 dark:text-foreground">Preset</span>
          <div class="w-40">
            <BaseSelect 
              modelValue="default"
              :options="[{ label: 'Default Dark', value: 'default' }]"
            />
          </div>
        </div>

        <div class="flex items-center justify-between p-3.5 border-b border-black/5 dark:border-border">
          <span class="text-sm text-slate-800 dark:text-foreground">Background</span>
          <div class="flex items-center gap-2 bg-white dark:bg-[#222] border border-black/10 dark:border-white/10 rounded-md px-2 py-1 min-w-[120px]">
            <div class="w-4 h-4 rounded-sm border border-white/20 shadow-sm" style="background-color: #101010;"></div>
            <span class="text-xs font-mono text-slate-600 dark:text-muted-foreground uppercase">#101010</span>
          </div>
        </div>

        <div class="flex items-center justify-between p-3.5">
          <span class="text-sm text-slate-800 dark:text-foreground">Foreground</span>
          <div class="flex items-center gap-2 bg-white dark:bg-[#222] border border-black/10 dark:border-white/10 rounded-md px-2 py-1 min-w-[120px]">
            <div class="w-4 h-4 rounded-sm border border-black/10 shadow-sm" style="background-color: #EEEEEE;"></div>
            <span class="text-xs font-mono text-slate-600 dark:text-muted-foreground uppercase">#EEEEEE</span>
          </div>
        </div>

      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref, inject, onMounted, watch } from 'vue';
import type { StateService } from '../../../services/StateService';
import type { ACPService } from '../../../services/ACPService';

// 导入原子化基础组件
import BaseSwitch from '../../ui/BaseSwitch.vue';
import BaseSelect from '../../ui/BaseSelect.vue';

const state = inject<StateService>('stateService')!;
const acp = inject<ACPService>('acpService')!;

const verboseChat = ref(true);
const currentTheme = ref('dark');

const applyTheme = async (val: string) => {
  state.set('selectedTheme', val);
  
  if (val === 'system') {
    const isDark = window.matchMedia && window.matchMedia('(prefers-color-scheme: dark)').matches;
    document.documentElement.setAttribute('data-theme', isDark ? 'dark' : 'light');
  } else {
    document.documentElement.setAttribute('data-theme', val);
  }
  
  await acp.storeSet('theme', val);
};

watch(currentTheme, (newVal) => {
  applyTheme(newVal);
});

onMounted(() => {
  currentTheme.value = state.get('selectedTheme') || 'dark';
});
</script>
