const { app, BrowserWindow, ipcMain } = require("electron");
const { spawn } = require("child_process");
const path = require("path");
const readline = require("readline");
const fs = require("fs");

// Production-grade logging
const log = require("electron-log");
log.initialize();
log.transports.file.level = "debug";
log.transports.console.level = "debug";

// Persistent user preferences (zero-dependency JSON store)
function createStore(defaults) {
  const userDataPath = app.getPath("userData");
  const storePath = path.join(userDataPath, "preferences.json");
  let data = { ...defaults };

  try {
    if (fs.existsSync(storePath)) {
      const raw = fs.readFileSync(storePath, "utf-8");
      data = { ...defaults, ...JSON.parse(raw) };
    }
  } catch {
    log.warn("Failed to load preferences, using defaults");
  }

  const save = () => {
    try {
      const dir = path.dirname(storePath);
      if (!fs.existsSync(dir)) fs.mkdirSync(dir, { recursive: true });
      fs.writeFileSync(storePath, JSON.stringify(data, null, 2), "utf-8");
    } catch (e) {
      log.warn(`Failed to save preferences: ${e.message}`);
    }
  };

  return {
    get(key) { return data[key]; },
    set(key, value) { data[key] = value; save(); },
  };
}

const store = createStore({
  model: "claude-opus-4-7",
  sidebarOpen: true,
  sidebarWidth: 260,
  windowBounds: { width: 1200, height: 800 },
});

// Native right-click context menu
const contextMenu = require("electron-context-menu");
contextMenu({
  showSearchWithGoogle: false,
  showCopyImage: false,
  showSaveImageAs: false,
  showInspectElement: false,
});

let mainWindow = null;
let agentProcess = null;
let agentReader = null;
let msgId = 0;
const pendingRequests = new Map();

// --- ACP Agent Process Management ---

function resolveOpencliBinary() {
  const exeName = process.platform === "win32" ? "opencli.exe" : "opencli";

  // Packaged app: binary bundled via electron-builder extraResources
  if (app.isPackaged) {
    return path.join(process.resourcesPath, exeName);
  }

  // Dev mode: look in rust target directories
  const releaseBin = path.resolve(app.getAppPath(), "..", "target", "release", exeName);
  const debugBin = path.resolve(app.getAppPath(), "..", "target", "debug", exeName);

  if (fs.existsSync(releaseBin)) return releaseBin;
  if (fs.existsSync(debugBin)) return debugBin;
  return releaseBin;
}

function startAgent() {
  const bin = resolveOpencliBinary();

  log.info(`Starting agent: ${bin}`);
  logToRenderer("system", `Starting agent: ${bin}`);

  agentProcess = spawn(bin, ["acp"], {
    stdio: ["pipe", "pipe", "pipe"],
    env: { ...process.env },
  });

  agentReader = readline.createInterface({
    input: agentProcess.stdout,
    crlfDelay: Infinity,
  });

  agentReader.on("line", (line) => {
    try {
      const msg = JSON.parse(line.trim());
      handleAgentMessage(msg);
    } catch {
      log.debug(`[agent stdout] ${line}`);
      logToRenderer("stderr", line);
    }
  });

  if (agentProcess.stderr) {
    const stderrReader = readline.createInterface({
      input: agentProcess.stderr,
      crlfDelay: Infinity,
    });
    stderrReader.on("line", (line) => {
      log.warn(`[agent stderr] ${line}`);
      logToRenderer("stderr", line);
    });
  }

  agentProcess.on("exit", (code) => {
    log.info(`Agent exited with code ${code}`);
    logToRenderer("system", `Agent exited with code ${code}`);
    agentProcess = null;
  });

  agentProcess.on("error", (err) => {
    log.error(`Agent process error: ${err.message}`);
    logToRenderer("error", `Agent process error: ${err.message}`);
  });
}

function isRendererAlive() {
  return mainWindow && !mainWindow.isDestroyed() && !mainWindow.webContents.isDestroyed();
}

