---
title: "Example: Safe Calculator"
---

This example introduces LaadleLang's robust error-handling infrastructure. It uses the `gopgop` keyword to explicitly throw errors, and the `koshish / pakad` (try/catch) block to intercept and gracefully recover from them rather than crashing the virtual machine.

## The Code

```laadle
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
```

<br>

## What's Happening?

1. **The Throw (`gopgop`):** Inside `safe_divide`, checking `b == 0` intercepts math faults and actively throws a text literal instead of triggering a Rust core panic during VM execution.
2. **The Binding (`koshish`):** Our `run_math` function wraps the dangerous division inside a `koshish` block. Behind the scenes, the Compiler issues a silent `PushErrHandler` opcode right before the block starts, taking a snapshot of the Call Stack depth and memory registers.
3. **The Unwind (`pakad err toh`):** When the division code triggers the `gopgop`, execution immediately halts. The VM jumps all the way back across `CallFrames`, binds the string "CRITICAL: Attempted to divide by zero!" into the `err` variable, and then resumes executing the handler.
4. **Clean Exit:** Because the error is completely handled locally, `The VM is still alive and running safely!` simply prints at the end, and the program exits successfully.
