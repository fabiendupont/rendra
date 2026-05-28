# Lightweight Web App Runtime Built on Servo

**Date:** 2026-05-27
**Status:** Draft
**Approach:** New runtime embedding libservo directly (Approach B)

## Overview

A lightweight, multi-platform desktop application runtime built on the Servo web engine. Apps use HTML/CSS/JS for the frontend and Rust for backend logic. The runtime ships as a single binary — no bundled Chromium, no bundled Node.js.

This is a platform play: a developer tool with its own CLI, packaging system, and ecosystem, positioned as a genuine alternative to Electron for developers who want small binaries, low resource usage, and native-feeling apps.

### Goals

- **Resource usage:** A fraction of Electron's memory footprint (target: under 100MB RSS for a typical app vs. Electron's 200-500MB).
- **Binary size:** Under 50MB for a minimal app (vs. Electron's 150-200MB).
- **Native feel:** Sub-500ms cold start, GPU-accelerated rendering via Servo's WebRender, responsive input handling.

### Relationship to Servo

Embed libservo (v0.1.0+, available on crates.io) as a dependency. Contribute missing embedding features back to the Servo project upstream. Do not fork.

### Relationship to Tauri

Tauri is pursuing Servo integration via the Verso project as an alternative webview backend within Tauri's existing architecture. This project is different: a purpose-built runtime designed from the ground up around Servo's strengths, with a curated web API surface and Rust-native developer experience. Not a layer on top of Tauri.

### Relationship to Verso

Verso wraps Servo's low-level APIs into a more ergonomic builder pattern. As of mid-2026, Verso as a standalone browser is no longer maintained, but the webview component continues for Tauri integration. Our runtime addresses the same ergonomic gap (Servo's raw APIs are too low-level) but with a different API surface optimized for app development rather than Tauri compatibility.

## Architecture

### Layer Diagram

```
┌─────────────────────────────────────────────────────────┐
│                    Application                          │
│  ┌───────────────┐  ┌──────────────┐  ┌──────────────┐  │
│  │  Frontend     │  │  Backend     │  │  App         │  │
│  │  (HTML/CSS/JS)│  │  (Rust)      │  │  Manifest    │  │
│  └──────┬────────┘  └──────┬───────┘  │  (TOML)      │  │
│         │                  │          └──────────────┘  │
├─────────┼──────────────────┼────────────────────────────┤
│         │      Runtime Core│                            │
│         ▼                  ▼                            │
│  ┌─────────────────────────────────┐                    │
│  │         IPC Bridge              │                    │
│  │  (Frontend ←→ Backend comms)    │                    │
│  └──────────┬──────────────────────┘                    │
│             │                                           │
│  ┌──────────▼──────────────────────┐                    │
│  │       Native API Layer          │                    │
│  │  FS │ Dialog │ Net │ Clipboard  │                    │
│  └─────────────────────────────────┘                    │
│                                                         │
│  ┌──────────────────────────────────────────────────┐   │
│  │              libservo (Web Engine)               │   │
│  │  SpiderMonkey (JS) │ WebRender (GPU) │ Stylo     │   │
│  └────────────────────────┬─────────────────────────┘   │
│                           │ renders into GL surface     │
│  ┌────────────────────────▼─────────────────────────┐   │
│  │              winit (Platform Layer)              │   │
│  │  Window + GL Surface │ Event Loop │ Input        │   │
│  │  X11 / Wayland / Cocoa / Win32                   │   │
│  └──────────────────────────────────────────────────┘   │
├─────────────────────────────────────────────────────────┤
│                  Operating System                       │
│     GPU (OpenGL/Vulkan)  │  Filesystem  │  Network      │
└─────────────────────────────────────────────────────────┘
```

### Layer Responsibilities

**winit (Platform Layer):** Creates the window, obtains the GL/Vulkan surface, runs the event loop, and forwards input events to libservo. This is the foundation — libservo renders into the surface winit provides.

**libservo (Web Engine):** Renders HTML/CSS via Stylo and WebRender. Executes frontend JavaScript via SpiderMonkey. Consumes the GL surface from winit. Servo v0.1.0 is the first crates.io release and provides an embedding API with multi-windowing, HTTP proxy management, localStorage/sessionStorage, cookies, and console messages.

