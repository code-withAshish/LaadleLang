# MeowLang VM Design — [LaadleVirtualMachineV1](file:///home/ashish/development/meowlang/src/vm.rs#11-16)

> This is a **design / learning document**, not the implementation.  
> It explains the *why* behind every opcode so you can implement them yourself.

---

## 1. What a Stack-Based VM Is

Your current VM is a **stack machine**: every value lives on a stack, and every
operation pops inputs from the stack and pushes its result back.

```
OpCode::Push(5)  →  stack: [5]
OpCode::Push(3)  →  stack: [5, 3]
OpCode::Add      →  pops 3 and 5, pushes 8  →  stack: [8]
```

Three pointers/registers drive execution:

| Name | Type | Role |
|------|------|------|
| `ip` (instruction pointer) | `usize` | Index of the current instruction in [program](file:///home/ashish/development/meowlang/src/vm.rs#65-69) |
| `stack` | `Vec<Value>` | Operand stack – main workspace |
| `call_stack` | `Vec<CallFrame>` | Tracks function calls (for `Kaam`/`Wapas`) |

---

## 2. The `Value` Type (upgrade from `i32`)

Right now your stack holds `i32`. To support **strings, booleans, null, and
errors** you need a richer type:

```rust
pub enum Value {
    Int(i32),
    Float(f64),
    Bool(bool),
    Str(String),
    Null,          // the "meow" void / no-value
    Error(String), // for GopGop
}
```

> **Your task later:** Define this enum. Every opcode operates on `Value` instead
> of `i32`.

---

## 3. VM State (full struct)

```
LaadleVirtualMachineV1
├── program:    Vec<OpCode>          — the flat list of instructions
├── ip:         usize                — instruction pointer
├── stack:      Vec<Value>           — operand stack
├── globals:    HashMap<String,Val>  — global variables (Laadle at top scope)
├── call_stack: Vec<CallFrame>       — one frame per active function call
└── error:      Option<Value>        — last thrown error (GopGop)
```

`CallFrame` (one per function call):
```
CallFrame
├── return_ip:  usize                — where to jump back after Wapas
├── locals:     HashMap<String,Val>  — local variables of this call
└── stack_base: usize                — stack depth at entry (for cleanup)
```

---

## 4. Complete Opcode Table

### 4a. Stack / Literals

| OpCode | Stack effect | What it does |
|--------|-------------|--------------|
| `Push(Value)` | `→ v` | Push a constant value |
| `Pop` | `v →` | Discard the top of stack |
| `Dup` | `v → v v` | Duplicate top of stack |
| `Swap` | `a b → b a` | Swap top two values |

---

### 4b. Arithmetic & Logic

| OpCode | Stack effect | Maps to token |
|--------|-------------|---------------|
| `Add` | `a b → a+b` | `Plus` |
| `Sub` | `a b → a-b` | `Minus` |
| `Mul` | `a b → a*b` | `Star` |
| `Div` | `a b → a/b` | `Slash` |
| `Neg` | `a → -a` | unary minus |
| `Not` | `a → !a` | logical not |
| `And` | `a b → a&&b` | `And` |
| `Or`  | `a b → a\|\|b` | `Or` |

---

### 4c. Comparison

| OpCode | Stack effect | Maps to token |
|--------|-------------|---------------|
| `Eq`  | `a b → a==b` | `EqualEqual` |
| `Neq` | `a b → a!=b` | `NotEqual` |
| `Gt`  | `a b → a>b`  | `Greater` |
| `Lt`  | `a b → a<b`  | `Less` |
| `Gte` | `a b → a>=b` | `GreaterEq` |
| `Lte` | `a b → a<=b` | `LessEq` |

---

### 4d. Variables — [Laadle](file:///home/ashish/development/meowlang/src/vm.rs#11-16) / `Hai`

These opcodes let the VM **store and recall named values**.

| OpCode | Stack effect | What it does |
|--------|-------------|--------------|
| `SetGlobal(name)` | `v →` | Pop `v`, store as `globals[name]` |
| `GetGlobal(name)` | `→ v` | Push `globals[name]` onto stack |
| `SetLocal(name)`  | `v →` | Pop `v`, store in current `CallFrame.locals` |
| `GetLocal(name)`  | `→ v` | Push from current `CallFrame.locals` |

**How `laadle x hai 5` compiles:**
```
Push(Int(5))
SetGlobal("x")
```

**How `bol x` compiles:**
```
GetGlobal("x")
Print
```

---

### 4e. Control Flow — `Agar` / `Warna` / `Jabtak`

Control flow is implemented with **jumps** — opcodes that change `ip`.

| OpCode | Stack effect | What it does |
|--------|-------------|--------------|
| `Jump(offset)` | — | Unconditionally set `ip = offset` |
| `JumpIfFalse(offset)` | `cond →` | Pop condition; if falsy, jump to `offset` |
| `JumpIfTrue(offset)`  | `cond →` | Pop condition; if truthy, jump to `offset` |

**How `agar (cond) toh ... warna ... bas` compiles:**

```
<emit condition>         ; leaves Bool on stack
JumpIfFalse(else_start)  ; if false, skip then-block
<then block>
Jump(end)                ; skip else-block
<else block>             ; ← else_start label
                         ; ← end label
```

**How `jabtak (cond) ... bas` (while loop) compiles:**

```
                          ; ← loop_start label
<emit condition>
JumpIfFalse(loop_end)
<loop body>
Jump(loop_start)          ; go back to condition
                          ; ← loop_end label
```

> **Key insight:** `Jump` and `JumpIfFalse` are all you need for `if/else`
> *and* `while`. The compiler fills in the exact offset numbers when it emits
> the bytecode.

---

### 4f. Loop Control — `Nikal` / `Aage`

| OpCode | Stack effect | What it does |
|--------|-------------|--------------|
| `Break` | — | Jump to after the current loop (`loop_end`) |
| `Continue` | — | Jump back to loop condition (`loop_start`) |

The compiler needs to track `loop_start` and `loop_end` offsets while emitting
the loop body and **patch** the `Break`/`Continue` targets.

---

### 4g. Functions — `Kaam` / `Wapas`

Functions in a bytecode VM are just **labeled positions** in the program.

| OpCode | Stack effect | What it does |
|--------|-------------|--------------|
| `Call(name, argc)` | `arg1..argN →` | Push a new `CallFrame`; jump to function start |
| `Return` | `retval →` (inner) `→ retval` (caller) | Pop `CallFrame`; restore `ip`; push return value |
| `MakeFunction(name, addr, params)` | — | Register `globals[name]` as a callable at `addr` |

**How a function call [add(3, 7)](file:///home/ashish/development/meowlang/src/vm.rs#75-94) compiles:**
```
Push(Int(3))     ; arg 1
Push(Int(7))     ; arg 2
Call("add", 2)   ; takes 2 args from stack, pushes CallFrame
```

**How `wapas x` compiles:**
```
GetLocal("x")  ; push return value
Return         ; pop frame, jump back, leave value on stack
```

---

### 4h. Output — `Bol`

| OpCode | Stack effect | What it does |
|--------|-------------|--------------|
| `Print` | `v →` | Pop and print the top value |
| `PrintNoNl` | `v →` | Same but no newline (optional) |

---

### 4i. Error Handling — `GopGop`

| OpCode | Stack effect | What it does |
|--------|-------------|--------------|
| `Throw` | `v →` | Pop value, store in `vm.error`, unwind to nearest catch |
| `PushErrHandler(offset)` | — | Register a handler at `offset` |
| `PopErrHandler` | — | Remove the innermost handler |

**How `gopgop "something went wrong"` compiles:**
```
Push(Str("something went wrong"))
Throw
```

**Error propagation** works by walking up the `call_stack` until a frame with
a registered error handler is found. If none exists, the VM halts with an
unhandled error message.

---

### 4j. Miscellaneous

| OpCode | Stack effect | What it does |
|--------|-------------|--------------|
| `Noop` | — | `Meow` — does nothing, skips one cycle |
| `Halt` | — | Stop the VM |

---

## 5. How the Pieces Fit Together (Pipeline)

```
Source code (MeowLang)
       ↓
   Tokenizer          →  Vec<Token>   (already started in tokenizer.rs)
       ↓
    Parser            →  AST          (ast.rs / parser.rs)
       ↓
   Compiler           →  Vec<OpCode>  (not yet created)
       ↓
      VM              →  runs it      (vm.rs)
```

The **compiler** is the new piece. It walks the AST and emits opcodes, filling
in jump offsets. This is where `agar`→`JumpIfFalse`, `jabtak`→`Jump` etc. get
wired up.

---

## 6. What To Implement Next (Suggested Order)

1. **Define `Value` enum** — upgrade the stack from `i32` to `Value`
2. **Add variable opcodes** (`SetGlobal`, `GetGlobal`) + `globals` HashMap in VM
3. **Add comparison opcodes** (`Eq`, `Gt`, etc.)
4. **Add jump opcodes** (`Jump`, `JumpIfFalse`) — test with hand-written bytecode
5. **Add `Print` for strings/booleans** — update match arm
6. **Add `CallFrame` + `call_stack`** + `Call`/`Return` opcodes
7. **Add `Throw` + error handling**
8. **Write the Compiler** that connects the AST → opcodes

> We can implement these **one step at a time** together. Start with step 1 and
> let me know when you're ready!