function handleAgentMessage(msg) {
  if (msg.id !== undefined && (msg.result !== undefined || msg.error !== undefined)) {
    const pending = pendingRequests.get(msg.id);
    if (pending) {
      pendingRequests.delete(msg.id);
      if (msg.error) {
        log.warn(`Agent RPC error: ${msg.error.message}`);
        pending.reject(new Error(msg.error.message || "Agent error"));
      } else {
        pending.resolve(msg.result);
      }
    }
    return;
  }

  if (msg.method === "session/update") {
    const update = msg.params || msg;
    if (isRendererAlive()) {
      mainWindow.webContents.send("acp:notification", {
        type: "sessionUpdate",
        sessionId: update.sessionId || update.session_id,
        update: update.update || update,
        meta: update._meta || update.meta,
      });
    }
    return;
  }

  if (isRendererAlive()) {
    mainWindow.webContents.send("acp:raw", msg);
  }
}

function sendToAgent(payload) {
  if (!agentProcess || agentProcess.killed) {
    return Promise.reject(new Error("Agent not running"));
  }

  const id = String(++msgId);
  const msg = {
    jsonrpc: "2.0",
    method: payload.method,
    params: payload.params || {},
    id,
  };

  log.debug(`RPC -> ${payload.method} (id=${id})`);

  return new Promise((resolve, reject) => {
    pendingRequests.set(id, { resolve, reject });
    agentProcess.stdin.write(JSON.stringify(msg) + "\n");
  });
}

// --- IPC Handlers ---

function setupIPC() {
  ipcMain.handle("acp:initialize", async () => {
    return sendToAgent({
      method: "initialize",
      params: {
        protocolVersion: "1",
        clientCapabilities: {},
      },
    });
  });

  ipcMain.handle("acp:newSession", async () => {
    const cwd = process.cwd();
    return sendToAgent({
      method: "session/new",
      params: { cwd, mcpServers: [] },
    });
  });

  ipcMain.handle("acp:prompt", async (_event, { sessionId, prompt }) => {
    log.info(`Prompt to session ${sessionId}: "${prompt.slice(0, 80)}"`);
    return sendToAgent({
      method: "session/prompt",
      params: {
        sessionId,
        prompt: [{ type: "text", text: prompt }],
      },
    });
  });

  ipcMain.handle("agent:start", () => {
    startAgent();
    return true;
  });

  ipcMain.handle("agent:status", () => {
    return {
      running: agentProcess !== null && !agentProcess.killed,
      pid: agentProcess?.pid || null,
    };
  });

  // Persisted preferences
  ipcMain.handle("store:get", (_event, key) => {
    return store.get(key);
  });

  ipcMain.handle("store:set", (_event, key, value) => {
    store.set(key, value);
    return true;
  });
}

// --- Log forwarding ---

function logToRenderer(level, text) {
  if (mainWindow && !mainWindow.isDestroyed() && !mainWindow.webContents.isDestroyed()) {
    mainWindow.webContents.send("acp:log", { level, text });
  }
}

// --- Window Creation ---

function createWindow() {
  const savedBounds = store.get("windowBounds");

  mainWindow = new BrowserWindow({
    width: savedBounds.width,
    height: savedBounds.height,
    minWidth: 800,
    minHeight: 500,
    title: "OpenCLI",
    backgroundColor: "#0a0a0f",
    titleBarStyle: "hiddenInset",
    trafficLightPosition: { x: 16, y: 16 },
    webPreferences: {
      preload: path.join(__dirname, "../preload/index.js"),
      contextIsolation: true,
      nodeIntegration: false,
    },
  });

  // Persist window bounds on resize
  mainWindow.on("resize", () => {
    if (!mainWindow.isDestroyed()) {
      const bounds = mainWindow.getBounds();
      store.set("windowBounds", { width: bounds.width, height: bounds.height });
    }
  });

  // Load renderer (dev server or built file)
  if (process.env.ELECTRON_RENDERER_URL) {
    mainWindow.loadURL(process.env.ELECTRON_RENDERER_URL);
  } else {
    mainWindow.loadFile(path.join(__dirname, "../renderer/index.html"));
  }

  mainWindow.once("ready-to-show", () => {
    startAgent();
  });
}

// --- App Lifecycle ---

app.whenReady().then(() => {
  setupIPC();
  createWindow();
  log.info("OpenCLI Desktop started");
});

app.on("window-all-closed", () => {
  if (agentProcess) agentProcess.kill();
  if (process.platform !== "darwin") app.quit();
});

app.on("activate", () => {
  if (BrowserWindow.getAllWindows().length === 0) createWindow();
});

app.on("before-quit", () => {
  if (agentProcess) agentProcess.kill();
  log.info("OpenCLI Desktop shutting down");
});
