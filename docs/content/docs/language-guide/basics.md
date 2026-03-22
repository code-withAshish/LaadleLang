---
title: "Language Basics"
---

LaadleLang uses a simple and intuitive syntax based on indentation, completely stripping away semi-colons and curly braces in favor of block structures.

## Variables

Variables are declared using the `laadle` keyword and assigned using the `hai` keyword (which acts as `=`). Since LaadleLang is dynamically typed, a single variable can be re-assigned differently over its lifetime.

```laadle
// Declaring variables
laadle x hai 10
laadle name hai "Alice"
laadle is_ready hai sahi

// Re-assignment (does not need 'laadle')
x hai x + 5
```

## Data Types

LaadleLang supports several native data types directly in the runtime Virtual Machine:

- **Integers**: `10`, `-5`
- **Floats**: `3.14`, `0.5`
- **Strings**: `"Hello World"`
- **Booleans**: `sahi` (true), `galat` (false)
- **Null**: `meow` (represents no value, void, or a null pointer)

## Operators

Standard operators are available for arithmetic and logic.

### Arithmetic
- Addition: `+`
- Subtraction: `-`
- Multiplication: `*`
- Division: `/`

*Note: Adding an integer and a float will automatically widen the integer to a float.*

### Comparison
- Equal: `==`
- Not Equal: `!=`
- Greater: `>`
- Less: `<`
- Greater/Equal: `>=`
- Less/Equal: `<=`

### Logical
- AND: `&&` (short-circuited: if the left is false, the right is never evaluated)
- OR: `||` (short-circuited: if the left is true, the right is never evaluated)
- NOT: `!`

## Printing to Console
To print a value to stdout, use the `bol` keyword:

```laadle
laadle greeting hai "Welcome!"
bol greeting
bol 10 + 20
```
