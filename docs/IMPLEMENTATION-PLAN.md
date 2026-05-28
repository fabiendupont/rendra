# Servo-Based Web App Runtime — Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Build a lightweight desktop application runtime that embeds Servo (v0.1.0+) for web rendering with a Rust backend, typed IPC bridge, and Flatpak-inspired sandboxing — targeting Linux (X11/Wayland) for v1.

**Architecture:** The runtime layers winit (platform/windowing) → libservo (web engine) → IPC bridge → native API traits → sandboxed app. Apps are defined by a TOML manifest, built with a CLI tool, and packaged as single-binary AppImages.

**Tech Stack:** Rust (stable), servo 0.1.0 (crates.io), winit 0.30+, serde/serde_json, toml, clap, syn/quote (proc macro), include_dir, rfd (file dialogs), arboard (clipboard)

---

## Project Structure (Target)

```
servo-runtime/
├── Cargo.toml                          # workspace root
├── crates/
│   ├── runtime-core/                   # libservo + winit embedding
│   │   ├── Cargo.toml
│   │   └── src/
│   │       ├── lib.rs                  # public API: App, AppBuilder
│   │       ├── servo_embed.rs          # Servo initialization, WebView lifecycle
│   │       ├── window.rs               # winit window management
│   │       └── event_loop.rs           # event loop integration
│   ├── runtime-ipc/                    # IPC bridge: command/event system
│   │   ├── Cargo.toml
│   │   └── src/
│   │       ├── lib.rs                  # IPC types, CommandRegistry
│   │       ├── bridge.rs               # JS ↔ Rust message passing
│   │       ├── command.rs              # Command trait, registration
│   │       └── event.rs                # Backend → Frontend events
│   ├── runtime-macros/                 # proc macros (#[command])
│   │   ├── Cargo.toml
│   │   └── src/
│   │       └── lib.rs                  # #[command] derive macro
│   ├── runtime-api/                    # native API traits + implementations
│   │   ├── Cargo.toml
│   │   └── src/
│   │       ├── lib.rs                  # re-exports
│   │       ├── filesystem.rs           # scoped filesystem trait + impl
│   │       ├── network.rs              # HTTPS fetch, WebSocket
│   │       ├── dialog.rs               # file picker (portal), message box
│   │       ├── clipboard.rs            # text + image clipboard
│   │       ├── shell.rs                # open URL/file in default app
│   │       └── window_api.rs           # resize, move, minimize, etc.
│   ├── runtime-sandbox/                # permission enforcement
│   │   ├── Cargo.toml
│   │   └── src/
│   │       ├── lib.rs                  # PermissionManager
│   │       ├── manifest.rs             # app.toml parsing + validation
│   │       └── scope.rs                # path scoping, app-private dirs
│   └── runtime-cli/                    # CLI tool
│       ├── Cargo.toml
│       └── src/
│           ├── main.rs                 # clap entry point
│           ├── init.rs                 # project scaffolding
│           ├── dev.rs                  # dev mode: build + serve + launch
│           ├── build.rs                # production build
│           ├── package.rs              # AppImage packaging
│           └── check_permissions.rs    # static analysis
├── runtime-bindgen/                    # TS bindings generator (standalone tool)
│   ├── Cargo.toml
│   └── src/
│       └── main.rs                     # parses #[command] fns → .ts
└── examples/
    └── hello-world/                    # minimal example app
        ├── app.toml
        ├── src/
        │   └── main.rs
        └── frontend/
            └── index.html
```

---

## Phase 1: Runtime Core (Window + Servo Embedding)

### Task 1: Initialize Workspace and runtime-core Crate

**Files:**
- Create: `Cargo.toml` (workspace root)
- Create: `crates/runtime-core/Cargo.toml`
- Create: `crates/runtime-core/src/lib.rs`

- [ ] **Step 1: Create workspace Cargo.toml**

```toml
[workspace]
resolver = "2"
members = [
    "crates/runtime-core",
]

[workspace.package]
edition = "2024"
license = "MPL-2.0"
rust-version = "1.85"

[workspace.dependencies]
serde = { version = "1", features = ["derive"] }
serde_json = "1"
toml = "0.8"
thiserror = "2"
tracing = "0.1"
tracing-subscriber = { version = "0.3", features = ["env-filter"] }
url = "2"
euclid = "0.22"
```

- [ ] **Step 2: Create runtime-core Cargo.toml**

```toml
[package]
name = "runtime-core"
version = "0.1.0"
edition.workspace = true
license.workspace = true

[dependencies]
servo = "0.1"
winit = { version = "0.30", features = ["rwh_06"] }
euclid.workspace = true
url.workspace = true
tracing.workspace = true
thiserror.workspace = true
raw-window-handle = "0.6"
rustls = { version = "0.23", features = ["aws-lc-rs"] }
```

- [ ] **Step 3: Create lib.rs with public API skeleton**

```rust
pub mod servo_embed;
pub mod window;
pub mod event_loop;

pub use servo_embed::ServoInstance;
pub use window::AppWindow;

pub struct App {
    _private: (),
}

pub struct AppBuilder {
    url: Option<url::Url>,
    title: String,
    width: u32,
    height: u32,
}

impl AppBuilder {
    pub fn new() -> Self {
        Self {
            url: None,
            title: "App".to_string(),
            width: 1024,
            height: 768,
        }
    }

    pub fn title(mut self, title: impl Into<String>) -> Self {
        self.title = title.into();
        self
    }

    pub fn size(mut self, width: u32, height: u32) -> Self {
        self.width = width;
        self.height = height;
        self
    }

    pub fn url(mut self, url: url::Url) -> Self {
        self.url = Some(url);
        self
    }
}

impl Default for AppBuilder {
    fn default() -> Self {
        Self::new()
    }
}
```

- [ ] **Step 4: Verify it compiles**

Run: `cargo check -p runtime-core`
Expected: Successful compilation (warnings about unused modules are OK at this point)

- [ ] **Step 5: Commit**

```bash
git add -A
git commit -s -m "feat: initialize workspace and runtime-core crate skeleton"
```

---

### Task 2: Implement Servo Embedding

**Files:**
- Create: `crates/runtime-core/src/servo_embed.rs`
- Create: `crates/runtime-core/src/window.rs`
- Create: `crates/runtime-core/src/event_loop.rs`

- [ ] **Step 1: Implement the winit event loop waker**

Write `crates/runtime-core/src/event_loop.rs`:

```rust
use winit::event_loop::EventLoop;

#[derive(Debug)]
pub struct WakerEvent;

#[derive(Clone)]
pub struct Waker(winit::event_loop::EventLoopProxy<WakerEvent>);

impl Waker {
    pub fn new(event_loop: &EventLoop<WakerEvent>) -> Self {
        Self(event_loop.create_proxy())
    }
}

impl servo::EventLoopWaker for Waker {
    fn clone_box(&self) -> Box<dyn servo::EventLoopWaker> {
        Box::new(Self(self.0.clone()))
    }

    fn wake(&self) {
        if let Err(e) = self.0.send_event(WakerEvent) {
            tracing::warn!(?e, "Failed to wake event loop");
        }
    }
}
```

- [ ] **Step 2: Implement window management**

Write `crates/runtime-core/src/window.rs`:

```rust
use std::cell::RefCell;
use std::rc::Rc;

use servo::{
    RenderingContext, Servo, WebView, WebViewBuilder, WindowRenderingContext,
};
use url::Url;
use winit::event_loop::ActiveEventLoop;
use winit::raw_window_handle::{HasDisplayHandle, HasWindowHandle};
use winit::window::Window;

pub struct AppWindow {
    pub window: Window,
    pub rendering_context: Rc<WindowRenderingContext>,
    pub webviews: RefCell<Vec<WebView>>,
}

impl AppWindow {
    pub fn new(event_loop: &ActiveEventLoop) -> Result<Self, Box<dyn std::error::Error>> {
        let display_handle = event_loop.display_handle()?;
        let window = event_loop.create_window(Window::default_attributes())?;
        let window_handle = window.window_handle()?;

        let rendering_context = Rc::new(
            WindowRenderingContext::new(display_handle, window_handle, window.inner_size())?,
        );
        let _ = rendering_context.make_current();

        Ok(Self {
            window,
            rendering_context,
            webviews: RefCell::new(Vec::new()),
        })
    }

    pub fn create_webview(&self, servo: &Servo, url: Url) {
        let scale = self.window.scale_factor() as f32;
        let webview = WebViewBuilder::new(servo, self.rendering_context.clone())
            .url(url)
            .hidpi_scale_factor(euclid::Scale::new(scale))
            .delegate(self as &dyn servo::WebViewDelegate)
            .build();
        self.webviews.borrow_mut().push(webview);
    }

    pub fn paint(&self) {
        if let Some(wv) = self.webviews.borrow().last() {
            wv.paint();
        }
        self.rendering_context.present();
    }

    pub fn resize(&self, new_size: winit::dpi::PhysicalSize<u32>) {
        if let Some(wv) = self.webviews.borrow().last() {
            wv.resize(new_size);
        }
    }
}
```

