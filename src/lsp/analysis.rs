use tower_lsp::lsp_types::*;

// We need to duplicate some core logic here since we can't easily share
// between binaries. In a real project, you'd use a library crate.

#[derive(Debug, Clone, PartialEq)]
enum TokenType {
    // Original keywords
    Stel, As, Anders, Terwyl, Druk, Waar, Vals,
    // Functional keywords
    Funksie, Fn, Gee, Laat, Mut,
    // Pattern matching
    Pas, Geval, Tipe,
    // Literals and identifiers
    Number(f64), Str(String), Identifier(String),
    // Operators
    Plus, Minus, Star, Slash, Percent,
    Equal, EqualEqual, Bang, BangEqual,
    Less, LessEqual, Greater, GreaterEqual,
    And, Or, Arrow,
    // Delimiters
    LeftParen, RightParen, LeftBrace, RightBrace,
    LeftBracket, RightBracket, Comma, Underscore,
    Newline, Eof,
}

#[derive(Debug, Clone)]
struct Token {
    token_type: TokenType,
    lexeme: String,
    line: u32,
    start_col: u32,
    end_col: u32,
}

struct Lexer {
    source: Vec<char>,
    tokens: Vec<Token>,
    start: usize,
    current: usize,
    line: u32,
    col: u32,
    start_col: u32,
}

impl Lexer {
    fn new(source: &str) -> Self {
        Lexer {
            source: source.chars().collect(),
            tokens: Vec::new(),
            start: 0,
            current: 0,
            line: 0,
            col: 0,
            start_col: 0,
        }
    }

    fn scan_tokens(&mut self) -> (Vec<Token>, Vec<Diagnostic>) {
        let mut diagnostics = Vec::new();

        while !self.is_at_end() {
            self.start = self.current;
            self.start_col = self.col;
            if let Err(e) = self.scan_token() {
                diagnostics.push(e);
            }
        }

        self.tokens.push(Token {
            token_type: TokenType::Eof,
            lexeme: String::new(),
            line: self.line,
            start_col: self.col,
            end_col: self.col,
        });

        (self.tokens.clone(), diagnostics)
    }

    fn scan_token(&mut self) -> std::result::Result<(), Diagnostic> {
        let c = self.advance();

        match c {
            '(' => self.add_token(TokenType::LeftParen),
            ')' => self.add_token(TokenType::RightParen),
            '{' => self.add_token(TokenType::LeftBrace),
            '}' => self.add_token(TokenType::RightBrace),
            '[' => self.add_token(TokenType::LeftBracket),
            ']' => self.add_token(TokenType::RightBracket),
            ',' => self.add_token(TokenType::Comma),
            '+' => self.add_token(TokenType::Plus),
            '-' => self.add_token(TokenType::Minus),
            '*' => self.add_token(TokenType::Star),
            '%' => self.add_token(TokenType::Percent),
            '/' => {
                if self.match_char('/') {
                    while self.peek() != '\n' && !self.is_at_end() {
                        self.advance();
                    }
                } else {
                    self.add_token(TokenType::Slash);
                }
            }
            '=' => {
                let token = if self.match_char('=') {
                    TokenType::EqualEqual
                } else if self.match_char('>') {
                    TokenType::Arrow
                } else {
                    TokenType::Equal
                };
                self.add_token(token);
            }
            '!' => {
                let token = if self.match_char('=') {
                    TokenType::BangEqual
                } else {
                    TokenType::Bang
                };
                self.add_token(token);
            }
            '<' => {
                let token = if self.match_char('=') {
                    TokenType::LessEqual
                } else {
                    TokenType::Less
                };
                self.add_token(token);
            }
            '>' => {
                let token = if self.match_char('=') {
                    TokenType::GreaterEqual
                } else {
                    TokenType::Greater
                };
                self.add_token(token);
            }
            '&' => {
                if self.match_char('&') {
                    self.add_token(TokenType::And);
                } else {
                    return Err(self.make_diagnostic("Onverwagte karakter '&'. Bedoel jy '&&'?"));
                }
            }
            '|' => {
                if self.match_char('|') {
                    self.add_token(TokenType::Or);
                } else {
                    return Err(self.make_diagnostic("Onverwagte karakter '|'. Bedoel jy '||'?"));
                }
            }
            '"' => {
                self.string()?;
            }
            '\n' => {
                self.add_token(TokenType::Newline);
                self.line += 1;
                self.col = 0;
            }
            ' ' | '\r' | '\t' => {}
            _ => {
                if c.is_ascii_digit() {
                    self.number();
                } else if c.is_alphabetic() || c == '_' {
                    self.identifier();
                } else {
                    return Err(self.make_diagnostic(&format!("Onverwagte karakter '{}'", c)));
                }
            }
        }

        Ok(())
    }

    fn number(&mut self) {
        while self.peek().is_ascii_digit() {
            self.advance();
        }
        if self.peek() == '.' && self.peek_next().is_ascii_digit() {
            self.advance();
            while self.peek().is_ascii_digit() {
                self.advance();
            }
        }
        let lexeme: String = self.source[self.start..self.current].iter().collect();
        let value: f64 = lexeme.parse().unwrap_or(0.0);
        self.add_token(TokenType::Number(value));
    }

    fn identifier(&mut self) {
        while self.peek().is_alphanumeric() || self.peek() == '_' {
            self.advance();
        }
        let lexeme: String = self.source[self.start..self.current].iter().collect();
        let token_type = match lexeme.as_str() {
            // Original keywords
            "stel" => TokenType::Stel,
            "as" => TokenType::As,
            "anders" => TokenType::Anders,
            "terwyl" => TokenType::Terwyl,
            "druk" => TokenType::Druk,
            "waar" => TokenType::Waar,
            "vals" => TokenType::Vals,
            // Functional keywords
            "funksie" => TokenType::Funksie,
            "fn" => TokenType::Fn,
            "gee" => TokenType::Gee,
            "laat" => TokenType::Laat,
            "mut" => TokenType::Mut,
            // Pattern matching
            "pas" => TokenType::Pas,
            "geval" => TokenType::Geval,
            "tipe" => TokenType::Tipe,
            // Wildcard
            "_" => TokenType::Underscore,
            _ => TokenType::Identifier(lexeme.clone()),
        };
        self.add_token(token_type);
    }

