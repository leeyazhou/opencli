<template>
  <main class="chat-area flex-1 flex flex-col h-full bg-transparent dark:bg-background/40 relative overflow-hidden">
    
    <!-- 顶部会话元数据平铺状态条 -->
    <div class="px-4 py-2 border-b border-white/5 bg-transparent flex items-center justify-between text-[10px] font-mono text-slate-500">
      <div class="flex items-center gap-1.5">
        <HashIcon class="w-3.5 h-3.5" />
        <span>ID:</span>
        <span class="text-slate-400 select-all font-semibold">{{ shortId(currentSessionId) }}</span>
      </div>

      <div class="flex items-center gap-3">
        <div class="flex items-center gap-1">
          <MessageSquareIcon class="w-3.5 h-3.5" />
          <span>{{ messagesCount }} messages</span>
        </div>
      </div>
    </div>

    <!-- 聊天内容视口区 -->
    <div 
      ref="chatViewportRef"
      @scroll="handleScroll"
      class="flex-1 overflow-y-auto p-4 flex flex-col gap-4 custom-scrollbar relative"
    >
      
      <!-- SPLASH 引导页 -->
      <div 
        v-if="messages.length === 0"
        class="splash max-w-[550px] mx-auto my-auto text-center flex flex-col items-center gap-5 p-6 rounded-2xl border border-black/5 dark:border-white/5 bg-white dark:bg-panel animate-in fade-in zoom-in duration-300"
      >
        <!-- Antigravity 标志小发光球 -->
        <div class="w-10 h-10 rounded-full bg-primary/10 border border-primary/20 flex items-center justify-center shadow-lg shadow-primary/10">
          <span class="w-2.5 h-2.5 rounded-full bg-primary animate-ping"></span>
        </div>

        <h2 class="text-lg font-mono font-bold tracking-widest text-slate-200">
          {{ t('chat.welcome.title') }}
        </h2>
        
        <p class="text-xs font-mono text-slate-400 leading-relaxed max-w-[450px]">
          {{ t('chat.welcome.subtitle') }}
        </p>

        <!-- 极客特征亮点列表 -->
        <div class="w-full flex flex-col gap-2.5 text-left border-t border-white/5 pt-4 mt-1.5 font-mono text-[11px] text-slate-500">
          <div class="flex items-start gap-2">
            <span class="text-primary">🚀</span>
            <span>{{ t('chat.welcome.feature1') }}</span>
          </div>
          <div class="flex items-start gap-2">
            <span class="text-primary">🔍</span>
            <span>{{ t('chat.welcome.feature2') }}</span>
          </div>
          <div class="flex items-start gap-2">
            <span class="text-primary">🛡️</span>
            <span>{{ t('chat.welcome.feature3') }}</span>
          </div>
        </div>

        <div class="text-[10px] font-mono text-slate-600 mt-2">
          {{ currentLang === 'zh' ? '向 OpenCLI 输入指令，开启 Workspace 全自动编程！' : 'Type commands to start full-automatic workspace engineering!' }}
        </div>
      </div>

      <!-- 消息列表渲染 -->
      <div 
        v-else 
        v-for="(msg, idx) in messages" 
        :key="idx"
        class="message flex flex-col gap-1.5"
        :class="[msg.type]"
      >
        <!-- 消息 Header -->
        <div class="message-header text-[10px] font-mono font-bold tracking-wider uppercase text-slate-500 flex items-center gap-1.5">
          <span :class="{ 'text-primary': msg.type === 'assistant' }">
            {{ typeLabels[msg.type] }}
          </span>

          <!-- Tool 专有状态角标 -->
          <span 
            v-if="msg.type === 'tool'" 
            class="px-1.5 py-0.5 rounded text-[8px] tracking-wider uppercase font-mono font-semibold"
            :class="statusClasses[msg.status || 'running']"
          >
            {{ msg.status }}
          </span>
        </div>

        <!-- 消息 Body -->
        <div 
          class="message-body text-xs font-mono text-slate-300 leading-relaxed break-all select-text"
          :class="[
            msg.type === 'thought' ? 'bg-purple-50/50 dark:bg-[#0f0e1c]/45 p-3 rounded-lg border border-purple-500/10 cursor-pointer hover:border-purple-500/20' : '',
            msg.type === 'user' ? 'bg-white/5 p-3 rounded-lg border border-white/5' : '',
            msg.type === 'error' ? 'bg-rose-500/5 p-3 rounded-lg border border-rose-500/15 text-rose-400 font-semibold' : '',
            msg.type === 'system' ? 'text-slate-500 italic p-1' : ''
          ]"
        >
          <!-- 思考过程支持折叠 -->
          <template v-if="msg.type === 'thought'">
            <div class="flex items-center gap-1.5 text-purple-400 text-[10px] select-none font-bold uppercase mb-1">
              <SparklesIcon class="w-3.5 h-3.5" />
              <span>{{ msg.expanded ? 'Hide Thinking Process' : 'Show Thinking Process' }}</span>
            </div>
            <div 
              v-show="msg.expanded"
              v-html="renderMarkdown(msg.text || '')"
              class="animate-in slide-in-from-top-1 duration-150 pl-2 border-l border-purple-500/20 text-[10px] text-purple-300/80"
            ></div>
          </template>

          <!-- 常规气泡 -->
          <template v-else-if="msg.type === 'tool'">
            <div class="flex items-center gap-2 p-2 rounded bg-black/30 border border-white/5">
              <CpuIcon class="w-4 h-4 text-primary/80" />
              <span>{{ msg.toolName }}</span>
            </div>
          </template>

          <template v-else>
            <div 
              v-html="renderMarkdown(msg.text || '')"
              class="markdown-content"
            ></div>
          </template>
        </div>
      </div>
    </div>

    <!-- 回到底部悬浮球 -->
    <button 
      v-show="showScrollBottomBtn"
      @click="scrollToBottom"
      class="absolute bottom-[90px] right-5 p-2 rounded-full border border-black/10 dark:border-white/10 bg-white dark:bg-panel shadow-2xl text-slate-400 hover:text-white transition-all cursor-pointer animate-in zoom-in-50"
      :title="currentLang === 'zh' ? '回到底部' : 'Scroll to bottom'"
    >
      <ArrowDownIcon class="w-4 h-4 animate-bounce" />
    </button>

    <!-- 底部 Prompt 输入舱 (磨砂玻璃面板) -->
    <div class="p-3 border-t border-white/5 bg-black/10">
      <div class="flex flex-col gap-2 px-3 py-2 rounded-xl border border-black/10 dark:border-white/10 bg-white dark:bg-panel relative z-10">
        <!-- 模型选择行 -->
        <div class="flex items-center justify-between">
          <!-- 模型下拉 -->
          <div class="relative">
            <button
              @click.stop="isDropdownOpen = !isDropdownOpen"
              class="flex items-center gap-1.5 px-2 py-1 rounded-lg bg-white/5 border border-white/10 hover:bg-white/10 text-[11px] font-mono text-slate-400 hover:text-white transition-all cursor-pointer"
            >
              <CpuIcon class="w-3 h-3 text-primary/70" />
              <span>{{ selectedModel }}</span>
              <ChevronDownIcon class="w-3 h-3 opacity-50 transition-transform" :class="{ 'rotate-180': isDropdownOpen }" />
            </button>

            <!-- 下拉菜单 -->
            <div
              v-if="isDropdownOpen"
              class="absolute bottom-full mb-1.5 left-0 w-[220px] rounded-lg border border-black/10 dark:border-white/10 bg-white dark:bg-panel shadow-xl py-1 z-[100] animate-in fade-in slide-in-from-bottom-2 duration-150"
            >
              <div class="px-2.5 py-1.5 text-[10px] font-mono font-semibold text-slate-500 uppercase tracking-wider border-b border-white/5">
                推理模型
              </div>
              <button
                v-for="model in models"
                :key="model"
                @click="selectModel(model)"
                class="w-full flex items-center justify-between px-3 py-1.5 text-left text-xs font-mono text-slate-300 hover:bg-white/5 hover:text-white transition-colors cursor-pointer"
                :class="{ 'bg-primary/10 text-primary': selectedModel === model }"
              >
                <span>{{ model }}</span>
                <CheckIcon v-if="selectedModel === model" class="w-3.5 h-3.5" />
              </button>
            </div>
          </div>
        </div>

        <!-- 输入行 -->
        <div class="flex items-end gap-2">
          <textarea
            ref="promptInputRef"
            v-model="promptInputText"
            @input="autoResizeInput"
            @keydown.enter.exact.prevent="sendMessage"
            :placeholder="t('chat.input.placeholder')"
            class="flex-1 resize-none bg-transparent outline-none border-none py-1 text-xs font-mono text-slate-100 placeholder-slate-500 custom-scrollbar max-h-[120px]"
            rows="1"
          ></textarea>

          <button
            @click="sendMessage"
            :disabled="isSending || !promptInputText.trim()"
            class="p-2 rounded-lg bg-primary hover:bg-primary-hover disabled:bg-white/5 disabled:text-slate-600 disabled:opacity-50 text-white transition-all cursor-pointer"
            :title="t('chat.button.send')"
          >
            <SendIcon class="w-4 h-4" />
          </button>
        </div>
      </div>
    </div>
  </main>
