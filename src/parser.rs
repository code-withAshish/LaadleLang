// ─────────────────────────────────────────────────────────────────────────────
// PARSER — LaadleLang  (Recursive Descent)
//
// Input:  Vec<Token>  (produced by Tokenizer)
// Output: Vec<Stmt>   (the AST, consumed by Compiler)
//
// Grammar (informal):
//
//   program      → stmt* EOF
//
//   stmt         → var_decl
//                | assign
//                | print_stmt
//                | if_stmt
//                | while_stmt
//                | break_stmt
//                | continue_stmt
//                | fn_decl
//                | return_stmt
//                | throw_stmt
//                | expr_stmt
//
//   var_decl     → "laadle" IDENT "hai" expr NEWLINE
//   assign       → IDENT "hai" expr NEWLINE          (no "laadle" prefix)
//   print_stmt   → "bol" expr NEWLINE
//   if_stmt      → "agar" expr "toh" NEWLINE block
//                  ("warna" NEWLINE block)?
//   while_stmt   → "jabtak" expr "toh" NEWLINE block
//   break_stmt   → "nikal" "ja" NEWLINE
//   continue_stmt→ "aage" "badh" NEWLINE
//   fn_decl      → "kaam" IDENT "(" params ")" "toh" NEWLINE block
//   return_stmt  → "wapas" "bhej" expr NEWLINE
//   throw_stmt   → "gopgop" expr NEWLINE
//   expr_stmt    → expr NEWLINE
//
//   block        → INDENT stmt+ DEDENT
//
//   expr         → or_expr
//   or_expr      → and_expr  ( "||" and_expr )*
//   and_expr     → equality  ( "&&" equality )*
//   equality     → comparison ( ("==" | "!=") comparison )*
//   comparison   → term       ( (">" | "<" | ">=" | "<=") term )*
//   term         → factor     ( ("+" | "-") factor )*
//   factor       → unary      ( ("*" | "/") unary )*
//   unary        → ("-" | "!") unary  |  primary
//   primary      → NUMBER | FLOAT | STRING | "sahi" | "galat" | "meow"
//                | IDENT "(" args ")"    (function call)
//                | IDENT                 (variable)
//                | "(" expr ")"          (grouping)
// ─────────────────────────────────────────────────────────────────────────────

use crate::ast::{BinaryOp, Expr, Stmt, UnaryOp};
use crate::tokenizer::Token;

// ─── PARSER STRUCT ────────────────────────────────────────────────────────────

pub struct Parser {
    tokens: Vec<Token>,
    /// Index of the token we are currently looking at
    current: usize,
    pub error: Option<String>,
}

impl Parser {
    pub fn new(tokens: Vec<Token>) -> Self {
        Self {
            tokens,
            current: 0,
            error: None,
        }
    }

    fn report_error(&mut self, msg: String) {
        if self.error.is_none() {
            self.error = Some(msg);
        }
    }

    // ── Entry point ───────────────────────────────────────────────────────────

    /// Parse the entire token stream into a list of top-level statements.
    pub fn parse(&mut self) -> Vec<Stmt> {
        let mut stmts = Vec::new();

        // Skip any leading newlines
        self.skip_newlines();

        while !self.is_at_end() {
            stmts.push(self.parse_stmt());
            self.skip_newlines();
        }

        stmts
    }

    // ── Statement dispatch ────────────────────────────────────────────────────

    fn parse_stmt(&mut self) -> Stmt {
        match self.peek() {
            Token::Laadle => self.parse_var_decl(),
            Token::Bol => self.parse_print(),
            Token::Agar => self.parse_if(),
            Token::Jabtak => self.parse_while(),
            Token::Nikal => self.parse_break(),
            Token::Aage => self.parse_continue(),
            Token::Kaam => self.parse_fn_decl(),
            Token::Wapas => self.parse_return(),
            Token::GopGop => self.parse_throw(),
            Token::Koshish => self.parse_try_catch(),
            // Identifier followed by `hai` → re-assignment
            Token::Identifier(_) if self.peek_at(1) == &Token::Hai => self.parse_assign(),
            // Everything else → expression statement (e.g. function call)
            _ => self.parse_expr_stmt(),
        }
    }

    // ── Statement parsers ─────────────────────────────────────────────────────

