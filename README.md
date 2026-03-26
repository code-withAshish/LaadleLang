# 🐱 LaadleLang

**LaadleLang** is a Pythonic, indentation-based scripting language built in Rust. It compiles to a custom, deterministic, stack-based Virtual Machine (VM) and can run natively or in a browser via WebAssembly.

![LaadleLang Playground](https://img.shields.io/badge/Status-Beta-brightgreen)
![Rust](https://img.shields.io/badge/Made%20with-Rust-orange)
![WASM](https://img.shields.io/badge/Runs%20on-WebAssembly-blueviolet)

---

## ✨ Features

- **Pythonic Syntax**: Indentation-based blocks (`Indent` / `Dedent`) and logical newlines. No semicolons or curly braces.
- **Graceful Error Handling**: Integrated `koshish`...`pakad` (try-catch) blocks that prevent native VM crashes and allow for robust error recovery.
- **Custom VM**: A stack-based bytecode executor with support for global and local variables, call frames, and nested functions.
- **WASM Playground**: A full-screen IDE powered by an ultra-lightweight PrismJS overlay, providing a VSCode-like experience entirely in the browser.
- **Embedded Web Server**: The CLI includes a zero-dependency internal server to host the playground instantly.
- **Self-Contained Binary**: A portable Rust executable that bundles all assets, documentation, and the IDE.
- **Interactive REPL**: Native shell with persistent state and command history.

---

## 🚀 Quick Example

```python
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

---

## 🛠️ Installation & Build

Since the entire frontend and WASM build pipeline is integrated seamlessly into Cargo via `build.rs`, compiling from source is incredibly simple:

### 1. Prerequisites
- [Rust](https://rustup.rs/) (edition 2024+)
- [Node.js](https://nodejs.org/) (v20+)
- `wasm-pack` (`cargo install wasm-pack`)

### 2. Build from Source
```bash
# 1. Clone the repository
git clone https://github.com/code-withAshish/LaadleLang.git
cd LaadleLang

# 2. Automate WASM building, JS bundling, and Rust compilation
cargo build --release

# 3. Run the interactive CLI
./target/release/laadlelang
```

### 💻 CLI Usage

The binary provides a unified interface for all LaadleLang tools:

```bash
# Start the interactive menu
./target/release/laadlelang

# Launch the embedded Playground (starts internal server)
./target/release/laadlelang playground

# Start the REPL directly
./target/release/laadlelang repl

# Execute a source file
./target/release/laadlelang run path/to/script.laadle
```

> [!TIP]
> The `playground` command starts an internal web server on `localhost:3000`. You don't need `npm` or `node` once the binary is built!

---

## 🏗️ Architecture

LaadleLang follows a classic compiler pipeline:

1.  **Tokenizer (`src/tokenizer.rs`)**: Converts source text into a stream of tokens, handling Python-style indentation.
2.  **Parser (`src/parser.rs`)**: A recursive-descent parser that builds an Abstract Syntax Tree (AST).
3.  **Compiler (`src/compiler.rs`)**: Walks the AST and emits flat, stack-based bytecode (`OpCode`).
4.  **Virtual Machine (`src/vm.rs`)**: Executes the bytecode on a custom stack with safe state management.

For a deeper dive into the VM internals, see [vm_design.md](./vm_design.md).

---

## 📄 License

This project is licensed under the MIT License - see the LICENSE file for details.
