use winit::window::Window;

pub struct WindowApi;

impl WindowApi {
    pub fn set_title(window: &Window, title: &str) {
        window.set_title(title);
    }

    pub fn set_minimized(window: &Window, minimized: bool) {
        window.set_minimized(minimized);
    }

    pub fn set_maximized(window: &Window, maximized: bool) {
        window.set_maximized(maximized);
    }

    pub fn set_fullscreen(window: &Window, fullscreen: bool) {
        if fullscreen {
            window.set_fullscreen(Some(winit::window::Fullscreen::Borderless(None)));
        } else {
            window.set_fullscreen(None);
        }
    }
}
