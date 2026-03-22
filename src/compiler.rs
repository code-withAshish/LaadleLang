// ─────────────────────────────────────────────────────────────────────────────
// COMPILER — LaadleLang
//
// Input:  Vec<Stmt>   (AST produced by Parser)
// Output: Vec<OpCode> (bytecode consumed by LaadleVirtualMachineV1)
//
// Key concepts:
//
// 1. EMIT — append an OpCode to the output list.
//    The list index of each emitted opcode is its "address" (used by jumps).
//
// 2. BACKPATCHING — when compiling an `agar` or `jabtak`, we don't know the
//    jump target yet (the else-block / loop-end hasn't been emitted).
//    So we emit a placeholder Jump(0), record its index, and once we know the
//    real target we go back and overwrite the placeholder. That's backpatching.
//
// 3. LOOP CONTEXT — while inside a loop body we keep a stack of
//    `LoopContext` entries. Every `nikal` (Break) and `aage`
//    (Continue) emits a placeholder jump and registers its index in the
//    current `LoopContext`. When the loop ends we patch all of them at once.
//
// 4. SCOPE — variables declared with `laadle` inside a function body use
//    SetLocal/GetLocal; at the top level they use SetGlobal/GetGlobal.
//    We track this with a simple `scope_depth` counter.
// ─────────────────────────────────────────────────────────────────────────────

use crate::ast::{BinaryOp, Expr, Stmt, UnaryOp};
use crate::vm::{OpCode, Value};

// ─── LOOP CONTEXT ────────────────────────────────────────────────────────────
// Tracks the addresses of Break / Continue placeholder jumps that need to be
// patched once the enclosing loop's boundaries are known.

struct LoopContext {
    /// Indices of Break placeholder instructions — patched to loop_end.
    break_patches: Vec<usize>,
    /// Indices of Continue placeholder instructions — patched to loop_start.
    continue_patches: Vec<usize>,
}

// ─── COMPILER ────────────────────────────────────────────────────────────────

pub struct Compiler {
    /// The bytecode we are building up.
    code: Vec<OpCode>,

    /// Stack of loop contexts (one per nested loop).
    loop_stack: Vec<LoopContext>,
    /// 0 = top level (use globals), >0 = inside a function (use locals).
    scope_depth: usize,
    pub error: Option<String>,
}

impl Default for Compiler {
    fn default() -> Self {
        Self::new()
    }
}

impl Compiler {
    pub fn new() -> Self {
        Self {
            code: Vec::new(),
            loop_stack: Vec::new(),
            scope_depth: 0,
            error: None,
        }
    }

    fn report_error(&mut self, msg: String) {
        if self.error.is_none() {
            self.error = Some(msg);
        }
    }

    // ── Entry point ───────────────────────────────────────────────────────────

    /// Compile a full program (list of top-level statements) into bytecode.
    pub fn compile(&mut self, stmts: &[Stmt]) -> Vec<OpCode> {
        for stmt in stmts {
            self.compile_stmt(stmt);
            if self.error.is_some() {
                return Vec::new(); // Stop compilation on first error
            }
        }
        self.emit(OpCode::Halt);
        self.code.clone()
    }

    // ── Helpers ───────────────────────────────────────────────────────────────

    /// Append one opcode and return its index (its "address").
    fn emit(&mut self, op: OpCode) -> usize {
        self.code.push(op);
        self.code.len() - 1
    }

    /// Overwrite the opcode at `idx` with a new one (used for backpatching).
    fn patch(&mut self, idx: usize, op: OpCode) {
        self.code[idx] = op;
    }

    /// Current instruction count — the next opcode will get this address.
    fn current_addr(&self) -> usize {
        self.code.len()
    }

    /// Emit a variable setter (SetGlobal at top level, SetLocal inside a fn).
    fn emit_set(&mut self, name: String) {
        if self.scope_depth == 0 {
            self.emit(OpCode::SetGlobal(name));
        } else {
            self.emit(OpCode::SetLocal(name));
        }
    }