</template>

<script setup lang="ts">
import { ref, inject, onMounted, onUnmounted, computed, nextTick } from "vue";
import { useI18n } from "../../hooks/useI18n";
import type { StateService } from "../../services/StateService";
import type { ACPService } from "../../services/ACPService";
import {
  HashIcon,
  MessageSquareIcon,
  SparklesIcon,
  CpuIcon,
  SendIcon,
  ArrowDownIcon,
  ChevronDownIcon,
  CheckIcon
} from "lucide-vue-next";

// 注入服务
const state = inject<StateService>("stateService")!;
const acp = inject<ACPService>("acpService")!;

// 订阅响应式 i18n 与语言
const { t, currentLang } = useI18n();

// 引用与状态
const chatViewportRef = ref<HTMLElement | null>(null);
const promptInputRef = ref<HTMLTextAreaElement | null>(null);

const currentSessionId = ref<string | null>(null);
const promptInputText = ref("");
const isSending = ref(false);
const showScrollBottomBtn = ref(false);

// 模型选择
const selectedModel = ref("Gemini 3.5 Flash (High)");
const isDropdownOpen = ref(false);
const models = [
  "Gemini 3.5 Flash (High)",
  "Gemini 3.5 Pro (Ultra)",
  "Claude 3.5 Sonnet",
  "GPT-4o Mini"
];