**Native API Layer:** Platform capabilities exposed to the Rust backend as traits. Gated by manifest permissions. Covers filesystem, networking, dialogs, clipboard, and window management.

**IPC Bridge:** Typed message passing between frontend JS and Rust backend. Async by default. Capability-based — only explicitly exposed commands are callable.

**Application:** The developer's code. HTML/CSS/JS frontend (any framework), Rust backend, and a TOML manifest.

### Data Flow

1. winit creates a window and obtains a GL surface
2. Runtime passes the GL surface to libservo
3. libservo renders web content into the GL surface
4. winit runs the event loop and forwards input events to libservo
5. Frontend JS calls `invoke()` → IPC bridge → Rust backend
6. Rust backend calls Native APIs → results return via IPC

## IPC Bridge

### Design Principles

- **Type-safe end-to-end.** Commands are defined in Rust with typed inputs and outputs. A build step generates TypeScript bindings so the frontend gets autocomplete and type checking.
- **Async by default.** All IPC calls return a `Promise` on the JS side and use `async fn` on the Rust side. Servo's renderer never blocks waiting for a backend response.
- **Capability-based.** Commands are explicitly exposed. The frontend can only call what the backend declares. No ambient access to native APIs from JS.

### Commands (Frontend → Backend)

```rust
// Backend: define a command
#[command]
async fn read_config(path: String) -> Result<Config, AppError> {
    let content = fs::read_to_string(&path).await?;
    toml::from_str(&content).map_err(Into::into)
}

// Register commands when building the app
App::builder()
    .command(read_config)
    .run();
```

```typescript
// Frontend: auto-generated TypeScript bindings
import { commands } from './bindings';

const config = await commands.readConfig("./config.toml");
// config is typed as Config, errors are typed as AppError
```

### Events (Backend → Frontend)

```rust
// Backend emits an event
app.emit("file-changed", &FileEvent { path, kind });
```

```typescript
// Frontend listens
import { events } from './bindings';

events.onFileChanged((event) => {
    console.log(event.path, event.kind);
});
```

### JS Bridge Injection

The runtime injects a small JS script into every page before app code runs. This script exposes the `invoke()` function and the event listener API as globals (or under a namespaced object like `window.__runtime`). The injection uses Servo's programmatic script injection API (merged upstream in early 2025), avoiding the temp-file workaround that Verso currently uses.

### Serialization

JSON via `serde_json`. Simple, debuggable, fast enough for IPC. Binary protocols (MessagePack, bincode) are a potential v2 optimization if profiling shows JSON is a bottleneck.

## App Manifest & Project Structure

### Manifest (`app.toml`)

```toml
[app]
name = "my-app"
version = "0.1.0"
description = "A lightweight desktop app"

[window]
title = "My App"
width = 1024
height = 768
resizable = true
decorations = true

[permissions]
network = ["https"]
clipboard = ["read", "write"]

[permissions.filesystem]
user-files = "portal"   # must use file picker dialog

[build]
frontend = "dist"
assets = ["icons/", "fonts/"]
```

### Project Structure

```
my-app/
├── app.toml              # manifest
├── src/
│   └── main.rs           # Rust backend (commands, app logic)
├── frontend/
│   ├── index.html
│   ├── src/
│   │   └── main.ts       # frontend code (any framework)
│   └── package.json      # frontend dependencies
├── build/
│   └── bindings.ts       # auto-generated from #[command] definitions
└── assets/
    └── icon.png
```

### Key Decisions

- **Frontend is framework-agnostic.** Vanilla HTML, React, Vue, Svelte — whatever produces HTML/CSS/JS.
- **Permissions are declared in the manifest.** This enables static analysis and audit. An app that doesn't declare `filesystem` can't call filesystem APIs.
- **The CLI tool** handles the build pipeline: compile Rust backend, build frontend, generate bindings, bundle into a single distributable.

## Sandboxing Model

Informed by Flatpak's sandboxing architecture and lessons learned from its real-world deployment.

### Principles

