---
title: "Example: Iterative Factorial"
---

This example highlights LaadleLang's `jabtak` (while) loops, variable mutation, and conditional breaking (`bas` and `chal`) inside an iterative sequence.

## The Code

```laadle
// A function that calculates a factorial iteratively without recursion
kaam calculate_factorial(n) toh
    agar n < 0 toh
        bol "Factorials are only for non-negative numbers!"
        wapas
    
    laadle result hai 1
    laadle current hai n
    
    jabtak current > 1 toh
        result hai result * current
        current hai current - 1
        
    wapas result

// Output the sequence 
laadle num hai 5
bol "Factorial of " + num + " is:"
bol calculate_factorial(num)
```

<br>

## What's Happening?

1. **Variables & Reassignment:** We declare variables using `laadle`. Inside the `jabtak` loop, we simply omit the `laadle` keyword to *mutate* the existing `result` and `current` variables rather than declaring new ones.
2. **Loop Condition:** The `jabtak current > 1 toh` loop continually runs its nested block until the condition becomes falsy.
3. **Execution Guarding:** The `agar n < 0 toh` code block acts as a safety barrier. If triggered, it prints a warning and invokes a bare `wapas` to break out directly without iterating or causing an infinite loop.
