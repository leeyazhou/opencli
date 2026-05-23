<template>
  <div class="fixed inset-0 z-[9999] bg-background flex items-center justify-center p-6 animate-in fade-in duration-300">
    <div class="max-w-md w-full text-center space-y-6">
      <div class="w-20 h-20 bg-rose-500/10 rounded-full flex items-center justify-center mx-auto text-rose-500">
        <AlertTriangleIcon class="w-10 h-10" />
      </div>
      
      <div class="space-y-2">
        <h1 class="text-2xl font-bold text-foreground">Backend Connection Lost</h1>
        <p class="text-muted-foreground text-sm">
          The OpenCLI agent process unexpectedly disconnected or crashed. 
          Please restart the agent to continue.
        </p>
      </div>
      
      <div class="pt-4 flex justify-center gap-4">
        <BaseButton @click="restartAgent" variant="primary">
          <RefreshCwIcon class="w-4 h-4 mr-2" />
          Restart Agent
        </BaseButton>
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import { AlertTriangleIcon, RefreshCwIcon } from 'lucide-vue-next';
import BaseButton from './BaseButton.vue';
import { ACPService } from '../../services/ACPService';
import { StateService } from '../../services/StateService';

const restartAgent = async () => {
  const acp = ACPService.getInstance();
  const state = StateService.getInstance();
  
  try {
    const started = await acp.startAgent();
    if (started) {
      state.set('agentRunning', true);
      // Wait for it to initialize
      await acp.initializeAgent();
    }
  } catch (e) {
    console.error("Failed to restart agent:", e);
  }
};
</script>