    /// Emit a variable getter (GetGlobal at top level, GetLocal inside a fn).
    fn emit_get(&mut self, name: String) {
        if self.scope_depth == 0 {
            self.emit(OpCode::GetGlobal(name));
        } else {
            self.emit(OpCode::GetLocal(name));
        }
    }

    // ── Statement compilation ─────────────────────────────────────────────────

    fn compile_stmt(&mut self, stmt: &Stmt) {
        match stmt {
            // ── laadle x hai <expr>  ─────────────────────────────────────────
            // 1. Compile the expression  → pushes value onto stack
            // 2. SetGlobal/SetLocal      → pops it into the variable table
            Stmt::VarDecl { name, value } => {
                self.compile_expr(value);
                self.emit_set(name.clone());
            }

            // ── x hai <expr>  ────────────────────────────────────────────────
            // Same as VarDecl but for re-assignment (no laadle keyword).
            Stmt::Assign { name, value } => {
                self.compile_expr(value);
                self.emit_set(name.clone());
            }

            // ── bol <expr>  ──────────────────────────────────────────────────
            Stmt::Print(expr) => {
                self.compile_expr(expr);
                self.emit(OpCode::Print);
            }

            // ── agar <cond> toh ... (warna ...)  ─────────────────────────────
            //
            // Emitted bytecode layout:
            //
            //   <condition>
            //   JumpIfFalse(else_start)   ← patched after then-block is emitted
            //   <then_branch>
            //   Jump(end)                  ← patched after else-block is emitted
            //   <else_branch>              ← else_start points here
            //   ...                        ← end points here
            Stmt::If {
                condition,
                then_branch,
                else_branch,
            } => {
                // Compile condition — leaves Bool on stack
                self.compile_expr(condition);

                // Placeholder: jump over then-block if condition is false
                let jif_idx = self.emit(OpCode::JumpIfFalse(0));

                // Compile then-block
                for s in then_branch {
                    self.compile_stmt(s);
                    if self.error.is_some() {
                        return;
                    }
                }

                // Placeholder: jump over else-block (always taken from then-block)
                let jump_idx = self.emit(OpCode::Jump(0));

                // Patch JumpIfFalse → start of else (or end if no else)
                let else_start = self.current_addr();
                self.patch(jif_idx, OpCode::JumpIfFalse(else_start));

                // Compile optional else-block
                if let Some(else_stmts) = else_branch {
                    for s in else_stmts {
                        self.compile_stmt(s);
                        if self.error.is_some() {
                            return;
                        }
                    }
                }

                // Patch Jump → after everything
                let end = self.current_addr();
                self.patch(jump_idx, OpCode::Jump(end));
            }

            // ── jabtak <cond> toh ...  ────────────────────────────────────────
            //
            // Emitted bytecode layout:
            //
            //   loop_start:               ← Continue jumps here
            //   <condition>
            //   JumpIfFalse(loop_end)
            //   <body>
            //   Jump(loop_start)
            //   loop_end:                 ← Break jumps here
            Stmt::While { condition, body } => {
                let loop_start = self.current_addr();

                // Push a loop context so Break/Continue can register patches
                self.loop_stack.push(LoopContext {
                    break_patches: Vec::new(),
                    continue_patches: Vec::new(),
                });

                // Compile condition
                self.compile_expr(condition);

                // Placeholder exit jump (patched to loop_end below)
                let jif_idx = self.emit(OpCode::JumpIfFalse(0));

                // Compile body
                for s in body {
                    self.compile_stmt(s);
                    if self.error.is_some() {
                        return;
                    }
                }

                // Jump back to condition
                self.emit(OpCode::Jump(loop_start));

                let loop_end = self.current_addr();

                // Patch the condition exit jump
                self.patch(jif_idx, OpCode::JumpIfFalse(loop_end));

                // Pop loop context and patch all Break / Continue holes
                let ctx = self.loop_stack.pop().unwrap();
                for idx in ctx.break_patches {
                    self.patch(idx, OpCode::Jump(loop_end));
                }
                for idx in ctx.continue_patches {
                    self.patch(idx, OpCode::Jump(loop_start));
                }
            }

            // ── nikal  ─────────────────────────────────────────────────────
            // Emit a Jump(0) placeholder and register it in the innermost loop.
            Stmt::Break => {
                let idx = self.emit(OpCode::Jump(0));
                match self.loop_stack.last_mut() {
                    Some(ctx) => ctx.break_patches.push(idx),
                    None => self.report_error(
                        "SyntaxError: `nikal` (break) used outside of a loop".to_string(),
                    ),
                }
            }

            // ── aage  ────────────────────────────────────────────────────
            // Emit a Jump(0) placeholder and register it as a Continue patch.
            Stmt::Continue => {
                let idx = self.emit(OpCode::Jump(0));
                match self.loop_stack.last_mut() {
                    Some(ctx) => ctx.continue_patches.push(idx),
                    None => self.report_error(
                        "SyntaxError: `aage` (continue) used outside of a loop".to_string(),
                    ),
                }
            }

            // ── kaam <name>(<params>) toh ...  ───────────────────────────────
            //
            // Emitted bytecode layout:
            //
            //   Jump(fn_end)         ← skip the function body during top-level execution
            //   fn_start:            ← MakeFunction records this address
            //   <body>
            //   Return               ← implicit return at end of body (pushes Null)
            //   fn_end:
            //   MakeFunction { name, addr: fn_start, params }
            //
            // Why Jump first? Because the VM executes linearly — we must skip
            // the function body when it's *defined*, and only run it when *called*.
            Stmt::FnDecl { name, params, body } => {
                // Emit a jump to skip the body (patched below)
                let skip_idx = self.emit(OpCode::Jump(0));

                let fn_start = self.current_addr();

                // Compile body in a local scope
                self.scope_depth += 1;
                for s in body {
                    self.compile_stmt(s);
                    if self.error.is_some() {
                        return;
                    }
                }
                // Implicit return Null if body didn't already return
                self.emit(OpCode::Push(Value::Null));
                self.emit(OpCode::Return);
                self.scope_depth -= 1;

                let fn_end = self.current_addr();

                // Patch the skip jump
                self.patch(skip_idx, OpCode::Jump(fn_end));

                // Register the function in the globals table
                self.emit(OpCode::MakeFunction {
                    name: name.clone(),
                    addr: fn_start,
                    params: params.clone(),
                });
            }

            // ── wapas <expr>  ────────────────────────────────────────────
            Stmt::Return(expr) => {
                self.compile_expr(expr);
                self.emit(OpCode::Return);
            }

            // ── gopgop <expr>  ────────────────────────────────────────────────
            Stmt::Throw(expr) => {
                self.compile_expr(expr);
                self.emit(OpCode::Throw);
            }

            // ── koshish toh ... pakad x toh ... ───────────────────────────────
            // Emit:
            //   PushErrHandler(catch_addr)
            //   <body>
            //   PopErrHandler
            //   Jump(end_addr)
            // catch_addr:
            //   SetGlobal/SetLocal(catch_var)   ← (VM pushes the error value here)
            //   <handler>
            // end_addr:
            Stmt::TryCatch {
                body,
                catch_var,
                handler,
            } => {
                let peh_idx = self.emit(OpCode::PushErrHandler(0)); // placeholder

                // 1. Compile `koshish` body
                for s in body {
                    self.compile_stmt(s);
                    if self.error.is_some() {
                        return;
                    }
                }

                // 2. If body succeeds, pop handler and skip catch block
                self.emit(OpCode::PopErrHandler);
                let jump_end_idx = self.emit(OpCode::Jump(0)); // placeholder

                // 3. Catch handler start address (patch the PushErrHandler)
                let catch_addr = self.current_addr();
                self.patch(peh_idx, OpCode::PushErrHandler(catch_addr));

                // 4. The VM will jump here on error, with the error value on the stack.
                // Store it in `catch_var`.
                self.emit_set(catch_var.clone());

                // 5. Compile the `pakad` block
                for s in handler {
                    self.compile_stmt(s);
                    if self.error.is_some() {
                        return;
                    }
                }

                // 6. End address (patch the success skip jump)
                let end_addr = self.current_addr();
                self.patch(jump_end_idx, OpCode::Jump(end_addr));
            }

            // ── bare expression statement  ────────────────────────────────────
            // The expression leaves a value on the stack; we Pop it because
            // the caller doesn't use it.
            Stmt::ExprStmt(expr) => {
                self.compile_expr(expr);
                self.emit(OpCode::Pop);
            }
        }
    }

