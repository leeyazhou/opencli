// ============================================
// OpenCLI Desktop — Renderer Application
// ============================================

// ---- State ----

const state = {
  sessionId: null,
  initialized: false,
  agentRunning: false,
  streamingMessageEl: null,
  streamingContent: "",
  messages: [],
  selectedModel: "claude-opus-4-7",
  sidebarOpen: true,
  activePanel: "sessions",
};

// ---- DOM refs ----

const $ = (sel) => document.querySelector(sel);
const chatMessages = $("#chatMessages");
const chatViewport = $("#chatViewport");
const promptInput = $("#promptInput");
const sendBtn = $("#sendBtn");
const splash = $("#splash");
const scrollBottomBtn = $("#scrollBottomBtn");
const agentStatus = $("#agentStatus");
const sidebar = $("#sidebar");
const statusDot = agentStatus.querySelector(".status-dot");
const statusLabel = agentStatus.querySelector(".status-label");

// ---- Agent lifecycle ----

async function initAgent() {
  try {
    setAgentStatus("loading", "connecting");
    const initResult = await window.opencli.initialize();
    console.log("ACP initialize:", initResult);

    const sessionResult = await window.opencli.newSession();
    state.sessionId = sessionResult.sessionId || sessionResult.session_id;
    state.initialized = true;
    state.agentRunning = true;
    setAgentStatus("online", "connected");

    // Update sidebar
    $("#sessionIdDisplay").textContent = shortId(state.sessionId);
    $("#projectAgentStatus").textContent = "connected";
    $("#projectCwd").textContent = sessionResult.cwd || "/";

    console.log("ACP session:", state.sessionId);
  } catch (err) {
    console.error("Agent init failed:", err);
    setAgentStatus("error", "offline");
    $("#projectAgentStatus").textContent = "offline";
    addSystemMessage("Failed to connect to agent. Check that opencli is built.");
  }
}

function setAgentStatus(status, label) {
  statusDot.className = "status-dot";
  if (status === "offline" || status === "error" || status === "loading") {
    statusDot.classList.add(status);
  }
  statusLabel.textContent = label;
}

function shortId(id) {
  return id ? id.slice(0, 8) : "--";
}

// ---- Sidebar toggle (VS Code-style) ----

function toggleSidebar(panel) {
  if (state.sidebarOpen && state.activePanel === panel) {
    state.sidebarOpen = false;
    sidebar.classList.add("collapsed");
    document.querySelectorAll(".activity-btn").forEach((b) => b.classList.remove("active"));
  } else {
    if (!state.sidebarOpen) {
      state.sidebarOpen = true;
      sidebar.classList.remove("collapsed");
    }
    state.activePanel = panel;
    document.querySelectorAll(".sidebar-panel").forEach((p) => p.classList.remove("active"));
    document.querySelectorAll(".activity-btn").forEach((b) => b.classList.remove("active"));

    const panelEl = $("#panel" + panel.charAt(0).toUpperCase() + panel.slice(1));
    if (panelEl) panelEl.classList.add("active");

    const btn = document.querySelector(`.activity-btn[data-panel="${panel}"]`);
    if (btn) btn.classList.add("active");
  }
  window.opencli.storeSet("sidebarOpen", state.sidebarOpen);
}

document.querySelectorAll(".activity-btn").forEach((btn) => {
  btn.addEventListener("click", () => {
    toggleSidebar(btn.dataset.panel);
  });
});

// Ctrl+B keyboard shortcut
document.addEventListener("keydown", (e) => {
  if ((e.ctrlKey || e.metaKey) && e.key === "b") {
    e.preventDefault();
    toggleSidebar("sessions");
  }
});

// ---- Sidebar resize ----

const resizeHandle = $("#sidebarResizeHandle");
let resizeStartX = 0;
let resizeStartWidth = 0;

resizeHandle.addEventListener("mousedown", (e) => {
  e.preventDefault();
  resizeStartX = e.clientX;
  resizeStartWidth = sidebar.offsetWidth;
  resizeHandle.classList.add("active");
  document.body.style.cursor = "col-resize";

  const onMove = (e) => {
    const diff = e.clientX - resizeStartX;
    const newWidth = Math.max(180, Math.min(500, resizeStartWidth + diff));
    sidebar.style.width = newWidth + "px";
  };

  const onUp = () => {
    resizeHandle.classList.remove("active");
    document.body.style.cursor = "";
    document.removeEventListener("mousemove", onMove);
    document.removeEventListener("mouseup", onUp);
    window.opencli.storeSet("sidebarWidth", sidebar.offsetWidth);
  };

  document.addEventListener("mousemove", onMove);
  document.addEventListener("mouseup", onUp);
});

