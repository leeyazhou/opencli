import { ref, watch, onMounted, onUnmounted } from 'vue';
import { StateService } from '../services/StateService';
import { ACPService } from '../services/ACPService';

export function useTheme() {
  const state = StateService.getInstance();
  const acp = ACPService.getInstance();
  
  const userPreference = ref<'light' | 'dark' | 'system'>('system');
  
  const mediaQuery = window.matchMedia('(prefers-color-scheme: dark)');

  const updateDocumentTheme = (isDark: boolean) => {
    if (isDark) {
      document.documentElement.setAttribute('data-theme', 'dark');
      document.documentElement.classList.add('dark');
    } else {
      document.documentElement.setAttribute('data-theme', 'light');
      document.documentElement.classList.remove('dark');
    }
  };

  const applyTheme = () => {
    if (userPreference.value === 'system') {
      updateDocumentTheme(mediaQuery.matches);
    } else {
      updateDocumentTheme(userPreference.value === 'dark');
    }
  };

  const handleMediaChange = (e: MediaQueryListEvent) => {
    if (userPreference.value === 'system') {
      updateDocumentTheme(e.matches);
    }
  };

  const setTheme = async (theme: 'light' | 'dark' | 'system') => {
    userPreference.value = theme;
    state.set('selectedTheme', theme);
    await acp.storeSet('theme', theme);
    applyTheme();
  };

  onMounted(() => {
    mediaQuery.addEventListener('change', handleMediaChange);
    
    // Subscribe to StateService in case theme is changed from somewhere else
    const unsubscribe = state.subscribe('selectedTheme', (theme) => {
      userPreference.value = theme as 'light' | 'dark' | 'system';
      applyTheme();
    });

    onUnmounted(() => {
      mediaQuery.removeEventListener('change', handleMediaChange);
      unsubscribe();
    });
  });

  return {
    userPreference,
    setTheme,
    applyTheme
  };
}
