<template>
  <div class="splash max-w-[550px] mx-auto my-auto text-center flex flex-col items-center gap-5 p-6 rounded-2xl border border-panelBorder bg-panel animate-in fade-in zoom-in duration-300">
    <!-- Antigravity 标志小发光球 -->
    <div class="w-10 h-10 rounded-full bg-primary/10 border border-primary/20 flex items-center justify-center shadow-lg shadow-primary/10">
      <span class="w-2.5 h-2.5 rounded-full bg-primary animate-ping"></span>
    </div>

    <h2 class="text-lg font-mono font-bold tracking-widest text-slate-800 dark:text-foreground">
      {{ t('chat.welcome.title') }}
    </h2>
    
    <p class="text-xs font-mono text-slate-600 dark:text-muted-foreground leading-relaxed max-w-[450px]">
      {{ t('chat.welcome.subtitle') }}
    </p>

    <!-- 极客特征亮点列表 -->
    <div class="w-full flex flex-col gap-2.5 text-left border-t border-panelBorder pt-4 mt-1.5 font-mono text-[11px] text-slate-500 dark:text-muted-foreground">
      <div class="flex items-start gap-2">
        <span class="text-primary">🚀</span>
        <span>{{ t('chat.welcome.feature1') }}</span>
      </div>
      <div class="flex items-start gap-2">
        <span class="text-primary">🔍</span>
        <span>{{ t('chat.welcome.feature2') }}</span>
      </div>
      <div class="flex items-start gap-2">
        <span class="text-primary">🛡️</span>
        <span>{{ t('chat.welcome.feature3') }}</span>
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import { useI18n } from '../../hooks/useI18n';

const { t } = useI18n();
</script>