Note: `WebViewDelegate` must be implemented on a type that is `Rc`-wrapped. The actual delegate pattern may require `AppWindow` to be wrapped in `Rc<AppWindow>` and implement `WebViewDelegate` on `AppWindow`. Consult the Servo docs for the exact delegate trait requirements at implementation time.

- [ ] **Step 3: Implement the servo initialization**

Write `crates/runtime-core/src/servo_embed.rs`:

```rust
use servo::{Servo, ServoBuilder};

use crate::event_loop::Waker;

pub struct ServoInstance {
    pub servo: Servo,
}

impl ServoInstance {
    pub fn new(waker: Waker) -> Self {
        Self {
            servo: ServoBuilder::default()
                .event_loop_waker(Box::new(waker))
                .build(),
        }
    }

    pub fn setup_logging(&self) {
        self.servo.setup_logging();
    }

    pub fn spin(&self) {
        self.servo.spin_event_loop();
    }
}
```

- [ ] **Step 4: Wire up lib.rs with AppBuilder::run()**

Update `crates/runtime-core/src/lib.rs` — add the `run` method to `AppBuilder`:

```rust
use std::rc::Rc;

use event_loop::{Waker, WakerEvent};
use servo_embed::ServoInstance;
use window::AppWindow;
use winit::application::ApplicationHandler;
use winit::event::WindowEvent;
use winit::event_loop::EventLoop;

impl AppBuilder {
    pub fn run(self) -> Result<(), Box<dyn std::error::Error>> {
        rustls::crypto::aws_lc_rs::default_provider()
            .install_default()
            .expect("Failed to install crypto provider");

        let event_loop = EventLoop::with_user_event().build()?;
        let waker = Waker::new(&event_loop);

        let mut handler = AppHandler {
            config: self,
            waker: Some(waker),
            state: None,
        };

        Ok(event_loop.run_app(&mut handler)?)
    }
}

struct AppState {
    app_window: Rc<AppWindow>,
    servo_instance: ServoInstance,
}

struct AppHandler {
    config: AppBuilder,
    waker: Option<Waker>,
    state: Option<AppState>,
}

impl ApplicationHandler<WakerEvent> for AppHandler {
    fn resumed(&mut self, event_loop: &winit::event_loop::ActiveEventLoop) {
        if self.state.is_some() {
            return;
        }

        let waker = self.waker.take().expect("Waker already consumed");
        let app_window = Rc::new(
            AppWindow::new(event_loop).expect("Failed to create window"),
        );

        let servo_instance = ServoInstance::new(waker);
        servo_instance.setup_logging();

        let url = self.config.url.clone().unwrap_or_else(|| {
            url::Url::parse("about:blank").unwrap()
        });
        app_window.create_webview(&servo_instance.servo, url);

        self.state = Some(AppState {
            app_window,
            servo_instance,
        });
    }

    fn user_event(
        &mut self,
        _event_loop: &winit::event_loop::ActiveEventLoop,
        _event: WakerEvent,
    ) {
        if let Some(state) = &self.state {
            state.servo_instance.spin();
        }
    }

    fn window_event(
        &mut self,
        event_loop: &winit::event_loop::ActiveEventLoop,
        _window_id: winit::window::WindowId,
        event: WindowEvent,
    ) {
        if let Some(state) = &self.state {
            state.servo_instance.spin();
        }

        match event {
            WindowEvent::CloseRequested => event_loop.exit(),
            WindowEvent::RedrawRequested => {
                if let Some(state) = &self.state {
                    state.app_window.paint();
                }
            }
            WindowEvent::Resized(new_size) => {
                if let Some(state) = &self.state {
                    state.app_window.resize(new_size);
                }
            }
            _ => {}
        }
    }
}
```

- [ ] **Step 5: Verify it compiles**

Run: `cargo check -p runtime-core`
Expected: Successful compilation. There may be trait bound issues with `WebViewDelegate` — resolve them by consulting `docs.rs/servo/0.1.0/servo/trait.WebViewDelegate.html` for the exact trait signature.

- [ ] **Step 6: Commit**

```bash
git add -A
git commit -s -m "feat: implement servo + winit embedding in runtime-core"
```

---

### Task 3: Hello World Example App

**Files:**
- Create: `examples/hello-world/Cargo.toml`
- Create: `examples/hello-world/src/main.rs`
- Create: `examples/hello-world/frontend/index.html`

- [ ] **Step 1: Create the example Cargo.toml**

```toml
[package]
name = "hello-world"
version = "0.1.0"
edition = "2024"

[dependencies]
runtime-core = { path = "../../crates/runtime-core" }
url = "2"
```

Add to workspace `Cargo.toml` members:
```toml
members = [
    "crates/runtime-core",
    "examples/hello-world",
]
```

- [ ] **Step 2: Create the frontend HTML**

Write `examples/hello-world/frontend/index.html`:

```html
<!DOCTYPE html>
<html>
<head>
    <meta charset="utf-8">
    <title>Hello World</title>
    <style>
        body {
            font-family: system-ui, sans-serif;
            display: flex;
            justify-content: center;
            align-items: center;
            min-height: 100vh;
            margin: 0;
            background: #1a1a2e;
            color: #eee;
        }
        .container {
            text-align: center;
        }
        h1 {
            font-size: 3rem;
            margin-bottom: 0.5rem;
        }
        p {
            font-size: 1.2rem;
            opacity: 0.7;
        }
    </style>
</head>
<body>
    <div class="container">
        <h1>Hello from Servo!</h1>
        <p>This app is running on the lightweight runtime.</p>
    </div>
</body>
</html>
```

- [ ] **Step 3: Create main.rs that loads the local HTML file**

```rust
use std::path::PathBuf;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let html_path = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("frontend")
        .join("index.html");

    let url = url::Url::from_file_path(&html_path)
        .map_err(|_| format!("Invalid path: {}", html_path.display()))?;

    runtime_core::AppBuilder::new()
        .title("Hello World")
        .size(800, 600)
        .url(url)
        .run()
}
```

- [ ] **Step 4: Build and run the example**

Run: `cargo run -p hello-world`
Expected: A window opens showing "Hello from Servo!" centered on a dark background. The window should be resizable and close cleanly.

- [ ] **Step 5: Commit**

```bash
git add -A
git commit -s -m "feat: add hello-world example app"
```

---

## Phase 2: App Manifest & Sandboxing Foundation

### Task 4: Manifest Parsing (runtime-sandbox crate)

**Files:**
- Create: `crates/runtime-sandbox/Cargo.toml`
- Create: `crates/runtime-sandbox/src/lib.rs`
- Create: `crates/runtime-sandbox/src/manifest.rs`
- Create: `crates/runtime-sandbox/src/scope.rs`
- Test: `crates/runtime-sandbox/tests/manifest_test.rs`

- [ ] **Step 1: Create runtime-sandbox crate**

Add to workspace members. Write `crates/runtime-sandbox/Cargo.toml`:

```toml
[package]
name = "runtime-sandbox"
version = "0.1.0"
edition.workspace = true
license.workspace = true

[dependencies]
serde.workspace = true
toml.workspace = true
thiserror.workspace = true
dirs = "6"

[dev-dependencies]
tempfile = "3"
```

- [ ] **Step 2: Write failing tests for manifest parsing**

Write `crates/runtime-sandbox/tests/manifest_test.rs`:

```rust
use runtime_sandbox::manifest::{AppManifest, FilesystemPermission};

#[test]
fn parse_minimal_manifest() {
    let toml_str = r#"
[app]
name = "test-app"
version = "0.1.0"
"#;
    let manifest: AppManifest = toml::from_str(toml_str).unwrap();
    assert_eq!(manifest.app.name, "test-app");
    assert_eq!(manifest.app.version, "0.1.0");
    assert!(manifest.window.is_none());
    assert!(manifest.permissions.is_none());
}

#[test]
fn parse_full_manifest() {
    let toml_str = r#"
[app]
name = "full-app"
version = "1.0.0"
description = "A full app"

[window]
title = "Full App"
width = 1280
height = 720
resizable = false
decorations = true

[permissions]
network = ["https"]
clipboard = ["read", "write"]

[permissions.filesystem]
user-files = "portal"

[build]
frontend = "dist"
assets = ["icons/"]
"#;
    let manifest: AppManifest = toml::from_str(toml_str).unwrap();
    assert_eq!(manifest.app.name, "full-app");
    let window = manifest.window.unwrap();
    assert_eq!(window.title.unwrap(), "Full App");
    assert_eq!(window.width.unwrap(), 1280);
    assert!(!window.resizable.unwrap());
    let permissions = manifest.permissions.unwrap();
    assert_eq!(permissions.network, Some(vec!["https".to_string()]));
    assert_eq!(
        permissions.filesystem.unwrap().user_files,
        Some(FilesystemPermission::Portal)
    );
}

#[test]
fn reject_unknown_permission() {
    let toml_str = r#"
[app]
name = "bad-app"
version = "0.1.0"

[permissions]
nuclear-launch = true
"#;
    let result: Result<AppManifest, _> = toml::from_str(toml_str);
    assert!(result.is_err());
}
```

