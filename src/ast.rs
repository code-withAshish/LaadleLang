// ─────────────────────────────────────────────────────────────────────────────
// AST  —  Abstract Syntax Tree for MeowLang
//
// The Parser produces Vec<Stmt>.
// The Compiler walks Vec<Stmt> and emits Vec<OpCode>.
// ─────────────────────────────────────────────────────────────────────────────

// ─── STATEMENTS ──────────────────────────────────────────────────────────────
// A statement is a complete unit of execution that does NOT produce a value
// on the stack by itself.

#[derive(Debug, Clone)]
pub enum Stmt {
    // ── Variables ────────────────────────────────────────────────────────────
    /// `laadle x hai <expr>`
    /// Compiles to: <compile expr> + SetGlobal("x")
    VarDecl { name: String, value: Expr },

    /// `x hai <expr>`  (re-assignment, not a new declaration)
    /// Compiles to: <compile expr> + SetGlobal("x")  (or SetLocal inside fn)
    Assign { name: String, value: Expr },

    // ── Output ───────────────────────────────────────────────────────────────
    /// `bol <expr>`
    /// Compiles to: <compile expr> + Print
    Print(Expr),

    // ── Control Flow ─────────────────────────────────────────────────────────
    /// `agar <cond> toh`
    ///      <indented then block>
    /// `warna`
    ///      <indented else block>   (optional)
    /// Compiles to: <cond> + JumpIfFalse + <then> + Jump + <else>
    If {
        condition: Expr,
        then_branch: Vec<Stmt>,
        else_branch: Option<Vec<Stmt>>,
    },

    /// `jabtak <cond> toh`
    ///      <indented body>
    /// Compiles to: loop_start + <cond> + JumpIfFalse(loop_end) + <body> + Jump(loop_start)
    While { condition: Expr, body: Vec<Stmt> },

    /// `nikal`  — break out of the nearest loop
    /// Compiles to: Break  (compiler patches the jump target)
    Break,

    /// `aage`  — skip to the next loop iteration
    /// Compiles to: Continue  (compiler patches the jump target)
    Continue,

    // ── Functions ────────────────────────────────────────────────────────────
    /// `kaam <name>(<params>) toh <body> bas`
    /// Compiles to: MakeFunction { name, addr, params }  +  the function body
    FnDecl {
        name: String,
        params: Vec<String>,
        body: Vec<Stmt>,
    },

    /// `wapas <expr>`
    /// Compiles to: <compile expr> + Return
    Return(Expr),

    // ── Error Handling ───────────────────────────────────────────────────────
    /// `gopgop <expr>`
    /// Compiles to: <compile expr> + Throw
    Throw(Expr),

    // ── Try / Catch ──────────────────────────────────────────────────────────
    /// `koshish toh`
    ///      `<body>`
    /// `pakad <ident> toh`
    ///      `<handler>`
    ///
    /// Compiles to:
    ///   PushErrHandler(catch_addr)
    ///   <body>
    ///   PopErrHandler
    ///   Jump(end)
    ///   catch_addr: SetGlobal/SetLocal(catch_var)   ← error value on stack
    ///   <handler>
    ///   end:
    TryCatch {
        body: Vec<Stmt>,
        catch_var: String,
        handler: Vec<Stmt>,
    },

    // ── Expression as Statement ──────────────────────────────────────────────
    /// Any bare expression used as a statement (e.g. a function call whose
    /// return value is discarded).
    /// Compiles to: <compile expr> + Pop   (discard the leftover stack value)
    ExprStmt(Expr),
}

// ─── EXPRESSIONS ─────────────────────────────────────────────────────────────
// An expression is something that, when compiled, leaves exactly ONE value on
// the VM stack.

#[derive(Debug, Clone)]
pub enum Expr {
    // ── Literals ─────────────────────────────────────────────────────────────
    /// `42`          — token Number(i32)
    /// Compiles to: Push(Value::Int(n))
    Int(i32),

    /// `3.14`        — token Float (if you add it to the tokenizer)
    /// Compiles to: Push(Value::Float(f))
    Float(f64),

    /// `sahi`  — boolean true  (token `Sahi`)
    /// Compiles to: Push(Value::Bool(true))
    Sahi,
    /// `galat`  — boolean false  (token `Galat`)
    /// Compiles to: Push(Value::Bool(false))
    Galat,

    /// `"hello"`     — token String(s)
    /// Compiles to: Push(Value::Str(s))
    Str(String),

    /// `meow`        — the null / void literal
    /// Compiles to: Push(Value::Null)
    Null,

    // ── Variable Reference ───────────────────────────────────────────────────
    /// `x`           — token Identifier(name)
    /// Compiles to: GetLocal("x")  or  GetGlobal("x")  (scope resolution)
    Variable(String),

    // ── Unary Operations ─────────────────────────────────────────────────────
    /// `-<expr>`     — arithmetic negation
    /// Compiles to: <compile expr> + Neg
    ///
    /// `!<expr>`     — logical NOT
    /// Compiles to: <compile expr> + Not
    Unary { op: UnaryOp, operand: Box<Expr> },

    // ── Binary Operations ────────────────────────────────────────────────────
    /// `<left> <op> <right>`
    /// Compiles to: <left> + <right> + <op opcode>
    Binary {
        left: Box<Expr>,
        op: BinaryOp,
        right: Box<Expr>,
    },

    // ── Function Call ─────────────────────────────────────────────────────────
    /// `add(3, 7)`   — token Identifier + LeftParen + args + RightParen
    /// Compiles to: <arg1> + <arg2> + ... + Call { name, argc }
    Call { name: String, args: Vec<Expr> },
}

// ─── OPERATORS ───────────────────────────────────────────────────────────────

/// Unary operators — applied to a single operand.
#[derive(Debug, Clone)]
pub enum UnaryOp {
    /// `-`   maps to OpCode::Neg
    Neg,
    /// `!`   maps to OpCode::Not
    Not,
}

/// Binary operators — applied to two operands.
/// Each variant maps directly to one comparison or arithmetic OpCode.
#[derive(Debug, Clone)]
pub enum BinaryOp {
    // Arithmetic (tokens Plus / Minus / Star / Slash)
    Add, // → OpCode::Add
    Sub, // → OpCode::Sub
    Mul, // → OpCode::Mul
    Div, // → OpCode::Div

    // Comparison (tokens EqualEqual / NotEqual / Greater / Less / GreaterEq / LessEq)
    Eq,  // → OpCode::Eq
    Neq, // → OpCode::Neq
    Gt,  // → OpCode::Gt
    Lt,  // → OpCode::Lt
    Gte, // → OpCode::Gte
    Lte, // → OpCode::Lte

    // Logical (tokens And / Or)
    And, // → OpCode::And
    Or,  // → OpCode::Or
}
