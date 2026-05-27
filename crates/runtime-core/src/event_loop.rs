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
            tracing::warn!("Failed to wake event loop: {e}");
        }
    }
}
