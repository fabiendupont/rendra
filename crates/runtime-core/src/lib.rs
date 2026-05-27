pub mod event_loop;
pub mod servo_embed;
pub mod window;

use euclid::Size2D;
use url::Url;

/// Builder for configuring and launching a Servo-based application.
pub struct AppBuilder {
    title: String,
    size: Size2D<i32, ()>,
    url: Url,
}

impl Default for AppBuilder {
    fn default() -> Self {
        Self {
            title: String::from("Servo Runtime"),
            size: Size2D::new(1024, 768),
            url: Url::parse("about:blank").expect("valid default URL"),
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
}
