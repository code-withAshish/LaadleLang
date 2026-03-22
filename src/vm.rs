use std::collections::HashMap;

// ─────────────────────────────────────────────────────────────────────────────
// VALUE — every runtime type the VM can hold on its stack
// ─────────────────────────────────────────────────────────────────────────────

#[derive(Debug, Clone, PartialEq)]
pub enum Value {
    Int(i32),
    Float(f64),
    Bool(bool),
    Str(String),
    Null,
    Error(String),
    /// A callable function: stored in globals by MakeFunction, retrieved by Call.
    Fn {
        name: String,
        addr: usize,
        params: Vec<String>,
    },
}

impl std::fmt::Display for Value {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Value::Int(n) => write!(f, "{}", n),
            Value::Float(n) => write!(f, "{}", n),
            Value::Bool(b) => write!(f, "{}", b),
            Value::Str(s) => write!(f, "{}", s),
            Value::Null => write!(f, "null"),
            Value::Error(e) => write!(f, "Error: {}", e),
            Value::Fn { name, .. } => write!(f, "<fn {}>", name),
        }
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// OPCODE — every instruction the VM understands
// ─────────────────────────────────────────────────────────────────────────────

#[derive(Debug, Clone)]
pub enum OpCode {
    // ── Stack ─────────────────────────────────────────────────────────────────
    /// Push a literal value onto the stack.
    Push(Value),
    /// Discard the top stack value.
    Pop,
    /// Duplicate the top stack value.
    Dup,
    /// Swap the top two stack values.
    Swap,

    // ── Arithmetic ────────────────────────────────────────────────────────────
    /// `a b → a + b`  (also concatenates Str+Str)
    Add,
    /// `a b → a - b`
    Sub,
    /// `a b → a * b`
    Mul,
    /// `a b → a / b`  (runtime err on int-div-by-zero)
    Div,
    /// `a → -a`  (unary negation)
    Neg,

    // ── Logic ─────────────────────────────────────────────────────────────────
    /// `a → !a`
    Not,
    /// `a b → a && b`
    And,
    /// `a b → a || b`
    Or,

    // ── Comparison ────────────────────────────────────────────────────────────
    /// `a b → a == b`
    Eq,
    /// `a b → a != b`
    Neq,
    /// `a b → a > b`
    Gt,
    /// `a b → a < b`
    Lt,
    /// `a b → a >= b`
    Gte,
    /// `a b → a <= b`
    Lte,

    // ── Variables ─────────────────────────────────────────────────────────────
    /// Pop value and store in global scope.
    SetGlobal(String),
    /// Push value from global scope.
    GetGlobal(String),
    /// Pop value and store in current CallFrame's locals.
    SetLocal(String),
    /// Push value from current CallFrame's locals.
    GetLocal(String),

    // ── Control Flow ──────────────────────────────────────────────────────────
    /// Unconditional jump to absolute address.
    Jump(usize),
    /// Pop cond; jump if falsy.
    JumpIfFalse(usize),
    /// Pop cond; jump if truthy.
    JumpIfTrue(usize),

    // ── Loop Control ──────────────────────────────────────────────────────────
    /// Compiled to Jump(loop_end) by the compiler — should never reach VM raw.
    Break,
    /// Compiled to Jump(loop_start) by the compiler — should never reach VM raw.
    Continue,

    // ── Functions ─────────────────────────────────────────────────────────────
    /// Register a function in globals so it can be called.
    MakeFunction {
        name: String,
        addr: usize,
        params: Vec<String>,
    },
    /// Call a function: pop argc args, push CallFrame, jump to function addr.
    Call { name: String, argc: usize },
    /// Return from function: pop CallFrame, restore ip, push return value.
    Return,

    // ── Output ────────────────────────────────────────────────────────────────
    /// Pop and print a value to stdout.
    Print,

    // ── Error Handling ────────────────────────────────────────────────────────
    /// Pop a value and throw it as an error.
    Throw,
    /// Register an error handler at a given address (stub for future try/catch).
    PushErrHandler(usize),
    /// Remove the innermost error handler (stub for future try/catch).
    PopErrHandler,

    // ── Misc ──────────────────────────────────────────────────────────────────
    /// Does nothing — useful as a placeholder.
    Noop,
    /// Stop the VM.
    Halt,
}

