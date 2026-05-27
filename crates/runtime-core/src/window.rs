use std::cell::RefCell;
use std::rc::Rc;

use servo::{RenderingContext, WebView, WebViewBuilder, WindowRenderingContext};
use url::Url;
use winit::event_loop::ActiveEventLoop;
use winit::raw_window_handle::{HasDisplayHandle, HasWindowHandle};
use winit::window::Window;

use crate::servo_embed::ServoInstance;

pub struct AppWindow {
    pub window: Window,
    pub rendering_context: Rc<WindowRenderingContext>,
    pub webviews: RefCell<Vec<WebView>>,
}

impl AppWindow {
    pub fn new(
        event_loop: &ActiveEventLoop,
        title: &str,
        width: u32,
        height: u32,
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
        }))
    }

    pub fn create_webview(
        self: &Rc<Self>,
        servo: &ServoInstance,
        url: Url,
    ) -> WebView {
        let scale = self.window.scale_factor() as f32;
        let webview = WebViewBuilder::new(&servo.servo, self.rendering_context.clone())
            .url(url)
            .hidpi_scale_factor(euclid::Scale::new(scale))
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

}

impl servo::WebViewDelegate for AppWindow {
    fn notify_new_frame_ready(&self, _webview: WebView) {
        self.window.request_redraw();
    }
}
