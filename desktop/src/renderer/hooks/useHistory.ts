import { ref, onMounted, onUnmounted } from 'vue';
import type { StateService } from '../services/StateService';

const backStack = ref<string[]>([]);
const forwardStack = ref<string[]>([]);
let isNavigating = false;
let initialized = false;

export function useHistory(state: StateService) {
  const navigateBack = () => {
    if (backStack.value.length === 0) return;
    const cur = state.get("sessionId");
    const prev = backStack.value.pop();
    if (prev) {
      isNavigating = true;
      if (cur) forwardStack.value.push(cur);
      state.emit("requestSwitchSession", prev);
      isNavigating = false;
    }
  };

  const navigateForward = () => {
    if (forwardStack.value.length === 0) return;
    const cur = state.get("sessionId");
    const next = forwardStack.value.pop();
    if (next) {
      isNavigating = true;
      if (cur) backStack.value.push(cur);
      state.emit("requestSwitchSession", next);
      isNavigating = false;
    }
  };

  if (!initialized) {
    state.subscribe("sessionId", (sessionId) => {
      if (isNavigating || !sessionId) return;
      const last = backStack.value[backStack.value.length - 1];
      if (last !== sessionId) {
        if (last) backStack.value.push(last);
        forwardStack.value = [];
      }
    });
    initialized = true;
  }

  return {
    backStack,
    forwardStack,
    navigateBack,
    navigateForward
  };
}
