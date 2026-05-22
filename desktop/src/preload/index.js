const { contextBridge, ipcRenderer } = require("electron");
const { marked } = require("marked");
const DOMPurify = require("dompurify");

// Configure marked
marked.setOptions({
  breaks: true,
  gfm: true,
});

contextBridge.exposeInMainWorld("opencli", {
  // ACP protocol
  initialize: () => ipcRenderer.invoke("acp:initialize"),
  newSession: () => ipcRenderer.invoke("acp:newSession"),
  prompt: (sessionId, prompt) =>
    ipcRenderer.invoke("acp:prompt", { sessionId, prompt }),

  // Agent lifecycle
  startAgent: () => ipcRenderer.invoke("agent:start"),
  agentStatus: () => ipcRenderer.invoke("agent:status"),

  // Persisted preferences
  storeGet: (key) => ipcRenderer.invoke("store:get", key),
  storeSet: (key, value) => ipcRenderer.invoke("store:set", key, value),

  // Markdown rendering with XSS sanitization
  // marked converts markdown to HTML, DOMPurify sanitizes it
  renderMarkdown: (text) => {
    if (!text) return "";
    try {
      const rawHtml = marked.parse(text);
      return DOMPurify.sanitize(rawHtml, {
        ALLOWED_TAGS: [
          "p", "br", "strong", "em", "u", "s", "del",
          "h1", "h2", "h3", "h4", "h5", "h6",
          "ul", "ol", "li",
          "blockquote",
          "pre", "code",
          "a", "img",
          "table", "thead", "tbody", "tr", "th", "td",
          "hr",
          "span", "div",
        ],
        ALLOWED_ATTR: ["href", "src", "alt", "class", "target", "rel"],
        ALLOW_DATA_ATTR: false,
      });
    } catch {
      return escapeHtml(text);
    }
  },

  // Events from main process
  onNotification: (callback) => {
    const handler = (_event, data) => callback(data);
    ipcRenderer.on("acp:notification", handler);
    return () => ipcRenderer.removeListener("acp:notification", handler);
  },

  onLog: (callback) => {
    const handler = (_event, data) => callback(data);
    ipcRenderer.on("acp:log", handler);
    return () => ipcRenderer.removeListener("acp:log", handler);
  },

  onRaw: (callback) => {
    const handler = (_event, data) => callback(data);
    ipcRenderer.on("acp:raw", handler);
    return () => ipcRenderer.removeListener("acp:raw", handler);
  },
});

function escapeHtml(text) {
  return text
    .replace(/&/g, "&amp;")
    .replace(/</g, "&lt;")
    .replace(/>/g, "&gt;");
}