- [ ] **Step 3: Run tests to verify they fail**

Run: `cargo test -p runtime-sandbox`
Expected: FAIL — module `manifest` does not exist

- [ ] **Step 4: Implement manifest types**

Write `crates/runtime-sandbox/src/manifest.rs`:

```rust
use serde::Deserialize;

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct AppManifest {
    pub app: AppSection,
    pub window: Option<WindowSection>,
    pub permissions: Option<PermissionsSection>,
    pub build: Option<BuildSection>,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct AppSection {
    pub name: String,
    pub version: String,
    pub description: Option<String>,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct WindowSection {
    pub title: Option<String>,
    pub width: Option<u32>,
    pub height: Option<u32>,
    pub resizable: Option<bool>,
    pub decorations: Option<bool>,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct PermissionsSection {
    pub network: Option<Vec<String>>,
    pub clipboard: Option<Vec<String>>,
    pub filesystem: Option<FilesystemSection>,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct FilesystemSection {
    #[serde(rename = "user-files")]
    pub user_files: Option<FilesystemPermission>,
}

#[derive(Debug, Deserialize, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum FilesystemPermission {
    Portal,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct BuildSection {
    pub frontend: Option<String>,
    pub assets: Option<Vec<String>>,
}

impl AppManifest {
    pub fn from_file(path: &std::path::Path) -> Result<Self, ManifestError> {
        let content = std::fs::read_to_string(path)
            .map_err(|e| ManifestError::Io(path.to_path_buf(), e))?;
        toml::from_str(&content)
            .map_err(|e| ManifestError::Parse(path.to_path_buf(), e))
    }

    pub fn has_permission(&self, feature: &str) -> bool {
        let Some(perms) = &self.permissions else {
            return false;
        };
        match feature {
            "network" => perms.network.is_some(),
            "clipboard" => perms.clipboard.is_some(),
            "filesystem" => perms.filesystem.is_some(),
            _ => false,
        }
    }
}

#[derive(Debug, thiserror::Error)]
pub enum ManifestError {
    #[error("failed to read manifest at {0}: {1}")]
    Io(std::path::PathBuf, std::io::Error),
    #[error("failed to parse manifest at {0}: {1}")]
    Parse(std::path::PathBuf, toml::de::Error),
}
```

- [ ] **Step 5: Implement path scoping**

Write `crates/runtime-sandbox/src/scope.rs`:

```rust
use std::path::PathBuf;

pub struct AppScope {
    app_name: String,
}

impl AppScope {
    pub fn new(app_name: impl Into<String>) -> Self {
        Self {
            app_name: app_name.into(),
        }
    }

    pub fn data_dir(&self) -> PathBuf {
        dirs::data_dir()
            .unwrap_or_else(|| PathBuf::from(".local/share"))
            .join(&self.app_name)
    }

    pub fn config_dir(&self) -> PathBuf {
        dirs::config_dir()
            .unwrap_or_else(|| PathBuf::from(".config"))
            .join(&self.app_name)
    }

    pub fn cache_dir(&self) -> PathBuf {
        dirs::cache_dir()
            .unwrap_or_else(|| PathBuf::from(".cache"))
            .join(&self.app_name)
    }

    pub fn is_within_scope(&self, path: &std::path::Path) -> bool {
        let canonical = match path.canonicalize() {
            Ok(p) => p,
            Err(_) => return false,
        };
        canonical.starts_with(self.data_dir())
            || canonical.starts_with(self.config_dir())
            || canonical.starts_with(self.cache_dir())
    }

    pub fn ensure_dirs_exist(&self) -> std::io::Result<()> {
        std::fs::create_dir_all(self.data_dir())?;
        std::fs::create_dir_all(self.config_dir())?;
        std::fs::create_dir_all(self.cache_dir())?;
        Ok(())
    }
}
```

- [ ] **Step 6: Wire up lib.rs**

Write `crates/runtime-sandbox/src/lib.rs`:

```rust
pub mod manifest;
pub mod scope;
```

- [ ] **Step 7: Run tests**

Run: `cargo test -p runtime-sandbox`
Expected: All 3 tests pass

- [ ] **Step 8: Commit**

```bash
git add -A
git commit -s -m "feat: add runtime-sandbox with manifest parsing and path scoping"
```

---

## Phase 3: IPC Bridge

### Task 5: IPC Types and Command Registry

**Files:**
- Create: `crates/runtime-ipc/Cargo.toml`
- Create: `crates/runtime-ipc/src/lib.rs`
- Create: `crates/runtime-ipc/src/command.rs`
- Create: `crates/runtime-ipc/src/event.rs`
- Create: `crates/runtime-ipc/src/bridge.rs`
- Test: `crates/runtime-ipc/tests/ipc_test.rs`

- [ ] **Step 1: Create runtime-ipc crate**

Add to workspace members. Write `crates/runtime-ipc/Cargo.toml`:

```toml
[package]
name = "runtime-ipc"
version = "0.1.0"
edition.workspace = true
license.workspace = true

[dependencies]
serde.workspace = true
serde_json.workspace = true
thiserror.workspace = true
```

- [ ] **Step 2: Write failing tests for the command registry**

Write `crates/runtime-ipc/tests/ipc_test.rs`:

```rust
use runtime_ipc::command::{CommandRegistry, CommandHandler};
use serde_json::json;

struct EchoHandler;

impl CommandHandler for EchoHandler {
    fn handle(&self, args: serde_json::Value) -> Result<serde_json::Value, runtime_ipc::IpcError> {
        Ok(args)
    }
}

struct GreetHandler;

impl CommandHandler for GreetHandler {
    fn handle(&self, args: serde_json::Value) -> Result<serde_json::Value, runtime_ipc::IpcError> {
        let name = args.get("name")
            .and_then(|v| v.as_str())
            .ok_or_else(|| runtime_ipc::IpcError::InvalidArgs("missing 'name'".into()))?;
        Ok(json!({ "greeting": format!("Hello, {}!", name) }))
    }
}

#[test]
fn register_and_invoke_command() {
    let mut registry = CommandRegistry::new();
    registry.register("echo", EchoHandler);
    let result = registry.invoke("echo", json!({"msg": "hi"})).unwrap();
    assert_eq!(result, json!({"msg": "hi"}));
}

#[test]
fn invoke_unknown_command_returns_error() {
    let registry = CommandRegistry::new();
    let result = registry.invoke("nonexistent", json!({}));
    assert!(result.is_err());
}

#[test]
fn invoke_greet_command() {
    let mut registry = CommandRegistry::new();
    registry.register("greet", GreetHandler);
    let result = registry.invoke("greet", json!({"name": "World"})).unwrap();
    assert_eq!(result, json!({"greeting": "Hello, World!"}));
}

#[test]
fn invoke_greet_missing_name() {
    let mut registry = CommandRegistry::new();
    registry.register("greet", GreetHandler);
    let result = registry.invoke("greet", json!({}));
    assert!(result.is_err());
}
```

- [ ] **Step 3: Run tests to verify they fail**

Run: `cargo test -p runtime-ipc`
Expected: FAIL — crate doesn't compile yet

- [ ] **Step 4: Implement command types**

Write `crates/runtime-ipc/src/command.rs`:

```rust
use std::collections::HashMap;

use serde_json::Value;

use crate::IpcError;

pub trait CommandHandler: Send + Sync {
    fn handle(&self, args: Value) -> Result<Value, IpcError>;
}

pub struct CommandRegistry {
    commands: HashMap<String, Box<dyn CommandHandler>>,
}

impl CommandRegistry {
    pub fn new() -> Self {
        Self {
            commands: HashMap::new(),
        }
    }

    pub fn register(&mut self, name: impl Into<String>, handler: impl CommandHandler + 'static) {
        self.commands.insert(name.into(), Box::new(handler));
    }

    pub fn invoke(&self, name: &str, args: Value) -> Result<Value, IpcError> {
        let handler = self.commands.get(name)
            .ok_or_else(|| IpcError::UnknownCommand(name.to_string()))?;
        handler.handle(args)
    }

    pub fn has_command(&self, name: &str) -> bool {
        self.commands.contains_key(name)
    }
}

impl Default for CommandRegistry {
    fn default() -> Self {
        Self::new()
    }
}
```

- [ ] **Step 5: Implement event emitter**

Write `crates/runtime-ipc/src/event.rs`:

```rust
use serde::Serialize;
use serde_json::Value;

pub struct EventEmitter {
    pending: Vec<EventMessage>,
}

#[derive(Debug, Clone)]
pub struct EventMessage {
    pub name: String,
    pub payload: Value,
}

impl EventEmitter {
    pub fn new() -> Self {
        Self {
            pending: Vec::new(),
        }
    }

    pub fn emit<T: Serialize>(&mut self, name: impl Into<String>, payload: &T) {
        let value = serde_json::to_value(payload).unwrap_or(Value::Null);
        self.pending.push(EventMessage {
            name: name.into(),
            payload: value,
        });
    }

    pub fn drain(&mut self) -> Vec<EventMessage> {
        std::mem::take(&mut self.pending)
    }
}

impl Default for EventEmitter {
    fn default() -> Self {
        Self::new()
    }
}
```

