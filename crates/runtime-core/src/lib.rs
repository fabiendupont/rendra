pub mod event_loop;
pub mod servo_embed;
pub mod window;

use std::rc::Rc;

use euclid::Size2D;
use url::Url;
use winit::application::ApplicationHandler;
use winit::event::WindowEvent;
use winit::event_loop::EventLoop;

use crate::event_loop::{Waker, WakerEvent};
use crate::servo_embed::ServoInstance;
use crate::window::AppWindow;

/// Builder for configuring and launching a Servo-based application.
pub struct AppBuilder {
    title: String,
    size: Size2D<i32, ()>,
    url: Url,
    commands: runtime_ipc::command::CommandRegistry,
}

impl Default for AppBuilder {
    fn default() -> Self {
        Self {
            title: String::from("Servo Runtime"),
            size: Size2D::new(1024, 768),
            url: Url::parse("about:blank").expect("valid default URL"),
            commands: runtime_ipc::command::CommandRegistry::new(),
        }
    }
}

impl AppBuilder {
    /// Create a new `AppBuilder` with default settings.
    pub fn new() -> Self {
        Self::default()
    }

    /// Set the window title.
    pub fn title(mut self, title: impl Into<String>) -> Self {
        self.title = title.into();
        self
    }

    /// Set the initial window size in logical pixels.
    pub fn size(mut self, width: i32, height: i32) -> Self {
        self.size = Size2D::new(width, height);
        self
    }

    /// Set the initial URL to load.
    pub fn url(mut self, url: Url) -> Self {
        self.url = url;
        self
    }

    /// Register an IPC command handler.
    pub fn command(mut self, name: impl Into<String>, handler: impl runtime_ipc::command::CommandHandler + 'static) -> Self {
        self.commands.register(name, handler);
        self
    }

    /// Build and run the application, consuming this builder.
    ///
    /// This initializes the TLS crypto provider, creates a winit event loop,
    /// and enters the main loop. This function does not return.
    pub fn run(self) -> Result<(), Box<dyn std::error::Error>> {
        // Initialize the rustls crypto provider (required by servo for TLS).
        rustls::crypto::aws_lc_rs::default_provider()
            .install_default()
            .expect("Failed to install rustls crypto provider");

        let event_loop: EventLoop<WakerEvent> = EventLoop::with_user_event()
            .build()?;
        let waker = Waker::new(&event_loop);

        let mut app = App {
            title: self.title,
            width: self.size.width.max(0) as u32,
            height: self.size.height.max(0) as u32,
            url: self.url,
            waker,
            commands: Some(self.commands),
            state: None,
        };

        event_loop.run_app(&mut app)?;
        Ok(())
    }
}

/// Holds the live servo + window state after the event loop resumes.
struct AppState {
    servo: ServoInstance,
    window: Rc<AppWindow>,
    commands: runtime_ipc::command::CommandRegistry,
}

/// The winit application handler.
struct App {
    title: String,
    width: u32,
    height: u32,
    url: Url,
    waker: Waker,
    commands: Option<runtime_ipc::command::CommandRegistry>,
    state: Option<AppState>,
}

impl ApplicationHandler<WakerEvent> for App {
    fn resumed(&mut self, event_loop: &winit::event_loop::ActiveEventLoop) {
        if self.state.is_some() {
            return;
        }

        let window = AppWindow::new(event_loop, &self.title, self.width, self.height)
            .expect("Failed to create window");

        let servo = ServoInstance::new(self.waker.clone());
        servo.setup_logging();

        let _webview = window.create_webview(&servo, self.url.clone());

        let commands = self.commands.take().unwrap_or_default();
        self.state = Some(AppState { servo, window, commands });
    }

    fn window_event(
        &mut self,
        event_loop: &winit::event_loop::ActiveEventLoop,
        _window_id: winit::window::WindowId,
        event: WindowEvent,
    ) {
        let Some(state) = self.state.as_ref() else {
            return;
        };

        match event {
            WindowEvent::CloseRequested => {
                event_loop.exit();
            }
            WindowEvent::RedrawRequested => {
                state.servo.spin();
                state.window.paint();
            }
            WindowEvent::Resized(_) => {
                state.window.resize();
            }
            _ => {}
        }
    }

    fn user_event(&mut self, _event_loop: &winit::event_loop::ActiveEventLoop, _event: WakerEvent) {
        if let Some(state) = self.state.as_ref() {
            state.servo.spin();
        }
    }
}