    /// `laadle x hai <expr>` NEWLINE
    fn parse_var_decl(&mut self) -> Stmt {
        self.expect(Token::Laadle);
        let name = self.expect_identifier("variable name after `laadle`");
        self.expect(Token::Hai);
        let value = self.parse_expr();
        self.expect_newline();
        Stmt::VarDecl { name, value }
    }

    /// `x hai <expr>` NEWLINE
    fn parse_assign(&mut self) -> Stmt {
        let name = self.expect_identifier("variable name for assignment");
        self.expect(Token::Hai);
        let value = self.parse_expr();
        self.expect_newline();
        Stmt::Assign { name, value }
    }

    /// `bol <expr>` NEWLINE
    fn parse_print(&mut self) -> Stmt {
        self.expect(Token::Bol);
        let expr = self.parse_expr();
        self.expect_newline();
        Stmt::Print(expr)
    }

    /// `agar <cond> toh` NEWLINE INDENT <then> DEDENT
    /// (`warna` NEWLINE INDENT <else> DEDENT)?
    fn parse_if(&mut self) -> Stmt {
        self.expect(Token::Agar);
        let condition = self.parse_expr();
        self.expect(Token::Toh);
        self.expect_newline();

        let then_branch = self.parse_block();

        // Optional `warna` (else) clause
        let else_branch = if self.peek() == &Token::Warna {
            self.advance(); // consume `warna`
            self.expect_newline();
            Some(self.parse_block())
        } else {
            None
        };

        Stmt::If {
            condition,
            then_branch,
            else_branch,
        }
    }

    /// `jabtak <cond> toh` NEWLINE INDENT <body> DEDENT
    fn parse_while(&mut self) -> Stmt {
        self.expect(Token::Jabtak);
        let condition = self.parse_expr();
        self.expect(Token::Toh);
        self.expect_newline();
        let body = self.parse_block();
        Stmt::While { condition, body }
    }

    /// `nikal` NEWLINE
    fn parse_break(&mut self) -> Stmt {
        self.expect(Token::Nikal);
        self.expect_newline();
        Stmt::Break
    }

    /// `aage` NEWLINE
    fn parse_continue(&mut self) -> Stmt {
        self.expect(Token::Aage);
        self.expect_newline();
        Stmt::Continue
    }

    /// `kaam <name>(<params>) toh` NEWLINE INDENT <body> DEDENT
    fn parse_fn_decl(&mut self) -> Stmt {
        self.expect(Token::Kaam);
        let name = self.expect_identifier("function name after `kaam`");
        self.expect(Token::LeftParen);

        let mut params = Vec::new();
        if self.peek() != &Token::RightParen {
            params.push(self.expect_identifier("parameter name"));
            while self.peek() == &Token::Comma {
                self.advance(); // consume ','
                params.push(self.expect_identifier("parameter name after ','"));
            }
        }

        self.expect(Token::RightParen);
        self.expect(Token::Toh);
        self.expect_newline();

        let body = self.parse_block();
        Stmt::FnDecl { name, params, body }
    }

    /// `wapas [<expr>]` NEWLINE
    fn parse_return(&mut self) -> Stmt {
        self.expect(Token::Wapas);
        let expr = if self.peek() == &Token::Newline {
            Expr::Null
        } else {
            self.parse_expr()
        };
        self.expect_newline();
        Stmt::Return(expr)
    }

    /// `gopgop <expr>` NEWLINE
    fn parse_throw(&mut self) -> Stmt {
        self.expect(Token::GopGop);
        let expr = self.parse_expr();
        self.expect_newline();
        Stmt::Throw(expr)
    }

    /// `koshish toh` NEWLINE INDENT <body> DEDENT
    /// `pakad <err_var> toh` NEWLINE INDENT <handler> DEDENT
    fn parse_try_catch(&mut self) -> Stmt {
        self.expect(Token::Koshish);
        self.expect(Token::Toh);
        self.expect_newline();
        let body = self.parse_block();

        self.expect(Token::Pakad);
        let catch_var = self.expect_identifier("error variable name after `pakad`");
        self.expect(Token::Toh);
        self.expect_newline();
        let handler = self.parse_block();

        Stmt::TryCatch {
            body,
            catch_var,
            handler,
        }
    }

