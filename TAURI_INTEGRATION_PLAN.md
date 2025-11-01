# 🦀 Tauri Integration Plan

## What We Just Set Up

✅ **Created:**
- `src-tauri/` - Tauri Rust backend
- `src-tauri/Cargo.toml` - Dependencies
- `src-tauri/tauri.conf.json` - App configuration
- `src-tauri/src/main.rs` - Entry point with embedded server
- Root `package.json` - Unified build scripts
- `TAURI_SETUP.md` - Technical documentation
- `QUICK_START.md` - User-friendly guide

## Current Architecture

```
TabAgent Desktop
│
├── src-tauri/           # Tauri Rust (NEW!)
│   ├── src/main.rs     # Entry point + embedded server
│   └── Cargo.toml      # Dependencies
│
├── Rust/               # Your existing server (TO INTEGRATE)
│   └── server/
│       └── src/main.rs # Current entry point
│
├── dashboard/          # React UI (READY!)
│   └── dist/          # Build output
│
└── agent-builder/      # Vue 3 UI (READY!)
    └── dist/          # Build output
```

## Next Steps to Complete Integration

### Step 1: Integrate Existing Rust Server

**Current:** `Rust/server/src/main.rs` is standalone
**Goal:** Merge into `src-tauri/src/main.rs`

You need to:
1. Copy server logic from `Rust/server/src/main.rs`
2. Merge with `src-tauri/src/main.rs`
3. Keep API routes, WebSocket, GRPC
4. Add Dashboard + Agent Builder serving

### Step 2: Update Tauri Configuration

**File:** `src-tauri/tauri.conf.json`

Key decisions:
- App name (currently "TabAgent Desktop")
- Window size (currently 1400x900)
- Allowed APIs (currently minimal for security)

### Step 3: Generate App Icons

```bash
# Need a 1024x1024 PNG source icon
npm run tauri icon path/to/icon.png
```

This generates all required icons automatically!

### Step 4: Update UI Routes

**Dashboard** should route:
- `/` → Main dashboard
- `/models` → Model management
- `/database` → Database explorer
- `/settings` → Settings

**Agent Builder** should route (under `/workflows`):
- `/workflows` → Workflow list
- `/workflows/new` → Create workflow
- `/workflows/:id` → Edit workflow

**No conflicts!**

### Step 5: Native Messaging Bridge

For Chrome Extension to talk to desktop app:

```rust
// Add to src-tauri/src/main.rs
mod native_messaging;

// Handle stdin/stdout from extension
tokio::spawn(async {
    native_messaging::start_listener().await;
});
```

## Port Strategy (Simplified!)

**Development:**
- Dashboard: `localhost:5173` (Vite dev)
- Agent Builder: `localhost:5175` (Vite dev)
- Tauri proxies both

**Production:**
- Everything: `localhost:3000` (Embedded in .exe)
- Dashboard: `localhost:3000/`
- Agent Builder: `localhost:3000/workflows`
- API: `localhost:3000/api/*`

## Build Commands Reference

```bash
# Development (hot reload)
npm run dev

# Build individual UIs
npm run build:dashboard
npm run build:builder

# Build everything + create .exe
npm run build

# Just build Rust without UIs
cd src-tauri && cargo build --release
```

## What Makes This Awesome

**vs n8n:**
- ❌ n8n: Tightly coupled, hard to extract
- ✅ Yours: Clean separation, easy to modify

**vs Docker:**
- ❌ Docker: Users need terminal, technical knowledge
- ✅ Yours: Double-click .exe, just works!

**vs Electron:**
- ❌ Electron: ~100MB bundles, slow startup
- ✅ Yours: ~5MB binaries, instant startup!

## Integration Complexity

**Easy ⭐⭐⭐⭐⭐:**
- Serving both UIs (already done!)
- Build scripts (already done!)

**Medium ⭐⭐⭐:**
- Integrating existing Rust server
- Native messaging for extension

**Hard ⭐:**
- App signing for distribution (optional)

## Expected Timeline

**Next Session:**
- Integrate Rust server (1-2 hours)
- Add app icons (15 min)
- First test build (30 min)

**Total:** ~2-3 hours to working `.exe`!

## Resources

- **Tauri Guide:** https://tauri.app/start/
- **Serving SPAs:** https://tauri.app/develop/calling-frontend/
- **Native Messaging:** Custom implementation needed

---

**You're 80% there! The hard part (UI) is done. Now just wrap it in Tauri!** 🎁

