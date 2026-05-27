use runtime_bindgen::generate_bindings;

#[test]
fn generates_ts_for_simple_command() {
    let source = r#"
        #[command]
        fn greet(name: String) -> Result<String, AppError> {
            Ok(format!("Hello, {}!", name))
        }
    "#;

    let output = generate_bindings(source);

    assert!(output.contains("greet"), "output should contain 'greet'");
    assert!(
        output.contains("name: string"),
        "output should contain 'name: string'"
    );
    assert!(
        output.contains("Promise<string>"),
        "output should contain 'Promise<string>'"
    );
}

#[test]
fn generates_ts_for_command_with_multiple_args() {
    let source = r#"
        #[command]
        fn add(a: i32, b: i32) -> Result<i32, AppError> {
            Ok(a + b)
        }
    "#;

    let output = generate_bindings(source);

    assert!(output.contains("add"), "output should contain 'add'");
    assert!(
        output.contains("a: number"),
        "output should contain 'a: number'"
    );
    assert!(
        output.contains("b: number"),
        "output should contain 'b: number'"
    );
    assert!(
        output.contains("Promise<number>"),
        "output should contain 'Promise<number>'"
    );
}