    fn string(&mut self) -> std::result::Result<(), Diagnostic> {
        let start_line = self.line;
        let start_col = self.start_col;

        while self.peek() != '"' && !self.is_at_end() {
            if self.peek() == '\n' {
                self.line += 1;
                self.col = 0;
            }
            if self.peek() == '\\' && !self.is_at_end() {
                self.advance(); // consume backslash
                if !self.is_at_end() {
                    self.advance(); // consume escaped char
                }
            } else {
                self.advance();
            }
        }

        if self.is_at_end() {
            return Err(Diagnostic {
                range: Range {
                    start: Position { line: start_line, character: start_col },
                    end: Position { line: self.line, character: self.col },
                },
                severity: Some(DiagnosticSeverity::ERROR),
                source: Some("arcane".to_string()),
                message: "Onbeeindigde string - verwag '\"'".to_string(),
                ..Default::default()
            });
        }

        self.advance(); // consume closing "

        let value: String = self.source[self.start + 1..self.current - 1].iter().collect();
        self.add_token(TokenType::Str(value));
        Ok(())
    }

    fn is_at_end(&self) -> bool {
        self.current >= self.source.len()
    }

    fn advance(&mut self) -> char {
        let c = self.source[self.current];
        self.current += 1;
        self.col += 1;
        c
    }

    fn peek(&self) -> char {
        if self.is_at_end() { '\0' } else { self.source[self.current] }
    }

    fn peek_next(&self) -> char {
        if self.current + 1 >= self.source.len() { '\0' } else { self.source[self.current + 1] }
    }

    fn match_char(&mut self, expected: char) -> bool {
        if self.is_at_end() || self.source[self.current] != expected {
            false
        } else {
            self.current += 1;
            self.col += 1;
            true
        }
    }

    fn add_token(&mut self, token_type: TokenType) {
        let lexeme: String = self.source[self.start..self.current].iter().collect();
        self.tokens.push(Token {
            token_type,
            lexeme,
            line: self.line,
            start_col: self.start_col,
            end_col: self.col,
        });
    }

    fn make_diagnostic(&self, message: &str) -> Diagnostic {
        Diagnostic {
            range: Range {
                start: Position { line: self.line, character: self.start_col },
                end: Position { line: self.line, character: self.col },
            },
            severity: Some(DiagnosticSeverity::ERROR),
            source: Some("arcane".to_string()),
            message: message.to_string(),
            ..Default::default()
        }
    }
}