    // ── Expression compilation ────────────────────────────────────────────────

    fn compile_expr(&mut self, expr: &Expr) {
        match expr {
            // ── Literals → Push a constant Value ─────────────────────────────
            Expr::Int(n) => {
                self.emit(OpCode::Push(Value::Int(*n)));
            }
            Expr::Float(f) => {
                self.emit(OpCode::Push(Value::Float(*f)));
            }
            Expr::Sahi => {
                self.emit(OpCode::Push(Value::Bool(true)));
            }
            Expr::Galat => {
                self.emit(OpCode::Push(Value::Bool(false)));
            }
            Expr::Str(s) => {
                self.emit(OpCode::Push(Value::Str(s.clone())));
            }
            Expr::Null => {
                self.emit(OpCode::Push(Value::Null));
            }

            // ── Variable reference → GetGlobal or GetLocal ────────────────────
            Expr::Variable(name) => {
                self.emit_get(name.clone());
            }

            // ── Unary  ────────────────────────────────────────────────────────
            // 1. Compile operand (pushes value)
            // 2. Emit the unary opcode (pops, transforms, pushes result)
            Expr::Unary { op, operand } => {
                self.compile_expr(operand);
                match op {
                    UnaryOp::Neg => {
                        self.emit(OpCode::Neg);
                    }
                    UnaryOp::Not => {
                        self.emit(OpCode::Not);
                    }
                }
            }

            // ── Binary  ───────────────────────────────────────────────────────
            // And / Or use short-circuit evaluation (right side may not run).
            // All other ops compile both sides eagerly then emit one opcode.
            Expr::Binary { left, op, right } => {
                match op {
                    // ── a && b  ───────────────────────────────────────────────
                    // Bytecode:
                    //   <left>
                    //   JumpIfFalse(false_addr)   ; short-circuit: pop left, skip right
                    //   <right>
                    //   Not; Not;                  ; normalise right to Bool
                    //   Jump(end)
                    //   false_addr: Push(false)
                    //   end:
                    BinaryOp::And => {
                        self.compile_expr(left);
                        let jif_idx = self.emit(OpCode::JumpIfFalse(0));
                        self.compile_expr(right);
                        self.emit(OpCode::Not);
                        self.emit(OpCode::Not); // → Bool
                        let jump_end_idx = self.emit(OpCode::Jump(0));
                        let false_addr = self.current_addr();
                        self.emit(OpCode::Push(Value::Bool(false)));
                        let end_addr = self.current_addr();
                        self.patch(jif_idx, OpCode::JumpIfFalse(false_addr));
                        self.patch(jump_end_idx, OpCode::Jump(end_addr));
                    }

                    // ── a || b  ───────────────────────────────────────────────
                    // Bytecode:
                    //   <left>
                    //   JumpIfTrue(true_addr)    ; short-circuit: pop left, skip right
                    //   <right>
                    //   Not; Not;                 ; normalise right to Bool
                    //   Jump(end)
                    //   true_addr: Push(true)
                    //   end:
                    BinaryOp::Or => {
                        self.compile_expr(left);
                        let jit_idx = self.emit(OpCode::JumpIfTrue(0));
                        self.compile_expr(right);
                        self.emit(OpCode::Not);
                        self.emit(OpCode::Not); // → Bool
                        let jump_end_idx = self.emit(OpCode::Jump(0));
                        let true_addr = self.current_addr();
                        self.emit(OpCode::Push(Value::Bool(true)));
                        let end_addr = self.current_addr();
                        self.patch(jit_idx, OpCode::JumpIfTrue(true_addr));
                        self.patch(jump_end_idx, OpCode::Jump(end_addr));
                    }

                    // ── All other binary ops: compile both sides then opcode ──
                    _ => {
                        self.compile_expr(left);
                        self.compile_expr(right);
                        match op {
                            BinaryOp::Add => {
                                self.emit(OpCode::Add);
                            }
                            BinaryOp::Sub => {
                                self.emit(OpCode::Sub);
                            }
                            BinaryOp::Mul => {
                                self.emit(OpCode::Mul);
                            }
                            BinaryOp::Div => {
                                self.emit(OpCode::Div);
                            }
                            BinaryOp::Eq => {
                                self.emit(OpCode::Eq);
                            }
                            BinaryOp::Neq => {
                                self.emit(OpCode::Neq);
                            }
                            BinaryOp::Gt => {
                                self.emit(OpCode::Gt);
                            }
                            BinaryOp::Lt => {
                                self.emit(OpCode::Lt);
                            }
                            BinaryOp::Gte => {
                                self.emit(OpCode::Gte);
                            }
                            BinaryOp::Lte => {
                                self.emit(OpCode::Lte);
                            }
                            BinaryOp::And | BinaryOp::Or => unreachable!(),
                        }
                    }
                }
            }

            // ── Function call  ────────────────────────────────────────────────
            // 1. Compile each argument in order (each pushes one value)
            // 2. Emit Call { name, argc } — the VM pops argc args and jumps
            Expr::Call { name, args } => {
                for arg in args {
                    self.compile_expr(arg);
                }
                self.emit(OpCode::Call {
                    name: name.clone(),
                    argc: args.len(),
                });
            }
        }
    }
}

