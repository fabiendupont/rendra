# Rendra

A lightweight multi-platform web app runtime built on Servo. An Electron
alternative that ships small binaries, uses minimal resources, and feels
native.

## Project Overview

Rendra embeds the Servo web engine (v0.1.0+, via crates.io) with winit for
windowing. Apps use HTML/CSS/JS for the frontend and Rust for backend logic.
The runtime ships as a single binary — no bundled Chromium, no bundled
Node.js.

## Architecture

```
Application (HTML/CSS/JS frontend + Rust backend + app.toml manifest)
  ↓
Runtime Core (IPC bridge, native API layer)
  ↓
libservo (SpiderMonkey JS, WebRender GPU, Stylo CSS)
  ↓
winit (window + GL surface + event loop)
  ↓
OS (X11/Wayland on Linux, Cocoa on macOS, Win32 on Windows)
```

## Workspace Crates

| Crate | Path | Purpose |
|-------|------|---------|
| `runtime-core` | `crates/runtime-core/` | Servo + winit embedding, AppBuilder, event loop, IPC wiring |
| `runtime-ipc` | `crates/runtime-ipc/` | CommandHandler trait, CommandRegistry, EventEmitter, JS bridge |
| `runtime-macros` | `crates/runtime-macros/` | `#[command]` proc macro |
| `runtime-api` | `crates/runtime-api/` | Native APIs: filesystem, network, clipboard, dialog, shell, window, permissions |
| `runtime-sandbox` | `crates/runtime-sandbox/` | Manifest parsing (app.toml), AppScope (scoped dirs), permission model |
| `runtime-cli` | `crates/runtime-cli/` | CLI: init, dev, build, package, check-permissions |
| `runtime-bindgen` | `runtime-bindgen/` | TypeScript bindings generator from `#[command]` functions |

## Examples

| Example | Path | What it demonstrates |
|---------|------|---------------------|
| `hello-world` | `examples/hello-world/` | Minimal app: text input + IPC greet command |
| `markdown-notes` | `examples/markdown-notes/` | Full app: split-pane Markdown editor, filesystem save/load, 5 IPC commands |
| `showcase` | `examples/showcase/` | Rendra UI widget library showcase — all 25 components |

## Rendra UI Widget Library

Location: `lib/ui/`

CSS + vanilla JS component library. 25 components with dark/light theming
via CSS custom properties (`--rd-*` prefix, `.rd-*` class prefix).

Build: `lib/ui/build.sh` → `rendra-ui.css` (26KB) + `rendra-ui.js` (9KB)

## Key Technical Details

### IPC Bridge

JS→Rust: `console.log('__IPC__:' + JSON.stringify(request))` intercepted by
`WebViewDelegate::show_console_message`.

Rust→JS: `webview.evaluate_javascript()` calls `window.__runtime._handleResponse()`.

**Critical**: Never call `evaluate_javascript` from inside a WebViewDelegate
callback — it re-enters Servo and segfaults. Queue responses and flush on
the next event loop tick (see `AppWindow::flush_pending()`).

### Window Close

Servo's Drop impl segfaults. We use `libc::_exit(0)` on CloseRequested to
avoid this. Known issue in Servo v0.1.x.

### Binary Size

Release profile uses `strip = true`, `lto = "thin"`, `opt-level = "s"`.
Produces ~101MB binary. UPX compression yields ~16MB.

### CSS Grid

Servo does not support CSS Grid (`display: grid`). The Rendra UI grid
component uses flexbox as a fallback.

### Keyboard Input

`crates/runtime-core/src/keyboard.rs` maps winit key types to Servo's
`keyboard_types` equivalents. winit uses `SuperLeft/SuperRight` where
keyboard-types uses `MetaLeft/MetaRight`.

## Build Commands

```bash
# Build an example
cargo build -p hello-world --release

# Run tests (exclude servo-dependent crates for speed)
cargo test -p runtime-sandbox -p runtime-ipc -p runtime-api -p runtime-cli -p runtime-bindgen

# Build Rendra UI
lib/ui/build.sh

# Full test suite (slow — compiles servo)
cargo test --workspace
```

## Conventions

- All commits must be signed off: `git commit -s`
- CSS prefix: `.rd-*`, CSS variables: `--rd-*`, JS namespace: `Rendra`
- IPC commands implement `runtime_ipc::command::CommandHandler` trait
- Manifest is `app.toml` (TOML, parsed by `runtime-sandbox`)
- Permissions are deny-by-default, declared in `app.toml [permissions]`
- Filesystem access scoped to app-private dirs (`~/.local/share/<app>/`)

## Design Spec

Full design specification: `docs/DESIGN.md`
Implementation plan: `docs/IMPLEMENTATION-PLAN.md`

## What's Next (v2 candidates)

See the Non-Goals section of the design spec for deferred features:
multi-window, system tray, native menus, auto-update, notifications,
WASM plugin system, macOS/Windows support, process isolation, seccomp.