// Simple parser for diagnostics
fn parse_for_diagnostics(tokens: &[Token]) -> Vec<Diagnostic> {
    let mut diagnostics = Vec::new();
    let mut i = 0;
    let mut paren_stack: Vec<&Token> = Vec::new();
    let mut brace_stack: Vec<&Token> = Vec::new();
    let mut bracket_stack: Vec<&Token> = Vec::new();

    // Track declared variables for undefined variable detection
    let mut declared_vars: std::collections::HashSet<String> = std::collections::HashSet::new();
    let mut declared_funcs: std::collections::HashSet<String> = std::collections::HashSet::new();

    // Built-in functions that are always valid
    let builtin_funcs: std::collections::HashSet<&str> = [
        // Output
        "druk",
        // Higher-order functions
        "kaart", "filter", "vou", "vir_elk",
        // List functions
        "lengte", "kop", "stert", "leeg", "voeg_by", "heg_aan", "ketting", "omgekeer",
    ].iter().cloned().collect();

    // First pass: collect all declared variables and function names
    let mut j = 0;
    while j < tokens.len() {
        // Track 'stel' declarations
        if matches!(tokens[j].token_type, TokenType::Stel) {
            if j + 1 < tokens.len() {
                if let TokenType::Identifier(name) = &tokens[j + 1].token_type {
                    declared_vars.insert(name.clone());
                }
            }
        }
        // Track 'laat' declarations (with optional 'mut')
        if matches!(tokens[j].token_type, TokenType::Laat) {
            let name_offset = if j + 1 < tokens.len()
                && matches!(tokens[j + 1].token_type, TokenType::Mut)
            {
                2 // laat mut name
            } else {
                1 // laat name
            };
            if j + name_offset < tokens.len() {
                if let TokenType::Identifier(name) = &tokens[j + name_offset].token_type {
                    declared_vars.insert(name.clone());
                }
            }
        }
        // Track 'funksie' declarations - the function name is also a valid identifier
        if matches!(tokens[j].token_type, TokenType::Funksie) {
            if j + 1 < tokens.len() {
                if let TokenType::Identifier(name) = &tokens[j + 1].token_type {
                    declared_vars.insert(name.clone());
                    declared_funcs.insert(name.clone());
                }
            }
        }
        j += 1;
    }

    // Second pass: collect function parameters
    j = 0;
    while j < tokens.len() {
        // For 'funksie name(' - collect parameters
        if matches!(tokens[j].token_type, TokenType::Funksie) {
            if j + 2 < tokens.len() && matches!(tokens[j + 2].token_type, TokenType::LeftParen) {
                let mut k = j + 3;
                // Collect parameters until closing paren
                while k < tokens.len() && !matches!(tokens[k].token_type, TokenType::RightParen) {
                    if let TokenType::Identifier(name) = &tokens[k].token_type {
                        declared_vars.insert(name.clone());
                    }
                    k += 1;
                }
            }
        }
        // For 'fn(' lambda - collect parameters
        if matches!(tokens[j].token_type, TokenType::Fn) {
            if j + 1 < tokens.len() && matches!(tokens[j + 1].token_type, TokenType::LeftParen) {
                let mut k = j + 2;
                while k < tokens.len() && !matches!(tokens[k].token_type, TokenType::RightParen) {
                    if let TokenType::Identifier(name) = &tokens[k].token_type {
                        declared_vars.insert(name.clone());
                    }
                    k += 1;
                }
            }
        }
        j += 1;
    }

    // Third pass: collect type constructors from 'tipe' definitions
    j = 0;
    while j < tokens.len() {
        if matches!(tokens[j].token_type, TokenType::Tipe) {
            // Skip type name and opening brace
            if j + 2 < tokens.len() && matches!(tokens[j + 2].token_type, TokenType::LeftBrace) {
                let mut k = j + 3;
                // Collect constructors until closing brace
                while k < tokens.len() && !matches!(tokens[k].token_type, TokenType::RightBrace) {
                    if let TokenType::Identifier(name) = &tokens[k].token_type {
                        // Constructor names start with uppercase
                        if name.chars().next().map(|c| c.is_uppercase()).unwrap_or(false) {
                            declared_funcs.insert(name.clone());
                        }
                    }
                    k += 1;
                }
            }
        }
        j += 1;
    }

    // Track if we're inside a type definition block
    let mut in_type_def = false;
    let mut type_def_brace_depth = 0;

    while i < tokens.len() {
        let token = &tokens[i];

        // Track entering/exiting type definitions
        if matches!(token.token_type, TokenType::Tipe) {
            in_type_def = true;
        }
        if in_type_def && matches!(token.token_type, TokenType::LeftBrace) {
            type_def_brace_depth += 1;
        }
        if type_def_brace_depth > 0 && matches!(token.token_type, TokenType::RightBrace) {
            type_def_brace_depth -= 1;
            if type_def_brace_depth == 0 {
                in_type_def = false;
            }
        }

        match &token.token_type {
            TokenType::LeftParen => paren_stack.push(token),
            TokenType::RightParen => {
                if paren_stack.pop().is_none() {
                    diagnostics.push(Diagnostic {
                        range: Range {
                            start: Position { line: token.line, character: token.start_col },
                            end: Position { line: token.line, character: token.end_col },
                        },
                        severity: Some(DiagnosticSeverity::ERROR),
                        source: Some("arcane".to_string()),
                        message: "Ongepaarde ')' - geen ooreenstemmende '(' gevind".to_string(),
                        ..Default::default()
                    });
                }
            }
            TokenType::LeftBrace => brace_stack.push(token),
            TokenType::RightBrace => {
                if brace_stack.pop().is_none() {
                    diagnostics.push(Diagnostic {
                        range: Range {
                            start: Position { line: token.line, character: token.start_col },
                            end: Position { line: token.line, character: token.end_col },
                        },
                        severity: Some(DiagnosticSeverity::ERROR),
                        source: Some("arcane".to_string()),
                        message: "Ongepaarde '}' - geen ooreenstemmende '{' gevind".to_string(),
                        ..Default::default()
                    });
                }
            }
            TokenType::LeftBracket => bracket_stack.push(token),
            TokenType::RightBracket => {
                if bracket_stack.pop().is_none() {
                    diagnostics.push(Diagnostic {
                        range: Range {
                            start: Position { line: token.line, character: token.start_col },
                            end: Position { line: token.line, character: token.end_col },
                        },
                        severity: Some(DiagnosticSeverity::ERROR),
                        source: Some("arcane".to_string()),
                        message: "Ongepaarde ']' - geen ooreenstemmende '[' gevind".to_string(),
                        ..Default::default()
                    });
                }
            }
            TokenType::Stel => {
                // Check for: stel <identifier> = <expr>
                if i + 1 < tokens.len() {
                    if !matches!(tokens[i + 1].token_type, TokenType::Identifier(_)) {
                        diagnostics.push(Diagnostic {
                            range: Range {
                                start: Position { line: token.line, character: token.start_col },
                                end: Position { line: token.line, character: token.end_col },
                            },
                            severity: Some(DiagnosticSeverity::ERROR),
                            source: Some("arcane".to_string()),
                            message: "Verwag veranderlike naam na 'stel'".to_string(),
                            ..Default::default()
                        });
                    } else if i + 2 < tokens.len() && !matches!(tokens[i + 2].token_type, TokenType::Equal) {
                        diagnostics.push(Diagnostic {
                            range: Range {
                                start: Position { line: tokens[i + 1].line, character: tokens[i + 1].end_col },
                                end: Position { line: tokens[i + 1].line, character: tokens[i + 1].end_col + 1 },
                            },
                            severity: Some(DiagnosticSeverity::ERROR),
                            source: Some("arcane".to_string()),
                            message: "Verwag '=' na veranderlike naam".to_string(),
                            ..Default::default()
                        });
                    }
                }
            }
            TokenType::Laat => {
                // Check for: laat [mut] <identifier> = <expr>
                let next_idx = if i + 1 < tokens.len()
                    && matches!(tokens[i + 1].token_type, TokenType::Mut)
                {
                    i + 2 // laat mut name
                } else {
                    i + 1 // laat name
                };

                if next_idx < tokens.len() {
                    if !matches!(tokens[next_idx].token_type, TokenType::Identifier(_)) {
                        diagnostics.push(Diagnostic {
                            range: Range {
                                start: Position { line: token.line, character: token.start_col },
                                end: Position { line: token.line, character: token.end_col },
                            },
                            severity: Some(DiagnosticSeverity::ERROR),
                            source: Some("arcane".to_string()),
                            message: "Verwag veranderlike naam na 'laat' (of 'laat mut')".to_string(),
                            ..Default::default()
                        });
                    } else if next_idx + 1 < tokens.len()
                        && !matches!(tokens[next_idx + 1].token_type, TokenType::Equal)
                    {
                        diagnostics.push(Diagnostic {
                            range: Range {
                                start: Position { line: tokens[next_idx].line, character: tokens[next_idx].end_col },
                                end: Position { line: tokens[next_idx].line, character: tokens[next_idx].end_col + 1 },
                            },
                            severity: Some(DiagnosticSeverity::ERROR),
                            source: Some("arcane".to_string()),
                            message: "Verwag '=' na veranderlike naam".to_string(),
                            ..Default::default()
                        });
                    }
                }
            }
            TokenType::Funksie => {
                // Check for: funksie <name>(<params>) {
                if i + 1 < tokens.len() {
                    if !matches!(tokens[i + 1].token_type, TokenType::Identifier(_)) {
                        diagnostics.push(Diagnostic {
                            range: Range {
                                start: Position { line: token.line, character: token.end_col },
                                end: Position { line: token.line, character: token.end_col + 1 },
                            },
                            severity: Some(DiagnosticSeverity::ERROR),
                            source: Some("arcane".to_string()),
                            message: "Verwag funksie naam na 'funksie'".to_string(),
                            ..Default::default()
                        });
                    } else if i + 2 < tokens.len()
                        && !matches!(tokens[i + 2].token_type, TokenType::LeftParen)
                    {
                        diagnostics.push(Diagnostic {
                            range: Range {
                                start: Position { line: tokens[i + 1].line, character: tokens[i + 1].end_col },
                                end: Position { line: tokens[i + 1].line, character: tokens[i + 1].end_col + 1 },
                            },
                            severity: Some(DiagnosticSeverity::ERROR),
                            source: Some("arcane".to_string()),
                            message: "Verwag '(' na funksie naam".to_string(),
                            ..Default::default()
                        });
                    }
                }
            }
            TokenType::Fn => {
                // Check for: fn(<params>) <expr> or fn(<params>) { ... }
                if i + 1 < tokens.len()
                    && !matches!(tokens[i + 1].token_type, TokenType::LeftParen)
                {
                    diagnostics.push(Diagnostic {
                        range: Range {
                            start: Position { line: token.line, character: token.end_col },
                            end: Position { line: token.line, character: token.end_col + 1 },
                        },
                        severity: Some(DiagnosticSeverity::ERROR),
                        source: Some("arcane".to_string()),
                        message: "Verwag '(' na 'fn'".to_string(),
                        ..Default::default()
                    });
                }
            }
            TokenType::Pas => {
                // Check for: pas(<expr>) {
                if i + 1 < tokens.len()
                    && !matches!(tokens[i + 1].token_type, TokenType::LeftParen)
                {
                    diagnostics.push(Diagnostic {
                        range: Range {
                            start: Position { line: token.line, character: token.end_col },
                            end: Position { line: token.line, character: token.end_col + 1 },
                        },
                        severity: Some(DiagnosticSeverity::ERROR),
                        source: Some("arcane".to_string()),
                        message: "Verwag '(' na 'pas'".to_string(),
                        ..Default::default()
                    });
                }
            }
            TokenType::Tipe => {
                // Check for: tipe <Name> {
                if i + 1 < tokens.len() {
                    if !matches!(tokens[i + 1].token_type, TokenType::Identifier(_)) {
                        diagnostics.push(Diagnostic {
                            range: Range {
                                start: Position { line: token.line, character: token.end_col },
                                end: Position { line: token.line, character: token.end_col + 1 },
                            },
                            severity: Some(DiagnosticSeverity::ERROR),
                            source: Some("arcane".to_string()),
                            message: "Verwag tipe naam na 'tipe'".to_string(),
                            ..Default::default()
                        });
                    } else if i + 2 < tokens.len()
                        && !matches!(tokens[i + 2].token_type, TokenType::LeftBrace)
                    {
                        diagnostics.push(Diagnostic {
                            range: Range {
                                start: Position { line: tokens[i + 1].line, character: tokens[i + 1].end_col },
                                end: Position { line: tokens[i + 1].line, character: tokens[i + 1].end_col + 1 },
                            },
                            severity: Some(DiagnosticSeverity::ERROR),
                            source: Some("arcane".to_string()),
                            message: "Verwag '{' na tipe naam".to_string(),
                            ..Default::default()
                        });
                    }
                }
            }
            TokenType::As | TokenType::Terwyl => {
                // Check for: as/terwyl (condition) {
                if i + 1 < tokens.len() && !matches!(tokens[i + 1].token_type, TokenType::LeftParen) {
                    diagnostics.push(Diagnostic {
                        range: Range {
                            start: Position { line: token.line, character: token.end_col },
                            end: Position { line: token.line, character: token.end_col + 1 },
                        },
                        severity: Some(DiagnosticSeverity::ERROR),
                        source: Some("arcane".to_string()),
                        message: format!("Verwag '(' na '{}'", token.lexeme),
                        ..Default::default()
                    });
                }
            }
            TokenType::Druk => {
                // Check for: druk(expr)
                if i + 1 < tokens.len() && !matches!(tokens[i + 1].token_type, TokenType::LeftParen) {
                    diagnostics.push(Diagnostic {
                        range: Range {
                            start: Position { line: token.line, character: token.end_col },
                            end: Position { line: token.line, character: token.end_col + 1 },
                        },
                        severity: Some(DiagnosticSeverity::ERROR),
                        source: Some("arcane".to_string()),
                        message: "Verwag '(' na 'druk'".to_string(),
                        ..Default::default()
                    });
                }
            }
            TokenType::Identifier(name) => {
                // Skip identifier checks inside type definitions (field names are not variables)
                if type_def_brace_depth > 0 {
                    i += 1;
                    continue;
                }

                // Check if this identifier is used as a variable (not being declared)
                let is_declaration = i > 0 && matches!(
                    tokens[i - 1].token_type,
                    TokenType::Stel | TokenType::Laat | TokenType::Mut
                );

                // Check for funksie declaration context
                let is_funksie_name = i > 0 && matches!(tokens[i - 1].token_type, TokenType::Funksie);

                // Check for tipe declaration context
                let is_type_name = i > 0 && matches!(tokens[i - 1].token_type, TokenType::Tipe);

                // Check for geval pattern context (inside pattern matching)
                let is_pattern = i > 0 && matches!(tokens[i - 1].token_type, TokenType::Geval);

                // Check if this is a function call (followed by '(')
                let is_function_call = i + 1 < tokens.len()
                    && matches!(tokens[i + 1].token_type, TokenType::LeftParen);

                if is_function_call && !is_funksie_name && !is_type_name {
                    // Valid if: built-in, user-defined funksie, ADT constructor, or variable holding function
                    let is_valid = builtin_funcs.contains(name.as_str())
                        || declared_funcs.contains(name)
                        || declared_vars.contains(name); // Variable could hold a function

                    if !is_valid {
                        diagnostics.push(Diagnostic {
                            range: Range {
                                start: Position { line: token.line, character: token.start_col },
                                end: Position { line: token.line, character: token.end_col },
                            },
                            severity: Some(DiagnosticSeverity::WARNING), // Warning, not error
                            source: Some("arcane".to_string()),
                            message: format!("Moontlike onbekende funksie: '{}'. Is dit gedefinieer?", name),
                            ..Default::default()
                        });
                    }
                } else if !is_declaration && !is_funksie_name && !is_type_name && !is_pattern {
                    if !declared_vars.contains(name) && !declared_funcs.contains(name) {
                        diagnostics.push(Diagnostic {
                            range: Range {
                                start: Position { line: token.line, character: token.start_col },
                                end: Position { line: token.line, character: token.end_col },
                            },
                            severity: Some(DiagnosticSeverity::ERROR),
                            source: Some("arcane".to_string()),
                            message: format!("Ongedefinieerde veranderlike: '{}'", name),
                            ..Default::default()
                        });
                    }
                }
            }
            _ => {}
        }

        i += 1;
    }

    // Report unclosed brackets
    for token in paren_stack {
        diagnostics.push(Diagnostic {
            range: Range {
                start: Position { line: token.line, character: token.start_col },
                end: Position { line: token.line, character: token.end_col },
            },
            severity: Some(DiagnosticSeverity::ERROR),
            source: Some("arcane".to_string()),
            message: "Ongeslote '(' - verwag ')'".to_string(),
            ..Default::default()
        });
    }

    for token in brace_stack {
        diagnostics.push(Diagnostic {
            range: Range {
                start: Position { line: token.line, character: token.start_col },
                end: Position { line: token.line, character: token.end_col },
            },
            severity: Some(DiagnosticSeverity::ERROR),
            source: Some("arcane".to_string()),
            message: "Ongeslote '{' - verwag '}'".to_string(),
            ..Default::default()
        });
    }

    for token in bracket_stack {
        diagnostics.push(Diagnostic {
            range: Range {
                start: Position { line: token.line, character: token.start_col },
                end: Position { line: token.line, character: token.end_col },
            },
            severity: Some(DiagnosticSeverity::ERROR),
            source: Some("arcane".to_string()),
            message: "Ongeslote '[' - verwag ']'".to_string(),
            ..Default::default()
        });
    }

    diagnostics
}