// ─── CONVENIENCE ─────────────────────────────────────────────────────────────

/// Compile a source string end-to-end: tokenize → parse → compile.
pub fn compile_source(source: &str) -> Vec<OpCode> {
    use crate::parser::parse_source;
    let ast = parse_source(source);
    Compiler::new().compile(&ast)
}

// ─── TESTS ───────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    fn compile(src: &str) -> Vec<OpCode> {
        compile_source(src)
    }

    /// Helper: collect the variant names of the opcodes for easy assertions.
    fn op_names(ops: &[OpCode]) -> Vec<&str> {
        ops.iter()
            .map(|op| match op {
                OpCode::Push(_) => "Push",
                OpCode::Pop => "Pop",
                OpCode::Dup => "Dup",
                OpCode::Swap => "Swap",
                OpCode::Add => "Add",
                OpCode::Sub => "Sub",
                OpCode::Mul => "Mul",
                OpCode::Div => "Div",
                OpCode::Neg => "Neg",
                OpCode::Not => "Not",
                OpCode::And => "And",
                OpCode::Or => "Or",
                OpCode::Eq => "Eq",
                OpCode::Neq => "Neq",
                OpCode::Gt => "Gt",
                OpCode::Lt => "Lt",
                OpCode::Gte => "Gte",
                OpCode::Lte => "Lte",
                OpCode::SetGlobal(_) => "SetGlobal",
                OpCode::GetGlobal(_) => "GetGlobal",
                OpCode::SetLocal(_) => "SetLocal",
                OpCode::GetLocal(_) => "GetLocal",
                OpCode::Jump(_) => "Jump",
                OpCode::JumpIfFalse(_) => "JumpIfFalse",
                OpCode::JumpIfTrue(_) => "JumpIfTrue",
                OpCode::Break => "Break",
                OpCode::Continue => "Continue",
                OpCode::MakeFunction { .. } => "MakeFunction",
                OpCode::Call { .. } => "Call",
                OpCode::Return => "Return",
                OpCode::Print => "Print",
                OpCode::Throw => "Throw",
                OpCode::PushErrHandler(_) => "PushErrHandler",
                OpCode::PopErrHandler => "PopErrHandler",
                OpCode::Noop => "Noop",
                OpCode::Halt => "Halt",
            })
            .collect()
    }

    #[test]
    fn test_var_decl() {
        // laadle x hai 42  →  Push(42) SetGlobal("x") Halt
        let ops = compile("laadle x hai 42\n");
        assert_eq!(op_names(&ops), ["Push", "SetGlobal", "Halt"]);
    }

    #[test]
    fn test_print() {
        let ops = compile("bol 1\n");
        assert_eq!(op_names(&ops), ["Push", "Print", "Halt"]);
    }

    #[test]
    fn test_binary_expr() {
        // bol 2 + 3  →  Push(2) Push(3) Add Print Halt
        let ops = compile("bol 2 + 3\n");
        assert_eq!(op_names(&ops), ["Push", "Push", "Add", "Print", "Halt"]);
    }

    #[test]
    fn test_unary_neg() {
        let ops = compile("bol -5\n");
        assert_eq!(op_names(&ops), ["Push", "Neg", "Print", "Halt"]);
    }

    #[test]
    fn test_if_no_else() {
        // agar sahi toh
        //     bol meow
        // should emit: Push(true) JumpIfFalse Jump Push(null) Print Halt
        let ops = compile("agar sahi toh\n    bol meow\n");
        let names = op_names(&ops);
        assert!(names.contains(&"JumpIfFalse"));
        assert!(names.contains(&"Jump"));
        assert!(names.contains(&"Print"));
    }

    #[test]
    fn test_if_with_else() {
        let src = "agar sahi toh\n    bol meow\nwarna\n    bol meow\n";
        let ops = compile(src);
        let names = op_names(&ops);
        // Should have a conditional jump AND an unconditional jump
        assert_eq!(names.iter().filter(|&&n| n == "JumpIfFalse").count(), 1);
        assert_eq!(names.iter().filter(|&&n| n == "Jump").count(), 1);
    }

    #[test]
    fn test_while_loop() {
        let src = "jabtak galat toh\n    bol meow\n";
        let ops = compile(src);
        let names = op_names(&ops);
        assert!(names.contains(&"JumpIfFalse")); // condition exit
        assert!(names.contains(&"Jump")); // loop-back jump
    }

    #[test]
    fn test_while_with_break() {
        let src = "jabtak sahi toh\n    nikal\n";
        let ops = compile(src);
        let names = op_names(&ops);
        // Break is compiled as Jump — verify 2 Jumps (break + loop-back)
        assert_eq!(names.iter().filter(|&&n| n == "Jump").count(), 2);
    }

    #[test]
    fn test_fn_decl_and_call() {
        let src = "kaam f() toh\n    wapas 1\nf()\n";
        let ops = compile(src);
        let names = op_names(&ops);
        // Should have: Jump(skip body), Push(1), Return, Push(Null), Return,
        //              MakeFunction, Call, Pop, Halt
        assert!(names.contains(&"MakeFunction"));
        assert!(names.contains(&"Call"));
        assert!(names.contains(&"Return"));
    }

    #[test]
    fn test_throw() {
        let ops = compile("gopgop \"oh no\"\n");
        assert_eq!(op_names(&ops), ["Push", "Throw", "Halt"]);
    }

    #[test]
    fn test_expr_stmt_pops() {
        // A bare call as a statement should Pop the return value
        let src = "kaam f() toh\n    wapas 1\nf()\n";
        let ops = compile(src);
        let names = op_names(&ops);
        assert!(names.contains(&"Pop")); // ExprStmt discards result
    }

    #[test]
    fn test_jump_targets_are_correct() {
        // After compilation of `agar sahi toh / bol 1 / warna / bol 2`,
        // verify that JumpIfFalse points past the then-block's Print,
        // and Jump points past the else-block's Print.
        let src = "agar sahi toh\n    bol 1\nwarna\n    bol 2\n";
        let ops = compile(src);
        // Find JumpIfFalse target
        let jif_target = ops
            .iter()
            .find_map(|op| {
                if let OpCode::JumpIfFalse(t) = op {
                    Some(*t)
                } else {
                    None
                }
            })
            .unwrap();
        // Find Jump (unconditional) target
        let jmp_target = ops
            .iter()
            .find_map(|op| {
                if let OpCode::Jump(t) = op {
                    Some(*t)
                } else {
                    None
                }
            })
            .unwrap();
        // JumpIfFalse target must be after the then-block (> JumpIfFalse position)
        // Jump target must be after the else-block (> JumpIfFalse target)
        assert!(jif_target > 0);
        assert!(jmp_target > jif_target);
    }
}
