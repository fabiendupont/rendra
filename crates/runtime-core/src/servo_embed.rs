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
