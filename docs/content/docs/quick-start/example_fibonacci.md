---
title: "Example: The Fibonacci Sequence"
---

This example demonstrates how to implement a recursive mathematical algorithm in LaadleLang. It highlights the use of the `kaam` keyword for defining functions, `agar` for base-case conditional logic, and isolated `CallFrame` recursion semantics.

## The Code

```laadle
// A recursive function to calculate the N-th Fibonacci number
kaam fibonacci(n) toh
    // Base cases: fib(0) = 0, fib(1) = 1
    agar n <= 1 toh
        wapas n
    
    // Recursive case
    laadle result hai fibonacci(n - 1) + fibonacci(n - 2)
    wapas result

// Calculate and print the 10th Fibonacci number
laadle term hai 10
bol "The " + term + "th Fibonacci number is:"
bol fibonacci(term)
```

<br>

## What's Happening?

1. **Function Definition:** We define the `fibonacci` function that accepts exactly one argument `n`.
2. **Conditional Base Case:** Using `agar n <= 1 toh`, we branch out and immediately return `n` if it's `0` or `1`. Note how LaadleLang safely performs runtime coercion if necessary without needing explicit type casting!
3. **Recursive Calls:** We recursively call `fibonacci(n - 1)` and `fibonacci(n - 2)`. Behind the scenes, the Virtual Machine allocates pristine, completely isolated `CallFrames` on the Call Stack for every single recursive invocation, completely eliminating variable crossover.
4. **Returning:** We cleanly push the math result back to the caller using `wapas`.