    /// Bare expression as a statement (e.g. a function call for side effects).
    fn parse_expr_stmt(&mut self) -> Stmt {
        let expr = self.parse_expr();
        self.expect_newline();
        Stmt::ExprStmt(expr)
    }

    // ── Block parsing ─────────────────────────────────────────────────────────

    /// Parse an indented block: INDENT stmt+ DEDENT
    fn parse_block(&mut self) -> Vec<Stmt> {
        self.expect(Token::Indent);
        let mut stmts = Vec::new();

        self.skip_newlines();
        while self.peek() != &Token::Dedent && !self.is_at_end() {
            stmts.push(self.parse_stmt());
            self.skip_newlines();
        }

        self.expect(Token::Dedent);
        stmts
    }

    // ── Expression parsers (Pratt / recursive descent by precedence) ──────────
    //
    // Precedence (low → high):
    //   or  →  and  →  equality  →  comparison  →  term  →  factor  →  unary  →  primary

    fn parse_expr(&mut self) -> Expr {
        self.parse_or()
    }

    /// or_expr → and_expr ( "||" and_expr )*
    fn parse_or(&mut self) -> Expr {
        let mut left = self.parse_and();
        while self.peek() == &Token::Or {
            self.advance();
            let right = self.parse_and();
            left = Expr::Binary {
                left: Box::new(left),
                op: BinaryOp::Or,
                right: Box::new(right),
            };
        }
        left
    }

    /// and_expr → equality ( "&&" equality )*
    fn parse_and(&mut self) -> Expr {
        let mut left = self.parse_equality();
        while self.peek() == &Token::And {
            self.advance();
            let right = self.parse_equality();
            left = Expr::Binary {
                left: Box::new(left),
                op: BinaryOp::And,
                right: Box::new(right),
            };
        }
        left
    }

    /// equality → comparison ( ("==" | "!=") comparison )*
    fn parse_equality(&mut self) -> Expr {
        let mut left = self.parse_comparison();
        loop {
            let op = match self.peek() {
                Token::EqualEqual => BinaryOp::Eq,
                Token::NotEqual => BinaryOp::Neq,
                _ => break,
            };
            self.advance();
            let right = self.parse_comparison();
            left = Expr::Binary {
                left: Box::new(left),
                op,
                right: Box::new(right),
            };
        }
        left
    }

    /// comparison → term ( (">" | "<" | ">=" | "<=") term )*
    fn parse_comparison(&mut self) -> Expr {
        let mut left = self.parse_term();
        loop {
            let op = match self.peek() {
                Token::Greater => BinaryOp::Gt,
                Token::Less => BinaryOp::Lt,
                Token::GreaterEq => BinaryOp::Gte,
                Token::LessEq => BinaryOp::Lte,
                _ => break,
            };
            self.advance();
            let right = self.parse_term();
            left = Expr::Binary {
                left: Box::new(left),
                op,
                right: Box::new(right),
            };
        }
        left
    }

    /// term → factor ( ("+" | "-") factor )*
    fn parse_term(&mut self) -> Expr {
        let mut left = self.parse_factor();
        loop {
            let op = match self.peek() {
                Token::Plus => BinaryOp::Add,
                Token::Minus => BinaryOp::Sub,
                _ => break,
            };
            self.advance();
            let right = self.parse_factor();
            left = Expr::Binary {
                left: Box::new(left),
                op,
                right: Box::new(right),
            };
        }
        left
    }

    /// factor → unary ( ("*" | "/") unary )*
    fn parse_factor(&mut self) -> Expr {
        let mut left = self.parse_unary();
        loop {
            let op = match self.peek() {
                Token::Star => BinaryOp::Mul,
                Token::Slash => BinaryOp::Div,
                _ => break,
            };
            self.advance();
            let right = self.parse_unary();
            left = Expr::Binary {
                left: Box::new(left),
                op,
                right: Box::new(right),
            };
        }
        left
    }

    /// unary → ("-" | "!") unary  |  primary
    fn parse_unary(&mut self) -> Expr {
        match self.peek().clone() {
            Token::Minus => {
                self.advance();
                let operand = self.parse_unary();
                Expr::Unary {
                    op: UnaryOp::Neg,
                    operand: Box::new(operand),
                }
            }
            Token::Bang => {
                self.advance();
                let operand = self.parse_unary();
                Expr::Unary {
                    op: UnaryOp::Not,
                    operand: Box::new(operand),
                }
            }
            _ => self.parse_primary(),
        }
    }