const selectModel = async (model: string) => {
  selectedModel.value = model;
  state.set("selectedModel", model);
  await acp.storeSet("model", model);
  isDropdownOpen.value = false;
};

// 点击外部关闭下拉
const closeDropdown = () => { isDropdownOpen.value = false; };

interface Message {
  type: "user" | "assistant" | "thought" | "tool" | "system" | "error";
  text?: string;
  toolName?: string;
  status?: string;
  expanded?: boolean;
}

const messages = ref<Message[]>([]);
const messagesCount = computed(() => messages.value.length);

const typeLabels = {
  user: "YOU",
  assistant: "ASSISTANT",
  thought: "THINKING",
  tool: "TOOL",
  system: "SYSTEM",
  error: "ERROR"
};

const statusClasses = {
  running: "bg-amber-500/10 text-amber-400 border border-amber-500/20",
  completed: "bg-emerald-500/10 text-emerald-400 border border-emerald-500/20",
  failed: "bg-rose-500/10 text-rose-400 border border-rose-500/20",
  success: "bg-emerald-500/10 text-emerald-400 border border-emerald-500/20"
};

// Markdown 渲染方法
const renderMarkdown = (text: string) => {
  return acp.renderMarkdown(text);
};

// 截取 ID
const shortId = (id: string | null) => {
  return id ? id.slice(0, 8) : "--";
};