pub fn analyze_document(text: &str) -> Vec<Diagnostic> {
    let mut lexer = Lexer::new(text);
    let (tokens, mut diagnostics) = lexer.scan_tokens();

    let parse_diagnostics = parse_for_diagnostics(&tokens);
    diagnostics.extend(parse_diagnostics);

    diagnostics
}

pub fn get_hover_info(text: &str, position: Position) -> Option<Hover> {
    let mut lexer = Lexer::new(text);
    let (tokens, _) = lexer.scan_tokens();

    // Find the token at the position
    for token in tokens {
        if token.line == position.line
            && position.character >= token.start_col
            && position.character < token.end_col
        {
            let info = match &token.token_type {
                TokenType::Stel => Some((
                    "**stel** (sleutelwoord)\n\nVerklaar 'n nuwe veranderlike.\n\n```arcane\nstel x = 10\n```",
                    "Declare a new variable"
                )),
                TokenType::As => Some((
                    "**as** (sleutelwoord)\n\nVoorwaardelike stelling (if statement).\n\n```arcane\nas (x > 5) {\n    druk(x)\n}\n```",
                    "Conditional statement (if)"
                )),
                TokenType::Anders => Some((
                    "**anders** (sleutelwoord)\n\nAlternatiewe tak van 'as' stelling.\n\n```arcane\nas (x > 5) {\n    druk(\"groot\")\n} anders {\n    druk(\"klein\")\n}\n```",
                    "Else branch"
                )),
                TokenType::Terwyl => Some((
                    "**terwyl** (sleutelwoord)\n\nHerhaal terwyl voorwaarde waar is.\n\n```arcane\nterwyl (x > 0) {\n    druk(x)\n    stel x = x - 1\n}\n```",
                    "While loop"
                )),
                TokenType::Druk => Some((
                    "**druk** (funksie)\n\nDruk 'n waarde na die konsole.\n\n```arcane\ndruk(42)\ndruk(waar)\n```",
                    "Print to console"
                )),
                TokenType::Waar => Some((
                    "**waar** (boolean)\n\nBoolean waarde vir 'waar' (true).",
                    "Boolean true"
                )),
                TokenType::Vals => Some((
                    "**vals** (boolean)\n\nBoolean waarde vir 'vals' (false).",
                    "Boolean false"
                )),
                TokenType::Funksie => Some((
                    "**funksie** (sleutelwoord)\n\nDefinieer 'n funksie.\n\n```arcane\nfunksie groet(naam) {\n    druk(\"Hallo \" + naam)\n}\n```",
                    "Define a function"
                )),
                TokenType::Fn => Some((
                    "**fn** (sleutelwoord)\n\nSkep 'n anonieme funksie (lambda).\n\n```arcane\nlaat dubbel = fn(x) x * 2\n```",
                    "Create anonymous function (lambda)"
                )),
                TokenType::Gee => Some((
                    "**gee** (sleutelwoord)\n\nGee 'n waarde terug uit 'n funksie.\n\n```arcane\nfunksie kwadraat(x) {\n    gee x * x\n}\n```",
                    "Return value from function"
                )),
                TokenType::Laat => Some((
                    "**laat** (sleutelwoord)\n\nVerklaar 'n onveranderlike veranderlike.\n\n```arcane\nlaat x = 42\n```",
                    "Declare immutable variable"
                )),
                TokenType::Mut => Some((
                    "**mut** (sleutelwoord)\n\nVerklaar 'n veranderlike wat kan verander (gebruik met laat).\n\n```arcane\nlaat mut teller = 0\n```",
                    "Declare mutable variable"
                )),
                TokenType::Pas => Some((
                    "**pas** (sleutelwoord)\n\nPatroon-passing uitdrukking.\n\n```arcane\npas(waarde) {\n    geval Sommige(x) => x\n    geval Niks => 0\n}\n```",
                    "Pattern matching expression"
                )),
                TokenType::Geval => Some((
                    "**geval** (sleutelwoord)\n\n'n Arm in 'n pas-uitdrukking.\n\n```arcane\ngeval Sommige(x) => x * 2\n```",
                    "Match arm in pattern matching"
                )),
                TokenType::Tipe => Some((
                    "**tipe** (sleutelwoord)\n\nDefinieer 'n algebraïese datatipe.\n\n```arcane\ntipe Opsie {\n    Niks\n    Sommige(waarde)\n}\n```",
                    "Define algebraic data type"
                )),
                TokenType::Identifier(name) => {
                    match name.as_str() {
                        "kaart" => Some((
                            "**kaart** (funksie)\n\nPas 'n funksie op elke element van 'n lys toe (map).\n\n```arcane\nlaat dubbel = kaart([1, 2, 3], fn(x) x * 2)\n// Resultaat: [2, 4, 6]\n```",
                            "Apply function to each element (map)"
                        )),
                        "filter" => Some((
                            "**filter** (funksie)\n\nFiltreer elemente wat aan 'n predikaat voldoen.\n\n```arcane\nlaat ewe = filter([1, 2, 3, 4], fn(x) x % 2 == 0)\n// Resultaat: [2, 4]\n```",
                            "Filter elements matching predicate"
                        )),
                        "vou" => Some((
                            "**vou** (funksie)\n\nVou 'n lys tot 'n enkele waarde (fold/reduce).\n\n```arcane\nlaat som = vou([1, 2, 3], 0, fn(acc, x) acc + x)\n// Resultaat: 6\n```",
                            "Fold list to single value (reduce)"
                        )),
                        "vir_elk" => Some((
                            "**vir_elk** (funksie)\n\nVoer 'n aksie uit vir elke element.\n\n```arcane\nvir_elk([1, 2, 3], fn(x) druk(x))\n```",
                            "Execute action for each element (forEach)"
                        )),
                        "lengte" => Some((
                            "**lengte** (funksie)\n\nGee die lengte van 'n lys of string.\n\n```arcane\ndruk(lengte([1, 2, 3]))  // 3\ndruk(lengte(\"hallo\"))   // 5\n```",
                            "Get length of list or string"
                        )),
                        "kop" => Some((
                            "**kop** (funksie)\n\nGee die eerste element van 'n lys.\n\n```arcane\ndruk(kop([1, 2, 3]))  // 1\n```",
                            "Get first element (head)"
                        )),
                        "stert" => Some((
                            "**stert** (funksie)\n\nGee alles behalwe die eerste element.\n\n```arcane\ndruk(stert([1, 2, 3]))  // [2, 3]\n```",
                            "Get all but first element (tail)"
                        )),
                        "leeg" => Some((
                            "**leeg** (funksie)\n\nKyk of 'n lys leeg is.\n\n```arcane\ndruk(leeg([]))      // waar\ndruk(leeg([1, 2]))  // vals\n```",
                            "Check if list is empty"
                        )),
                        "voeg_by" => Some((
                            "**voeg_by** (funksie)\n\nVoeg 'n element voor 'n lys by (prepend).\n\n```arcane\ndruk(voeg_by(0, [1, 2, 3]))  // [0, 1, 2, 3]\n```",
                            "Prepend element to list"
                        )),
                        "heg_aan" => Some((
                            "**heg_aan** (funksie)\n\nVoeg 'n element aan die einde van 'n lys (append).\n\n```arcane\ndruk(heg_aan([1, 2, 3], 4))  // [1, 2, 3, 4]\n```",
                            "Append element to list"
                        )),
                        "ketting" => Some((
                            "**ketting** (funksie)\n\nVoeg twee lyste saam (concatenate).\n\n```arcane\ndruk(ketting([1, 2], [3, 4]))  // [1, 2, 3, 4]\n```",
                            "Concatenate two lists"
                        )),
                        "omgekeer" => Some((
                            "**omgekeer** (funksie)\n\nKeer 'n lys om (reverse).\n\n```arcane\ndruk(omgekeer([1, 2, 3]))  // [3, 2, 1]\n```",
                            "Reverse a list"
                        )),
                        _ => None,
                    }
                }
                _ => None,
            };

            if let Some((afrikaans, english)) = info {
                return Some(Hover {
                    contents: HoverContents::Markup(MarkupContent {
                        kind: MarkupKind::Markdown,
                        value: format!("{}\n\n---\n*{}*", afrikaans, english),
                    }),
                    range: Some(Range {
                        start: Position { line: token.line, character: token.start_col },
                        end: Position { line: token.line, character: token.end_col },
                    }),
                });
            }
        }
    }

    None
}

