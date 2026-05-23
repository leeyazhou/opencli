<template>
  <div 
    class="message flex flex-col gap-1.5"
    :class="[msg.type]"
  >
    <!-- 消息 Header -->
    <div class="message-header text-[10px] font-mono font-bold tracking-wider uppercase text-muted-foreground flex items-center gap-1.5">
      <span :class="{ 'text-primary': msg.type === 'assistant' }">
        {{ typeLabels[msg.type] || msg.type }}
      </span>

      <!-- Tool 专有状态角标 -->
      <span 
        v-if="msg.type === 'tool'" 
        class="px-1.5 py-0.5 rounded text-[8px] tracking-wider uppercase font-mono font-semibold"
        :class="statusClasses[msg.status || 'running']"
      >
        {{ msg.status }}
      </span>
    </div>

    <!-- 消息 Body -->
    <component
      :is="['thought', 'user', 'error'].includes(msg.type) ? BaseCard : 'div'"
      class="message-body text-xs font-mono text-foreground leading-relaxed break-all select-text"
      :class="[
        msg.type === 'thought' ? 'p-3 cursor-pointer hover:border-primary/20' : '',
        msg.type === 'user' ? 'p-3 bg-panel border-border' : '',
        msg.type === 'error' ? 'bg-rose-500/5 p-3 border-rose-500/15 text-rose-400 font-semibold' : '',
        msg.type === 'system' ? 'text-muted-foreground italic p-1' : ''
      ]"
      :shadow="false"
      :border-style="msg.type === 'thought' ? 'glass' : 'solid'"
    >
      <!-- 思考过程支持折叠 -->
      <template v-if="msg.type === 'thought'">
        <div class="flex items-center gap-1.5 text-purple-400 text-[10px] select-none font-bold uppercase mb-1">
          <SparklesIcon class="w-3.5 h-3.5" />
          <span>{{ msg.expanded ? 'Hide Thinking Process' : 'Show Thinking Process' }}</span>
        </div>
        <div 
          v-show="msg.expanded"
          v-html="renderedThought"
          class="animate-in slide-in-from-top-1 duration-150 pl-2 border-l border-purple-500/20 text-[10px] text-purple-300/80"
        ></div>
      </template>

      <!-- 常规气泡 -->
      <template v-else-if="msg.type === 'tool'">
        <div class="flex items-center gap-2 p-2 rounded bg-black/5 dark:bg-black/30 border border-border">
          <CpuIcon class="w-4 h-4 text-primary/80" />
          <span>{{ msg.toolName }}</span>
        </div>
      </template>

      <template v-else>
        <div 
          v-html="renderedText"
          class="markdown-content"
        ></div>
      </template>
    </component>
  </div>
</template>

<script setup lang="ts">
import { SparklesIcon, CpuIcon } from 'lucide-vue-next';
import { marked } from 'marked';
import DOMPurify from 'dompurify';
import BaseCard from '../ui/BaseCard.vue';

import { computed } from 'vue';

const props = defineProps<{
  msg: any;
}>();

const typeLabels: Record<string, string> = {
  user: "You",
  assistant: "OpenCLI",
  system: "System",
  error: "Error",
  thought: "Agent Thought",
  tool: "Tool Call"
};

const statusClasses: Record<string, string> = {
  running: "bg-blue-500/20 text-blue-400 border border-blue-500/30 animate-pulse",
  success: "bg-emerald-500/20 text-emerald-400 border border-emerald-500/30",
  error: "bg-rose-500/20 text-rose-400 border border-rose-500/30",
};

const renderedThought = computed(() => {
  if (props.msg.type !== 'thought') return '';
  return DOMPurify.sanitize(marked.parse(props.msg.text || '') as string);
});

const renderedText = computed(() => {
  if (props.msg.type === 'thought' || props.msg.type === 'tool') return '';
  return DOMPurify.sanitize(marked.parse(props.msg.text || '') as string);
});
</script>
