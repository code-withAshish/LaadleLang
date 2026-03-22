#![cfg(target_arch = "wasm32")]

use laadlelang::run_laadle_code;
use wasm_bindgen_test::*;

wasm_bindgen_test_configure!(run_in_browser);

#[wasm_bindgen_test]
fn sanity_check() {
    assert_eq!(2 + 2, 4);
}

#[wasm_bindgen_test]
fn test_fibonacci_output() {
    let source = r#"
kaam fibonacci(n) toh
    agar n <= 1 toh
        wapas n
    laadle result hai fibonacci(n - 1) + fibonacci(n - 2)
    wapas result

laadle num hai 5
bol "Fibonacci of " + num + " is:"
bol fibonacci(num)
"#;

    let output = run_laadle_code(source);

    assert!(output.contains("Fibonacci of 5 is:"));
    assert!(output.contains("5"));
}

#[wasm_bindgen_test]
fn test_divide_by_zero_trapped_securely() {
    let source = r#"
kaam safe_divide(a, b) toh
    agar b == 0 toh
        gopgop "CRITICAL: Attempted to divide by zero!"
    wapas a / b

koshish toh
    safe_divide(10, 0)
pakad err toh
    bol "Caught!"
    bol err
"#;

    let output = run_laadle_code(source);

    assert!(output.contains("Caught!"));
    assert!(output.contains("CRITICAL: Attempted to divide by zero!"));
    assert!(!output.contains("[VM Crash] Uncaught Error:"));
}

#[wasm_bindgen_test]
fn test_empty_wapas_syntax_recovery() {
    let source = r#"
kaam calculate_something(n) toh
    agar n < 0 toh
        wapas
"#;

    let output = run_laadle_code(source);

    assert_eq!(output.trim(), "");
}
