/**
 * @typedef {import('../renderer/types/acp').ACPInitializeResponse} ACPInitializeResponse
 * @typedef {import('../renderer/types/acp').ACPSession} ACPSession
 * @typedef {import('../renderer/types/acp').ACPAgentStatus} ACPAgentStatus
 */

const { contextBridge, ipcRenderer } = require("electron");

contextBridge.exposeInMainWorld("opencli", {
  // ACP protocol
  initialize: () => ipcRenderer.invoke("acp:initialize"),
  newSession: () => ipcRenderer.invoke("acp:newSession"),
  prompt: (sessionId, prompt, model) =>
    ipcRenderer.invoke("acp:prompt", { sessionId, prompt, model }),

  // Session management
  listSessions: () => ipcRenderer.invoke("acp:listSessions"),
  loadSession: (id) => ipcRenderer.invoke("acp:loadSession", { id }),
  deleteSession: (id) => ipcRenderer.invoke("acp:deleteSession", { id }),

  // Agent lifecycle
  startAgent: () => ipcRenderer.invoke("agent:start"),
  agentStatus: () => ipcRenderer.invoke("agent:status"),

  // Persisted preferences
  storeGet: (key) => ipcRenderer.invoke("store:get", key),
  storeSet: (key, value) => ipcRenderer.invoke("store:set", key, value),

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
