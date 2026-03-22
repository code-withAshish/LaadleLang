// ─────────────────────────────────────────────────────────────────────────────
// TOKENIZER — LaadleLang
//
// Pythonic / indentation-based syntax:
//   • Newlines end statements (no semicolons)
//   • Indentation opens blocks (INDENT token), de-indentation closes them (DEDENT)
//   • `toh` marks the start of a block header (like `:` in Python)
//   • `(` `)` `,` are kept for function calls and declarations
//   • `//` line comments are skipped
//
// INDENT/DEDENT algorithm (same as CPython):
//   - Maintain a stack of indentation levels, starting at [0]
//   - At the beginning of every non-empty, non-comment line:
//       • measure leading spaces (tabs count as 4 spaces)
//       • if level > top of stack → emit INDENT, push level
//       • if level < top of stack → pop and emit DEDENT until level matches
//       • if level == top → nothing (just a new statement)
// ─────────────────────────────────────────────────────────────────────────────

#[derive(Debug, Clone, PartialEq)]
pub enum Token {
    // ── Core keywords ────────────────────────────────────────────────────────
    /// Variable declaration  →  `laadle x hai 10`
    Laadle,
    /// Assignment operator   →  `hai`  (plays the role of `=`)
    Hai,
    /// Print statement       →  `bol x`
    Bol,
    /// If keyword            →  `agar x > 5 toh`
    Agar,
    /// Else keyword          →  `warna`
    Warna,
    /// While keyword         →  `jabtak i < 10 toh`
    Jabtak,
    /// Block-open marker     →  `toh`  (like `:` in Python)
    Toh,

    // ── Control flow ─────────────────────────────────────────────────────────
    /// Function declaration  →  `kaam add(a, b) toh`
    Kaam,
    /// Return                →  `wapas x`
    Wapas,
    /// Break                 →  `nikal`
    Nikal,
    /// Continue              →  `aage`
    Aage,

    // ── Operators ────────────────────────────────────────────────────────────
    Plus,  // +
    Minus, // -
    Star,  // *
    Slash, // /

    EqualEqual, // ==
    NotEqual,   // !=
    Greater,    // >
    Less,       // <
    GreaterEq,  // >=
    LessEq,     // <=

    And,  // &&
    Or,   // ||
    Bang, // !  (unary NOT)

    // ── Literals ─────────────────────────────────────────────────────────────
    /// Variable / function name
    Identifier(String),
    /// Integer literal
    Number(i32),
    /// Floating-point literal
    Float(f64),
    /// String literal  →  `"hello"`
    Str(String),
    /// Boolean true    →  `sahi`
    Sahi,
    /// Boolean false   →  `galat`
    Galat,
    /// Void / null     →  `meow`
    Meow,

    // ── Structure symbols ────────────────────────────────────────────────────
    LeftParen,  // (   — function call / declaration
    RightParen, // )
    Comma,      // ,   — argument separator

    // ── Indentation (block structure) ────────────────────────────────────────
    /// Emitted when indentation increases — opens a block
    Indent,
    /// Emitted when indentation decreases — closes a block
    Dedent,
    /// Logical newline — ends a statement
    Newline,

    // ── Error / misc ─────────────────────────────────────────────────────────
    /// Throw error  →  `gopgop "message"`
    GopGop,
    /// Try block    →  `koshish toh`
    Koshish,
    /// Catch block  →  `pakad <ident> toh`
    Pakad,
    /// End of file
    Eof,
}

// ─── TOKENIZER ───────────────────────────────────────────────────────────────

pub struct Tokenizer {
    /// Source characters
    source: Vec<char>,
    /// Current position in `source`
    current: usize,
    /// Indentation stack — starts at [0]
    indent_stack: Vec<usize>,
    /// Whether we are at the very beginning of a new line (need to measure indent)
    at_line_start: bool,
}

impl Tokenizer {
    pub fn new(input: &str) -> Self {
        Self {
            source: input.chars().collect(),
            current: 0,
            indent_stack: vec![0],
            at_line_start: true,
        }
    }

    // ── Public entry point ────────────────────────────────────────────────────

