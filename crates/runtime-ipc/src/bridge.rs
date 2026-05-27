use serde::{Deserialize, Serialize};

/// An IPC request sent from the frontend to the Rust backend.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IpcRequest {
    pub id: u64,
    pub command: String,
    pub args: serde_json::Value,
}

/// An IPC response sent from the Rust backend to the frontend.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IpcResponse {
    pub id: u64,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub result: Option<serde_json::Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<String>,
}

impl IpcResponse {
    /// Create a successful response.
    pub fn success(id: u64, value: serde_json::Value) -> Self {
        Self {
            id,
            result: Some(value),
            error: None,
        }
    }

    /// Create an error response.
    pub fn error(id: u64, message: impl Into<String>) -> Self {
        Self {
            id,
            result: None,
            error: Some(message.into()),
        }
    }
}

/// JavaScript injected into pages to provide `window.__runtime.invoke()` and
/// `window.__runtime.on()`.
pub const BRIDGE_JS: &str = r#"
(function() {
    "use strict";

    const pending = new Map();
    const listeners = new Map();
    let nextId = 1;

    window.__runtime = {
        invoke(command, args) {
            return new Promise((resolve, reject) => {
                const id = nextId++;
                pending.set(id, { resolve, reject });
                window.__runtime._send(JSON.stringify({ id, command, args: args || {} }));
            });
        },

        on(event, callback) {
            if (!listeners.has(event)) {
                listeners.set(event, []);
            }
            listeners.get(event).push(callback);
        },

        _handleResponse(msg) {
            const { id, result, error } = JSON.parse(msg);
            const p = pending.get(id);
            if (!p) return;
            pending.delete(id);
            if (error) {
                p.reject(new Error(error));
            } else {
                p.resolve(result);
            }
        },

        _handleEvent(msg) {
            const { name, payload } = JSON.parse(msg);
            const cbs = listeners.get(name);
            if (cbs) {
                for (const cb of cbs) {
                    try { cb(payload); } catch(e) { console.error(e); }
                }
            }
        },

        _send(json) {
            console.log("__IPC__:" + json);
        },
    };
})();
"#;
