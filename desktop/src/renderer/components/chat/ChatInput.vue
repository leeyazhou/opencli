<template>
  <div class="p-3 border-t border-panelBorder bg-panel/50">
    <div class="flex flex-col gap-2 px-3 py-2 rounded-xl border border-panelBorder bg-panel relative z-10">
      <!-- 模型选择行 -->
      <div class="flex items-center justify-between">
        <!-- 模型下拉 -->
        <div class="relative">
          <button
            @click.stop="isDropdownOpen = !isDropdownOpen"
            class="flex items-center gap-1.5 px-2 py-1 rounded-lg bg-black/5 dark:bg-transparent dark:bg-white/5 border border-transparent hover:border-panelBorder text-[11px] font-mono text-slate-600 dark:text-muted-foreground hover:text-slate-900 dark:hover:text-white transition-all cursor-pointer"
          >
            <span>{{ selectedModel }}</span>
            <ChevronDownIcon class="w-3 h-3 opacity-50" />
          </button>

          <!-- 展开面板 -->
          <div
            v-if="isDropdownOpen"
            class="absolute bottom-full left-0 mb-1 w-48 rounded-lg border border-panelBorder bg-panel shadow-xl overflow-hidden animate-in fade-in slide-in-from-bottom-2 duration-200 z-50"
          >
            <div class="py-1">
              <button
                v-for="model in models"
                :key="model"
                @click="selectModel(model)"
                class="w-full flex items-center justify-between px-3 py-1.5 text-left text-[11px] font-mono transition-colors cursor-pointer"
                :class="selectedModel === model ? 'bg-primary/10 text-primary' : 'text-slate-600 dark:text-foreground hover:bg-black/5 dark:hover:bg-transparent dark:bg-white/10'"
              >
                <span>{{ model }}</span>
                <CheckIcon v-if="selectedModel === model" class="w-3 h-3" />
              </button>
            </div>
          </div>
        </div>
      </div>

      <!-- 输入行 -->
      <div class="flex items-end gap-2">
        <textarea
          ref="promptInputRef"
          v-model="promptInputText"
          @input="autoResizeInput"
          @keydown.enter.exact.prevent="handleSend"
          :placeholder="placeholderText"
          class="flex-1 resize-none bg-transparent outline-none border-none py-1 text-xs font-mono text-slate-800 dark:text-foreground placeholder-slate-400 dark:placeholder-slate-500 custom-scrollbar max-h-[120px]"
          rows="1"
        ></textarea>

        <button
          @click="handleSend"
          :disabled="isSending || !promptInputText.trim()"
          class="p-2 rounded-lg bg-primary hover:bg-primary/90 disabled:bg-black/5 dark:disabled:bg-transparent dark:bg-white/5 disabled:text-slate-400 dark:disabled:text-muted-foreground disabled:opacity-50 text-white transition-all cursor-pointer"
          :title="'Send Message'"
        >
          <SendIcon class="w-4 h-4" />
        </button>
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref, onMounted, onUnmounted } from 'vue';
import { SendIcon, ChevronDownIcon, CheckIcon } from 'lucide-vue-next';

const props = defineProps<{
  isSending: boolean;
  selectedModel: string;
  placeholderText?: string;
}>();

const emit = defineEmits<{
  (e: 'send', text: string): void;
  (e: 'update:model', model: string): void;
}>();

const promptInputText = ref('');
const promptInputRef = ref<HTMLTextAreaElement | null>(null);
const isDropdownOpen = ref(false);

const models = [
  "Gemini 3.5 Flash (High)",
  "Gemini 3.5 Pro (Ultra)",
  "Claude 3.5 Sonnet",
  "GPT-4o Mini",
  "GPT-OSS 120B (Medium)"
];

const selectModel = (model: string) => {
  emit('update:model', model);
  isDropdownOpen.value = false;
};

const closeDropdown = () => {
  isDropdownOpen.value = false;
};

onMounted(() => {
  document.addEventListener('click', closeDropdown);
});

onUnmounted(() => {
  document.removeEventListener('click', closeDropdown);
});

const autoResizeInput = () => {
  if (promptInputRef.value) {
    promptInputRef.value.style.height = 'auto';
    promptInputRef.value.style.height = Math.min(promptInputRef.value.scrollHeight, 120) + 'px';
  }
};

const handleSend = () => {
  if (props.isSending || !promptInputText.value.trim()) return;
  emit('send', promptInputText.value.trim());
  promptInputText.value = '';
  if (promptInputRef.value) {
    promptInputRef.value.style.height = 'auto';
  }
};
</script>
