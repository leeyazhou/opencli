import { ref, inject, onMounted, onUnmounted } from "vue";
import type { I18nService } from "../services/I18nService";
import type { StateService } from "../services/StateService";

export function useI18n() {
  const i18n = inject<I18nService>("i18nService")!;
  const state = inject<StateService>("stateService")!;
  
  // 建立响应式语言状态，初始化
  const currentLang = ref(state.getState().language || "zh");

  let unsubscribe: (() => void) | null = null;

  onMounted(() => {
    // 订阅 StateService 中 language 的变更，并保持响应式
    unsubscribe = state.subscribe("language", (newLang) => {
      currentLang.value = newLang;
    });
  });

  onUnmounted(() => {
    if (unsubscribe) {
      unsubscribe();
    }
  });

  /**
   * 翻译函数，包装 I18nService 的 t 方法
   * 显式关联 currentLang.value 以便当语言切换时 Vue 3 自动重绘所有相关的 DOM
   */
  const t = (key: string, variables?: Record<string, string>): string => {
    // 强制依赖收集
    const _ = currentLang.value;
    return i18n.t(key, variables);
  };

  return {
    t,
    currentLang,
  };
}