    /// primary → literal | IDENT ["(" args ")"] | "(" expr ")"
    fn parse_primary(&mut self) -> Expr {
        match self.peek().clone() {
            Token::Number(n) => {
                self.advance();
                Expr::Int(n)
            }
            Token::Float(f) => {
                self.advance();
                Expr::Float(f)
            }
            Token::Str(s) => {
                self.advance();
                Expr::Str(s)
            }
            Token::Sahi => {
                self.advance();
                Expr::Sahi
            }
            Token::Galat => {
                self.advance();
                Expr::Galat
            }
            Token::Meow => {
                self.advance();
                Expr::Null
            }
            Token::Identifier(name) => {
                self.advance();
                // Check for function call: IDENT "(" ...
                if self.peek() == &Token::LeftParen {
                    self.advance(); // consume '('
                    let mut args = Vec::new();
                    if self.peek() != &Token::RightParen {
                        args.push(self.parse_expr());
                        while self.peek() == &Token::Comma {
                            self.advance(); // consume ','
                            args.push(self.parse_expr());
                        }
                    }
                    self.expect(Token::RightParen);
                    Expr::Call { name, args }
                } else {
                    Expr::Variable(name)
                }
            }
            Token::LeftParen => {
                self.advance(); // consume '('
                let inner = self.parse_expr();
                self.expect(Token::RightParen);
                inner
            }
            other => {
                self.report_error(format!("Unexpected token in expression: {:?}", other));
                self.advance(); // Prevent infinite loops on bad tokens
                Expr::Int(0) // Dummy fallback node
            }
        }
    }

    // ── Token stream helpers ──────────────────────────────────────────────────

    /// Return a reference to the current token without consuming it.
    fn peek(&self) -> &Token {
        &self.tokens[self.current]
    }

    /// Return a reference to a token `offset` positions ahead without consuming.
    fn peek_at(&self, offset: usize) -> &Token {
        let idx = self.current + offset;
        if idx >= self.tokens.len() {
            &Token::Eof
        } else {
            &self.tokens[idx]
        }
    }

    /// Consume and return the current token.
    fn advance(&mut self) -> Token {
        let tok = self.tokens[self.current].clone();
        if !self.is_at_end() {
            self.current += 1;
        }
        tok
    }

    /// Consume the current token only if it matches `expected`, else panic.
    fn expect(&mut self, expected: Token) -> &Token {
        if self.peek() == &expected {
            self.advance();
            &self.tokens[self.current - 1]
        } else {
            self.report_error(format!(
                "Expected {:?} but found {:?}",
                expected,
                self.peek()
            ));
            &self.tokens[self.current] // Dummy traversal logic
        }
    }

    /// Consume the current token as an Identifier and return the inner String.
    fn expect_identifier(&mut self, context: &str) -> String {
        match self.advance() {
            Token::Identifier(name) => name,
            other => {
                self.report_error(format!(
                    "Expected identifier ({}) but found {:?}",
                    context, other
                ));
                "$$ERROR$$".to_string()
            }
        }
    }

    /// Consume a Newline token (end of statement).
    fn expect_newline(&mut self) {
        match self.peek() {
            Token::Newline => {
                self.advance();
            }
            Token::Eof => {} // end of file also terminates a statement
            other => {
                self.report_error(format!(
                    "Expected newline at end of statement, found {:?}",
                    other
                ));
                self.advance();
            }
        }
    }

    /// Skip any run of Newline tokens (used between statements in a block).
    fn skip_newlines(&mut self) {
        while self.peek() == &Token::Newline {
            self.advance();
        }
    }

    /// True when we've consumed all tokens (or are sitting on EOF).
    fn is_at_end(&self) -> bool {
        matches!(self.peek(), Token::Eof)
    }
}

// ─── CONVENIENCE ─────────────────────────────────────────────────────────────

/// Parse a source string end-to-end: tokenize then parse.
pub fn parse_source(source: &str) -> Vec<Stmt> {
    use crate::tokenizer::Tokenizer;
    let tokens = Tokenizer::new(source).tokenize();
    Parser::new(tokens).parse()
}

