<template>
  <div 
    class="rounded-xl border bg-panel transition-colors"
    :class="[
      borderStyle === 'glass' ? 'border-borderGlass' : 'border-border',
      shadow ? 'shadow-lg shadow-black/5 dark:shadow-black/20' : '',
      customClass
    ]"
  >
    <slot></slot>
  </div>
</template>

<script setup lang="ts">
defineProps({
  borderStyle: {
    type: String,
    default: 'solid', // 'solid' | 'glass'
  },
  shadow: {
    type: Boolean,
    default: true
  },
  customClass: {
    type: String,
    default: ''
  }
});
</script>