- [ ] **Step 6: Implement the JS bridge protocol**

Write `crates/runtime-ipc/src/bridge.rs`:

```rust
use serde::{Deserialize, Serialize};
use serde_json::Value;

#[derive(Debug, Serialize, Deserialize)]
pub struct IpcRequest {
    pub id: u64,
    pub command: String,
    pub args: Value,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct IpcResponse {
    pub id: u64,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub result: Option<Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<String>,
}

impl IpcResponse {
    pub fn success(id: u64, result: Value) -> Self {
        Self {
            id,
            result: Some(result),
            error: None,
        }
    }

    pub fn error(id: u64, error: impl Into<String>) -> Self {
        Self {
            id,
            result: None,
            error: Some(error.into()),
        }
    }
}

pub const BRIDGE_JS: &str = r#"
(function() {
    let _nextId = 1;
    const _pending = new Map();

    window.__runtime = {
        invoke(command, args) {
            return new Promise((resolve, reject) => {
                const id = _nextId++;
                _pending.set(id, { resolve, reject });
                window.__runtime_send(JSON.stringify({ id, command, args: args || {} }));
            });
        },
        _handleResponse(json) {
            const response = JSON.parse(json);
            const pending = _pending.get(response.id);
            if (!pending) return;
            _pending.delete(response.id);
            if (response.error) {
                pending.reject(new Error(response.error));
            } else {
                pending.resolve(response.result);
            }
        },
        _handleEvent(json) {
            const event = JSON.parse(json);
            const listeners = _eventListeners.get(event.name) || [];
            listeners.forEach(fn => fn(event.payload));
        },
    };

    const _eventListeners = new Map();
    window.__runtime.on = function(name, callback) {
        if (!_eventListeners.has(name)) {
            _eventListeners.set(name, []);
        }
        _eventListeners.get(name).push(callback);
    };
})();
"#;
```

- [ ] **Step 7: Wire up lib.rs**

Write `crates/runtime-ipc/src/lib.rs`:

```rust
pub mod bridge;
pub mod command;
pub mod event;

#[derive(Debug, thiserror::Error)]
pub enum IpcError {
    #[error("unknown command: {0}")]
    UnknownCommand(String),
    #[error("invalid arguments: {0}")]
    InvalidArgs(String),
    #[error("handler error: {0}")]
    HandlerError(String),
    #[error("serialization error: {0}")]
    Serialization(#[from] serde_json::Error),
}
```

- [ ] **Step 8: Run tests**

Run: `cargo test -p runtime-ipc`
Expected: All 4 tests pass

- [ ] **Step 9: Commit**

```bash
git add -A
git commit -s -m "feat: add runtime-ipc with command registry, event emitter, and JS bridge"
```

---

### Task 6: #[command] Proc Macro

**Files:**
- Create: `crates/runtime-macros/Cargo.toml`
- Create: `crates/runtime-macros/src/lib.rs`
- Test: inline doc tests and integration via hello-world example

- [ ] **Step 1: Create runtime-macros crate**

Add to workspace members. Write `crates/runtime-macros/Cargo.toml`:

```toml
[package]
name = "runtime-macros"
version = "0.1.0"
edition.workspace = true
license.workspace = true

[lib]
proc-macro = true

[dependencies]
syn = { version = "2", features = ["full"] }
quote = "1"
proc-macro2 = "1"
```

- [ ] **Step 2: Implement the #[command] macro**

Write `crates/runtime-macros/src/lib.rs`:

```rust
use proc_macro::TokenStream;
use quote::quote;
use syn::{parse_macro_input, FnArg, ItemFn, Pat, ReturnType};

#[proc_macro_attribute]
pub fn command(_attr: TokenStream, item: TokenStream) -> TokenStream {
    let input_fn = parse_macro_input!(item as ItemFn);
    let fn_name = &input_fn.sig.ident;
    let fn_name_str = fn_name.to_string();

    let args: Vec<_> = input_fn.sig.inputs.iter().filter_map(|arg| {
        if let FnArg::Typed(pat_type) = arg {
            if let Pat::Ident(ident) = pat_type.pat.as_ref() {
                let name = &ident.ident;
                let ty = &pat_type.ty;
                return Some((name.clone(), ty.clone()));
            }
        }
        None
    }).collect();

    let arg_extractions: Vec<_> = args.iter().map(|(name, ty)| {
        let name_str = name.to_string();
        quote! {
            let #name: #ty = serde_json::from_value(
                args.get(#name_str)
                    .cloned()
                    .unwrap_or(serde_json::Value::Null)
            ).map_err(|e| runtime_ipc::IpcError::InvalidArgs(
                format!("field '{}': {}", #name_str, e)
            ))?;
        }
    }).collect();

    let has_result_return = matches!(&input_fn.sig.output, ReturnType::Type(_, ty) if {
        let type_str = quote!(#ty).to_string();
        type_str.contains("Result")
    });

    let call_and_return = if has_result_return {
        quote! {
            let result = #fn_name(#(#args.iter().map(|(n, _)| n)),*).map_err(|e| runtime_ipc::IpcError::HandlerError(e.to_string()))?;
            serde_json::to_value(result).map_err(|e| runtime_ipc::IpcError::Serialization(e))
        }
    } else {
        let arg_names: Vec<_> = args.iter().map(|(n, _)| n).collect();
        quote! {
            let result = #fn_name(#(#arg_names),*);
            serde_json::to_value(result).map_err(|e| runtime_ipc::IpcError::Serialization(e))
        }
    };

    let handler_name = syn::Ident::new(
        &format!("__command_handler_{}", fn_name),
        fn_name.span(),
    );

    let expanded = quote! {
        #input_fn

        pub struct #handler_name;

        impl runtime_ipc::command::CommandHandler for #handler_name {
            fn handle(&self, args: serde_json::Value) -> Result<serde_json::Value, runtime_ipc::IpcError> {
                #(#arg_extractions)*
                #call_and_return
            }
        }

        impl #handler_name {
            pub const NAME: &'static str = #fn_name_str;
        }
    };

    TokenStream::from(expanded)
}
```

Note: This is a v1 proc macro that handles synchronous commands. Async support requires additional work (spawning onto a runtime). The macro generates a `CommandHandler` impl struct named `__command_handler_{fn_name}` that can be registered with the `CommandRegistry`.

- [ ] **Step 3: Verify it compiles**

Run: `cargo check -p runtime-macros`
Expected: Compiles successfully

- [ ] **Step 4: Commit**

```bash
git add -A
git commit -s -m "feat: add #[command] proc macro for IPC command registration"
```

---

## Phase 4: Native API Layer

### Task 7: Native API Traits and Implementations

**Files:**
- Create: `crates/runtime-api/Cargo.toml`
- Create: `crates/runtime-api/src/lib.rs`
- Create: `crates/runtime-api/src/filesystem.rs`
- Create: `crates/runtime-api/src/clipboard.rs`
- Create: `crates/runtime-api/src/dialog.rs`
- Create: `crates/runtime-api/src/shell.rs`
- Create: `crates/runtime-api/src/network.rs`
- Create: `crates/runtime-api/src/window_api.rs`
- Test: `crates/runtime-api/tests/filesystem_test.rs`

- [ ] **Step 1: Create runtime-api crate**

Add to workspace members. Write `crates/runtime-api/Cargo.toml`:

```toml
[package]
name = "runtime-api"
version = "0.1.0"
edition.workspace = true
license.workspace = true

[dependencies]
runtime-sandbox = { path = "../runtime-sandbox" }
thiserror.workspace = true
serde.workspace = true
serde_json.workspace = true
rfd = "0.15"
arboard = "3"
open = "5"
reqwest = { version = "0.12", default-features = false, features = ["rustls-tls", "json"] }
tokio = { version = "1", features = ["fs"] }

[dev-dependencies]
tempfile = "3"
tokio = { version = "1", features = ["rt", "macros"] }
```

- [ ] **Step 2: Write failing filesystem tests**

Write `crates/runtime-api/tests/filesystem_test.rs`:

