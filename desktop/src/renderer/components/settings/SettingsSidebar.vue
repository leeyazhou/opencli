<template>
  <aside class="w-[240px] flex flex-col bg-sidebar border-r border-border py-4">
    <div class="flex-1 overflow-y-auto px-3 flex flex-col gap-1 custom-scrollbar">
      
      <!-- General Group -->
      <div class="text-[11px] font-semibold text-muted-foreground uppercase tracking-wider mb-1 mt-2 px-3">General</div>
      <button v-for="tab in ['Account', 'Permissions', 'Appearance', 'Models', 'Customizations', 'Browser', 'App']" 
              :key="tab"
              @click="$emit('update:activeTab', tab)"
              class="w-full text-left px-3 py-1.5 rounded-lg text-sm transition-all cursor-pointer"
              :class="activeTab === tab 
                ? 'bg-black/5 dark:bg-transparent dark:bg-white/10 font-medium text-slate-900 dark:text-white shadow-sm border border-black/5 dark:border-border' 
                : 'text-slate-600 dark:text-muted-foreground hover:bg-black/5 dark:hover:bg-transparent dark:bg-white/5'">
        {{ tab }}
      </button>
      
      <!-- Projects Group -->
      <div class="text-[11px] font-semibold text-muted-foreground uppercase tracking-wider mb-1 mt-5 px-3">Projects</div>
      <button v-for="tab in ['opencli', 'travel-ai', 'quant-vue']" 
              :key="tab"
              @click="$emit('update:activeTab', tab)"
              class="w-full text-left px-3 py-1.5 rounded-lg text-sm transition-all cursor-pointer text-slate-600 dark:text-muted-foreground hover:bg-black/5 dark:hover:bg-transparent dark:bg-white/5"
              :class="activeTab === tab ? 'bg-black/5 dark:bg-transparent dark:bg-white/10 font-medium text-slate-900 dark:text-white' : ''">
        {{ tab }}
      </button>
      <button class="w-full text-left px-3 py-1.5 rounded-lg text-xs text-muted-foreground hover:text-slate-700 dark:hover:text-foreground transition-colors mt-1">
        Show all
      </button>

      <!-- Not in Project Group -->
      <div class="text-[11px] font-semibold text-muted-foreground uppercase tracking-wider mb-1 mt-5 px-3">Not in Project</div>
      <button @click="$emit('update:activeTab', 'Conversations')"
              class="w-full text-left px-3 py-1.5 rounded-lg text-sm transition-all cursor-pointer text-slate-600 dark:text-muted-foreground hover:bg-black/5 dark:hover:bg-transparent dark:bg-white/5"
              :class="activeTab === 'Conversations' ? 'bg-black/5 dark:bg-transparent dark:bg-white/10 font-medium text-slate-900 dark:text-white' : ''">
        Conversations
      </button>
    </div>

    <!-- 底部快捷入口 -->
    <div class="px-3 mt-4 flex flex-col gap-1">
      <button class="w-full text-left px-3 py-1.5 rounded-lg text-sm transition-all cursor-pointer text-slate-600 dark:text-muted-foreground hover:bg-black/5 dark:hover:bg-transparent dark:bg-white/5">
        Shortcuts
      </button>
      <button class="w-full text-left px-3 py-1.5 rounded-lg text-sm transition-all cursor-pointer text-slate-600 dark:text-muted-foreground hover:bg-black/5 dark:hover:bg-transparent dark:bg-white/5">
        Provide Feedback
      </button>
    </div>
  </aside>
</template>

<script setup lang="ts">
defineProps<{ activeTab: string }>();
defineEmits(['update:activeTab']);
</script>

<style scoped>
.custom-scrollbar::-webkit-scrollbar { width: 6px; }
.custom-scrollbar::-webkit-scrollbar-track { background: transparent; }
.custom-scrollbar::-webkit-scrollbar-thumb { background: rgba(150, 150, 150, 0.2); border-radius: 99px; }
.custom-scrollbar::-webkit-scrollbar-thumb:hover { background: rgba(150, 150, 150, 0.4); }
</style>
