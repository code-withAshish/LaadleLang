---
title: "Quick Start"
---

## Prerequisites

To run LaadleLang, you need to have **Rust** and **Cargo** installed on your system.
If you don't have Rust installed, follow the instructions on [rustup.rs](https://rustup.rs).

## Compiling and Running

1. Clone or download the LaadleLang repository.
2. Open your terminal in the project directory.
3. Run the project using standard cargo commands:

```bash
cargo run
```

Currently, `cargo run` will execute the example program found in `src/main.rs`. You can inspect the `src/main.rs` file to see how the code is tokenized, parsed, compiled, and executed by the VM.

## Running the Tests

LaadleLang has a comprehensive test suite covering the Tokenizer, Parser, Compiler, and Virtual Machine via both Unit and End-to-End (E2E) tests.

Run the entire suite using:

```bash
cargo test
```

## Creating Documentation

This documentation is built using [mdBook](https://rust-lang.github.io/mdBook/).

To build and serve this documentation locally:

```bash
cargo install mdbook
mdbook serve docs --open
```
