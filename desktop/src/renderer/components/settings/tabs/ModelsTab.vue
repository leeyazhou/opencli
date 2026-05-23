<template>
  <div class="flex flex-col gap-6 animate-in fade-in duration-300">
    <!-- 模型提供商设置 -->
    <section class="mb-8">
      <h3 class="text-sm font-semibold text-slate-800 dark:text-foreground mb-3">Model Providers</h3>
      <div class="rounded-xl border border-black/5 dark:border-white/10 bg-transparent dark:bg-white/5 overflow-hidden">
        
        <!-- OpenAI Compatible -->
        <div class="p-3.5 border-b border-black/5 dark:border-border flex items-center justify-between hover:bg-black/5 dark:hover:bg-white/5 transition-colors cursor-pointer">
          <div class="flex items-center gap-3">
            <div class="w-8 h-8 rounded-lg bg-emerald-500/10 flex items-center justify-center text-emerald-500">
              <CpuIcon class="w-4 h-4" />
            </div>
            <div class="flex flex-col">
              <span class="text-sm font-medium text-slate-800 dark:text-foreground">OpenAI Compatible</span>
              <span class="text-xs text-slate-500 dark:text-muted-foreground mt-0.5">DeepSeek, Qwen, vLLM endpoints</span>
            </div>
          </div>
          <button class="px-3 py-1.5 rounded-md bg-black/5 dark:bg-white/10 text-xs text-slate-700 dark:text-white font-medium hover:bg-black/10 dark:hover:bg-white/20 transition-colors">
            Configure
          </button>
        </div>

        <!-- Anthropic -->
        <div class="p-3.5 flex items-center justify-between hover:bg-black/5 dark:hover:bg-white/5 transition-colors cursor-pointer">
          <div class="flex items-center gap-3">
            <div class="w-8 h-8 rounded-lg bg-orange-500/10 flex items-center justify-center text-orange-500">
              <CpuIcon class="w-4 h-4" />
            </div>
            <div class="flex flex-col">
              <span class="text-sm font-medium text-slate-800 dark:text-foreground">Anthropic</span>
              <span class="text-xs text-slate-500 dark:text-muted-foreground mt-0.5">Claude 3.5 Sonnet / Opus</span>
            </div>
          </div>
          <button class="px-3 py-1.5 rounded-md bg-black/5 dark:bg-white/10 text-xs text-slate-700 dark:text-white font-medium hover:bg-black/10 dark:hover:bg-white/20 transition-colors">
            Configure
          </button>
        </div>
      </div>
    </section>

    <!-- 高级模型设置 -->
    <section class="mb-8">
      <h3 class="text-sm font-semibold text-slate-800 dark:text-foreground mb-3">Advanced</h3>
      <div class="rounded-xl border border-black/5 dark:border-white/10 bg-transparent dark:bg-white/5 overflow-hidden">
        <div class="flex items-center justify-between p-3.5">
          <div class="flex flex-col">
            <span class="text-sm font-medium text-slate-800 dark:text-foreground">Context Window Limit</span>
            <span class="text-xs text-slate-500 dark:text-muted-foreground mt-0.5">Truncate conversation history to fit context</span>
          </div>
          <div class="w-32 flex items-center">
            <input type="range" min="4000" max="128000" step="4000" value="32000" class="w-full accent-primary" />
          </div>
        </div>
      </div>
    </section>
  </div>
</template>

<script setup lang="ts">
import { CpuIcon } from 'lucide-vue-next';
</script>
