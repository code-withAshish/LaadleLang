//! End-to-end integration tests for LaadleLang.
//!
//! Each test runs the full pipeline:  source → tokenize → parse → compile → VM
//!
//! These tests live in `tests/` so they exercise only the crate's public API,
//! just like a real user of the language would.

use laadlelang::compiler::compile_source;
use laadlelang::vm::{LaadleVirtualMachineV1, Value};

// Helper: compile source and run to completion, return the VM state.
fn run(src: &str) -> LaadleVirtualMachineV1 {
    let ops = compile_source(src);
    let mut vm = LaadleVirtualMachineV1::new(ops);
    vm.run();
    vm
}

// ── Expressions & arithmetic ──────────────────────────────────────────────────

#[test]
fn e2e_arithmetic_precedence() {
    // 2 + 3 * 4 = 14  (multiplication binds tighter)
    let vm = run("laadle x hai 2 + 3 * 4\n");
    assert_eq!(vm.globals.get("x"), Some(&Value::Int(14)));
}

#[test]
fn e2e_parenthesised_expr() {
    // (2 + 3) * 4 = 20
    let vm = run("laadle x hai (2 + 3) * 4\n");
    assert_eq!(vm.globals.get("x"), Some(&Value::Int(20)));
}

#[test]
fn e2e_unary_neg() {
    let vm = run("laadle x hai -7\n");
    assert_eq!(vm.globals.get("x"), Some(&Value::Int(-7)));
}

#[test]
fn e2e_boolean_not() {
    let vm = run("laadle x hai !galat\n");
    assert_eq!(vm.globals.get("x"), Some(&Value::Bool(true)));
}

#[test]
fn e2e_comparison_neq() {
    let vm = run("laadle x hai 10 != 20\n");
    assert_eq!(vm.globals.get("x"), Some(&Value::Bool(true)));
}

// ── Variables ─────────────────────────────────────────────────────────────────

#[test]
fn e2e_variable_reassignment() {
    let src = "laadle x hai 1\nx hai x + 1\nx hai x + 1\n";
    let vm = run(src);
    assert_eq!(vm.globals.get("x"), Some(&Value::Int(3)));
}

// ── If / else ─────────────────────────────────────────────────────────────────

#[test]
fn e2e_if_true_branch() {
    let src = "laadle result hai 0\nagar 5 > 3 toh\n    result hai 1\nwarna\n    result hai 2\n";
    let vm = run(src);
    assert_eq!(vm.globals.get("result"), Some(&Value::Int(1)));
}

#[test]
fn e2e_if_false_branch() {
    let src = "laadle result hai 0\nagar 3 > 5 toh\n    result hai 1\nwarna\n    result hai 2\n";
    let vm = run(src);
    assert_eq!(vm.globals.get("result"), Some(&Value::Int(2)));
}

#[test]
fn e2e_nested_if() {
    let src = "laadle r hai 0\nagar sahi toh\n    agar 10 == 10 toh\n        r hai 42\n";
    let vm = run(src);
    assert_eq!(vm.globals.get("r"), Some(&Value::Int(42)));
}

// ── While loop ────────────────────────────────────────────────────────────────

#[test]
fn e2e_while_count_to_5() {
    let src = "laadle i hai 0\njabtak i < 5 toh\n    i hai i + 1\n";
    let vm = run(src);
    assert_eq!(vm.globals.get("i"), Some(&Value::Int(5)));
}

#[test]
fn e2e_while_break() {
    // Loop would run forever (jabtak sahi), but nikal exits at j == 3.
    let src =
        "laadle j hai 0\njabtak sahi toh\n    j hai j + 1\n    agar j == 3 toh\n        nikal\n";
    let vm = run(src);
    assert_eq!(vm.globals.get("j"), Some(&Value::Int(3)));
}

// ── Functions ─────────────────────────────────────────────────────────────────

#[test]
fn e2e_function_return_value() {
    let src = "kaam add(a, b) toh\n    wapas a + b\nlaadle result hai add(10, 32)\n";
    let vm = run(src);
    assert_eq!(vm.globals.get("result"), Some(&Value::Int(42)));
}