    pub fn tokenize(&mut self) -> Vec<Token> {
        let mut tokens: Vec<Token> = Vec::new();

        while !self.is_at_end() {
            // ── Handle line-start indentation ────────────────────────────────
            if self.at_line_start {
                self.at_line_start = false;

                // Measure this line's leading whitespace
                let level = self.measure_indent();

                // Skip blank lines and comment-only lines entirely.
                // A line is blank if after the indent there's a newline or comment.
                if self.peek() == '\n' || (self.peek() == '/' && self.peek_next() == '/') {
                    // Skip to end of line without emitting anything
                    self.skip_to_newline();
                    if !self.is_at_end() {
                        self.advance(); // consume the '\n'
                    }
                    self.at_line_start = true;
                    continue;
                }

                // Compare measured level to top of indent stack
                let top = *self.indent_stack.last().unwrap();

                if level > top {
                    self.indent_stack.push(level);
                    tokens.push(Token::Indent);
                } else if level < top {
                    while *self.indent_stack.last().unwrap() > level {
                        self.indent_stack.pop();
                        tokens.push(Token::Dedent);
                    }
                    // If we popped past the target level it's an indentation error
                    if *self.indent_stack.last().unwrap() != level {
                        panic!(
                            "IndentationError: unindent does not match any outer indentation level"
                        );
                    }
                }
                // level == top → no indent token needed, just continue scanning
            }

            // ── Skip inline whitespace (spaces/tabs that aren't newlines) ────
            if self.peek() == ' ' || self.peek() == '\t' {
                self.advance();
                continue;
            }

            // ── Newline → end of statement ───────────────────────────────────
            if self.peek() == '\n' {
                self.advance();
                // Only emit Newline if there's something meaningful before it
                // (avoid duplicate Newlines from blank lines — those are handled above)
                if let Some(last) = tokens.last()
                    && !matches!(last, Token::Newline | Token::Indent | Token::Dedent)
                {
                    tokens.push(Token::Newline);
                }
                self.at_line_start = true;
                continue;
            }

            // ── Line comment  `// ...` ────────────────────────────────────────
            if self.peek() == '/' && self.peek_next() == '/' {
                self.skip_to_newline();
                continue;
            }

            // ── Scan next token ───────────────────────────────────────────────
            let ch = self.advance();

            let tok = match ch {
                // Single-char operators / symbols
                '+' => Token::Plus,
                '-' => Token::Minus,
                '*' => Token::Star,
                '(' => Token::LeftParen,
                ')' => Token::RightParen,
                ',' => Token::Comma,

                // Slash — could be `/` or start of `//` comment (handled above)
                '/' => Token::Slash,

                // Two-char operators
                '=' => {
                    if self.peek() == '=' {
                        self.advance();
                        Token::EqualEqual
                    } else {
                        panic!(
                            "Unexpected `=` — did you mean `hai` for assignment or `==` for equality?"
                        )
                    }
                }
                '!' => {
                    if self.peek() == '=' {
                        self.advance();
                        Token::NotEqual
                    } else {
                        Token::Bang
                    }
                }
                '>' => {
                    if self.peek() == '=' {
                        self.advance();
                        Token::GreaterEq
                    } else {
                        Token::Greater
                    }
                }
                '<' => {
                    if self.peek() == '=' {
                        self.advance();
                        Token::LessEq
                    } else {
                        Token::Less
                    }
                }
                '&' => {
                    if self.peek() == '&' {
                        self.advance();
                        Token::And
                    } else {
                        panic!("Unexpected `&` — did you mean `&&`?")
                    }
                }
                '|' => {
                    if self.peek() == '|' {
                        self.advance();
                        Token::Or
                    } else {
                        panic!("Unexpected `|` — did you mean `||`?")
                    }
                }

                // String literal  "..."
                '"' => self.scan_string(),

                // Number literal
                c if c.is_ascii_digit() => self.scan_number(c),

                // Identifier or keyword
                c if c.is_alphabetic() || c == '_' => self.scan_word(c),

                other => panic!("Unexpected character: `{}`", other),
            };

            tokens.push(tok);
        }

        // ── End of file: close any remaining open blocks ──────────────────────
        // Emit Newline to close the last statement (if needed)
        if let Some(last) = tokens.last()
            && !matches!(last, Token::Newline | Token::Indent | Token::Dedent)
        {
            tokens.push(Token::Newline);
        }
        // Pop all remaining indentation levels
        while self.indent_stack.len() > 1 {
            self.indent_stack.pop();
            tokens.push(Token::Dedent);
        }

        tokens.push(Token::Eof);
        tokens
    }

    // ── Scanning helpers ──────────────────────────────────────────────────────

    /// Count leading spaces on the current line without consuming them.
    /// Tabs are normalised to 4 spaces.
    fn measure_indent(&mut self) -> usize {
        let mut level = 0usize;
        while !self.is_at_end() && (self.peek() == ' ' || self.peek() == '\t') {
            if self.peek() == '\t' {
                level += 4;
            } else {
                level += 1;
            }
            self.advance();
        }
        level
    }

    /// Skip characters until end of line (does NOT consume the `\n`).
    fn skip_to_newline(&mut self) {
        while !self.is_at_end() && self.peek() != '\n' {
            self.advance();
        }
    }

