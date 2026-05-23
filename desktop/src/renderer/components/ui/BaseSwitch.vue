<template>
  <button 
    type="button"
    role="switch"
    :aria-checked="modelValue"
    @click="$emit('update:modelValue', !modelValue)"
    class="w-11 h-6 rounded-full relative transition-colors duration-200 ease-in-out cursor-pointer focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-primary/50 focus-visible:ring-offset-2 dark:focus-visible:ring-offset-slate-900"
    :class="modelValue ? 'bg-primary' : 'bg-slate-300 dark:bg-slate-600'"
  >
    <span 
      class="absolute top-1 left-1 bg-white w-4 h-4 rounded-full transition-transform duration-200 ease-in-out shadow-sm"
      :class="modelValue ? 'translate-x-5' : 'translate-x-0'"
    ></span>
  </button>
</template>

<script setup lang="ts">
defineProps<{ modelValue: boolean }>();
defineEmits(['update:modelValue']);
</script>
