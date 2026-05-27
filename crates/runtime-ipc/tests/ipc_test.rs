use runtime_ipc::command::{CommandHandler, CommandRegistry};
use runtime_ipc::IpcError;
use serde_json::json;

struct EchoHandler;

impl CommandHandler for EchoHandler {
    fn handle(&self, args: serde_json::Value) -> Result<serde_json::Value, IpcError> {
        Ok(args)
    }
}

struct GreetHandler;

impl CommandHandler for GreetHandler {
    fn handle(&self, args: serde_json::Value) -> Result<serde_json::Value, IpcError> {
        let name = args
            .get("name")
            .and_then(|v| v.as_str())
            .ok_or_else(|| IpcError::InvalidArgs("missing 'name' field".into()))?;
        Ok(json!({ "greeting": format!("Hello, {name}!") }))
    }
}

#[test]
fn register_and_invoke_command() {
    let mut registry = CommandRegistry::new();
    registry.register("echo", EchoHandler);
    let result = registry.invoke("echo", json!({"msg": "hi"})).unwrap();
    assert_eq!(result, json!({"msg": "hi"}));
}

#[test]
fn invoke_unknown_command_returns_error() {
    let registry = CommandRegistry::new();
    let err = registry.invoke("nope", json!({})).unwrap_err();
    assert!(matches!(err, IpcError::UnknownCommand(_)));
}

#[test]
fn invoke_greet_command() {
    let mut registry = CommandRegistry::new();
    registry.register("greet", GreetHandler);
    let result = registry.invoke("greet", json!({"name": "World"})).unwrap();
    assert_eq!(result, json!({"greeting": "Hello, World!"}));
}

#[test]
fn invoke_greet_missing_name() {
    let mut registry = CommandRegistry::new();
    registry.register("greet", GreetHandler);
    let err = registry.invoke("greet", json!({})).unwrap_err();
    assert!(matches!(err, IpcError::InvalidArgs(_)));
}