pub fn get_completions(text: &str, position: Position) -> Vec<CompletionItem> {
    let mut completions = vec![
        // Variable declarations
        CompletionItem {
            label: "laat".to_string(),
            kind: Some(CompletionItemKind::KEYWORD),
            detail: Some("Verklaar onveranderlike veranderlike".to_string()),
            insert_text: Some("laat ${1:naam} = ${0:waarde}".to_string()),
            insert_text_format: Some(InsertTextFormat::SNIPPET),
            ..Default::default()
        },
        CompletionItem {
            label: "stel".to_string(),
            kind: Some(CompletionItemKind::KEYWORD),
            detail: Some("Verklaar veranderlike (veranderlik)".to_string()),
            insert_text: Some("stel ${1:naam} = ${0:waarde}".to_string()),
            insert_text_format: Some(InsertTextFormat::SNIPPET),
            ..Default::default()
        },
        // Functions
        CompletionItem {
            label: "funksie".to_string(),
            kind: Some(CompletionItemKind::KEYWORD),
            detail: Some("Definieer funksie".to_string()),
            insert_text: Some("funksie ${1:naam}(${2:params}) {\n\t${0}\n}".to_string()),
            insert_text_format: Some(InsertTextFormat::SNIPPET),
            ..Default::default()
        },
        CompletionItem {
            label: "fn".to_string(),
            kind: Some(CompletionItemKind::KEYWORD),
            detail: Some("Lambda uitdrukking".to_string()),
            insert_text: Some("fn(${1:params}) ${0:uitdrukking}".to_string()),
            insert_text_format: Some(InsertTextFormat::SNIPPET),
            ..Default::default()
        },
        CompletionItem {
            label: "gee".to_string(),
            kind: Some(CompletionItemKind::KEYWORD),
            detail: Some("Gee waarde terug".to_string()),
            insert_text: Some("gee ${0}".to_string()),
            insert_text_format: Some(InsertTextFormat::SNIPPET),
            ..Default::default()
        },
        // Pattern matching
        CompletionItem {
            label: "pas".to_string(),
            kind: Some(CompletionItemKind::KEYWORD),
            detail: Some("Patroon-passing".to_string()),
            insert_text: Some("pas(${1:waarde}) {\n\tgeval ${2:patroon} => ${0}\n}".to_string()),
            insert_text_format: Some(InsertTextFormat::SNIPPET),
            ..Default::default()
        },
        CompletionItem {
            label: "geval".to_string(),
            kind: Some(CompletionItemKind::KEYWORD),
            detail: Some("Patroon arm".to_string()),
            insert_text: Some("geval ${1:patroon} => ${0}".to_string()),
            insert_text_format: Some(InsertTextFormat::SNIPPET),
            ..Default::default()
        },
        // Types
        CompletionItem {
            label: "tipe".to_string(),
            kind: Some(CompletionItemKind::KEYWORD),
            detail: Some("Definieer datatipe".to_string()),
            insert_text: Some("tipe ${1:Naam} {\n\t${0}\n}".to_string()),
            insert_text_format: Some(InsertTextFormat::SNIPPET),
            ..Default::default()
        },
        // Control flow
        CompletionItem {
            label: "as".to_string(),
            kind: Some(CompletionItemKind::KEYWORD),
            detail: Some("As-stelling (if)".to_string()),
            insert_text: Some("as (${1:voorwaarde}) {\n\t${0}\n}".to_string()),
            insert_text_format: Some(InsertTextFormat::SNIPPET),
            ..Default::default()
        },
        CompletionItem {
            label: "anders".to_string(),
            kind: Some(CompletionItemKind::KEYWORD),
            detail: Some("Anders-tak (else)".to_string()),
            insert_text: Some("anders {\n\t${0}\n}".to_string()),
            insert_text_format: Some(InsertTextFormat::SNIPPET),
            ..Default::default()
        },
        CompletionItem {
            label: "terwyl".to_string(),
            kind: Some(CompletionItemKind::KEYWORD),
            detail: Some("Terwyl-lus (while)".to_string()),
            insert_text: Some("terwyl (${1:voorwaarde}) {\n\t${0}\n}".to_string()),
            insert_text_format: Some(InsertTextFormat::SNIPPET),
            ..Default::default()
        },
        // Built-in functions
        CompletionItem {
            label: "druk".to_string(),
            kind: Some(CompletionItemKind::FUNCTION),
            detail: Some("Druk na konsole".to_string()),
            insert_text: Some("druk(${0})".to_string()),
            insert_text_format: Some(InsertTextFormat::SNIPPET),
            ..Default::default()
        },
        // List functions
        CompletionItem {
            label: "kaart".to_string(),
            kind: Some(CompletionItemKind::FUNCTION),
            detail: Some("Pas funksie op elke element toe".to_string()),
            insert_text: Some("kaart(${1:lys}, ${0:fn})".to_string()),
            insert_text_format: Some(InsertTextFormat::SNIPPET),
            ..Default::default()
        },
        CompletionItem {
            label: "filter".to_string(),
            kind: Some(CompletionItemKind::FUNCTION),
            detail: Some("Filtreer lys met predikaat".to_string()),
            insert_text: Some("filter(${1:lys}, ${0:predikaat})".to_string()),
            insert_text_format: Some(InsertTextFormat::SNIPPET),
            ..Default::default()
        },
        CompletionItem {
            label: "vou".to_string(),
            kind: Some(CompletionItemKind::FUNCTION),
            detail: Some("Vou lys tot enkele waarde".to_string()),
            insert_text: Some("vou(${1:lys}, ${2:begin}, ${0:fn})".to_string()),
            insert_text_format: Some(InsertTextFormat::SNIPPET),
            ..Default::default()
        },
        CompletionItem {
            label: "vir_elk".to_string(),
            kind: Some(CompletionItemKind::FUNCTION),
            detail: Some("Voer aksie uit vir elke element".to_string()),
            insert_text: Some("vir_elk(${1:lys}, fn(${2:x}) {\n\t${0}\n})".to_string()),
            insert_text_format: Some(InsertTextFormat::SNIPPET),
            ..Default::default()
        },
        CompletionItem {
            label: "lengte".to_string(),
            kind: Some(CompletionItemKind::FUNCTION),
            detail: Some("Kry lengte van lys of string".to_string()),
            insert_text: Some("lengte(${0})".to_string()),
            insert_text_format: Some(InsertTextFormat::SNIPPET),
            ..Default::default()
        },
        CompletionItem {
            label: "kop".to_string(),
            kind: Some(CompletionItemKind::FUNCTION),
            detail: Some("Kry eerste element van lys".to_string()),
            insert_text: Some("kop(${0})".to_string()),
            insert_text_format: Some(InsertTextFormat::SNIPPET),
            ..Default::default()
        },
        CompletionItem {
            label: "stert".to_string(),
            kind: Some(CompletionItemKind::FUNCTION),
            detail: Some("Kry stert van lys (sonder kop)".to_string()),
            insert_text: Some("stert(${0})".to_string()),
            insert_text_format: Some(InsertTextFormat::SNIPPET),
            ..Default::default()
        },
        CompletionItem {
            label: "leeg".to_string(),
            kind: Some(CompletionItemKind::FUNCTION),
            detail: Some("Kyk of lys leeg is".to_string()),
            insert_text: Some("leeg(${0})".to_string()),
            insert_text_format: Some(InsertTextFormat::SNIPPET),
            ..Default::default()
        },
        CompletionItem {
            label: "voeg_by".to_string(),
            kind: Some(CompletionItemKind::FUNCTION),
            detail: Some("Voeg element voor lys by".to_string()),
            insert_text: Some("voeg_by(${1:element}, ${0:lys})".to_string()),
            insert_text_format: Some(InsertTextFormat::SNIPPET),
            ..Default::default()
        },
        CompletionItem {
            label: "heg_aan".to_string(),
            kind: Some(CompletionItemKind::FUNCTION),
            detail: Some("Voeg element aan einde van lys".to_string()),
            insert_text: Some("heg_aan(${1:lys}, ${0:element})".to_string()),
            insert_text_format: Some(InsertTextFormat::SNIPPET),
            ..Default::default()
        },
        CompletionItem {
            label: "ketting".to_string(),
            kind: Some(CompletionItemKind::FUNCTION),
            detail: Some("Voeg twee lyste saam".to_string()),
            insert_text: Some("ketting(${1:lys1}, ${0:lys2})".to_string()),
            insert_text_format: Some(InsertTextFormat::SNIPPET),
            ..Default::default()
        },
        CompletionItem {
            label: "omgekeer".to_string(),
            kind: Some(CompletionItemKind::FUNCTION),
            detail: Some("Keer lys om".to_string()),
            insert_text: Some("omgekeer(${0:lys})".to_string()),
            insert_text_format: Some(InsertTextFormat::SNIPPET),
            ..Default::default()
        },
        // Booleans
        CompletionItem {
            label: "waar".to_string(),
            kind: Some(CompletionItemKind::CONSTANT),
            detail: Some("Boolean waar (true)".to_string()),
            ..Default::default()
        },
        CompletionItem {
            label: "vals".to_string(),
            kind: Some(CompletionItemKind::CONSTANT),
            detail: Some("Boolean vals (false)".to_string()),
            ..Default::default()
        },
    ];

    // Extract variable names from the document
    let mut lexer = Lexer::new(text);
    let (tokens, _) = lexer.scan_tokens();

    let mut seen_vars = std::collections::HashSet::new();
    let mut seen_funcs = std::collections::HashSet::new();
    let mut i = 0;
    while i < tokens.len() {
        // Variable declarations (stel or laat)
        if matches!(tokens[i].token_type, TokenType::Stel | TokenType::Laat) {
            // Check for 'mut' keyword after 'laat'
            let name_offset = if matches!(tokens[i].token_type, TokenType::Laat)
                && i + 1 < tokens.len()
                && matches!(tokens[i + 1].token_type, TokenType::Mut)
            {
                2 // laat mut name
            } else {
                1 // stel name or laat name
            };

            if i + name_offset < tokens.len() {
                if let TokenType::Identifier(name) = &tokens[i + name_offset].token_type {
                    if !seen_vars.contains(name) {
                        seen_vars.insert(name.clone());
                        completions.push(CompletionItem {
                            label: name.clone(),
                            kind: Some(CompletionItemKind::VARIABLE),
                            detail: Some("Veranderlike".to_string()),
                            ..Default::default()
                        });
                    }
                }
            }
        }
        // Function declarations
        if matches!(tokens[i].token_type, TokenType::Funksie) {
            if i + 1 < tokens.len() {
                if let TokenType::Identifier(name) = &tokens[i + 1].token_type {
                    if !seen_funcs.contains(name) {
                        seen_funcs.insert(name.clone());
                        completions.push(CompletionItem {
                            label: name.clone(),
                            kind: Some(CompletionItemKind::FUNCTION),
                            detail: Some("Funksie".to_string()),
                            insert_text: Some(format!("{}(${{0}})", name)),
                            insert_text_format: Some(InsertTextFormat::SNIPPET),
                            ..Default::default()
                        });
                    }
                }
            }
        }
        i += 1;
    }

    // Extract type constructors from document
    let mut seen_constructors = std::collections::HashSet::new();
    i = 0;
    while i < tokens.len() {
        if matches!(tokens[i].token_type, TokenType::Tipe) {
            // Skip type name and opening brace
            if i + 2 < tokens.len() && matches!(tokens[i + 2].token_type, TokenType::LeftBrace) {
                let mut k = i + 3;
                while k < tokens.len() && !matches!(tokens[k].token_type, TokenType::RightBrace) {
                    if let TokenType::Identifier(name) = &tokens[k].token_type {
                        // Constructor names typically start uppercase
                        if name.chars().next().map(|c| c.is_uppercase()).unwrap_or(false)
                            && !seen_constructors.contains(name)
                        {
                            seen_constructors.insert(name.clone());
                            completions.push(CompletionItem {
                                label: name.clone(),
                                kind: Some(CompletionItemKind::CONSTRUCTOR),
                                detail: Some("Tipe konstruktor".to_string()),
                                insert_text: Some(format!("{}(${{0}})", name)),
                                insert_text_format: Some(InsertTextFormat::SNIPPET),
                                ..Default::default()
                            });
                        }
                    }
                    k += 1;
                }
            }
        }
        i += 1;
    }

    completions
}