1. **Deny by default.** Apps start with no access to host resources beyond their own data directory. Capabilities are added explicitly via the manifest.
2. **Portal-style file access.** Inspired by Flatpak's XDG Desktop Portals. To access files outside the app's scope, the app must use a file picker dialog. The user chooses the file; only that file becomes accessible.
3. **Scoped app-private storage.** Each app gets its own directories, always accessible without declaration:
   - `~/.local/share/<app-name>/` — persistent data
   - `~/.config/<app-name>/` — configuration
   - `~/.cache/<app-name>/` — cache (system can clear)
4. **File handles, not paths.** Portal-style file access returns handles, not path strings. The handle proves the user granted access.
5. **Static permissions are an upper bound.** The manifest declares what the app *can* request. Runtime portals/dialogs mediate actual access with user involvement.
6. **No broad escape hatches.** Unlike Flatpak's `filesystem=home`, there is no mechanism to request blanket host access.

### Permission Tiers

| Tier | Scope | User involvement |
|------|-------|-----------------|
| **Always available** | App-private dirs (`~/.local/share/<app>/`, etc.) | None |
| **Manifest-declared** | Network (HTTPS), clipboard, window management | None at runtime (declared upfront) |
| **Portal-mediated** | User files outside app scope | User picks files via native dialog |

### Static Analysis

The CLI's `check permissions` command analyzes the Rust backend to detect which native APIs are called, then compares against the manifest:
- Warns if the manifest declares permissions the code doesn't use (over-permissioned)
- Errors if the code uses APIs not declared in the manifest (under-permissioned)

### Flatpak Interop

Apps distributed as Flatpaks benefit from defense in depth: the runtime's app-level sandboxing runs inside Flatpak's OS-level sandboxing. Two independent layers.

## Native API Layer (v1)

Each module is a Rust trait, gated by manifest permissions.

| Module | Purpose | Key operations |
|--------|---------|---------------|
| **Window** | Window management | resize, move, minimize, maximize, fullscreen, close, set title |
| **Filesystem** | Scoped file access | read, write, create, delete, watch — sandboxed to app-private dirs |
| **Network** | HTTP client | HTTPS fetch, WebSocket |
| **Dialog** | Native dialogs | open file (portal), save file (portal), message box, confirmation |
| **Clipboard** | Read/write clipboard | text and image |
| **Shell** | Open external resources | open URL in default browser, open file in default app |

### Design Principles

- **Trait-based.** Real implementations for production; mock implementations for tests. Prepares the boundary for the future WASM plugin system.
- **Scoped filesystem.** The filesystem API enforces the sandbox. Apps cannot escape to arbitrary paths.
- **No ambient shell access.** `Shell::open` launches URLs or files in the default handler. There is no `child_process.exec()` equivalent.
- **Progressive disclosure.** The minimal app needs zero native APIs — just a window rendering HTML. Each module is opt-in.

### Example

```rust
use runtime::fs;

#[command]
async fn load_document(app: &App, name: String) -> Result<String, AppError> {
    let path = app.data_dir().join(&name);
    fs::read_to_string(&path).await.map_err(Into::into)
}
```

## CLI Tooling

| Command | Purpose |
|---------|---------|
| `init` | Scaffold a new project (app.toml, src/main.rs, frontend/) |
| `dev` | Dev mode — build backend, serve frontend with hot reload, launch app window |
| `build` | Production build — compile Rust, bundle frontend, generate single binary |
| `generate bindings` | Regenerate TypeScript bindings from `#[command]` definitions |
| `package` | Create distributable (AppImage on Linux) |
| `check permissions` | Static analysis — audit manifest vs. actual API usage |

### Dev Mode Workflow