// 自动计算高度
const autoResizeInput = () => {
  if (promptInputRef.value) {
    promptInputRef.value.style.height = "auto";
    promptInputRef.value.style.height = Math.min(promptInputRef.value.scrollHeight, 120) + "px";
  }
};

// 滚动到底部
const scrollToBottom = async () => {
  await nextTick();
  if (chatViewportRef.value) {
    chatViewportRef.value.scrollTop = chatViewportRef.value.scrollHeight;
  }
};

const isNearBottom = () => {
  if (!chatViewportRef.value) return true;
  const threshold = 120;
  return chatViewportRef.value.scrollHeight - chatViewportRef.value.scrollTop - chatViewportRef.value.clientHeight < threshold;
};

const handleScroll = () => {
  showScrollBottomBtn.value = !isNearBottom();
};

// 发送消息核心控制
const sendMessage = async () => {
  const text = promptInputText.value.trim();
  if (!text || isSending.value) return;

  const initialized = state.get("initialized");
  let sessionId = state.get("sessionId");

  if (!initialized || !sessionId) {
    addSystemMessage("Agent 尚未准备就绪，正在重新初始化中...");
    state.emit("requestInitializeAgent");
    return;
  }

  // 1. 追加用户泡泡
  messages.value.push({
    type: "user",
    text
  });

  promptInputText.value = "";
  autoResizeInput();
  scrollToBottom();
  isSending.value = true;

  const selectedModel = state.get("selectedModel");

  try {
    // 2. 发起远程 ACP 推理请求
    const result = await acp.prompt(sessionId, text, selectedModel);

    // 如果返回了非流式的整块回包
    if (result && (result.content || result.text || result.message)) {
      const responseText = extractText(result.content || result.text || result.message || result);
      // 检查当前最后一条是不是 assistant，如果不是，追加气泡
      const last = messages.value[messages.value.length - 1];
      if (responseText && (!last || last.type !== "assistant")) {
        messages.value.push({
          type: "assistant",
          text: responseText
        });
        scrollToBottom();
      }
    }
  } catch (err: any) {
    console.error("Prompt 响应异常:", err);
    addErrorMessage(err.message || "Prompt 请求响应异常");
  } finally {
    isSending.value = false;
    // 刷新侧边栏
    state.emit("requestRefreshSessionList");
  }
};

// 插入日志
const addSystemMessage = (text: string) => {
  messages.value.push({
    type: "system",
    text
  });
  scrollToBottom();
};

const addErrorMessage = (text: string) => {
  messages.value.push({
    type: "error",
    text
  });
  scrollToBottom();
};

// 提取内容
const extractText = (content: any): string => {
  if (!content) return "";
  if (typeof content === "string") return content;
  if (content.text) return content.text;
  if (Array.isArray(content)) {
    return content.map((b) => (typeof b === "string" ? b : b.text || "")).join("");
  }
  return "";
};

// 切换会话并还原历史
const switchSession = async (sessionId: string) => {
  try {
    const result = await acp.loadSession(sessionId);
    state.set("sessionId", result.id);
    currentSessionId.value = result.id;
    state.emit("updateProjectCwd", result.cwd || "/");

    messages.value = [];
    if (result.messages && result.messages.length > 0) {
      result.messages.forEach((m: any) => {
        if (m.role === "user") {
          messages.value.push({
            type: "user",
            text: m.content
          });
        } else if (m.role === "assistant") {
          messages.value.push({
            type: "assistant",
            text: m.content
          });
        }
      });
    }
    scrollToBottom();
    // 刷新侧边
    state.emit("requestRefreshSessionList");
  } catch (err: any) {
    addErrorMessage(`会话加载异常: ${err.message}`);
  }
};

// 删除会话
const deleteSession = async (sessionId: string) => {
  try {
    await acp.deleteSession(sessionId);
    const current = state.get("sessionId");
    if (current === sessionId) {
      await createNewSession();
    } else {
      state.emit("requestRefreshSessionList");
    }
  } catch (err: any) {
    addErrorMessage(`会话删除异常: ${err.message}`);
  }
};

