<template>
  <button 
    :class="[
      'inline-flex items-center justify-center rounded-md font-medium transition-colors focus-visible:outline-none disabled:pointer-events-none disabled:opacity-50 cursor-pointer',
      variants[variant],
      sizes[size],
      $attrs.class
    ]"
    v-bind="$attrs"
  >
    <slot />
  </button>
</template>

<script setup lang="ts">
import { defineProps, withDefaults } from 'vue';

type ButtonVariant = 'default' | 'primary' | 'secondary' | 'ghost' | 'outline' | 'danger';
type ButtonSize = 'sm' | 'md' | 'lg' | 'icon';

const props = withDefaults(defineProps<{
  variant?: ButtonVariant;
  size?: ButtonSize;
}>(), {
  variant: 'default',
  size: 'md'
});

const variants: Record<ButtonVariant, string> = {
  default: 'bg-white text-slate-900 hover:bg-slate-100 dark:bg-slate-800 dark:text-slate-50 dark:hover:bg-slate-700',
  primary: 'bg-primary text-white hover:bg-primary/90 shadow-sm',
  secondary: 'bg-slate-100 text-slate-900 hover:bg-slate-200 dark:bg-slate-800 dark:text-slate-100 dark:hover:bg-slate-700',
  ghost: 'hover:bg-black/5 dark:hover:bg-transparent dark:bg-white/10 text-slate-700 dark:text-foreground',
  outline: 'border border-slate-200 bg-transparent hover:bg-slate-100 dark:border-slate-800 dark:text-slate-100 dark:hover:bg-slate-800',
  danger: 'bg-red-500 text-white hover:bg-red-600'
};

const sizes: Record<ButtonSize, string> = {
  sm: 'h-8 px-3 text-xs',
  md: 'h-9 px-4 py-2 text-sm',
  lg: 'h-10 px-8 text-sm',
  icon: 'h-9 w-9 p-2'
};
</script>