// ─── TESTS ───────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ast::*;

    fn parse(src: &str) -> Vec<Stmt> {
        parse_source(src)
    }

    #[test]
    fn test_var_decl() {
        let ast = parse("laadle x hai 42\n");
        assert!(matches!(&ast[0], Stmt::VarDecl { name, .. } if name == "x"));
    }

    #[test]
    fn test_assign() {
        let ast = parse("x hai 10\n");
        assert!(matches!(&ast[0], Stmt::Assign { name, .. } if name == "x"));
    }

    #[test]
    fn test_print() {
        let ast = parse("bol meow\n");
        assert!(matches!(&ast[0], Stmt::Print(Expr::Null)));
    }

    #[test]
    fn test_if_no_else() {
        let src = "agar x toh\n    bol meow\n";
        let ast = parse(src);
        assert!(matches!(
            &ast[0],
            Stmt::If {
                else_branch: None,
                ..
            }
        ));
    }

    #[test]
    fn test_if_with_else() {
        let src = "agar sahi toh\n    bol meow\nwarna\n    bol meow\n";
        let ast = parse(src);
        assert!(matches!(
            &ast[0],
            Stmt::If {
                else_branch: Some(_),
                ..
            }
        ));
    }

    #[test]
    fn test_while_loop() {
        let src = "jabtak sahi toh\n    bol meow\n";
        let ast = parse(src);
        assert!(matches!(&ast[0], Stmt::While { .. }));
    }

    #[test]
    fn test_break_continue() {
        let ast = parse("nikal\naage\n");
        assert!(matches!(&ast[0], Stmt::Break));
        assert!(matches!(&ast[1], Stmt::Continue));
    }

    #[test]
    fn test_fn_decl() {
        let src = "kaam add(a, b) toh\n    wapas a\n";
        let ast = parse(src);
        assert!(matches!(&ast[0], Stmt::FnDecl { name, params, .. }
            if name == "add" && params.len() == 2
        ));
    }

    #[test]
    fn test_return() {
        let src = "kaam f() toh\n    wapas 1\n";
        let ast = parse(src);
        if let Stmt::FnDecl { body, .. } = &ast[0] {
            assert!(matches!(&body[0], Stmt::Return(Expr::Int(1))));
        } else {
            panic!("expected FnDecl");
        }
    }

    #[test]
    fn test_throw() {
        let ast = parse("gopgop \"oops\"\n");
        assert!(matches!(&ast[0], Stmt::Throw(Expr::Str(_))));
    }

    #[test]
    fn test_fn_call_expr_stmt() {
        let ast = parse("add(1, 2)\n");
        assert!(matches!(&ast[0], Stmt::ExprStmt(Expr::Call { name, args })
            if name == "add" && args.len() == 2
        ));
    }

    #[test]
    fn test_binary_precedence() {
        // 2 + 3 * 4  should parse as  2 + (3 * 4)
        let ast = parse("bol 2 + 3 * 4\n");
        if let Stmt::Print(Expr::Binary { op, right, .. }) = &ast[0] {
            assert!(matches!(op, BinaryOp::Add));
            assert!(matches!(
                right.as_ref(),
                Expr::Binary {
                    op: BinaryOp::Mul,
                    ..
                }
            ));
        } else {
            panic!("expected Print with binary expr");
        }
    }

    #[test]
    fn test_unary_neg() {
        let ast = parse("bol -5\n");
        assert!(matches!(
            &ast[0],
            Stmt::Print(Expr::Unary {
                op: UnaryOp::Neg,
                ..
            })
        ));
    }

    #[test]
    fn test_grouped_expr() {
        // (2 + 3) * 4 — the grouping should make Add the outer op
        let ast = parse("bol (2 + 3) * 4\n");
        if let Stmt::Print(Expr::Binary { op, left, .. }) = &ast[0] {
            assert!(matches!(op, BinaryOp::Mul));
            assert!(matches!(
                left.as_ref(),
                Expr::Binary {
                    op: BinaryOp::Add,
                    ..
                }
            ));
        } else {
            panic!("expected Print with Mul as outer op");
        }
    }
}