// 新建会话
const createNewSession = async () => {
  try {
    state.emit("requestUpdateAgentStatus", { status: "loading", label: "connecting" });
    const result = await acp.newSession();
    
    const newId = result.sessionId || result.session_id;
    state.set("sessionId", newId);
    currentSessionId.value = newId;
    state.emit("requestUpdateAgentStatus", { status: "online", label: "connected" });
    state.emit("updateProjectCwd", result.cwd || "/");

    messages.value = [];
    promptInputText.value = "";
    autoResizeInput();
    state.emit("requestRefreshSessionList");
  } catch (err) {
    addErrorMessage("新建会话创建失败");
  }
};

// 监听流式 Chunk 块
const handleSessionUpdate = (data: any) => {
  const update = data.update || data;
  const kind = update.update || update.kind || "";

  if (kind === "agent_message_chunk" || kind === "AgentMessageChunk") {
    const text = extractText(update.content || update);
    if (!text) return;

    const last = messages.value[messages.value.length - 1];
    if (last && last.type === "assistant") {
      last.text += text;
    } else {
      messages.value.push({
        type: "assistant",
        text
      });
    }
    scrollToBottom();
  } else if (kind === "agent_thought_chunk" || kind === "AgentThoughtChunk") {
    const text = extractText(update.content || update);
    if (!text) return;

    messages.value.push({
      type: "thought",
      text,
      expanded: true
    });
    
    // 3 秒后自动收拢
    const currentIdx = messages.value.length - 1;
    setTimeout(() => {
      if (messages.value[currentIdx]) {
        messages.value[currentIdx].expanded = false;
      }
    }, 3000);
    scrollToBottom();
  } else if (kind === "tool_call" || kind === "ToolCall") {
    const toolName = update.name || update.tool_name || "tool";
    const status = update.status || "running";

    // 查找是否已存在相同运行的工具
    const existingIdx = messages.value.findIndex(m => m.type === "tool" && m.toolName === toolName);
    if (existingIdx !== -1) {
      messages.value[existingIdx].status = status;
    } else {
      messages.value.push({
        type: "tool",
        toolName,
        status
      });
    }
    scrollToBottom();
  }
};

let unsubscribers: (() => void)[] = [];

onMounted(() => {
  // 1. 全局快捷键聚焦输入
  state.on("focusInputHotKey", () => {
    promptInputRef.value?.focus();
  });

  // 2. 状态监听绑定
  unsubscribers.push(
    state.subscribe("sessionId", (id) => {
      currentSessionId.value = id;
    })
  );

  // 订阅模型变更
  unsubscribers.push(
    state.subscribe("selectedModel", (model) => {
      selectedModel.value = model;
    })
  );

  // 3. 订阅流式事件广播
  acp.subscribeNotification(handleSessionUpdate);

  // 4. 绑定 Session 各种信号
  state.on("requestSwitchSession", (id: string) => {
    switchSession(id);
  });

  state.on("requestNewSession", () => {
    createNewSession();
  });

  state.on("requestDeleteSession", (id: string) => {
    deleteSession(id);
  });

  state.on("addSystemMessage", (text: string) => {
    addSystemMessage(text);
  });

  // 点击外部关闭模型下拉
  document.addEventListener("click", closeDropdown);
});

onUnmounted(() => {
  document.removeEventListener("click", closeDropdown);

  unsubscribers.forEach(un => un());
});
</script>

<style scoped>
.custom-scrollbar::-webkit-scrollbar {
  width: 4px;
}
.custom-scrollbar::-webkit-scrollbar-track {
  background: transparent;
}
.custom-scrollbar::-webkit-scrollbar-thumb {
  background: rgba(255, 255, 255, 0.05);
  border-radius: 99px;
}
.custom-scrollbar::-webkit-scrollbar-thumb:hover {
  background: rgba(255, 255, 255, 0.15);
}
.bg-primary-hover:hover {
  background-color: #e62e2e;
}
</style>
