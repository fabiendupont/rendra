use std::collections::HashMap;

use crate::IpcError;

/// Trait for handling IPC commands dispatched from frontend JavaScript.
pub trait CommandHandler: Send + Sync {
    fn handle(&self, args: serde_json::Value) -> Result<serde_json::Value, IpcError>;
}

/// Registry that maps command names to their handlers.
pub struct CommandRegistry {
    handlers: HashMap<String, Box<dyn CommandHandler>>,
}

impl CommandRegistry {
    pub fn new() -> Self {
        Self {
            handlers: HashMap::new(),
        }
    }

    /// Register a handler for the given command name.
    pub fn register(&mut self, name: impl Into<String>, handler: impl CommandHandler + 'static) {
        self.handlers.insert(name.into(), Box::new(handler));
    }

    /// Invoke a command by name with the given arguments.
    pub fn invoke(&self, name: &str, args: serde_json::Value) -> Result<serde_json::Value, IpcError> {
        let handler = self
            .handlers
            .get(name)
            .ok_or_else(|| IpcError::UnknownCommand(name.to_string()))?;
        handler.handle(args)
    }

    /// Check whether a command is registered.
    pub fn has_command(&self, name: &str) -> bool {
        self.handlers.contains_key(name)
    }
}

impl Default for CommandRegistry {
    fn default() -> Self {
        Self::new()
    }
}