// ---- Model selector ----

const modelSelectorBtn = $("#modelSelectorBtn");
const modelDropdown = $("#modelDropdown");
const currentModelName = $("#currentModelName");

modelSelectorBtn.addEventListener("click", (e) => {
  e.stopPropagation();
  modelDropdown.classList.toggle("open");
});

document.addEventListener("click", () => {
  modelDropdown.classList.remove("open");
});

modelDropdown.addEventListener("click", (e) => {
  e.stopPropagation();
  const option = e.target.closest(".model-option");
  if (!option) return;

  state.selectedModel = option.dataset.model;
  window.opencli.storeSet("model", option.dataset.model);

  document.querySelectorAll(".model-option").forEach((o) => o.classList.remove("active"));
  option.classList.add("active");
  currentModelName.textContent = option.dataset.model;
  modelDropdown.classList.remove("open");
});

// ---- Notification handler ----

window.opencli.onNotification((data) => {
  if (data.type === "sessionUpdate") {
    handleSessionUpdate(data);
  }
});

window.opencli.onLog((data) => {
  console.log(`[agent:${data.level}]`, data.text);
});

window.opencli.onRaw((data) => {
  console.log("[agent:raw]", data);
});

function handleSessionUpdate(data) {
  const update = data.update || data;
  const kind = update.update || update.kind || "";

  if (kind === "agent_message_chunk" || kind === "AgentMessageChunk") {
    handleMessageChunk(update);
  } else if (kind === "agent_thought_chunk" || kind === "AgentThoughtChunk") {
    handleThoughtChunk(update);
  } else if (kind === "tool_call" || kind === "ToolCall") {
    handleToolCall(update);
  }
}

function handleMessageChunk(update) {
  const text = extractText(update.content || update);
  if (!text) return;

  state.streamingContent += text;

  if (!state.streamingMessageEl) {
    hideSplash();
    state.streamingMessageEl = createMessageBubble("assistant", "");
    chatMessages.appendChild(state.streamingMessageEl);
  }

  const body = state.streamingMessageEl.querySelector(".message-body");
  body.innerHTML = renderMd(state.streamingContent) + '<span class="streaming-cursor"></span>';
  scrollToBottom();
}

function handleThoughtChunk(update) {
  const text = extractText(update.content || update);
  if (!text) return;

  hideSplash();
  const el = createMessageBubble("thought", text);
  chatMessages.appendChild(el);

  setTimeout(() => {
    el.classList.remove("expanded");
  }, 3000);

  scrollToBottom();
}

function handleToolCall(update) {
  hideSplash();
  const toolName = update.name || update.tool_name || "tool";
  const status = update.status || "running";
  const el = createToolMessage(toolName, status);
  chatMessages.appendChild(el);

  // Update if tool already exists
  const existing = document.querySelector(`.message.tool[data-tool-name="${toolName}"]`);
  if (existing && existing !== el) {
    existing.querySelector(".tool-badge").textContent = status;
    existing.querySelector(".tool-badge").className = `tool-badge ${status}`;
    el.remove();
  } else {
    scrollToBottom();
  }
}

function extractText(content) {
  if (!content) return "";
  if (typeof content === "string") return content;
  if (content.text) return content.text;
  if (Array.isArray(content)) {
    return content.map((b) => (typeof b === "string" ? b : b.text || "")).join("");
  }
  return "";
}

// ---- Message rendering ----

function createMessageBubble(type, text) {
  const el = document.createElement("div");
  el.className = `message ${type}`;

  const header = document.createElement("div");
  header.className = "message-header";
  header.textContent = typeLabels[type] || type;

  const body = document.createElement("div");
  body.className = "message-body";
  body.innerHTML = renderMd(text);

  el.appendChild(header);
  el.appendChild(body);

  if (type === "thought") {
    el.classList.add("expanded");
    el.addEventListener("click", () => el.classList.toggle("expanded"));
  }

  return el;
}

function createToolMessage(name, status) {
  const el = document.createElement("div");
  el.className = "message tool";
  el.dataset.toolName = name;

  const header = document.createElement("div");
  header.className = "message-header";
  header.textContent = "TOOL";

  const badge = document.createElement("span");
  badge.className = `tool-badge ${status}`;
  badge.textContent = status;
  header.appendChild(badge);

  const body = document.createElement("div");
  body.className = "message-body";
  body.textContent = name;

  el.appendChild(header);
  el.appendChild(body);

  return el;
}