// Human-readable disassembly — used by main.rs with `println!("{}", op)`.
impl std::fmt::Display for OpCode {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            OpCode::Push(v) => write!(f, "Push({:?})", v),
            OpCode::Pop => write!(f, "Pop"),
            OpCode::Dup => write!(f, "Dup"),
            OpCode::Swap => write!(f, "Swap"),
            OpCode::Add => write!(f, "Add"),
            OpCode::Sub => write!(f, "Sub"),
            OpCode::Mul => write!(f, "Mul"),
            OpCode::Div => write!(f, "Div"),
            OpCode::Neg => write!(f, "Neg"),
            OpCode::Not => write!(f, "Not"),
            OpCode::And => write!(f, "And"),
            OpCode::Or => write!(f, "Or"),
            OpCode::Eq => write!(f, "Eq"),
            OpCode::Neq => write!(f, "Neq"),
            OpCode::Gt => write!(f, "Gt"),
            OpCode::Lt => write!(f, "Lt"),
            OpCode::Gte => write!(f, "Gte"),
            OpCode::Lte => write!(f, "Lte"),
            OpCode::SetGlobal(n) => write!(f, "SetGlobal({})", n),
            OpCode::GetGlobal(n) => write!(f, "GetGlobal({})", n),
            OpCode::SetLocal(n) => write!(f, "SetLocal({})", n),
            OpCode::GetLocal(n) => write!(f, "GetLocal({})", n),
            OpCode::Jump(t) => write!(f, "Jump(→ {})", t),
            OpCode::JumpIfFalse(t) => write!(f, "JumpIfFalse(→ {})", t),
            OpCode::JumpIfTrue(t) => write!(f, "JumpIfTrue(→ {})", t),
            OpCode::Break => write!(f, "Break"),
            OpCode::Continue => write!(f, "Continue"),
            OpCode::MakeFunction { name, addr, params } => write!(
                f,
                "MakeFunction({}, addr={}, params={:?})",
                name, addr, params
            ),
            OpCode::Call { name, argc } => write!(f, "Call({}, argc={})", name, argc),
            OpCode::Return => write!(f, "Return"),
            OpCode::Print => write!(f, "Print"),
            OpCode::Throw => write!(f, "Throw"),
            OpCode::PushErrHandler(t) => write!(f, "PushErrHandler(→ {})", t),
            OpCode::PopErrHandler => write!(f, "PopErrHandler"),
            OpCode::Noop => write!(f, "Noop"),
            OpCode::Halt => write!(f, "Halt"),
        }
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// CALL FRAME — saved state for one active function call
// ─────────────────────────────────────────────────────────────────────────────

#[derive(Debug)]
pub struct CallFrame {
    /// Where to resume in the caller after Return.
    pub return_ip: usize,
    /// Local variables (parameters + laadle declarations inside the fn).
    pub locals: HashMap<String, Value>,
    /// Stack depth when this frame was entered — used by Return to clean up.
    pub stack_base: usize,
}

impl CallFrame {
    pub fn new(return_ip: usize, stack_base: usize) -> Self {
        Self {
            return_ip,
            locals: HashMap::new(),
            stack_base,
        }
    }
}

// ─── ERROR HANDLER ───────────────────────────────────────────────────────────
// Stores the state for an active `koshish` (try) block so `gopgop` (throw)
// knows where to jump and how far to unwind the stack.

pub struct ErrHandler {
    /// Where to jump when an error is thrown
    pub catch_addr: usize,
    /// Call stack depth when the try block started (to unwind functions)
    pub call_depth: usize,
    /// VM stack size when the try block started (to discard leftovers)
    pub stack_base: usize,
}

// ─────────────────────────────────────────────────────────────────────────────
// VM
// ─────────────────────────────────────────────────────────────────────────────

pub struct LaadleVirtualMachineV1 {
    pub program: Vec<OpCode>,
    pub ip: usize,
    pub stack: Vec<Value>,
    pub globals: HashMap<String, Value>,
    pub call_stack: Vec<CallFrame>,
    pub err_stack: Vec<ErrHandler>,
    pub error: Option<Value>,
    pub output: String,
}

impl LaadleVirtualMachineV1 {
    pub fn new(program: Vec<OpCode>) -> Self {
        Self {
            program,
            ip: 0,
            stack: Vec::new(),
            globals: HashMap::new(),
            call_stack: Vec::new(),
            err_stack: Vec::new(),
            error: None,
            output: String::new(),
        }
    }

