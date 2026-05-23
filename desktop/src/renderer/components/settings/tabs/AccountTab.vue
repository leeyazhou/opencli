<template>
  <div class="flex flex-col gap-6 animate-in fade-in duration-300">
    <section class="mb-8">
      <div class="rounded-xl border border-black/5 dark:border-white/10 bg-transparent dark:bg-white/5 overflow-hidden">
        <div class="flex items-center gap-4 p-4">
          <div class="w-14 h-14 rounded-full bg-gradient-to-tr from-primary to-blue-500 flex items-center justify-center text-white text-2xl font-bold shadow-sm">
            O
          </div>
          <div class="flex flex-col">
            <h3 class="text-base font-semibold text-slate-800 dark:text-foreground">opencli User</h3>
            <span class="text-sm text-slate-500 dark:text-muted-foreground mt-0.5">user@example.com</span>
          </div>
          <div class="ml-auto">
            <button class="px-3 py-1.5 rounded-md bg-black/5 dark:bg-white/10 text-xs text-slate-700 dark:text-white font-medium hover:bg-black/10 dark:hover:bg-white/20 transition-colors">
              Sign Out
            </button>
          </div>
        </div>
      </div>
    </section>

    <section class="mb-8">
      <h3 class="text-sm font-semibold text-slate-800 dark:text-foreground mb-3">Subscription</h3>
      <div class="rounded-xl border border-black/5 dark:border-white/10 bg-transparent dark:bg-white/5 overflow-hidden">
        <div class="p-4 flex flex-col">
          <div class="flex items-center justify-between mb-2">
            <span class="text-sm font-medium text-slate-800 dark:text-foreground">Free Plan</span>
            <span class="px-2 py-0.5 rounded bg-primary/10 text-primary text-[10px] font-bold uppercase tracking-wider">Active</span>
          </div>
          <span class="text-xs text-slate-500 dark:text-muted-foreground mb-4">You are currently on the free tier. Upgrade for unlimited pro models and premium support.</span>
          <div>
            <button class="px-4 py-2 rounded-lg bg-primary hover:bg-primary-hover text-white text-xs font-semibold shadow-sm transition-colors">
              Upgrade to Pro
            </button>
          </div>
        </div>
      </div>
    </section>
  </div>
</template>

<script setup lang="ts">
</script>
