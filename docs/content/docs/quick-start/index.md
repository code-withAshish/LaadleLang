---
title: "Quick Start"
---

## Prerequisites

To run LaadleLang or build it from source, you need to have **Rust**, **Cargo**, and **Node.js** installed on your system.

## Compiling from Source

Since LaadleLang embeds a high-fidelity WebAssembly-powered Native IDE interface, the build process has been completely automated via `build.rs`.

1. Clone or download the LaadleLang repository.
2. Open your terminal in the project directory.
3. Run the project using the standard cargo release command:

```bash
cargo build --release
```

This single command natively compiles the WASM core via `wasm-pack`, bundles the Vanilla JS playground output directly into the Next.js static asset registry, and links everything securely into a self-contained 3.4MB native Rust binary.

## Running the Interactive CLI

Once built (or downloaded from our pre-compiled GitHub Releases page), the unified executable acts as the entrypoint for everything:

```bash
# Start the interactive REPL and Menu
./target/release/laadlelang

# Launch the locally embedded Playground IDE (starts internal tiny_http server)
./target/release/laadlelang playground

# Execute a source file directly
./target/release/laadlelang run path/to/script.laadle
```

## Running the Documentation Site Locally

This entire documentation site is built using [Fumadocs](https://fumadocs.vercel.app/) and Next.js.

To run the documentation locally:

```bash
cd docs
npm ci
npm run dev
```

The docs will be available at `http://localhost:3000`.