    // ── Helpers ───────────────────────────────────────────────────────────────

    fn push(&mut self, val: Value) {
        self.stack.push(val);
    }

    fn pop(&mut self) -> Value {
        self.stack.pop().expect("stack underflow — compiler bug")
    }

    fn peek(&self) -> &Value {
        self.stack.last().expect("peek on empty stack")
    }

    /// Truthiness rules: false / 0 / 0.0 / "" / null → false; everything else → true.
    fn is_truthy(val: &Value) -> bool {
        match val {
            Value::Bool(b) => *b,
            Value::Int(n) => *n != 0,
            Value::Float(f) => *f != 0.0,
            Value::Str(s) => !s.is_empty(),
            Value::Null => false,
            Value::Error(_) => false,
            Value::Fn { .. } => true,
        }
    }

    /// Numeric comparison: returns –1 / 0 / +1.
    fn cmp_values(a: &Value, b: &Value) -> Result<i32, String> {
        use std::cmp::Ordering::*;
        let ord = match (a, b) {
            (Value::Int(x), Value::Int(y)) => x.cmp(y),
            (Value::Float(x), Value::Float(y)) => x.partial_cmp(y).unwrap_or(Equal),
            (Value::Int(x), Value::Float(y)) => (*x as f64).partial_cmp(y).unwrap_or(Equal),
            (Value::Float(x), Value::Int(y)) => x.partial_cmp(&(*y as f64)).unwrap_or(Equal),
            (Value::Str(x), Value::Str(y)) => x.cmp(y),
            _ => return Err(format!("TypeError: cannot compare {:?} and {:?}", a, b)),
        };
        Ok(match ord {
            Less => -1,
            Equal => 0,
            Greater => 1,
        })
    }

    // ── Error Helper ──────────────────────────────────────────────────────────

    pub fn trigger_error(&mut self, msg: String) {
        let val = Value::Error(msg);
        self.error = Some(val.clone());

        if let Some(handler) = self.err_stack.pop() {
            self.call_stack.truncate(handler.call_depth);
            self.stack.truncate(handler.stack_base);
            self.push(val);
            self.ip = handler.catch_addr;
            self.error = None; // Reset the VM error state because it was trapped securely
        } else {
            self.output.push_str(&format!("\n💥 Uncaught GopGopError: {}\n", val));
            eprintln!("\n💥 Uncaught GopGopError: {}\n", val);
            self.ip = self.program.len(); // Halt the VM cleanly
        }
    }

    // ── Main execution loop ───────────────────────────────────────────────────

