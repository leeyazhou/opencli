<template>
  <div class="relative">
    <select 
      :value="modelValue" 
      @change="$emit('update:modelValue', ($event.target as HTMLSelectElement).value)"
      class="w-full appearance-none bg-black/5 dark:bg-white/10 border border-transparent hover:border-black/10 dark:hover:border-white/20 text-slate-800 dark:text-slate-200 text-sm rounded-lg pl-3 pr-8 py-1.5 focus:outline-none focus:ring-2 focus:ring-primary/50 transition-colors cursor-pointer"
    >
      <option v-for="opt in options" :key="opt.value" :value="opt.value">
        {{ opt.label }}
      </option>
    </select>
    <ChevronDownIcon class="w-4 h-4 text-slate-500 absolute right-2.5 top-1/2 -translate-y-1/2 pointer-events-none" />
  </div>
</template>

<script setup lang="ts">
import { ChevronDownIcon } from 'lucide-vue-next';

defineProps<{ 
  modelValue: string | number;
  options: { label: string; value: string | number }[] 
}>();

defineEmits(['update:modelValue']);
</script>