```rust
use runtime_api::filesystem::ScopedFs;
use runtime_sandbox::scope::AppScope;
use tempfile::TempDir;

#[test]
fn read_within_scope_succeeds() {
    let tmp = TempDir::new().unwrap();
    let scope = AppScope::new("test-app");
    let fs = ScopedFs::new_with_root(tmp.path().to_path_buf());

    let file_path = tmp.path().join("test.txt");
    std::fs::write(&file_path, "hello").unwrap();

    let content = fs.read_to_string(&file_path).unwrap();
    assert_eq!(content, "hello");
}

#[test]
fn read_outside_scope_fails() {
    let tmp = TempDir::new().unwrap();
    let fs = ScopedFs::new_with_root(tmp.path().to_path_buf());

    let result = fs.read_to_string(std::path::Path::new("/etc/passwd"));
    assert!(result.is_err());
}

#[test]
fn write_within_scope_succeeds() {
    let tmp = TempDir::new().unwrap();
    let fs = ScopedFs::new_with_root(tmp.path().to_path_buf());

    let file_path = tmp.path().join("output.txt");
    fs.write(&file_path, "data").unwrap();

    let content = std::fs::read_to_string(&file_path).unwrap();
    assert_eq!(content, "data");
}

#[test]
fn path_traversal_attack_blocked() {
    let tmp = TempDir::new().unwrap();
    let fs = ScopedFs::new_with_root(tmp.path().to_path_buf());

    let malicious = tmp.path().join("..").join("..").join("etc").join("passwd");
    let result = fs.read_to_string(&malicious);
    assert!(result.is_err());
}
```

- [ ] **Step 3: Run tests to verify they fail**

Run: `cargo test -p runtime-api`
Expected: FAIL — module doesn't exist

- [ ] **Step 4: Implement scoped filesystem**

Write `crates/runtime-api/src/filesystem.rs`:

```rust
use std::path::{Path, PathBuf};

#[derive(Debug, thiserror::Error)]
pub enum FsError {
    #[error("access denied: path {0} is outside the app scope")]
    OutOfScope(PathBuf),
    #[error("io error: {0}")]
    Io(#[from] std::io::Error),
}

pub struct ScopedFs {
    root: PathBuf,
}

impl ScopedFs {
    pub fn new_with_root(root: PathBuf) -> Self {
        Self { root }
    }

    fn validate_path(&self, path: &Path) -> Result<PathBuf, FsError> {
        let canonical_root = self.root.canonicalize()
            .map_err(FsError::Io)?;

        let canonical = if path.exists() {
            path.canonicalize().map_err(FsError::Io)?
        } else {
            let parent = path.parent()
                .ok_or_else(|| FsError::OutOfScope(path.to_path_buf()))?;
            let parent_canonical = parent.canonicalize().map_err(FsError::Io)?;
            let file_name = path.file_name()
                .ok_or_else(|| FsError::OutOfScope(path.to_path_buf()))?;
            parent_canonical.join(file_name)
        };

        if !canonical.starts_with(&canonical_root) {
            return Err(FsError::OutOfScope(path.to_path_buf()));
        }
        Ok(canonical)
    }

    pub fn read_to_string(&self, path: &Path) -> Result<String, FsError> {
        let validated = self.validate_path(path)?;
        std::fs::read_to_string(validated).map_err(FsError::Io)
    }

    pub fn write(&self, path: &Path, contents: impl AsRef<[u8]>) -> Result<(), FsError> {
        let validated = self.validate_path(path)?;
        std::fs::write(validated, contents).map_err(FsError::Io)
    }

    pub fn read(&self, path: &Path) -> Result<Vec<u8>, FsError> {
        let validated = self.validate_path(path)?;
        std::fs::read(validated).map_err(FsError::Io)
    }

    pub fn remove_file(&self, path: &Path) -> Result<(), FsError> {
        let validated = self.validate_path(path)?;
        std::fs::remove_file(validated).map_err(FsError::Io)
    }

    pub fn create_dir_all(&self, path: &Path) -> Result<(), FsError> {
        let validated = self.validate_path(path)?;
        std::fs::create_dir_all(validated).map_err(FsError::Io)
    }
}
```

- [ ] **Step 5: Implement remaining API modules**

Write `crates/runtime-api/src/clipboard.rs`:

```rust
use arboard::Clipboard;

#[derive(Debug, thiserror::Error)]
pub enum ClipboardError {
    #[error("clipboard error: {0}")]
    Access(#[from] arboard::Error),
}

pub struct ClipboardApi;

impl ClipboardApi {
    pub fn read_text() -> Result<String, ClipboardError> {
        let mut clipboard = Clipboard::new()?;
        Ok(clipboard.get_text()?)
    }

    pub fn write_text(text: &str) -> Result<(), ClipboardError> {
        let mut clipboard = Clipboard::new()?;
        clipboard.set_text(text)?;
        Ok(())
    }
}
```

Write `crates/runtime-api/src/dialog.rs`:

```rust
use std::path::PathBuf;

pub struct DialogApi;

impl DialogApi {
    pub fn open_file(title: &str, filters: &[(&str, &[&str])]) -> Option<PathBuf> {
        let mut dialog = rfd::FileDialog::new().set_title(title);
        for (name, exts) in filters {
            dialog = dialog.add_filter(*name, exts);
        }
        dialog.pick_file()
    }

    pub fn save_file(title: &str, default_name: &str) -> Option<PathBuf> {
        rfd::FileDialog::new()
            .set_title(title)
            .set_file_name(default_name)
            .save_file()
    }

    pub fn message(title: &str, description: &str) {
        rfd::MessageDialog::new()
            .set_title(title)
            .set_description(description)
            .show();
    }

    pub fn confirm(title: &str, description: &str) -> bool {
        rfd::MessageDialog::new()
            .set_title(title)
            .set_description(description)
            .set_buttons(rfd::MessageButtons::YesNo)
            .show()
            == rfd::MessageDialogResult::Yes
    }
}
```

Write `crates/runtime-api/src/shell.rs`:

```rust
#[derive(Debug, thiserror::Error)]
pub enum ShellError {
    #[error("failed to open: {0}")]
    Open(#[from] std::io::Error),
}

pub struct ShellApi;

impl ShellApi {
    pub fn open_url(url: &str) -> Result<(), ShellError> {
        open::that(url)?;
        Ok(())
    }

    pub fn open_path(path: &std::path::Path) -> Result<(), ShellError> {
        open::that(path)?;
        Ok(())
    }
}
```

Write `crates/runtime-api/src/network.rs`:

```rust
use serde_json::Value;

#[derive(Debug, thiserror::Error)]
pub enum NetworkError {
    #[error("request error: {0}")]
    Request(#[from] reqwest::Error),
    #[error("only HTTPS is allowed")]
    HttpNotAllowed,
}

pub struct NetworkApi;

impl NetworkApi {
    pub async fn fetch(url: &str) -> Result<String, NetworkError> {
        if !url.starts_with("https://") {
            return Err(NetworkError::HttpNotAllowed);
        }
        let body = reqwest::get(url).await?.text().await?;
        Ok(body)
    }

    pub async fn fetch_json(url: &str) -> Result<Value, NetworkError> {
        if !url.starts_with("https://") {
            return Err(NetworkError::HttpNotAllowed);
        }
        let body = reqwest::get(url).await?.json().await?;
        Ok(body)
    }
}
```

Write `crates/runtime-api/src/window_api.rs`:

```rust
pub struct WindowApi;

impl WindowApi {
    pub fn set_title(window: &winit::window::Window, title: &str) {
        window.set_title(title);
    }

    pub fn set_minimized(window: &winit::window::Window, minimized: bool) {
        window.set_minimized(minimized);
    }

    pub fn set_maximized(window: &winit::window::Window, maximized: bool) {
        window.set_maximized(maximized);
    }

    pub fn set_fullscreen(window: &winit::window::Window, fullscreen: bool) {
        if fullscreen {
            window.set_fullscreen(Some(winit::window::Fullscreen::Borderless(None)));
        } else {
            window.set_fullscreen(None);
        }
    }
}
```

Note: `window_api.rs` depends on `winit`. Add `winit` to `runtime-api` dependencies.

- [ ] **Step 6: Wire up lib.rs**

Write `crates/runtime-api/src/lib.rs`:

```rust
pub mod clipboard;
pub mod dialog;
pub mod filesystem;
pub mod network;
pub mod shell;
pub mod window_api;
```

- [ ] **Step 7: Run tests**

Run: `cargo test -p runtime-api`
Expected: All 4 filesystem tests pass

- [ ] **Step 8: Commit**

```bash
git add -A
git commit -s -m "feat: add runtime-api with filesystem, clipboard, dialog, shell, network, and window APIs"
```

---

## Phase 5: CLI Tooling

### Task 8: CLI — Init Command

**Files:**
- Create: `crates/runtime-cli/Cargo.toml`
- Create: `crates/runtime-cli/src/main.rs`
- Create: `crates/runtime-cli/src/init.rs`
- Test: `crates/runtime-cli/tests/init_test.rs`

- [ ] **Step 1: Create runtime-cli crate**

Add to workspace members. Write `crates/runtime-cli/Cargo.toml`:

```toml
[package]
name = "runtime-cli"
version = "0.1.0"
edition.workspace = true
license.workspace = true

[[bin]]
name = "runtime-cli"
path = "src/main.rs"

[dependencies]
runtime-sandbox = { path = "../runtime-sandbox" }
clap = { version = "4", features = ["derive"] }
thiserror.workspace = true
toml.workspace = true
serde.workspace = true

[dev-dependencies]
tempfile = "3"
assert_cmd = "2"
```

