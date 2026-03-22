//! LaadleLang — a Pythonic scripting language that compiles to a stack-based VM.
//!
//! Public module layout:
//! - [`tokenizer`] — source → tokens (handles indentation, INDENT/DEDENT)
//! - [`ast`]       — AST node types (`Stmt`, `Expr`)
//! - [`parser`]    — tokens → AST (recursive descent)
//! - [`compiler`]  — AST → bytecode (`Vec<OpCode>`)
//! - [`vm`]        — executes bytecode (`LaadleVirtualMachineV1`)

pub mod ast;
pub mod compiler;
pub mod parser;
pub mod tokenizer;
pub mod vm;

use crate::vm::LaadleVirtualMachineV1;
use wasm_bindgen::prelude::*;

#[wasm_bindgen]
pub fn run_laadle_code(source: &str) -> String {
    console_error_panic_hook::set_once();

    let mut lexer = crate::tokenizer::Tokenizer::new(source);
    let tokens = lexer.tokenize();

    let mut parser = crate::parser::Parser::new(tokens);
    let stmts = parser.parse();
    if let Some(err) = parser.error {
        return format!("Syntax Error: {}", err);
    }

    let mut compiler = crate::compiler::Compiler::new();
    let program = compiler.compile(&stmts);
    if let Some(err) = compiler.error {
        return format!("Compilation Error: {}", err);
    }

    let mut vm = LaadleVirtualMachineV1::new(program);
    vm.run();

    if let Some(err) = vm.error {
        vm.output
            .push_str(&format!("\n[VM Crash] Uncaught Error: {}", err));
    }

    vm.output
}
