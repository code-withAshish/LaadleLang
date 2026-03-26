---
title: "Quick Start"
---

LaadleLang is designed to be extremely easy to build and run.

### 1. Prerequisites
- [Rust](https://rustup.rs/) (edition 2024+)
- [Node.js](https://nodejs.org/) (v20+)
- `wasm-pack` (`cargo install wasm-pack`)

### 2. Build from Source
```bash
# Clone the repository
git clone https://github.com/code-withAshish/LaadleLang.git
cd LaadleLang

# Build everything (WASM, JS, and Rust)
cargo build --release
```

### 3. Run the CLI
Once built, the unified executable acts as the entrypoint for everything:

```bash
# Start the interactive REPL and Menu
./target/release/laadlelang

# Launch the locally embedded Playground IDE
./target/release/laadlelang playground

# Execute a source file directly
./target/release/laadlelang run path/to/script.laadle
```

### 4. Running Docs Locally (Optional)
This documentation site is built with Fumadocs.

```bash
cd docs
npm ci
npm run dev
```
