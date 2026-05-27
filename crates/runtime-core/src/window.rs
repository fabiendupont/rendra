use std::cell::RefCell;
use std::rc::Rc;

use servo::{
    ConsoleLogLevel, RenderingContext, UserContentManager, UserScript, WebView, WebViewBuilder,
    WindowRenderingContext,
};
use url::Url;
use winit::event_loop::ActiveEventLoop;
use winit::raw_window_handle::{HasDisplayHandle, HasWindowHandle};
use winit::window::Window;

use crate::servo_embed::ServoInstance;

const IPC_PREFIX: &str = "__IPC__:";

pub struct AppWindow {
    pub window: Window,
    pub rendering_context: Rc<WindowRenderingContext>,
    pub webviews: RefCell<Vec<WebView>>,
    commands: RefCell<runtime_ipc::command::CommandRegistry>,
    pending_responses: RefCell<Vec<String>>,
}

impl AppWindow {
    pub fn new(
        event_loop: &ActiveEventLoop,
        title: &str,
        width: u32,
        height: u32,
        commands: runtime_ipc::command::CommandRegistry,
    ) -> Result<Rc<Self>, Box<dyn std::error::Error>> {
        let attrs = Window::default_attributes()
            .with_title(title)
            .with_inner_size(winit::dpi::LogicalSize::new(width, height));
        let window = event_loop.create_window(attrs)?;

        let display_handle = window.display_handle()?;
        let window_handle = window.window_handle()?;

        let rendering_context = Rc::new(
            WindowRenderingContext::new(display_handle, window_handle, window.inner_size())
                .expect("Could not create RenderingContext"),
        );
        let _ = rendering_context.make_current();

        Ok(Rc::new(Self {
            window,
            rendering_context,
            webviews: RefCell::new(Vec::new()),
            commands: RefCell::new(commands),
            pending_responses: RefCell::new(Vec::new()),
        }))
    }

    pub fn create_webview(
        self: &Rc<Self>,
        servo: &ServoInstance,
        url: Url,
    ) -> WebView {
        let ucm = Rc::new(UserContentManager::new(&servo.servo));
        let bridge_script = Rc::new(UserScript::new(
            runtime_ipc::bridge::BRIDGE_JS.to_string(),
            None,
        ));
        ucm.add_script(bridge_script);

        let scale = self.window.scale_factor() as f32;
        let webview = WebViewBuilder::new(&servo.servo, self.rendering_context.clone())
            .url(url)
            .hidpi_scale_factor(euclid::Scale::new(scale))
            .user_content_manager(ucm)
            .delegate(self.clone() as Rc<dyn servo::WebViewDelegate>)
            .build();
        self.webviews.borrow_mut().push(webview.clone());
        webview
    }

    pub fn paint(&self) {
        for webview in self.webviews.borrow().iter() {
            webview.paint();
        }
        self.rendering_context.present();
    }

    pub fn resize(&self) {
        let size = self.window.inner_size();
        self.rendering_context.resize(size);
    }

    pub fn flush_pending_responses(&self) {
        let responses: Vec<String> = self.pending_responses.borrow_mut().drain(..).collect();
        if responses.is_empty() {
            return;
        }
        let webviews = self.webviews.borrow();
        let Some(webview) = webviews.last() else {
            return;
        };
        for response_json in responses {
            let escaped = response_json.replace('\\', "\\\\").replace('\'', "\\'");
            let script = format!("window.__runtime._handleResponse('{escaped}')");
            webview.evaluate_javascript(script, |_| {});
        }
    }

    fn handle_ipc_message(&self, json: &str) {
        let request: runtime_ipc::bridge::IpcRequest = match serde_json::from_str(json) {
            Ok(r) => r,
            Err(e) => {
                tracing::warn!("Invalid IPC request: {e}");
                return;
            }
        };

        let response = match self.commands.borrow().invoke(&request.command, request.args) {
            Ok(value) => runtime_ipc::bridge::IpcResponse::success(request.id, value),
            Err(e) => runtime_ipc::bridge::IpcResponse::error(request.id, e.to_string()),
        };

        match serde_json::to_string(&response) {
            Ok(json) => self.pending_responses.borrow_mut().push(json),
            Err(e) => tracing::warn!("Failed to serialize IPC response: {e}"),
        }

        self.window.request_redraw();
    }
}

impl servo::WebViewDelegate for AppWindow {
    fn notify_new_frame_ready(&self, _webview: WebView) {
        self.window.request_redraw();
    }

    fn show_console_message(&self, _webview: WebView, level: ConsoleLogLevel, message: String) {
        if let Some(json) = message.strip_prefix(IPC_PREFIX) {
            self.handle_ipc_message(json);
            return;
        }

        match level {
            ConsoleLogLevel::Error => tracing::error!("[console] {message}"),
            ConsoleLogLevel::Warn => tracing::warn!("[console] {message}"),
            _ => tracing::info!("[console] {message}"),
        }
    }
}
