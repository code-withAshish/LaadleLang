use laadlelang::{compiler::compile_source, vm::LaadleVirtualMachineV1};

// ── The example LaadleLang program ───────────────────────────────────────────
const PROGRAM: &str = r#"
// A dangerous math operation that will throw an error if abused
kaam safe_divide(a, b) toh
    agar b == 0 toh
        gopgop "CRITICAL: Attempted to divide by zero!"
    wapas a / b

// Outer runner function implementing try/catch handler
kaam run_math() toh
    laadle result hai 0
    koshish toh
        // Success case
        bol "10 / 2 = " + safe_divide(10, 2)
        
        // This line fails and forces VM to jump immediately
        result hai safe_divide(10, 0)
        
        // The panic jumped over this line, so it never prints
        bol "You will never see me!"
        
    pakad err toh
        bol "Math error caught successfully!"
        bol err 

    bol "The VM is still alive and running safely!"

run_math()
"#;

fn main() {
    // Compile the source to opcodes
    let opcodes = compile_source(PROGRAM);
    let mut vm = LaadleVirtualMachineV1::new(opcodes);
    vm.run();
}