- [ ] **Step 2: Write failing test for init**

Write `crates/runtime-cli/tests/init_test.rs`:

```rust
use std::path::Path;
use tempfile::TempDir;

#[test]
fn init_creates_project_structure() {
    let tmp = TempDir::new().unwrap();
    let project_dir = tmp.path().join("my-app");

    runtime_cli::init::scaffold_project(&project_dir, "my-app").unwrap();

    assert!(project_dir.join("app.toml").exists());
    assert!(project_dir.join("src").join("main.rs").exists());
    assert!(project_dir.join("frontend").join("index.html").exists());
    assert!(project_dir.join("Cargo.toml").exists());

    let manifest_content = std::fs::read_to_string(project_dir.join("app.toml")).unwrap();
    assert!(manifest_content.contains("my-app"));

    let cargo_content = std::fs::read_to_string(project_dir.join("Cargo.toml")).unwrap();
    assert!(cargo_content.contains("my-app"));
}

#[test]
fn init_fails_if_directory_exists() {
    let tmp = TempDir::new().unwrap();
    let project_dir = tmp.path().join("existing");
    std::fs::create_dir_all(&project_dir).unwrap();
    std::fs::write(project_dir.join("something.txt"), "data").unwrap();

    let result = runtime_cli::init::scaffold_project(&project_dir, "existing");
    assert!(result.is_err());
}
```

- [ ] **Step 3: Run tests to verify they fail**

Run: `cargo test -p runtime-cli`
Expected: FAIL — module doesn't exist

- [ ] **Step 4: Implement init module**

Write `crates/runtime-cli/src/init.rs`:

```rust
use std::path::Path;

#[derive(Debug, thiserror::Error)]
pub enum InitError {
    #[error("directory already exists and is not empty: {0}")]
    DirectoryExists(std::path::PathBuf),
    #[error("io error: {0}")]
    Io(#[from] std::io::Error),
}

pub fn scaffold_project(path: &Path, name: &str) -> Result<(), InitError> {
    if path.exists() && path.read_dir()?.next().is_some() {
        return Err(InitError::DirectoryExists(path.to_path_buf()));
    }

    std::fs::create_dir_all(path)?;
    std::fs::create_dir_all(path.join("src"))?;
    std::fs::create_dir_all(path.join("frontend"))?;
    std::fs::create_dir_all(path.join("assets"))?;

    std::fs::write(
        path.join("app.toml"),
        format!(
            r#"[app]
name = "{name}"
version = "0.1.0"
description = ""

[window]
title = "{name}"
width = 1024
height = 768
resizable = true
decorations = true

[build]
frontend = "frontend"
"#
        ),
    )?;

    std::fs::write(
        path.join("Cargo.toml"),
        format!(
            r#"[package]
name = "{name}"
version = "0.1.0"
edition = "2024"

[dependencies]
runtime-core = {{ git = "https://github.com/user/servo-runtime" }}
runtime-ipc = {{ git = "https://github.com/user/servo-runtime" }}
runtime-macros = {{ git = "https://github.com/user/servo-runtime" }}
runtime-api = {{ git = "https://github.com/user/servo-runtime" }}
url = "2"
serde = {{ version = "1", features = ["derive"] }}
serde_json = "1"
"#
        ),
    )?;

    std::fs::write(
        path.join("src").join("main.rs"),
        r#"use std::path::PathBuf;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let html_path = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("frontend")
        .join("index.html");

    let url = url::Url::from_file_path(&html_path)
        .map_err(|_| format!("Invalid path: {}", html_path.display()))?;

    runtime_core::AppBuilder::new()
        .title("My App")
        .size(1024, 768)
        .url(url)
        .run()
}
"#,
    )?;

    std::fs::write(
        path.join("frontend").join("index.html"),
        r#"<!DOCTYPE html>
<html>
<head>
    <meta charset="utf-8">
    <title>My App</title>
    <style>
        body {
            font-family: system-ui, sans-serif;
            display: flex;
            justify-content: center;
            align-items: center;
            min-height: 100vh;
            margin: 0;
            background: #1a1a2e;
            color: #eee;
        }
    </style>
</head>
<body>
    <h1>Welcome to your app!</h1>
</body>
</html>
"#,
    )?;

    Ok(())
}
```

- [ ] **Step 5: Write the CLI main entry point**

Write `crates/runtime-cli/src/main.rs`:

```rust
use clap::{Parser, Subcommand};
use std::path::PathBuf;

#[derive(Parser)]
#[command(name = "runtime-cli", about = "CLI for the Servo-based app runtime")]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    Init {
        #[arg(help = "Project name")]
        name: String,
        #[arg(short, long, help = "Directory to create project in")]
        path: Option<PathBuf>,
    },
}

fn main() {
    let cli = Cli::parse();

    match cli.command {
        Commands::Init { name, path } => {
            let project_path = path.unwrap_or_else(|| PathBuf::from(&name));
            match runtime_cli::init::scaffold_project(&project_path, &name) {
                Ok(()) => println!("Project '{}' created at {}", name, project_path.display()),
                Err(e) => {
                    eprintln!("Error: {e}");
                    std::process::exit(1);
                }
            }
        }
    }
}
```

- [ ] **Step 6: Wire up lib.rs for tests**

Create `crates/runtime-cli/src/lib.rs`:

```rust
pub mod init;
```

Update `Cargo.toml` to expose as lib too:

```toml
[lib]
name = "runtime_cli"
path = "src/lib.rs"
```

- [ ] **Step 7: Run tests**

Run: `cargo test -p runtime-cli`
Expected: Both tests pass

- [ ] **Step 8: Test the CLI manually**

Run: `cargo run -p runtime-cli -- init test-project`
Expected: Creates a `test-project/` directory with app.toml, Cargo.toml, src/main.rs, frontend/index.html

Clean up: `rm -rf test-project`

- [ ] **Step 9: Commit**

```bash
git add -A
git commit -s -m "feat: add runtime-cli with init command"
```

---

### Task 9: CLI — Dev Command (Build + Serve + Launch)

**Files:**
- Create: `crates/runtime-cli/src/dev.rs`
- Modify: `crates/runtime-cli/src/main.rs`
- Modify: `crates/runtime-cli/src/lib.rs`

- [ ] **Step 1: Implement dev command**

Write `crates/runtime-cli/src/dev.rs`:

```rust
use std::path::Path;
use std::process::{Command, Stdio};

use runtime_sandbox::manifest::AppManifest;

#[derive(Debug, thiserror::Error)]
pub enum DevError {
    #[error("manifest error: {0}")]
    Manifest(#[from] runtime_sandbox::manifest::ManifestError),
    #[error("build failed: {0}")]
    Build(String),
    #[error("io error: {0}")]
    Io(#[from] std::io::Error),
    #[error("no app.toml found in current directory")]
    NoManifest,
}

pub fn run_dev(project_dir: &Path) -> Result<(), DevError> {
    let manifest_path = project_dir.join("app.toml");
    if !manifest_path.exists() {
        return Err(DevError::NoManifest);
    }

    let manifest = AppManifest::from_file(&manifest_path)?;
    println!("Building {}...", manifest.app.name);

    let status = Command::new("cargo")
        .arg("build")
        .current_dir(project_dir)
        .stdout(Stdio::inherit())
        .stderr(Stdio::inherit())
        .status()?;

    if !status.success() {
        return Err(DevError::Build("cargo build failed".into()));
    }

    let bin_name = &manifest.app.name;
    let bin_path = project_dir
        .join("target")
        .join("debug")
        .join(bin_name);

    println!("Launching {}...", bin_path.display());

    let status = Command::new(&bin_path)
        .current_dir(project_dir)
        .stdout(Stdio::inherit())
        .stderr(Stdio::inherit())
        .status()?;

    if !status.success() {
        return Err(DevError::Build(format!(
            "{} exited with {}",
            bin_name,
            status
        )));
    }

    Ok(())
}
```

- [ ] **Step 2: Add dev subcommand to main.rs**

Add to the `Commands` enum in `main.rs`:

```rust
    Dev {
        #[arg(short, long, default_value = ".", help = "Project directory")]
        path: PathBuf,
    },
```

Add the match arm:

```rust
        Commands::Dev { path } => {
            match runtime_cli::dev::run_dev(&path) {
                Ok(()) => {}
                Err(e) => {
                    eprintln!("Error: {e}");
                    std::process::exit(1);
                }
            }
        }
```

- [ ] **Step 3: Update lib.rs**

```rust
pub mod dev;
pub mod init;
```

- [ ] **Step 4: Verify it compiles**

Run: `cargo check -p runtime-cli`
Expected: Compiles successfully

- [ ] **Step 5: Commit**

```bash
git add -A
git commit -s -m "feat: add dev command to runtime-cli"
```

---

### Task 10: CLI — Build Command (Production Binary)

