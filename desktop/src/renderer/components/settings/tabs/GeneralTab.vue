<template>
  <div class="flex flex-col gap-6 animate-in fade-in duration-300">
    <section class="mb-8">
      <h3 class="text-sm font-semibold text-slate-800 dark:text-foreground mb-3">Workspace</h3>
      <div class="rounded-xl border border-black/5 dark:border-white/10 bg-transparent dark:bg-white/5 overflow-hidden">
        <div class="p-4 flex items-center justify-between">
          <div class="flex flex-col">
            <span class="text-sm font-medium text-slate-800 dark:text-foreground">Default Project Path</span>
            <span class="text-xs text-slate-500 dark:text-muted-foreground mt-0.5">Where new projects and sessions are stored</span>
          </div>
          <button class="px-3 py-1.5 rounded-md bg-black/5 dark:bg-white/10 text-xs text-slate-700 dark:text-white font-medium hover:bg-black/10 dark:hover:bg-white/20 transition-colors">
            Change
          </button>
        </div>
        <div class="px-4 pb-4">
          <div class="text-xs font-mono text-slate-600 dark:text-muted-foreground bg-transparent dark:bg-black/20 p-2 rounded border border-black/5 dark:border-white/5">
            /Users/opencli/workspace
          </div>
        </div>
      </div>
    </section>

    <section class="mb-8">
      <h3 class="text-sm font-semibold text-slate-800 dark:text-foreground mb-3">Language & Region</h3>
      <div class="rounded-xl border border-black/5 dark:border-white/10 bg-transparent dark:bg-white/5 overflow-hidden">
        <div class="p-4 flex items-center justify-between">
          <div class="flex flex-col">
            <span class="text-sm font-medium text-slate-800 dark:text-foreground">UI Language</span>
            <span class="text-xs text-slate-500 dark:text-muted-foreground mt-0.5">Change the application interface language</span>
          </div>
          <div class="flex gap-2">
            <button 
              @click="changeLanguage('zh')"
              class="px-3 py-1.5 rounded-md text-xs font-medium transition-colors border"
              :class="currentLang === 'zh' 
                ? 'bg-primary/10 border-primary/30 text-primary' 
                : 'border-transparent bg-black/5 dark:bg-white/10 text-slate-700 dark:text-white hover:bg-black/10 dark:hover:bg-white/20'"
            >
              简体中文
            </button>
            <button 
              @click="changeLanguage('en')"
              class="px-3 py-1.5 rounded-md text-xs font-medium transition-colors border"
              :class="currentLang === 'en' 
                ? 'bg-primary/10 border-primary/30 text-primary' 
                : 'border-transparent bg-black/5 dark:bg-white/10 text-slate-700 dark:text-white hover:bg-black/10 dark:hover:bg-white/20'"
            >
              English
            </button>
          </div>
        </div>
      </div>
    </section>

    <section class="mb-8">
      <h3 class="text-sm font-semibold text-slate-800 dark:text-foreground mb-3">Startup</h3>
      <div class="rounded-xl border border-black/5 dark:border-white/10 bg-transparent dark:bg-white/5 overflow-hidden">
        <div class="p-4 flex items-center justify-between hover:bg-black/5 dark:hover:bg-white/5 transition-colors cursor-pointer">
          <div class="flex flex-col">
            <span class="text-sm font-medium text-slate-800 dark:text-foreground">Launch at login</span>
            <span class="text-xs text-slate-500 dark:text-muted-foreground mt-0.5">Start opencli automatically when you log in</span>
          </div>
          <!-- Switch Toggle -->
          <BaseSwitch v-model="launchAtLogin" />
        </div>
      </div>
    </section>
  </div>
</template>

<script setup lang="ts">
import { ref, inject } from 'vue';
import BaseSwitch from '../../ui/BaseSwitch.vue';
import type { StateService } from '../../../services/StateService';
import type { ACPService } from '../../../services/ACPService';
import { useI18n } from '../../../hooks/useI18n';

const state = inject<StateService>('stateService')!;
const acp = inject<ACPService>('acpService')!;
const { currentLang } = useI18n();

const launchAtLogin = ref(true);

const changeLanguage = async (lang: 'zh' | 'en') => {
  state.set('language', lang);
  await acp.storeSet('language', lang);
};
</script>