    pub fn run(&mut self) {
        loop {
            if self.ip >= self.program.len() {
                break;
            }

            let op = self.program[self.ip].clone();
            self.ip += 1;

            match op {
                // ── Stack / Literals ─────────────────────────────────────────
                OpCode::Push(val) => self.push(val),
                OpCode::Pop => {
                    self.pop();
                }
                OpCode::Dup => {
                    let v = self.peek().clone();
                    self.push(v);
                }
                OpCode::Swap => {
                    let b = self.pop();
                    let a = self.pop();
                    self.push(b);
                    self.push(a);
                }

                // ── Arithmetic ───────────────────────────────────────────────
                // Int+Int → Int, Float+Float → Float, Int+Float → Float.
                // Add also concatenates Str+Str.
                OpCode::Add => {
                    let b = self.pop();
                    let a = self.pop();
                    match (a.clone(), b.clone()) {
                        // Numeric Addition
                        (Value::Int(x), Value::Int(y)) => self.push(Value::Int(x + y)),
                        (Value::Float(x), Value::Float(y)) => self.push(Value::Float(x + y)),
                        (Value::Int(x), Value::Float(y)) => self.push(Value::Float(x as f64 + y)),
                        (Value::Float(x), Value::Int(y)) => self.push(Value::Float(x + y as f64)),
                        
                        // String Interpolation & Concatenation
                        (Value::Str(x), val) => self.push(Value::Str(format!("{}{}", x, val))),
                        (val, Value::Str(y)) => self.push(Value::Str(format!("{}{}", val, y))),
                        
                        (a, b) => {
                            self.trigger_error(format!("TypeError: cannot Add {:?} and {:?}", a, b));
                            continue;
                        }
                    }
                }
                OpCode::Sub => {
                    let b = self.pop();
                    let a = self.pop();
                    match (a.clone(), b.clone()) {
                        (Value::Int(x), Value::Int(y)) => self.push(Value::Int(x - y)),
                        (Value::Float(x), Value::Float(y)) => self.push(Value::Float(x - y)),
                        (Value::Int(x), Value::Float(y)) => self.push(Value::Float(x as f64 - y)),
                        (Value::Float(x), Value::Int(y)) => self.push(Value::Float(x - y as f64)),
                        (a, b) => {
                            self.trigger_error(format!("TypeError: cannot Sub {:?} and {:?}", a, b));
                            continue;
                        }
                    }
                }
                OpCode::Mul => {
                    let b = self.pop();
                    let a = self.pop();
                    match (a.clone(), b.clone()) {
                        (Value::Int(x), Value::Int(y)) => self.push(Value::Int(x * y)),
                        (Value::Float(x), Value::Float(y)) => self.push(Value::Float(x * y)),
                        (Value::Int(x), Value::Float(y)) => self.push(Value::Float(x as f64 * y)),
                        (Value::Float(x), Value::Int(y)) => self.push(Value::Float(x * y as f64)),
                        (a, b) => {
                            self.trigger_error(format!("TypeError: cannot Mul {:?} and {:?}", a, b));
                            continue;
                        }
                    }
                }
                OpCode::Div => {
                    let b = self.pop();
                    let a = self.pop();
                    match (a.clone(), b.clone()) {
                        (Value::Int(_), Value::Int(0)) => {
                            self.trigger_error("ZeroDivisionError: integer division by zero".to_string());
                            continue;
                        }
                        (Value::Int(x), Value::Int(y)) => self.push(Value::Int(x / y)),
                        (Value::Float(x), Value::Float(y)) => self.push(Value::Float(x / y)),
                        (Value::Int(x), Value::Float(y)) => self.push(Value::Float(x as f64 / y)),
                        (Value::Float(x), Value::Int(y)) => self.push(Value::Float(x / y as f64)),
                        (a, b) => {
                            self.trigger_error(format!("TypeError: cannot Div {:?} and {:?}", a, b));
                            continue;
                        }
                    }
                }
                OpCode::Neg => {
                    let a = self.pop();
                    match a.clone() {
                        Value::Int(x) => self.push(Value::Int(-x)),
                        Value::Float(x) => self.push(Value::Float(-x)),
                        a => {
                            self.trigger_error(format!("TypeError: cannot negate {:?}", a));
                            continue;
                        }
                    }
                }

                // ── Logic ────────────────────────────────────────────────────
                OpCode::Not => {
                    let a = self.pop();
                    self.push(Value::Bool(!Self::is_truthy(&a)));
                }
                OpCode::And => {
                    let b = self.pop();
                    let a = self.pop();
                    self.push(Value::Bool(Self::is_truthy(&a) && Self::is_truthy(&b)));
                }
                OpCode::Or => {
                    let b = self.pop();
                    let a = self.pop();
                    self.push(Value::Bool(Self::is_truthy(&a) || Self::is_truthy(&b)));
                }

                // ── Comparison ───────────────────────────────────────────────
                OpCode::Eq => {
                    let b = self.pop();
                    let a = self.pop();
                    self.push(Value::Bool(a == b));
                }
                OpCode::Neq => {
                    let b = self.pop();
                    let a = self.pop();
                    self.push(Value::Bool(a != b));
                }
                OpCode::Gt => {
                    let b = self.pop();
                    let a = self.pop();
                    match Self::cmp_values(&a, &b) {
                        Ok(res) => self.push(Value::Bool(res > 0)),
                        Err(msg) => { self.trigger_error(msg); continue; }
                    }
                }
                OpCode::Lt => {
                    let b = self.pop();
                    let a = self.pop();
                    match Self::cmp_values(&a, &b) {
                        Ok(res) => self.push(Value::Bool(res < 0)),
                        Err(msg) => { self.trigger_error(msg); continue; }
                    }
                }
                OpCode::Gte => {
                    let b = self.pop();
                    let a = self.pop();
                    match Self::cmp_values(&a, &b) {
                        Ok(res) => self.push(Value::Bool(res >= 0)),
                        Err(msg) => { self.trigger_error(msg); continue; }
                    }
                }
                OpCode::Lte => {
                    let b = self.pop();
                    let a = self.pop();
                    match Self::cmp_values(&a, &b) {
                        Ok(res) => self.push(Value::Bool(res <= 0)),
                        Err(msg) => { self.trigger_error(msg); continue; }
                    }
                }

                // ── Variables ────────────────────────────────────────────────
                OpCode::SetGlobal(name) => {
                    let val = self.pop();
                    self.globals.insert(name, val);
                }
                OpCode::GetGlobal(name) => {
                    if let Some(val) = self.globals.get(&name) {
                        self.push(val.clone());
                    } else {
                        self.trigger_error(format!("NameError: undefined variable `{}`", name));
                        continue;
                    }
                }
                OpCode::SetLocal(name) => {
                    let val = self.pop();
                    if let Some(frame) = self.call_stack.last_mut() {
                        frame.locals.insert(name, val);
                    } else {
                        self.trigger_error(format!("ScopeError: SetLocal `{}` outside of a function", name));
                        continue;
                    }
                }
                OpCode::GetLocal(name) => {
                    // Look up locals first, then fall back to globals.
                    // This lets functions read variables declared at the top level.
                    if let Some(frame) = self.call_stack.last() {
                        if let Some(val) = frame.locals.get(&name).cloned().or_else(|| self.globals.get(&name).cloned()) {
                            self.push(val);
                        } else {
                            self.trigger_error(format!("NameError: undefined variable `{}`", name));
                            continue;
                        }
                    } else {
                        self.trigger_error(format!("ScopeError: GetLocal `{}` outside of a function", name));
                        continue;
                    }
                }

                // ── Control Flow ─────────────────────────────────────────────
                // All jump targets are absolute instruction indices.
                OpCode::Jump(target) => {
                    self.ip = target;
                }
                OpCode::JumpIfFalse(target) => {
                    let cond = self.pop();
                    if !Self::is_truthy(&cond) {
                        self.ip = target;
                    }
                }
                OpCode::JumpIfTrue(target) => {
                    let cond = self.pop();
                    if Self::is_truthy(&cond) {
                        self.ip = target;
                    }
                }

                // ── Loop Control ─────────────────────────────────────────────
                // These should have been patched to Jump by the compiler.
                OpCode::Break | OpCode::Continue => {
                    self.trigger_error("InternalError: Break/Continue reached the VM".to_string());
                    continue;
                }

                // ── Functions ────────────────────────────────────────────────
                //
                // MakeFunction: store the function descriptor in globals so
                // Call can look it up by name.
                OpCode::MakeFunction { name, addr, params } => {
                    self.globals
                        .insert(name.clone(), Value::Fn { name, addr, params });
                }
                //
                // Call:
                //   1. Pop `argc` args (they're in forward order on the stack — reverse to get arg0 first)
                //   2. Resolve function by name in globals
                //   3. Push a new CallFrame (saves return_ip and stack_base)
                //   4. Bind each parameter name → argument value in frame.locals
                //   5. Jump to the function's start address
                OpCode::Call { name, argc } => {
                    let mut args = Vec::with_capacity(argc);
                    for _ in 0..argc {
                        args.push(self.pop());
                    }
                    args.reverse();

                    let func = match self.globals.get(&name) {
                        Some(val) => val.clone(),
                        None => {
                            self.trigger_error(format!("NameError: undefined function `{}`", name));
                            continue;
                        }
                    };

                    let (fn_addr, params) = match func {
                        Value::Fn { addr, params, .. } => (addr, params),
                        _ => {
                            self.trigger_error(format!("TypeError: `{}` is not a function", name));
                            continue;
                        }
                    };

                    if args.len() != params.len() {
                        self.trigger_error(format!("ArgumentError: `{}` expects {} args, got {}", name, params.len(), args.len()));
                        continue;
                    }

                    let mut frame = CallFrame::new(self.ip, self.stack.len());
                    for (param, arg) in params.iter().zip(args) {
                        frame.locals.insert(param.clone(), arg);
                    }
                    self.call_stack.push(frame);
                    self.ip = fn_addr;
                }
                //
                // Return:
                //   1. Pop the return value
                //   2. Pop the CallFrame, restoring ip
                //   3. Truncate the stack back to stack_base (discards any
                //      leftover values from the function body)
                //   4. Push the return value for the caller
                OpCode::Return => {
                    let return_val = self.pop();
                    if let Some(frame) = self.call_stack.pop() {
                        self.ip = frame.return_ip;
                        self.stack.truncate(frame.stack_base);
                        self.push(return_val);
                    } else {
                        self.trigger_error("ScopeError: Return with no active call frame".to_string());
                        continue;
                    }
                }

                // ── Output ───────────────────────────────────────────────────
                OpCode::Print => {
                    let val = self.pop();
                    self.output.push_str(&format!("{}\n", val));
                    println!("{}", val);
                }

                // ── Error Handling ───────────────────────────────────────────
                OpCode::PushErrHandler(catch_addr) => {
                    self.err_stack.push(ErrHandler {
                        catch_addr,
                        call_depth: self.call_stack.len(),
                        stack_base: self.stack.len(),
                    });
                }
                OpCode::PopErrHandler => {
                    if self.err_stack.pop().is_none() {
                        self.trigger_error("InternalError: PopErrHandler without pushed handler".to_string());
                        continue;
                    }
                }
                OpCode::Throw => {
                    let val = self.pop();
                    self.error = Some(val.clone());

                    if let Some(handler) = self.err_stack.pop() {
                        // Unwind call stack up to the try block's depth
                        self.call_stack.truncate(handler.call_depth);
                        // Truncate VM stack to the exact state before try block started
                        self.stack.truncate(handler.stack_base);

                        // Push the error value for the catch block and jump
                        self.push(val);
                        self.ip = handler.catch_addr;
                        self.error = None; // The error was fully safely trapped
                    } else {
                        // Uncaught error: halt the VM cleanly and print
                        self.output.push_str(&format!("\n💥 Uncaught GopGopError: {}\n", val));
                        eprintln!("\n💥 Uncaught GopGopError: {}\n", val);
                        break;
                    }
                }

                // ── Misc ─────────────────────────────────────────────────────
                OpCode::Noop => { /* meow */ }
                OpCode::Halt => break,
            }
        }
    }