**Files:**
- Create: `crates/runtime-cli/src/build.rs`
- Modify: `crates/runtime-cli/src/main.rs`
- Modify: `crates/runtime-cli/src/lib.rs`

- [ ] **Step 1: Implement build command**

Write `crates/runtime-cli/src/build.rs`:

```rust
use std::path::Path;
use std::process::{Command, Stdio};

use runtime_sandbox::manifest::AppManifest;

#[derive(Debug, thiserror::Error)]
pub enum BuildError {
    #[error("manifest error: {0}")]
    Manifest(#[from] runtime_sandbox::manifest::ManifestError),
    #[error("build failed: {0}")]
    Build(String),
    #[error("io error: {0}")]
    Io(#[from] std::io::Error),
    #[error("no app.toml found in current directory")]
    NoManifest,
}

pub fn run_build(project_dir: &Path) -> Result<(), BuildError> {
    let manifest_path = project_dir.join("app.toml");
    if !manifest_path.exists() {
        return Err(BuildError::NoManifest);
    }

    let manifest = AppManifest::from_file(&manifest_path)?;
    println!("Building {} (release)...", manifest.app.name);

    let status = Command::new("cargo")
        .args(["build", "--release"])
        .current_dir(project_dir)
        .stdout(Stdio::inherit())
        .stderr(Stdio::inherit())
        .status()?;

    if !status.success() {
        return Err(BuildError::Build("cargo build --release failed".into()));
    }

    let bin_name = &manifest.app.name;
    let bin_path = project_dir
        .join("target")
        .join("release")
        .join(bin_name);

    let size_mb = std::fs::metadata(&bin_path)?.len() as f64 / 1_048_576.0;
    println!("Built: {} ({:.1} MB)", bin_path.display(), size_mb);

    Ok(())
}
```

- [ ] **Step 2: Add build subcommand to main.rs**

Add to the `Commands` enum:

```rust
    Build {
        #[arg(short, long, default_value = ".", help = "Project directory")]
        path: PathBuf,
    },
```

Add the match arm:

```rust
        Commands::Build { path } => {
            match runtime_cli::build::run_build(&path) {
                Ok(()) => {}
                Err(e) => {
                    eprintln!("Error: {e}");
                    std::process::exit(1);
                }
            }
        }
```

- [ ] **Step 3: Update lib.rs**

```rust
pub mod build;
pub mod dev;
pub mod init;
```

- [ ] **Step 4: Verify it compiles**

Run: `cargo check -p runtime-cli`
Expected: Compiles successfully

- [ ] **Step 5: Commit**

```bash
git add -A
git commit -s -m "feat: add build command (release) to runtime-cli"
```

---

## Phase 6: Integration

### Task 11: Wire IPC into Runtime Core

**Files:**
- Modify: `crates/runtime-core/Cargo.toml`
- Modify: `crates/runtime-core/src/lib.rs`

- [ ] **Step 1: Add runtime-ipc dependency to runtime-core**

Add to `crates/runtime-core/Cargo.toml`:

```toml
runtime-ipc = { path = "../runtime-ipc" }
runtime-sandbox = { path = "../runtime-sandbox" }
```

- [ ] **Step 2: Integrate CommandRegistry into AppBuilder**

Update `AppBuilder` in `lib.rs` to hold a command registry and inject the bridge JS:

```rust
use runtime_ipc::command::{CommandHandler, CommandRegistry};

pub struct AppBuilder {
    url: Option<url::Url>,
    title: String,
    width: u32,
    height: u32,
    commands: CommandRegistry,
}

impl AppBuilder {
    pub fn new() -> Self {
        Self {
            url: None,
            title: "App".to_string(),
            width: 1024,
            height: 768,
            commands: CommandRegistry::new(),
        }
    }

    pub fn command(mut self, name: impl Into<String>, handler: impl CommandHandler + 'static) -> Self {
        self.commands.register(name, handler);
        self
    }

    // ... existing methods unchanged
}
```

Note: Actually wiring `__runtime_send` from JS to the Rust `CommandRegistry` requires hooking into Servo's `UserContentManager` for script injection and implementing a custom protocol or console message handler for the return path. The exact mechanism depends on Servo's API for JS-to-embedder communication — consult `docs.rs/servo/0.1.0` for `WebViewDelegate` methods that handle messages from JS (e.g., console messages, custom URI schemes, or `postMessage`). This wiring is the most Servo-API-dependent part of the project and may require upstream contributions if no suitable callback exists.

- [ ] **Step 3: Verify it compiles**

Run: `cargo check -p runtime-core`
Expected: Compiles successfully

- [ ] **Step 4: Commit**

```bash
git add -A
git commit -s -m "feat: integrate IPC command registry into runtime-core AppBuilder"
```

---

### Task 12: End-to-End Integration Test

**Files:**
- Modify: `examples/hello-world/src/main.rs`
- Modify: `examples/hello-world/Cargo.toml`
- Modify: `examples/hello-world/frontend/index.html`

- [ ] **Step 1: Add IPC usage to hello-world example**

Update `examples/hello-world/Cargo.toml` to add dependencies:

```toml
[dependencies]
runtime-core = { path = "../../crates/runtime-core" }
runtime-ipc = { path = "../../crates/runtime-ipc" }
runtime-macros = { path = "../../crates/runtime-macros" }
url = "2"
serde = { version = "1", features = ["derive"] }
serde_json = "1"
```

Update `examples/hello-world/src/main.rs`:

```rust
use std::path::PathBuf;

use runtime_ipc::command::CommandHandler;
use serde_json::{json, Value};

struct GreetHandler;

impl CommandHandler for GreetHandler {
    fn handle(&self, args: Value) -> Result<Value, runtime_ipc::IpcError> {
        let name = args.get("name")
            .and_then(|v| v.as_str())
            .unwrap_or("World");
        Ok(json!({ "greeting": format!("Hello, {}!", name) }))
    }
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let html_path = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("frontend")
        .join("index.html");

    let url = url::Url::from_file_path(&html_path)
        .map_err(|_| format!("Invalid path: {}", html_path.display()))?;

    runtime_core::AppBuilder::new()
        .title("Hello World")
        .size(800, 600)
        .url(url)
        .command("greet", GreetHandler)
        .run()
}
```

- [ ] **Step 2: Update frontend to use IPC (ready for when bridge is wired)**

Update `examples/hello-world/frontend/index.html`:

```html
<!DOCTYPE html>
<html>
<head>
    <meta charset="utf-8">
    <title>Hello World</title>
    <style>
        body {
            font-family: system-ui, sans-serif;
            display: flex;
            justify-content: center;
            align-items: center;
            min-height: 100vh;
            margin: 0;
            background: #1a1a2e;
            color: #eee;
        }
        .container { text-align: center; }
        h1 { font-size: 3rem; margin-bottom: 0.5rem; }
        p { font-size: 1.2rem; opacity: 0.7; }
        button {
            margin-top: 1rem;
            padding: 0.5rem 1.5rem;
            font-size: 1rem;
            border: 1px solid #eee;
            background: transparent;
            color: #eee;
            border-radius: 4px;
            cursor: pointer;
        }
        button:hover { background: rgba(255,255,255,0.1); }
        #result { margin-top: 1rem; font-size: 1.1rem; }
    </style>
</head>
<body>
    <div class="container">
        <h1>Hello from Servo!</h1>
        <p>This app is running on the lightweight runtime.</p>
        <button onclick="greet()">Greet</button>
        <div id="result"></div>
    </div>
    <script>
        async function greet() {
            if (window.__runtime) {
                const response = await window.__runtime.invoke('greet', { name: 'Servo' });
                document.getElementById('result').textContent = response.greeting;
            } else {
                document.getElementById('result').textContent = 'IPC bridge not available yet';
            }
        }
    </script>
</body>
</html>
```

- [ ] **Step 3: Build and run**

