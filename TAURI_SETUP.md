# 🦀 Tauri Desktop App Setup

## Architecture Overview

```
TabAgent Desktop (.exe/.app/linux binary)
│
├── Tauri Shell (Rust)
│   └── Embedded Web Server (Port 3000)
│       ├── / → Dashboard (React)
│       ├── /workflows → Agent Builder (Vue 3)
│       └── /api/* → REST API
│
├── Chrome Extension
│   └── Native Messaging → localhost:3000
│
└── User Interface (System Webview)
    └── Loads: http://localhost:3000
```

## Development

### Prerequisites
- Node.js 18+
- Rust (via rustup)
- Platform-specific tools:
  - **Windows:** Microsoft Visual Studio C++ Build Tools
  - **macOS:** Xcode Command Line Tools
  - **Linux:** webkit2gtk, libappindicator

### Install Dependencies
```bash
# Root dependencies (Tauri CLI)
npm install

# Dashboard dependencies
cd dashboard && npm install && cd ..

# Agent Builder dependencies  
cd agent-builder && npm install && cd ..
```

### Run Development Mode
```bash
# Starts ALL components:
# - Dashboard dev server (port 5173)
# - Agent Builder dev server (port 5175)
# - Tauri desktop window
npm run dev
```

This opens a native window showing your app with hot reload!

## Production Build

### Build Everything
```bash
npm run build
```

This will:
1. Build Dashboard → `dashboard/dist/`
2. Build Agent Builder → `agent-builder/dist/`
3. Bundle into Tauri app → `src-tauri/target/release/`

### Output Locations

**Windows:**
- `src-tauri/target/release/tabagent-desktop.exe`
- `src-tauri/target/release/bundle/msi/TabAgent Desktop_0.1.0_x64_en-US.msi`

**macOS:**
- `src-tauri/target/release/tabagent-desktop`
- `src-tauri/target/release/bundle/dmg/TabAgent Desktop_0.1.0_x64.dmg`
- `src-tauri/target/release/bundle/macos/TabAgent Desktop.app`

**Linux:**
- `src-tauri/target/release/tabagent-desktop`
- `src-tauri/target/release/bundle/deb/tabagent-desktop_0.1.0_amd64.deb`
- `src-tauri/target/release/bundle/appimage/tabagent-desktop_0.1.0_amd64.AppImage`

## Port Configuration

### Single Port Strategy (localhost:3000)
```
/                → Dashboard (default view)
/workflows       → Agent Builder  
/api/*          → REST API
/ws             → WebSocket
```

### Why Port 3000?
- Extension expects `localhost:3000`
- Single port = simpler for users
- Easy to remember
- No firewall issues

## Integrating Existing Rust Server

Your existing `Rust/server/src/main.rs` needs to be integrated into `src-tauri/src/main.rs`:

1. Import your server modules
2. Start server in background task
3. Serve both UIs from single router
4. Keep native messaging for extension

## Chrome Extension Integration

The extension connects via:
1. **Native Messaging:** stdin/stdout (no port)
2. **HTTP API:** `localhost:3000/api/*`
3. **WebSocket:** `localhost:3000/ws`

## Benefits

✅ **Single Binary:** One `.exe` for Windows, `.app` for Mac, etc.
✅ **Small Size:** ~3-5MB (vs Electron's ~100MB)
✅ **Fast:** Uses system webview (Chrome/Edge/Safari)
✅ **Cross-platform:** Windows, macOS, Linux
✅ **Professional:** Proper desktop app with taskbar, notifications
✅ **Easy Distribution:** Just send users the installer!
✅ **Mom & Pop Friendly:** Double-click to run, no technical knowledge needed

## Next Steps

1. ✅ Install Tauri CLI
2. ✅ Create `src-tauri/` structure
3. ✅ Configure `tauri.conf.json`
4. ⏳ Add app icons
5. ⏳ Integrate existing Rust server
6. ⏳ Test development mode
7. ⏳ Build production binary

## Resources

- [Tauri Docs](https://tauri.app/)
- [Tauri API Reference](https://tauri.app/reference/)
- [Tauri Examples](https://github.com/tauri-apps/tauri/tree/dev/examples)