#[test]
fn e2e_function_with_loop() {
    // sum(n) = 1 + 2 + … + n
    let src = "kaam sum(n) toh\n    laadle acc hai 0\n    laadle i hai 1\n    jabtak i <= n toh\n        acc hai acc + i\n        i hai i + 1\n    wapas acc\nlaadle result hai sum(5)\n";
    let vm = run(src);
    assert_eq!(vm.globals.get("result"), Some(&Value::Int(15)));
}

#[test]
fn e2e_recursive_factorial() {
    // kaam fact(n): if n <= 1 → 1, else n * fact(n-1)
    let src = concat!(
        "kaam fact(n) toh\n",
        "    agar n <= 1 toh\n",
        "        wapas 1\n",
        "    wapas n * fact(n - 1)\n",
        "laadle result hai fact(5)\n",
    );
    let vm = run(src);
    assert_eq!(vm.globals.get("result"), Some(&Value::Int(120)));
}

#[test]
fn e2e_multiple_function_calls() {
    let src = concat!(
        "kaam double(x) toh\n",
        "    wapas x * 2\n",
        "laadle a hai double(3)\n",
        "laadle b hai double(a)\n",
    );
    let vm = run(src);
    assert_eq!(vm.globals.get("a"), Some(&Value::Int(6)));
    assert_eq!(vm.globals.get("b"), Some(&Value::Int(12)));
}

// ── Fix 3: Short-circuit && / || ─────────────────────────────────────────────

#[test]
fn e2e_and_short_circuit_true() {
    // sahi && sahi → true
    let vm = run("laadle x hai sahi && sahi\n");
    assert_eq!(vm.globals.get("x"), Some(&Value::Bool(true)));
}

#[test]
fn e2e_and_short_circuit_false() {
    // galat && sahi → false (right side should NOT be evaluated)
    let vm = run("laadle x hai galat && sahi\n");
    assert_eq!(vm.globals.get("x"), Some(&Value::Bool(false)));
}

#[test]
fn e2e_or_short_circuit_true() {
    // sahi || galat → true (right side should NOT be evaluated)
    let vm = run("laadle x hai sahi || galat\n");
    assert_eq!(vm.globals.get("x"), Some(&Value::Bool(true)));
}

#[test]
fn e2e_or_short_circuit_false() {
    // galat || galat → false
    let vm = run("laadle x hai galat || galat\n");
    assert_eq!(vm.globals.get("x"), Some(&Value::Bool(false)));
}

#[test]
fn e2e_compound_boolean() {
    // 5 > 3 && 10 < 20 → true
    let vm = run("laadle x hai 5 > 3 && 10 < 20\n");
    assert_eq!(vm.globals.get("x"), Some(&Value::Bool(true)));
}

// ── Fix 2: Global variable access inside functions ────────────────────────────

#[test]
fn e2e_function_reads_global() {
    // The function adds the global `base` to its argument.
    let src = concat!(
        "laadle base hai 100\n",
        "kaam add_base(n) toh\n",
        "    wapas n + base\n",
        "laadle result hai add_base(42)\n",
    );
    let vm = run(src);
    assert_eq!(vm.globals.get("result"), Some(&Value::Int(142)));
}

#[test]
fn e2e_function_uses_global_fn() {
    // double is defined globally; triple calls it.
    let src = concat!(
        "kaam double(x) toh\n",
        "    wapas x * 2\n",
        "kaam triple(x) toh\n",
        "    wapas double(x) + x\n",
        "laadle result hai triple(5)\n",
    );
    let vm = run(src);
    assert_eq!(vm.globals.get("result"), Some(&Value::Int(15)));
}

// ── meow (null literal) ───────────────────────────────────────────────────────

#[test]
fn e2e_meow_null_literal() {
    let vm = run("laadle x hai meow\n");
    assert_eq!(vm.globals.get("x"), Some(&Value::Null));
}

