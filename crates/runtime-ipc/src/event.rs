use serde::Serialize;

/// A single event destined for the frontend.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct EventMessage {
    pub name: String,
    pub payload: serde_json::Value,
}

/// Collects events to be flushed to the frontend.
pub struct EventEmitter {
    queue: Vec<EventMessage>,
}

impl EventEmitter {
    pub fn new() -> Self {
        Self { queue: Vec::new() }
    }

    /// Queue an event with a serializable payload.
    pub fn emit(&mut self, name: impl Into<String>, payload: impl Serialize) {
        let payload = serde_json::to_value(payload).unwrap_or(serde_json::Value::Null);
        self.queue.push(EventMessage {
            name: name.into(),
            payload,
        });
    }

    /// Drain all queued events, returning them in order.
    pub fn drain(&mut self) -> Vec<EventMessage> {
        std::mem::take(&mut self.queue)
    }
}

impl Default for EventEmitter {
    fn default() -> Self {
        Self::new()
    }
}
