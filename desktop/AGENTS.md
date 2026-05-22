# OpenCLI Desktop

Electron desktop client for the opencli AI coding assistant. Communicates with the `opencli acp` Rust backend via ACP (Agent Client Protocol) JSON-RPC 2.0 over stdio. Built with electron-vite, targets macOS / Linux / Windows.

## Architecture

Three-layer process model:

```
[Renderer]  ← IPC (contextBridge) →  [Main]  ← stdin/stdout JSON-RPC →  [opencli acp child]
 app.js         preload/index.js    index.js                               Rust ACP agent
 style.css                           Window mgmt                           Session persistence
 index.html                          Agent lifecycle                       Tool execution
```

### Main process (`src/main/index.js`)

- **Window management**: creates the BrowserWindow with `titleBarStyle: hiddenInset`, `contextIsolation: true`, `nodeIntegration: false`
- **Agent lifecycle**: spawns `opencli acp` as a child process, pipes stdin/stdout via readline
- **RPC dispatch**: assigns monotonically increasing `msgId` to outgoing requests, resolves/rejects via `pendingRequests` Map when responses arrive
- **Notification forwarding**: `session/update` notifications are forwarded to renderer via `mainWindow.webContents.send("acp:notification", ...)`
- **IPC handlers**: `acp:initialize`, `acp:newSession`, `acp:prompt`, `acp:listSessions`, `acp:loadSession`, `acp:deleteSession`, `agent:start`, `agent:status`, `store:get`, `store:set`
- **JSON store**: zero-dependency preferences file at `{userData}/preferences.json` (replaces electron-store which is ESM-only)
- **Binary resolution**: `resolveOpencliBinary()` searches `process.resourcesPath` (packaged) then `target/release` / `target/debug` (dev)
- **Safety**: `isRendererAlive()` guard before every `webContents.send()` to prevent "Object has been destroyed" crashes

### Preload (`src/preload/index.js`)

- Exposes `window.opencli` API via `contextBridge.exposeInMainWorld()`:
  - ACP protocol: `initialize()`, `newSession()`, `prompt(sessionId, prompt, model)`
  - Session management: `listSessions()`, `loadSession(id)`, `deleteSession(id)`
  - Agent lifecycle: `startAgent()`, `agentStatus()`
  - Preferences: `storeGet(key)`, `storeSet(key, value)`
  - Markdown: `renderMarkdown(text)` — marked → DOMPurify (allowlist: p, br, strong, em, code, pre, table, blockquote, ul, ol, li, h1-h6, a, img, hr, span, div)
  - Events: `onNotification(cb)`, `onLog(cb)`, `onRaw(cb)`

### Renderer (`src/renderer/`)

| File | Role |
|---|---|
| `index.html` | VS Code-style layout: activity bar (48px) + collapsible sidebar (260px) + chat area. Activity bar buttons for sessions/project/settings panels. Model selector dropdown in input toolbar. Resize handle on sidebar right edge. |
| `app.js` | Application state machine: session lifecycle, message rendering, streaming chunk handling, command palette, theme switching, keyboard shortcuts |
| `style.css` | "Antigravity 2.0" glassmorphism design system. 7 built-in themes via `[data-theme="..."]` CSS selectors. Particle canvas background. |

### Message flow (prompt lifecycle)

1. User types prompt, presses Enter → `sendMessage()`
2. Renderer calls `window.opencli.prompt(sessionId, text, model)` → IPC `acp:prompt`
3. Main process sends JSON-RPC: `{ method: "session/prompt", params: { sessionId, prompt, _meta: { model } } }`
4. ACP agent loads session from disk, appends user message, runs agent turn, saves session
5. Agent sends `session/update` notifications (thought chunks, tool calls, message chunks) back to main
6. Main forwards to renderer via `acp:notification` → `handleSessionUpdate()` renders in real-time
7. On completion, agent responds with `PromptResponse(stopReason)` → resolved promise → renderer finalizes streaming

## Directory structure

```
desktop/
├── AGENTS.md                  # This file
├── package.json               # Dependencies, scripts, electron-builder config
├── electron.vite.config.mjs   # Build: main + preload (esbuild) + renderer (Vite)
├── pnpm-lock.yaml             # Lockfile (pnpm)
├── .gitignore                 # node_modules/, out/, dist/, resources/*/opencli*
├── build/                     # electron-builder build resources (icons)
├── resources/
│   ├── mac/.gitkeep           # macOS opencli binary (CI copies here)
│   ├── linux/.gitkeep         # Linux opencli binary
│   └── win/.gitkeep           # Windows opencli.exe binary
├── src/
│   ├── main/index.js          # Main process (entry point)
│   ├── preload/index.js       # Preload script (context bridge)
│   └── renderer/
│       ├── index.html         # Chat UI shell
│       ├── app.js             # Renderer application logic
│       └── style.css          # Theme system + all styles
└── out/                       # electron-vite build output (gitignored)
    ├── main/index.js
    └── preload/index.js
```

## Key patterns & conventions