Run: `cargo run -p hello-world`
Expected: Window opens. The "Greet" button shows "IPC bridge not available yet" (since the JS→Rust wiring depends on Servo's delegate API). The window renders correctly and closes cleanly.

- [ ] **Step 4: Commit**

```bash
git add -A
git commit -s -m "feat: add IPC greet command to hello-world example"
```

---

## Phase 7: TypeScript Bindings Generator

### Task 13: Bindings Generator (Standalone Tool)

**Files:**
- Create: `runtime-bindgen/Cargo.toml`
- Create: `runtime-bindgen/src/main.rs`
- Test: `runtime-bindgen/tests/bindgen_test.rs`

- [ ] **Step 1: Create the bindgen crate**

Add to workspace members. Write `runtime-bindgen/Cargo.toml`:

```toml
[package]
name = "runtime-bindgen"
version = "0.1.0"
edition.workspace = true
license.workspace = true

[dependencies]
syn = { version = "2", features = ["full", "parsing", "visit"] }
quote = "1"
proc-macro2 = "1"
clap = { version = "4", features = ["derive"] }
walkdir = "2"
```

- [ ] **Step 2: Write failing test**

Write `runtime-bindgen/tests/bindgen_test.rs`:

```rust
use runtime_bindgen::generate_bindings;

#[test]
fn generates_ts_for_simple_command() {
    let rust_source = r#"
#[command]
fn greet(name: String) -> Result<String, AppError> {
    Ok(format!("Hello, {}!", name))
}
"#;
    let ts = generate_bindings(rust_source);
    assert!(ts.contains("greet"));
    assert!(ts.contains("name: string"));
    assert!(ts.contains("Promise<string>"));
}

#[test]
fn generates_ts_for_command_with_multiple_args() {
    let rust_source = r#"
#[command]
fn add(a: i32, b: i32) -> Result<i32, AppError> {
    Ok(a + b)
}
"#;
    let ts = generate_bindings(rust_source);
    assert!(ts.contains("add"));
    assert!(ts.contains("a: number"));
    assert!(ts.contains("b: number"));
    assert!(ts.contains("Promise<number>"));
}
```

- [ ] **Step 3: Run tests to verify they fail**

Run: `cargo test -p runtime-bindgen`
Expected: FAIL

- [ ] **Step 4: Implement the bindings generator**

Write `runtime-bindgen/src/main.rs` (also exposed as lib):

Create `runtime-bindgen/src/lib.rs`:

```rust
use syn::{FnArg, ItemFn, Pat, ReturnType, Type, visit::Visit};

pub fn generate_bindings(source: &str) -> String {
    let file = syn::parse_file(source).expect("Failed to parse Rust source");
    let mut visitor = CommandVisitor { commands: vec![] };
    visitor.visit_file(&file);

    let mut output = String::new();
    output.push_str("// Auto-generated TypeScript bindings\n");
    output.push_str("// Do not edit manually\n\n");

    output.push_str("declare const __runtime: {\n");
    output.push_str("  invoke(command: string, args?: any): Promise<any>;\n");
    output.push_str("  on(event: string, callback: (payload: any) => void): void;\n");
    output.push_str("};\n\n");

    output.push_str("export const commands = {\n");
    for cmd in &visitor.commands {
        let args_type = if cmd.args.is_empty() {
            String::new()
        } else {
            let fields: Vec<String> = cmd.args.iter()
                .map(|(name, ty)| format!("{}: {}", name, ty))
                .collect();
            format!("args: {{ {} }}", fields.join(", "))
        };

        output.push_str(&format!(
            "  async {}({}): Promise<{}> {{\n",
            cmd.name, args_type, cmd.return_type
        ));
        if cmd.args.is_empty() {
            output.push_str(&format!(
                "    return __runtime.invoke('{}');\n",
                cmd.name
            ));
        } else {
            output.push_str(&format!(
                "    return __runtime.invoke('{}', args);\n",
                cmd.name
            ));
        }
        output.push_str("  },\n");
    }
    output.push_str("};\n");

    output
}

struct CommandInfo {
    name: String,
    args: Vec<(String, String)>,
    return_type: String,
}

struct CommandVisitor {
    commands: Vec<CommandInfo>,
}

impl<'ast> Visit<'ast> for CommandVisitor {
    fn visit_item_fn(&mut self, node: &'ast ItemFn) {
        let has_command_attr = node.attrs.iter().any(|attr| {
            attr.path().is_ident("command")
        });

        if !has_command_attr {
            return;
        }

        let name = node.sig.ident.to_string();
        let args: Vec<(String, String)> = node.sig.inputs.iter().filter_map(|arg| {
            if let FnArg::Typed(pat_type) = arg {
                if let Pat::Ident(ident) = pat_type.pat.as_ref() {
                    let arg_name = ident.ident.to_string();
                    let ts_type = rust_type_to_ts(&pat_type.ty);
                    return Some((arg_name, ts_type));
                }
            }
            None
        }).collect();

        let return_type = match &node.sig.output {
            ReturnType::Default => "void".to_string(),
            ReturnType::Type(_, ty) => extract_result_ok_type(ty),
        };

        self.commands.push(CommandInfo {
            name,
            args,
            return_type,
        });
    }
}

fn rust_type_to_ts(ty: &Type) -> String {
    let type_str = quote::quote!(#ty).to_string().replace(' ', "");
    match type_str.as_str() {
        "String" | "&str" => "string".to_string(),
        "i8" | "i16" | "i32" | "i64" | "u8" | "u16" | "u32" | "u64" | "f32" | "f64" | "isize" | "usize" => {
            "number".to_string()
        }
        "bool" => "boolean".to_string(),
        _ => "any".to_string(),
    }
}

fn extract_result_ok_type(ty: &Type) -> String {
    let type_str = quote::quote!(#ty).to_string();
    if let Some(start) = type_str.find("Result<") {
        let after_result = &type_str[start + 7..];
        if let Some(comma) = after_result.find(',') {
            let ok_type = after_result[..comma].trim();
            return rust_type_to_ts(&syn::parse_str::<Type>(ok_type).unwrap_or_else(|_| {
                syn::parse_str::<Type>("()").unwrap()
            }));
        }
    }
    "any".to_string()
}
```

Update `runtime-bindgen/Cargo.toml` to add lib section:

```toml
[lib]
name = "runtime_bindgen"
path = "src/lib.rs"
```

- [ ] **Step 5: Run tests**

Run: `cargo test -p runtime-bindgen`
Expected: Both tests pass

- [ ] **Step 6: Write the CLI entry point**

Write `runtime-bindgen/src/main.rs`:

```rust
use clap::Parser;
use std::path::PathBuf;
use walkdir::WalkDir;

#[derive(Parser)]
#[command(name = "runtime-bindgen", about = "Generate TypeScript bindings from #[command] functions")]
struct Cli {
    #[arg(help = "Source directory to scan")]
    source: PathBuf,
    #[arg(short, long, default_value = "build/bindings.ts", help = "Output file")]
    output: PathBuf,
}

fn main() {
    let cli = Cli::parse();

    let mut all_source = String::new();
    for entry in WalkDir::new(&cli.source)
        .into_iter()
        .filter_map(|e| e.ok())
        .filter(|e| e.path().extension().is_some_and(|ext| ext == "rs"))
    {
        if let Ok(content) = std::fs::read_to_string(entry.path()) {
            all_source.push_str(&content);
            all_source.push('\n');
        }
    }

    let bindings = runtime_bindgen::generate_bindings(&all_source);

    if let Some(parent) = cli.output.parent() {
        std::fs::create_dir_all(parent).expect("Failed to create output directory");
    }
    std::fs::write(&cli.output, &bindings).expect("Failed to write bindings");
    println!("Generated bindings at {}", cli.output.display());
}
```

- [ ] **Step 7: Commit**

```bash
git add -A
git commit -s -m "feat: add runtime-bindgen for TypeScript bindings generation"
```

---

## Summary

| Phase | Tasks | What it produces |
|-------|-------|-----------------|
| **1: Runtime Core** | Tasks 1-3 | A working window rendering HTML via Servo |
| **2: Manifest & Sandbox** | Task 4 | `app.toml` parsing, scoped paths, permission model |
| **3: IPC Bridge** | Tasks 5-6 | Command registry, event emitter, JS bridge, `#[command]` macro |
| **4: Native APIs** | Task 7 | Filesystem, clipboard, dialog, shell, network, window APIs |
| **5: CLI** | Tasks 8-10 | `init`, `dev`, `build` commands |
| **6: Integration** | Tasks 11-12 | IPC wired into runtime core, end-to-end example |
| **7: Bindgen** | Task 13 | TypeScript bindings generator |

### Follow-on Tasks (Not in This Plan)

These are documented in the spec but deferred to keep this plan at a manageable size:

- **`package` CLI command** — AppImage generation using `appimagetool`. Depends on a working `build` command (Task 10).
- **`check permissions` CLI command** — Static analysis using `syn` to scan for `runtime_api::*` usage and compare against `app.toml`. Depends on manifest parsing (Task 4) and the API modules (Task 7).
- **`generate bindings` CLI subcommand** — Wire `runtime-bindgen` (Task 13) into the CLI as a subcommand.
- **JS Bridge wiring** — The actual `__runtime_send` → Rust callback path. Depends on Servo's `WebViewDelegate` or `UserContentManager` APIs. This is the most uncertain piece and may require upstream work.

### Key Risks & Notes

1. **Servo's WebViewDelegate API** — The exact mechanism for JS→Rust message passing (how `__runtime_send` calls back into the embedder) depends on Servo APIs not fully documented in the 0.1.0 docs.rs listing. The `WebViewDelegate` trait and `UserContentManager` are the most likely candidates. This may require reading Servo's source code or contributing upstream.

2. **Build times** — Servo is a large dependency. First build will take 15-30 minutes. Incremental builds should be fast. Consider using `sccache` or `mold` linker.

3. **Binary size** — The 30-50MB target is ambitious. Servo's dependencies (SpiderMonkey, WebRender, Stylo) are substantial. Profile with `cargo bloat` and enable LTO + `strip` in release profile.

4. **Platform-specific behavior** — winit and Servo may behave differently on X11 vs Wayland. Test both.
