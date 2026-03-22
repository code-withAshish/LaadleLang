---
title: "Control Flow"
---

LaadleLang provides simple but powerful control flow mechanisms.

## If / Else

Conditional branching is handled with `agar` (if) and `warna` (else). Conditions are followed by the `toh` keyword to open the block. Blocks must be indented.

```laadle
laadle score hai 85

agar score >= 90 toh
    bol "A"
warna
    agar score >= 80 toh
        bol "B"
    warna
        bol "C"
```

## While Loops

Loops are created using `jabtak` (while). It repeats the block as long as the condition is truthy.

```laadle
laadle i hai 0
jabtak i < 5 toh
    bol i
    i hai i + 1
```

### Break and Continue
You can exit loops early or skip to the next iteration using `nikal` (break) and `aage` (continue).

These behave identically to `break` and `continue` in C, Python, or Rust. The code below demonstrates skipping even numbers and stopping when it reaches 7.

```laadle
laadle i hai 0
jabtak i < 10 toh
    i hai i + 1
    
    // Skip even numbers
    agar i % 2 == 0 toh
        aage
    
    // Stop at 7
    agar i == 7 toh
        nikal
        
    bol i
```