### Security
- `contextIsolation: true`, `nodeIntegration: false` — non-negotiable.
- All renderer ↔ main communication goes through `contextBridge` API.
- Markdown rendered with DOMPurify whitelist (no `data-*` attributes, no `script`, no `iframe`).
- CSP meta tag: `default-src 'self'; style-src 'self' 'unsafe-inline'; script-src 'self'`.
- **Renderer External Navigation Defense**:
  - Intercept the `will-navigate` event on `webContents` to prevent malicious redirection of the main window. Allow only local file/localhost URLs, and route other navigations to the system's default browser via `shell.openExternal`.
- **Window Open & XSS Isolation**:
  - Intercept and deny `setWindowOpenHandler` calls. Redirect all external http/https requests to the native browser to isolate the application environment from external JavaScript.
- **Hardware Permission Lock**:
  - Rigidly block all sensitive API and hardware access requests (camera, microphone, geolocation) using `session.defaultSession.setPermissionRequestHandler`.
- **Uncaught Error Watchdog**:
  - Register global error traps (`uncaughtException` and `unhandledRejection`) in the main process to ensure any stray async failures are logged rather than causing silent application crashes.

### Window lifecycle & Process Management
- Always call `isRendererAlive()` before `mainWindow.webContents.send()`.
- Persist window bounds on `resize` event to `preferences.json` using the local JSON store.
- **RPC Resiliency & Cleanup on Backend Crash**:
  - In `agentProcess.on("exit")` and `agentProcess.on("error")`, iterate through all outstanding promises in `pendingRequests` and reject them with a structured error, clearing the map immediately. This prevents the frontend from freezing in a perpetual loading state if the agent dies.
- **Graceful Shutdown with Timeout SIGKILL Fallback**:
  - Instead of brute-force SIGKILL, initiate termination of the Rust agent by closing its input stream (`agentProcess.stdin.end()`), which sends a Stdin EOF.
  - Wait up to a configured threshold (e.g., 2 seconds) for a graceful exit, after which fall back to `SIGKILL` to clean up resource-heavy instances. Trigger this graceful flow on `before-quit` and `window-all-closed` events.

### ACP communication
- Outgoing RPC requests get a string `msgId` (monotonic counter), stored in `pendingRequests` Map
- Incoming agent messages: if the message has `id` + `result`/`error` → resolve pending request; if `method === "session/update"` → forward as notification to renderer
- Custom RPC methods (`session/list`, `session/load`, `session/delete`) are handled by the ACP agent's dispatch fallback — no protocol extension needed

### Theme system
- 7 built-in themes: `dark` (default), `light`, `dracula`, `monokai`, `nord`, `one-dark-pro`, `solarized-dark`
- Each theme is a `[data-theme="id"]` block in `style.css` defining all color variables
- Theme switching: `document.documentElement.setAttribute("data-theme", id)` + persist to store
- Color transitions: `body { transition: background 400ms ease, color 400ms ease; }`

### Session management
- Sessions are stored as JSON files by the Rust session crate (`~/.config/opencli/sessions/{id}.json`)
- Desktop lists/loads/deletes sessions via custom ACP methods
- `session/new` returns the session ID; `session/prompt` loads existing session, appends messages, runs agent, saves back
- Session list refreshes after each prompt completes

## Development

### Prerequisites
- Rust toolchain (for building the `opencli` binary that the desktop spawns)
- Node.js ≥ 22
- pnpm ≥ 10

### Setup & run
```bash
# Build the Rust backend first
cargo build --release

# Install JS dependencies and start dev mode
cd desktop
pnpm install
pnpm dev
```

Dev mode uses Vite HMR for the renderer; main/preload changes require restart.

### Build & package
```bash
pnpm dist          # All platforms (via electron-builder)
pnpm dist:mac      # macOS (dmg + zip, arm64 + x64)
pnpm dist:linux    # Linux (AppImage + deb, x64)
pnpm dist:win      # Windows (nsis + zip, x64)
```

Artifacts land in `desktop/dist/`.

### Dependencies

| Package | Version | Purpose |
|---|---|---|
| electron | ^42.2.0 | Runtime |
| electron-vite | ^5.0.0 | Build toolchain (esbuild + Vite) |
| electron-builder | ^26.0.0 | Packaging / code signing |
| electron-log | ^5.4.0 | Production logging |
| electron-context-menu | ^3.6.0 | Native right-click menu |
| marked | ^17.0.0 | Markdown → HTML |
| dompurify | ^3.3.0 | XSS sanitization |
| highlight.js | ^11.11.0 | Code syntax highlighting |

### Gotchas
- **pnpm postinstall**: `electron` postinstall may not run automatically. If you get "Electron failed to install", run `node node_modules/electron/install.js` manually.
- **electron-store**: v9+ is ESM-only, can't `require()` from CJS. Replaced with a zero-dependency JSON file store using `app.getPath('userData')`.
- **`ELECTRON_OVERRIDE_DIST_PATH`**: must be set to `./node_modules/electron/dist` in dev scripts, otherwise electron-vite can't find the Electron binary.
- **`--locked` in CI**: `Cargo.lock` is tracked in git — do not add it to `.gitignore`.
- **Binary path resolution**: `resolveOpencliBinary()` checks `process.resourcesPath` first (packaged app), then falls back to `target/release` / `target/debug` for dev.