1. Developer runs `cli dev`
2. CLI compiles the Rust backend
3. CLI starts a local dev server for the frontend (or uses the project's — Vite, webpack, etc.)
4. CLI launches a runtime window pointing at the dev server
5. Frontend changes hot-reload instantly (Servo reloads the page)
6. Backend changes trigger recompile + app restart

## Packaging & Distribution

### Single Binary

The production build produces a single self-contained binary:

```
┌─────────────────────────────┐
│      Application Binary     │
│  ┌───────────────────────┐  │
│  │  Rust backend          │  │
│  ├───────────────────────┤  │
│  │  Frontend assets       │  │
│  │  (HTML/CSS/JS embedded)│  │
│  ├───────────────────────┤  │
│  │  Servo resources       │  │
│  │  (UA stylesheet, etc.) │  │
│  ├───────────────────────┤  │
│  │  App manifest (baked)  │  │
│  └───────────────────────┘  │
└─────────────────────────────┘
```

Target binary size: **30-50MB** for a minimal app.

### Linux Distribution Formats (v1)

| Format | Why |
|--------|-----|
| **AppImage** | Single file, no installation, works on any distro. Matches single-binary philosophy. |
| **Flatpak** | Defense in depth — app sandbox inside Flatpak sandbox. Good for Flathub distribution. |
| **Raw binary + .desktop** | For power users managing installation themselves. |

No Snap for v1.

### App Signing (v1)

GPG signing of the binary for authenticity verification. Platform-specific code signing (Apple notarization, Windows Authenticode) comes with platform expansion.

## Language Strategy

### Frontend

JavaScript runs natively in Servo via SpiderMonkey. Any frontend framework that produces HTML/CSS/JS works. This is not compiled to WASM — it's standard web rendering.

### Backend

Rust only for v1. No embedded Node.js or Deno.

### Web API Compatibility

Servo does not implement everything Chromium does. The project adopts a **curated subset** approach: document what Servo supports, let developers build for the platform they're targeting. Do not polyfill gaps. This is a feature, not a limitation — it keeps the runtime lean.

### Future: WASM Plugins (v2)

The trait-based native API design prepares for a WASM plugin system in v2. Plugins written in any language that compiles to WASM could extend the runtime with new capabilities. This is architecturally cleaner than embedding a JS runtime and aligns with the WASI ecosystem direction (WASI 0.2 shipping, WASI 1.0 planned for 2026).

## v1 Scope

### What v1 Delivers

- Runtime core: libservo + winit, single-window apps on Linux (X11 + Wayland)
- IPC bridge: typed commands + events with generated TypeScript bindings
- Native APIs: window management, scoped filesystem, HTTPS networking, file picker dialog, clipboard
- Sandboxing: deny-by-default permissions, portal-style file access, app-private storage
- CLI: `init`, `dev`, `build`, `generate bindings`, `package` (AppImage)
- Documentation: getting started guide, API reference, example apps

### Success Criteria

1. A developer can `init` a project, write a Rust backend + HTML frontend, and `build` a working AppImage in under 30 minutes
2. The resulting binary is under 50MB
3. App cold start is under 500ms
4. The sandboxing model prevents an app from accessing files or network beyond its declared permissions

### Timeline Consideration

Servo is targeting Summer 2026 for its Linux/macOS Alpha. Aligning v1 with Servo's stabilization makes strategic sense — the project would be one of the first serious embedders of the crates.io release.

## Non-Goals (Deferred to Future Versions)

These are explicitly out of scope for v1 but documented here for future exploration.

### v2 Candidates

| Feature | Rationale for deferral | Notes for future exploration |
|---------|----------------------|------------------------------|
| **WASM plugin system** | The trait-based API design prepares the boundary, but implementing the WASM host, sandboxing plugins, and defining a stable ABI is significant work. | Evaluate WASI 1.0 (expected late 2026) as the plugin interface. Javy and ComponentizeJS could enable JS-authored plugins compiled to WASM. |
| **Multi-window apps** | Servo v0.1.0 supports multi-windowing, but the IPC and lifecycle complexity (which window owns which commands, cross-window events) needs careful design. | Design a window-scoped IPC model. Each window could have its own command namespace. |
| **System tray** | Requires platform-specific integration (libappindicator on Linux, NSStatusItem on macOS, Windows notification area). | Consider ksni crate for Linux. Define a cross-platform trait in the native API layer. |
| **Custom native menus** | Native menus are platform-specific (GTK on Linux, Cocoa on macOS, Win32 on Windows). | For v1, apps use in-app HTML menus. Native menus should be a progressive enhancement, not a requirement. |
| **Auto-update** | Requires a server-side component, update protocol, binary diffing, and rollback mechanism. | Evaluate existing solutions (e.g., Sparkle-like protocol). The single-binary architecture simplifies this — replace one file. |
| **Notifications** | Desktop notification APIs are fragmented across platforms and desktop environments (libnotify, D-Bus, UNUserNotificationCenter). | XDG Desktop Portal has a notifications interface — could reuse the portal pattern from our sandboxing model. |

### Platform Expansion

| Platform | Rationale for deferral | Notes for future exploration |
|----------|----------------------|------------------------------|
| **macOS** | Requires Cocoa windowing (winit handles this), `.app` bundle packaging, Apple notarization, and testing across macOS versions. | winit already supports macOS. Servo has improved macOS builds as of 2026. Main work is packaging and signing. |
| **Windows** | Requires Win32/WinUI windowing, `.msi` packaging, Windows Authenticode signing, and WebView2-like distribution considerations. | Consider whether an "evergreen shared runtime" (like Verso's long-term goal) makes sense on Windows to avoid per-app bundling. |
| **Mobile (Android/iOS)** | Fundamentally different UX paradigm — no system tray, no window management, different input model, app store distribution requirements. | Servo has Android support. Evaluate whether the same manifest/IPC model can work on mobile or if it needs a separate runtime profile. |

### Developer Experience

| Feature | Rationale for deferral | Notes for future exploration |
|---------|----------------------|------------------------------|
| **Built-in DevTools** | Building a full devtools UI is a massive undertaking. Servo supports Firefox remote debugging protocol as a stopgap. | Evaluate whether embedding a stripped-down devtools panel (inspector + console + network) is feasible using Servo itself to render the devtools UI. |
| **Electron migration tools** | Trying to be Electron-compatible undermines the lightweight premise. This is a new platform, not a compatibility layer. | Could offer a migration guide documenting Electron API → runtime API mappings. Automated codemods for common patterns may be feasible. |
| **App store / registry** | Premature — needs critical mass of apps and developers first. | Distribute via GitHub releases and Flathub initially. Evaluate whether a curated registry adds value once there are 50+ apps. |
| **Hot module replacement for Rust** | Rust recompilation is fast with incremental builds but still requires an app restart. True HMR for Rust backend code is a research problem. | Evaluate hot-reloading via dynamic libraries (`libloading`) for development mode only. |

### Security Hardening

| Feature | Rationale for deferral | Notes for future exploration |
|---------|----------------------|------------------------------|
| **Process isolation** | Running the backend and frontend in separate OS processes (like Chromium's multi-process model) adds security but significant IPC complexity. | Evaluate after v1 — if the single-process model proves to be a security concern in practice. |
| **Seccomp filters** | System call filtering (as Flatpak uses) would harden the sandbox at the OS level. | Define a minimal syscall allowlist for the runtime process. This is Linux-specific but aligns with the Linux-first strategy. |
| **Content Security Policy** | Enforcing CSP for frontend content would prevent XSS in the app's own UI. | Servo's CSP implementation status needs evaluation. Could be an app.toml option. |

## References

- [Servo project](https://servo.org/) — web engine
- [Servo v0.1.0 on crates.io](https://servo.org/blog/2026/04/13/servo-0.1.0-release/) — first crate release (April 2026)
- [Servo roadmap](https://github.com/servo/servo/wiki/Roadmap) — upstream development plans
- [Tauri architecture](https://v2.tauri.app/concept/architecture/) — reference for IPC and plugin patterns
- [Tauri-Verso integration](https://v2.tauri.app/blog/tauri-verso-integration/) — experimental Servo backend for Tauri
- [Verso project](https://github.com/versotile-org/verso) — Servo webview wrapper
- [NLnet Servo Webview for Tauri](https://nlnet.nl/project/Tauri-Servo/) — funding and goals
- [NLnet Verso WebView grant](https://nlnet.nl/project/Verso-WebView/) — ongoing funding
- [Flatpak sandbox permissions](https://docs.flatpak.org/en/latest/sandbox-permissions.html) — sandboxing model reference
- [XDG Desktop Portal](https://flatpak.github.io/xdg-desktop-portal/) — portal API reference
- [Javy](https://github.com/bytecodealliance/javy) — JS-to-WASM toolchain (relevant for future WASM plugins)
- [winit](https://github.com/rust-windowing/winit) — cross-platform windowing library