function addSystemMessage(text) {
  hideSplash();
  const el = document.createElement("div");
  el.className = "message system";
  el.textContent = text;
  chatMessages.appendChild(el);
  scrollToBottom();
}

function addErrorMessage(text) {
  hideSplash();
  const el = document.createElement("div");
  el.className = "message error";

  const header = document.createElement("div");
  header.className = "message-header";
  header.textContent = "ERROR";

  const body = document.createElement("div");
  body.className = "message-body";
  body.textContent = text;

  el.appendChild(header);
  el.appendChild(body);
  chatMessages.appendChild(el);
  scrollToBottom();
}

function addUserMessage(text) {
  hideSplash();
  const el = createMessageBubble("user", text);
  chatMessages.appendChild(el);
  scrollToBottom();
}

const typeLabels = {
  user: "YOU",
  assistant: "ASSISTANT",
  thought: "THINKING",
  tool: "TOOL",
  system: "",
  error: "ERROR",
};

// ---- Rendering (uses marked via preload) ----

function renderMd(text) {
  return window.opencli.renderMarkdown(text);
}

// ---- Sending messages ----

async function sendMessage() {
  const text = promptInput.value.trim();
  if (!text) return;

  if (!state.initialized || !state.sessionId) {
    addSystemMessage("Agent not ready. Initializing...");
    await initAgent();
    if (!state.sessionId) {
      addErrorMessage("Could not connect to agent.");
      return;
    }
  }

  addUserMessage(text);
  updateMsgCount();
  promptInput.value = "";
  autoResize();
  sendBtn.disabled = true;

  if (state.streamingMessageEl) {
    const cursor = state.streamingMessageEl.querySelector(".streaming-cursor");
    if (cursor) cursor.remove();
    state.streamingMessageEl = null;
    state.streamingContent = "";
  }

  try {
    const result = await window.opencli.prompt(state.sessionId, text);
    console.log("Prompt result:", result);

    if (state.streamingMessageEl) {
      const cursor = state.streamingMessageEl.querySelector(".streaming-cursor");
      if (cursor) cursor.remove();
      state.streamingMessageEl = null;
      state.streamingContent = "";
    }

    if (result.content || result.text || result.message) {
      const responseText = extractText(result.content || result.text || result.message || result);
      if (responseText && !state.streamingMessageEl) {
        const el = createMessageBubble("assistant", responseText);
        chatMessages.appendChild(el);
        scrollToBottom();
      }
    }
  } catch (err) {
    console.error("Prompt failed:", err);
    addErrorMessage(err.message || "Prompt request failed");
  } finally {
    sendBtn.disabled = false;
    updateMsgCount();
  }
}

function updateMsgCount() {
  const count = chatMessages.querySelectorAll(".message").length;
  $("#projectMsgCount").textContent = count;
}

// ---- Auto-resize textarea ----

function autoResize() {
  promptInput.style.height = "auto";
  promptInput.style.height = Math.min(promptInput.scrollHeight, 120) + "px";
}

// ---- Scroll management ----

function scrollToBottom() {
  chatViewport.scrollTop = chatViewport.scrollHeight;
}

function isNearBottom() {
  const threshold = 80;
  return chatViewport.scrollHeight - chatViewport.scrollTop - chatViewport.clientHeight < threshold;
}

chatViewport.addEventListener("scroll", () => {
  if (isNearBottom()) {
    scrollBottomBtn.classList.remove("visible");
  } else {
    scrollBottomBtn.classList.add("visible");
  }
});

scrollBottomBtn.addEventListener("click", () => {
  scrollToBottom();
  scrollBottomBtn.classList.remove("visible");
});

function hideSplash() {
  if (splash) splash.style.display = "none";
}

// ---- New session button ----

$("#newSessionBtn").addEventListener("click", async () => {
  try {
    setAgentStatus("loading", "connecting");
    const result = await window.opencli.newSession();
    state.sessionId = result.sessionId || result.session_id;
    setAgentStatus("online", "connected");
    $("#sessionIdDisplay").textContent = shortId(state.sessionId);

    // Clear chat
    chatMessages.innerHTML = "";
    const splashDiv = document.createElement("div");
    splashDiv.className = "splash";
    splashDiv.id = "splash";
    splashDiv.innerHTML = splashOriginalHTML;
    chatMessages.appendChild(splashDiv);

    addSystemMessage("New session created.");
    updateMsgCount();
  } catch (err) {
    addErrorMessage("Failed to create new session.");
  }
});

const splashOriginalHTML = splash ? splash.innerHTML : "";

// ---- Keyboard shortcuts ----