    /// Convenience: build a VM and run it in one call.
    pub fn run_program(program: Vec<OpCode>) {
        LaadleVirtualMachineV1::new(program).run();
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// UNIT TESTS  —  pure VM-level tests, no compiler or parser involved
//
// E2E tests (source → tokenize → parse → compile → run) live in
// tests/e2e.rs as Rust integration tests.
// ─────────────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    // Helper: build + run a bytecode program and return the vm state.
    fn exec(program: Vec<OpCode>) -> LaadleVirtualMachineV1 {
        let mut vm = LaadleVirtualMachineV1::new(program);
        vm.run();
        vm
    }

    // ── Stack basics ──────────────────────────────────────────────────────────

    #[test]
    fn test_push_and_print() {
        // No panic = pass
        LaadleVirtualMachineV1::run_program(vec![
            OpCode::Push(Value::Int(42)),
            OpCode::Print,
            OpCode::Halt,
        ]);
    }

    #[test]
    fn test_dup_and_swap() {
        let vm = exec(vec![
            OpCode::Push(Value::Int(1)),
            OpCode::Push(Value::Int(2)),
            OpCode::Swap, // stack: [2, 1]
            OpCode::Dup,  // stack: [2, 1, 1]
            OpCode::Halt,
        ]);
        assert_eq!(vm.stack, vec![Value::Int(2), Value::Int(1), Value::Int(1)]);
    }

    #[test]
    fn test_halt_early() {
        let vm = exec(vec![OpCode::Halt, OpCode::Push(Value::Int(99))]);
        assert!(vm.stack.is_empty());
    }

    #[test]
    fn test_noop() {
        let vm = exec(vec![
            OpCode::Noop,
            OpCode::Push(Value::Bool(true)),
            OpCode::Halt,
        ]);
        assert_eq!(vm.stack, vec![Value::Bool(true)]);
    }

    // ── Arithmetic ────────────────────────────────────────────────────────────

    #[test]
    fn test_add_int() {
        let vm = exec(vec![
            OpCode::Push(Value::Int(3)),
            OpCode::Push(Value::Int(4)),
            OpCode::Add,
            OpCode::Halt,
        ]);
        assert_eq!(vm.stack, vec![Value::Int(7)]);
    }

    #[test]
    fn test_sub_mul_div() {
        // (10 - 4) * 2 / 3 = 4  (integer division)
        let vm = exec(vec![
            OpCode::Push(Value::Int(10)),
            OpCode::Push(Value::Int(4)),
            OpCode::Sub,
            OpCode::Push(Value::Int(2)),
            OpCode::Mul,
            OpCode::Push(Value::Int(3)),
            OpCode::Div,
            OpCode::Halt,
        ]);
        assert_eq!(vm.stack, vec![Value::Int(4)]);
    }

    #[test]
    fn test_neg() {
        let vm = exec(vec![OpCode::Push(Value::Int(7)), OpCode::Neg, OpCode::Halt]);
        assert_eq!(vm.stack, vec![Value::Int(-7)]);
    }

    #[test]
    fn test_str_concat() {
        let vm = exec(vec![
            OpCode::Push(Value::Str("hello".into())),
            OpCode::Push(Value::Str(" world".into())),
            OpCode::Add,
            OpCode::Halt,
        ]);
        assert_eq!(vm.stack, vec![Value::Str("hello world".into())]);
    }

    // ── Logic & comparison ────────────────────────────────────────────────────

    #[test]
    fn test_not() {
        let vm = exec(vec![
            OpCode::Push(Value::Bool(false)),
            OpCode::Not,
            OpCode::Halt,
        ]);
        assert_eq!(vm.stack, vec![Value::Bool(true)]);
    }

    #[test]
    fn test_eq_neq() {
        let vm = exec(vec![
            OpCode::Push(Value::Int(5)),
            OpCode::Push(Value::Int(5)),
            OpCode::Eq,
            OpCode::Push(Value::Int(5)),
            OpCode::Push(Value::Int(6)),
            OpCode::Neq,
            OpCode::Halt,
        ]);
        assert_eq!(vm.stack, vec![Value::Bool(true), Value::Bool(true)]);
    }

    #[test]
    fn test_gt_lt() {
        let vm = exec(vec![
            OpCode::Push(Value::Int(10)),
            OpCode::Push(Value::Int(3)),
            OpCode::Gt,
            OpCode::Push(Value::Int(1)),
            OpCode::Push(Value::Int(2)),
            OpCode::Lt,
            OpCode::Halt,
        ]);
        assert_eq!(vm.stack, vec![Value::Bool(true), Value::Bool(true)]);
    }

    // ── Variables ─────────────────────────────────────────────────────────────

    #[test]
    fn test_global_set_get() {
        let vm = exec(vec![
            OpCode::Push(Value::Int(42)),
            OpCode::SetGlobal("x".into()),
            OpCode::GetGlobal("x".into()),
            OpCode::Halt,
        ]);
        assert_eq!(vm.stack, vec![Value::Int(42)]);
    }

    // ── Control flow ──────────────────────────────────────────────────────────

    #[test]
    fn test_jump() {
        let vm = exec(vec![
            OpCode::Jump(2),              // [0] skip [1]
            OpCode::Push(Value::Int(1)),  // [1] skipped
            OpCode::Push(Value::Int(99)), // [2]
            OpCode::Halt,
        ]);
        assert_eq!(vm.stack, vec![Value::Int(99)]);
    }

    #[test]
    fn test_jump_if_false() {
        let vm = exec(vec![
            OpCode::Push(Value::Bool(false)), // [0]
            OpCode::JumpIfFalse(3),           // [1] → skip [2]
            OpCode::Push(Value::Int(1)),      // [2] skipped
            OpCode::Push(Value::Int(2)),      // [3]
            OpCode::Halt,
        ]);
        assert_eq!(vm.stack, vec![Value::Int(2)]);
    }

    // ── Functions ─────────────────────────────────────────────────────────────

    #[test]
    fn test_function_call_and_return() {
        // double(n) = n + n;  double(21) = 42
        let vm = exec(vec![
            // [0] skip body
            OpCode::Jump(6),
            // [1..5] body of double(n)
            OpCode::GetLocal("n".into()),
            OpCode::GetLocal("n".into()),
            OpCode::Add,
            OpCode::Return,
            OpCode::Push(Value::Null), // implicit trailing return (unreachable)
            // [6] register function
            OpCode::MakeFunction {
                name: "double".into(),
                addr: 1,
                params: vec!["n".into()],
            },
            // [7] call double(21)
            OpCode::Push(Value::Int(21)),
            OpCode::Call {
                name: "double".into(),
                argc: 1,
            },
            // [9]
            OpCode::Halt,
        ]);
        assert_eq!(vm.stack, vec![Value::Int(42)]);
    }
}