    /// Scan a string literal (opening `"` already consumed).
    /// Supports `\"` escape inside strings.
    fn scan_string(&mut self) -> Token {
        let mut s = String::new();
        while !self.is_at_end() && self.peek() != '"' {
            let c = self.advance();
            if c == '\\' && self.peek() == '"' {
                self.advance(); // consume the escaped quote
                s.push('"');
            } else {
                s.push(c);
            }
        }
        if self.is_at_end() {
            panic!("Unterminated string literal");
        }
        self.advance(); // closing `"`
        Token::Str(s)
    }

    /// Scan an integer or float, first digit already consumed.
    fn scan_number(&mut self, first: char) -> Token {
        let mut num = String::from(first);
        while !self.is_at_end() && self.peek().is_ascii_digit() {
            num.push(self.advance());
        }
        // Check for decimal point
        if self.peek() == '.' && self.peek_next().is_ascii_digit() {
            num.push(self.advance()); // consume '.'
            while !self.is_at_end() && self.peek().is_ascii_digit() {
                num.push(self.advance());
            }
            Token::Float(num.parse().unwrap())
        } else {
            Token::Number(num.parse().unwrap())
        }
    }

    /// Scan an identifier or keyword, first char already consumed.
    fn scan_word(&mut self, first: char) -> Token {
        let mut word = String::from(first);
        while !self.is_at_end() && (self.peek().is_alphanumeric() || self.peek() == '_') {
            word.push(self.advance());
        }
        // Match against all reserved keywords
        match word.as_str() {
            "laadle" => Token::Laadle,
            "hai" => Token::Hai,
            "bol" => Token::Bol,
            "agar" => Token::Agar,
            "warna" => Token::Warna,
            "jabtak" => Token::Jabtak,
            "toh" => Token::Toh,
            "kaam" => Token::Kaam,
            "wapas" => Token::Wapas,
            "nikal" => Token::Nikal,
            "aage" => Token::Aage,
            "sahi" => Token::Sahi,
            "galat" => Token::Galat,
            "meow" => Token::Meow,
            "gopgop" => Token::GopGop,
            "koshish" => Token::Koshish,
            "pakad" => Token::Pakad,
            _ => Token::Identifier(word),
        }
    }

    // ── Low-level character helpers ───────────────────────────────────────────

    fn is_at_end(&self) -> bool {
        self.current >= self.source.len()
    }

    fn advance(&mut self) -> char {
        let ch = self.source[self.current];
        self.current += 1;
        ch
    }

    fn peek(&self) -> char {
        if self.is_at_end() {
            '\0'
        } else {
            self.source[self.current]
        }
    }

    fn peek_next(&self) -> char {
        if self.current + 1 >= self.source.len() {
            '\0'
        } else {
            self.source[self.current + 1]
        }
    }
}

// ─── TESTS ───────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    fn tok(src: &str) -> Vec<Token> {
        Tokenizer::new(src).tokenize()
    }

    #[test]
    fn test_var_decl() {
        // laadle x hai 10
        let tokens = tok("laadle x hai 10\n");
        assert_eq!(
            tokens,
            vec![
                Token::Laadle,
                Token::Identifier("x".into()),
                Token::Hai,
                Token::Number(10),
                Token::Newline,
                Token::Eof,
            ]
        );
    }

    #[test]
    fn test_string_literal() {
        let tokens = tok("bol \"meow meow\"\n");
        assert_eq!(
            tokens,
            vec![
                Token::Bol,
                Token::Str("meow meow".into()),
                Token::Newline,
                Token::Eof,
            ]
        );
    }

    #[test]
    fn test_indent_dedent() {
        // agar sahi toh
        //     bol meow
        let src = "agar sahi toh\n    bol meow\n";
        let tokens = tok(src);
        assert_eq!(
            tokens,
            vec![
                Token::Agar,
                Token::Sahi,
                Token::Toh,
                Token::Newline,
                Token::Indent,
                Token::Bol,
                Token::Meow,
                Token::Newline,
                Token::Dedent,
                Token::Eof,
            ]
        );
    }

    #[test]
    fn test_operators() {
        let tokens = tok("x == y\n");
        assert_eq!(
            tokens,
            vec![
                Token::Identifier("x".into()),
                Token::EqualEqual,
                Token::Identifier("y".into()),
                Token::Newline,
                Token::Eof,
            ]
        );
    }

    #[test]
    fn test_comment_skipped() {
        let tokens = tok("// this is a comment\nlaadle x hai 5\n");
        assert_eq!(
            tokens,
            vec![
                Token::Laadle,
                Token::Identifier("x".into()),
                Token::Hai,
                Token::Number(5),
                Token::Newline,
                Token::Eof,
            ]
        );
    }

    #[test]
    fn test_float() {
        let tokens = tok("laadle pi hai 3.14\n");
        assert_eq!(
            tokens,
            vec![
                Token::Laadle,
                Token::Identifier("pi".into()),
                Token::Hai,
                Token::Float(3.14),
                Token::Newline,
                Token::Eof,
            ]
        );
    }
}