promptInput.addEventListener("keydown", (e) => {
  if (e.key === "Enter" && !e.shiftKey) {
    e.preventDefault();
    sendMessage();
  }
});

// ---- Particle background ----

function initParticles() {
  const container = document.getElementById("particles");
  if (!container) return;

  const canvas = document.createElement("canvas");
  container.appendChild(canvas);
  const ctx = canvas.getContext("2d");

  let width, height;
  const particles = [];
  const PARTICLE_COUNT = 60;

  function resize() {
    width = canvas.width = window.innerWidth;
    height = canvas.height = window.innerHeight;
  }
  resize();
  window.addEventListener("resize", resize);

  class Particle {
    constructor() {
      this.reset(true);
    }

    reset(initial) {
      this.x = Math.random() * width;
      this.y = initial ? Math.random() * height : height + 10;
      this.size = Math.random() * 1.5 + 0.5;
      this.speedY = -(Math.random() * 0.3 + 0.1);
      this.speedX = (Math.random() - 0.5) * 0.2;
      this.opacity = Math.random() * 0.4 + 0.1;
      this.fadeRate = Math.random() * 0.002 + 0.001;
    }

    update() {
      this.x += this.speedX;
      this.y += this.speedY;
      this.opacity -= this.fadeRate;
      this.x += Math.sin(this.y * 0.02) * 0.05;

      if (this.opacity <= 0 || this.y < -10) {
        this.reset(false);
      }
    }

    draw(ctx) {
      ctx.beginPath();
      ctx.arc(this.x, this.y, this.size, 0, Math.PI * 2);
      ctx.fillStyle = `rgba(99, 102, 241, ${this.opacity})`;
      ctx.fill();
    }
  }

  for (let i = 0; i < PARTICLE_COUNT; i++) {
    particles.push(new Particle());
  }

  function animate() {
    ctx.clearRect(0, 0, width, height);

    for (const p of particles) {
      p.update();
      p.draw(ctx);
    }

    for (let i = 0; i < particles.length; i++) {
      for (let j = i + 1; j < particles.length; j++) {
        const dx = particles[i].x - particles[j].x;
        const dy = particles[i].y - particles[j].y;
        const dist = Math.sqrt(dx * dx + dy * dy);

        if (dist < 80) {
          const alpha = (1 - dist / 80) * 0.06;
          ctx.beginPath();
          ctx.moveTo(particles[i].x, particles[i].y);
          ctx.lineTo(particles[j].x, particles[j].y);
          ctx.strokeStyle = `rgba(99, 102, 241, ${alpha})`;
          ctx.lineWidth = 0.5;
          ctx.stroke();
        }
      }
    }

    requestAnimationFrame(animate);
  }

  animate();
}

// ---- Bootstrap ----

async function bootstrap() {
  initParticles();
  promptInput.addEventListener("input", autoResize);
  sendBtn.addEventListener("click", sendMessage);

  // Restore persisted preferences
  try {
    const savedModel = await window.opencli.storeGet("model");
    if (savedModel) {
      state.selectedModel = savedModel;
      currentModelName.textContent = savedModel;
      document.querySelectorAll(".model-option").forEach((o) => {
        o.classList.toggle("active", o.dataset.model === savedModel);
      });
    }

    const savedSidebarOpen = await window.opencli.storeGet("sidebarOpen");
    if (savedSidebarOpen !== undefined && savedSidebarOpen !== null) {
      state.sidebarOpen = savedSidebarOpen;
      if (!savedSidebarOpen) {
        sidebar.classList.add("collapsed");
        document.querySelectorAll(".activity-btn").forEach((b) => b.classList.remove("active"));
      }
    }

    const savedSidebarWidth = await window.opencli.storeGet("sidebarWidth");
    if (savedSidebarWidth) {
      sidebar.style.width = savedSidebarWidth + "px";
    }
  } catch (e) {
    // Store may not be available on first run
  }

  try {
    const status = await window.opencli.agentStatus();
    if (status.running) {
      state.agentRunning = true;
      setAgentStatus("online", "connected");
      await initAgent();
    } else {
      setAgentStatus("offline", "starting");
      await window.opencli.startAgent();
      setTimeout(async () => {
        await initAgent();
      }, 500);
    }
  } catch (err) {
    console.error("Bootstrap error:", err);
    try {
      await window.opencli.startAgent();
      setTimeout(async () => {
        await initAgent();
      }, 800);
    } catch (e) {
      setAgentStatus("error", "offline");
      addSystemMessage("Could not start agent. Build `opencli` first: cargo build --release");
    }
  }
}

bootstrap();
