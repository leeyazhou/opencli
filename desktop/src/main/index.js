const { app, BrowserWindow, ipcMain, shell, session } = require("electron");
const { spawn } = require("child_process");
const path = require("path");
const readline = require("readline");
const fs = require("fs");

// Production-grade logging
const log = require("electron-log");
log.initialize();
log.transports.file.level = "debug";
log.transports.console.level = "debug";

// Global uncaught exception watchdog for main process resilience
process.on("uncaughtException", (err) => {
  log.error("CRITICAL: Uncaught Exception in Main Process:", err);
});
process.on("unhandledRejection", (reason, promise) => {
  log.error("CRITICAL: Unhandled Rejection at:", promise, "reason:", reason);
});

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

// Clear outstanding RPC requests on backend crash or shutdown to avoid perpetual loading states
function rejectAllPendingRequests(reason) {
  log.warn(`Rejecting all pending RPC requests: ${reason}`);
  for (const [id, pending] of pendingRequests.entries()) {
    try {
      pending.reject(new Error(`${reason} (msg_id: ${id})`));
    } catch (e) {
      log.error(`Failed to reject pending request ${id}: ${e.message}`);
    }
  }
  pendingRequests.clear();
}

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
    rejectAllPendingRequests(`Agent process exited with code ${code}`);
  });

  agentProcess.on("error", (err) => {
    log.error(`Agent process error: ${err.message}`);
    logToRenderer("error", `Agent process error: ${err.message}`);
    rejectAllPendingRequests(`Agent process error: ${err.message}`);
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

  ipcMain.handle("acp:prompt", async (_event, { sessionId, prompt, model }) => {
    log.info(`Prompt to session ${sessionId} [${model}]: "${prompt.slice(0, 80)}"`);
    return sendToAgent({
      method: "session/prompt",
      params: {
        sessionId,
        prompt: [{ type: "text", text: prompt }],
        _meta: model ? { model } : undefined,
      },
    });
  });

  ipcMain.handle("acp:listSessions", async () => {
    return sendToAgent({ method: "session/list", params: {} });
  });

  ipcMain.handle("acp:loadSession", async (_event, { id }) => {
    return sendToAgent({ method: "session/load", params: { id } });
  });

  ipcMain.handle("acp:deleteSession", async (_event, { id }) => {
    return sendToAgent({ method: "session/delete", params: { id } });
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

  const winOptions = {
    width: savedBounds.width,
    height: savedBounds.height,
    minWidth: 800,
    minHeight: 500,
    title: "OpenCLI",
    backgroundColor: "#0a0a0f",
    titleBarStyle: "hidden",
    webPreferences: {
      preload: path.join(__dirname, "../preload/index.js"),
      contextIsolation: true,
      nodeIntegration: false,
    },
  };

  if (process.platform === "darwin") {
    winOptions.trafficLightPosition = { x: 16, y: 12 };
  }

  mainWindow = new BrowserWindow(winOptions);


  // 自动在开发环境下开启开发者工具以抓取并诊断前端白屏 JS 运行时报错
  mainWindow.webContents.openDevTools();


  // Persist window bounds on resize
  mainWindow.on("resize", () => {
    if (!mainWindow.isDestroyed()) {
      const bounds = mainWindow.getBounds();
      store.set("windowBounds", { width: bounds.width, height: bounds.height });
    }
  });

  // Renderer Security: Intercept will-navigate to block external navigation hijack
  mainWindow.webContents.on("will-navigate", (event, url) => {
    const isLocal = url.startsWith("file://") || 
                    url.startsWith("http://localhost:") || 
                    url.startsWith("http://127.0.0.1:") || 
                    url.startsWith("http://[::1]:");
    if (!isLocal) {
      event.preventDefault();
      log.warn(`Blocked main window navigation to external URL: ${url}`);
      shell.openExternal(url).catch((err) => {
        log.error(`Failed to open external link: ${err.message}`);
      });
    }
  });

  // Renderer Security: Force all external window opens to load in default browser
  mainWindow.webContents.setWindowOpenHandler(({ url }) => {
    if (url.startsWith("http:") || url.startsWith("https:")) {
      log.info(`Forwarding external window open to system browser: ${url}`);
      shell.openExternal(url).catch((err) => {
        log.error(`Failed to open external link: ${err.message}`);
      });
    } else {
      log.warn(`Blocked local or non-standard window open attempt: ${url}`);
    }
    return { action: "deny" };
  });

  // Hardened sandbox: Block all device hardware and permission requests
  session.defaultSession.setPermissionRequestHandler((webContents, permission, callback) => {
    log.warn(`Blocked unprivileged web application permission request: ${permission}`);
    callback(false);
  });

  // 提前启动 Rust 代理进程，让其与渲染进程并行热身，从根源上消灭启动竞态问题
  startAgent();

  // Load renderer (dev server or built file)
  if (process.env.ELECTRON_RENDERER_URL) {
    mainWindow.loadURL(process.env.ELECTRON_RENDERER_URL);
  } else {
    mainWindow.loadFile(path.join(__dirname, "../renderer/index.html"));
  }

  mainWindow.once("ready-to-show", () => {
    if (process.platform === "darwin" && typeof mainWindow.setTrafficLightPosition === "function") {
      try {
        mainWindow.setTrafficLightPosition({ x: 16, y: 12 });
      } catch (err) {
        log.error(`Failed to set traffic light position: ${err.message}`);
      }
    }
  });
}


// --- App Lifecycle ---

app.whenReady().then(() => {
  setupIPC();
  createWindow();
  log.info("OpenCLI Desktop started");
});

// Gracefully terminate the Rust child process using Stdin EOF, falling back to SIGKILL
function stopAgentGracefully() {
  if (!agentProcess || agentProcess.killed) {
    return Promise.resolve();
  }
  log.info("Initiating graceful shutdown of Rust agent via stdin EOF...");
  
  return new Promise((resolve) => {
    const proc = agentProcess;
    const timer = setTimeout(() => {
      if (proc && !proc.killed) {
        log.warn("Agent graceful shutdown timed out, enforcing SIGKILL...");
        try {
          proc.kill("SIGKILL");
        } catch (e) {
          log.error(`Failed to force kill agent: ${e.message}`);
        }
      }
      resolve();
    }, 2000); // 2 second threshold for Rust persistence flush

    proc.once("exit", () => {
      clearTimeout(timer);
      log.info("Agent process exited cleanly.");
      resolve();
    });

    try {
      proc.stdin.end(); // Closing stdin pipe triggers Stdin EOF in Rust ACP loop
    } catch (e) {
      log.error(`Failed to close agent stdin: ${e.message}`);
      try {
        proc.kill();
      } catch (_) {}
      resolve();
    }
  });
}

app.on("window-all-closed", async () => {
  await stopAgentGracefully();
  if (process.platform !== "darwin") app.quit();
});

app.on("activate", () => {
  if (BrowserWindow.getAllWindows().length === 0) createWindow();
});

app.on("before-quit", async (event) => {
  // Prevent immediate quit to allow asynchronous graceful child process cleanup
  if (agentProcess && !agentProcess.killed) {
    event.preventDefault();
    await stopAgentGracefully();
    app.quit();
  } else {
    log.info("OpenCLI Desktop shutting down");
  }
});