#[test]
fn e2e_meow_null_equality() {
    let vm = run("laadle x hai meow == meow\n");
    assert_eq!(vm.globals.get("x"), Some(&Value::Bool(true)));
}

// ── Float literals ────────────────────────────────────────────────────────────

#[test]
fn e2e_float_literal() {
    let vm = run("laadle x hai 3.14\n");
    assert_eq!(vm.globals.get("x"), Some(&Value::Float(3.14)));
}

#[test]
fn e2e_float_arithmetic() {
    // 1.5 + 2.5 = 4.0
    let vm = run("laadle x hai 1.5 + 2.5\n");
    assert_eq!(vm.globals.get("x"), Some(&Value::Float(4.0)));
}

#[test]
fn e2e_int_float_widening() {
    // int + float → float
    let vm = run("laadle x hai 1 + 2.5\n");
    assert_eq!(vm.globals.get("x"), Some(&Value::Float(3.5)));
}

// ── aage (continue) ──────────────────────────────────────────────────────

#[test]
fn e2e_continue_skips_body() {
    // Sum only even numbers from 1..=6: 2+4+6 = 12.
    // When i is odd, `aage` skips the acc += i line.
    let src = concat!(
        "laadle acc hai 0\n",
        "laadle i hai 0\n",
        "jabtak i < 6 toh\n",
        "    i hai i + 1\n",
        "    agar i == 1 || i == 3 || i == 5 toh\n",
        "        aage\n",
        "    acc hai acc + i\n",
    );
    let vm = run(src);
    assert_eq!(vm.globals.get("acc"), Some(&Value::Int(12)));
}

// ── Throw halts VM gracefully (no panic) ─────────────────────────────────────

#[test]
fn e2e_throw_halts_gracefully() {
    // After gopgop, the VM should stop; the error field should be set.
    // Instructions after the throw must NOT execute.
    let src = concat!(
        "laadle before hai 1\n",
        "gopgop \"something went wrong\"\n",
        "laadle after hai 2\n", // this line must NOT run
    );
    let vm = run(src);
    assert_eq!(vm.globals.get("before"), Some(&Value::Int(1)));
    assert!(vm.error.is_some());
    // `after` must not be set — VM halted before reaching that line
    assert_eq!(vm.globals.get("after"), None);
}

// ── Try / Catch (koshish / pakad) ─────────────────────────────────────────────

#[test]
fn e2e_try_catch_basic() {
    let src = concat!(
        "laadle result hai 0\n",
        "koshish toh\n",
        "    gopgop 42\n",
        "    result hai 1    // skipped\n",
        "pakad err toh\n",
        "    result hai err\n",
    );
    let vm = run(src);
    assert_eq!(vm.globals.get("result"), Some(&Value::Int(42)));
    assert_eq!(vm.globals.get("err"), Some(&Value::Int(42)));
}

#[test]
fn e2e_try_catch_unwind_call_stack() {
    // Tests that throwing inside a deeply nested function correctly unwinds
    // the call stack all the way up to the try block.
    let src = concat!(
        "kaam level3() toh\n",
        "    gopgop \"boom\"\n",
        "kaam level2() toh\n",
        "    level3()\n",
        "kaam level1() toh\n",
        "    level2()\n",
        "laadle result hai \"ok\"\n",
        "koshish toh\n",
        "    level1()\n",
        "    result hai \"unreachable\"\n",
        "pakad err toh\n",
        "    result hai err\n",
    );
    let vm = run(src);
    assert_eq!(
        vm.globals.get("result"),
        Some(&Value::Str("boom".to_string()))
    );
}

#[test]
fn e2e_try_catch_success_skips_catch() {
    // If no error is thrown, the pakad block must be skipped.
    let src = concat!(
        "laadle result hai 0\n",
        "koshish toh\n",
        "    result hai 100\n",
        "pakad err toh\n",
        "    result hai 200\n",
    );
    let vm = run(src);
    assert_eq!(vm.globals.get("result"), Some(&Value::Int(100)));
    assert_eq!(vm.globals.get("err"), None);
}
