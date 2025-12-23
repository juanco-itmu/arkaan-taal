use crate::ast::{Expr, LambdaBody, Literal, MatchArm, Pattern, Stmt, TypeConstructor};
use crate::token::{Token, TokenType};

pub struct Parser {
    tokens: Vec<Token>,
    current: usize,
}

impl Parser {
    pub fn new(tokens: Vec<Token>) -> Self {
        Parser { tokens, current: 0 }
    }

    pub fn parse(&mut self) -> Result<Vec<Stmt>, String> {
        let mut statements = Vec::new();

        while !self.is_at_end() {
            self.skip_newlines();
            if !self.is_at_end() {
                statements.push(self.declaration()?);
            }
        }

        Ok(statements)
    }

    fn declaration(&mut self) -> Result<Stmt, String> {
        if self.check(&TokenType::Funksie) {
            self.advance();
            self.fun_declaration()
        } else if self.check(&TokenType::Stel) {
            self.advance();
            self.var_declaration(true)  // stel = mutable
        } else if self.check(&TokenType::Laat) {
            self.advance();
            self.var_declaration(false)  // laat = immutable
        } else if self.check(&TokenType::Tipe) {
            self.advance();
            self.type_declaration()
        } else {
            self.statement()
        }
    }

    fn type_declaration(&mut self) -> Result<Stmt, String> {
        let name = self.consume_identifier("Verwag tipe naam.")?;
        self.skip_newlines();
        self.consume(&TokenType::LeftBrace, "Verwag '{' na tipe naam.")?;
        self.skip_newlines();

        let mut constructors = Vec::new();

        while !self.check(&TokenType::RightBrace) && !self.is_at_end() {
            let constructor = self.parse_constructor()?;
            constructors.push(constructor);
            self.skip_newlines();
        }

        self.consume(&TokenType::RightBrace, "Verwag '}' na tipe definisie.")?;

        if constructors.is_empty() {
            return Err(format!(
                "Tipe '{}' moet ten minste een konstruktor hê.",
                name
            ));
        }

        Ok(Stmt::TypeDecl { name, constructors })
    }

    fn parse_constructor(&mut self) -> Result<TypeConstructor, String> {
        let name = self.consume_identifier("Verwag konstruktor naam.")?;

        let fields = if self.check(&TokenType::LeftParen) {
            self.advance();
            let mut fields = Vec::new();

            if !self.check(&TokenType::RightParen) {
                loop {
                    fields.push(self.consume_identifier("Verwag veld naam.")?);
                    if !self.check(&TokenType::Comma) {
                        break;
                    }
                    self.advance();
                }
            }

            self.consume(&TokenType::RightParen, "Verwag ')' na konstruktor velde.")?;
            fields
        } else {
            Vec::new()
        };

        Ok(TypeConstructor { name, fields })
    }

    fn fun_declaration(&mut self) -> Result<Stmt, String> {
        let name = self.consume_identifier("Verwag funksie naam.")?;

        self.consume(&TokenType::LeftParen, "Verwag '(' na funksie naam.")?;
        let mut params = Vec::new();

        if !self.check(&TokenType::RightParen) {
            loop {
                if params.len() >= 255 {
                    return Err(format!(
                        "Kan nie meer as 255 parameters hê nie. (lyn {})",
                        self.peek().line
                    ));
                }
                params.push(self.consume_identifier("Verwag parameter naam.")?);

                if !self.check(&TokenType::Comma) {
                    break;
                }
                self.advance(); // consume comma
            }
        }

        self.consume(&TokenType::RightParen, "Verwag ')' na parameters.")?;
        self.skip_newlines();
        self.consume(&TokenType::LeftBrace, "Verwag '{' voor funksie liggaam.")?;
        let body = self.block()?;

        Ok(Stmt::FunDecl { name, params, body })
    }

    fn var_declaration(&mut self, is_mutable: bool) -> Result<Stmt, String> {
        let name = self.consume_identifier("Verwag veranderlike naam.")?;
        self.consume(&TokenType::Equal, "Verwag '=' na veranderlike naam.")?;
        let initializer = self.expression()?;
        self.consume_newline_or_eof()?;
        Ok(Stmt::VarDecl { name, initializer, is_mutable })
    }

    fn statement(&mut self) -> Result<Stmt, String> {
        if self.check(&TokenType::Druk) {
            self.advance();
            self.print_statement()
        } else if self.check(&TokenType::Gee) {
            self.advance();
            self.return_statement()
        } else if self.check(&TokenType::As) {
            self.advance();
            self.if_statement()
        } else if self.check(&TokenType::Terwyl) {
            self.advance();
            self.while_statement()
        } else if self.check(&TokenType::LeftBrace) {
            self.advance();
            Ok(Stmt::Block(self.block()?))
        } else {
            self.expression_statement()
        }
    }

    fn return_statement(&mut self) -> Result<Stmt, String> {
        let value = if self.check(&TokenType::Newline) || self.is_at_end() || self.check(&TokenType::RightBrace) {
            None
        } else {
            Some(self.expression()?)
        };

        self.consume_newline_or_eof()?;
        Ok(Stmt::Return { value })
    }

    fn print_statement(&mut self) -> Result<Stmt, String> {
        self.consume(&TokenType::LeftParen, "Verwag '(' na 'druk'.")?;
        let value = self.expression()?;
        self.consume(&TokenType::RightParen, "Verwag ')' na uitdrukking.")?;
        self.consume_newline_or_eof()?;
        Ok(Stmt::Print(value))
    }

    fn if_statement(&mut self) -> Result<Stmt, String> {
        self.consume(&TokenType::LeftParen, "Verwag '(' na 'as'.")?;
        let condition = self.expression()?;
        self.consume(&TokenType::RightParen, "Verwag ')' na voorwaarde.")?;
        self.skip_newlines();

        self.consume(&TokenType::LeftBrace, "Verwag '{' na 'as' voorwaarde.")?;
        let then_branch = Stmt::Block(self.block()?);
        self.skip_newlines();

        let else_branch = if self.check(&TokenType::Anders) {
            self.advance();
            self.skip_newlines();
            self.consume(&TokenType::LeftBrace, "Verwag '{' na 'anders'.")?;
            Some(Box::new(Stmt::Block(self.block()?)))
        } else {
            None
        };

        Ok(Stmt::If {
            condition,
            then_branch: Box::new(then_branch),
            else_branch,
        })
    }

    fn while_statement(&mut self) -> Result<Stmt, String> {
        self.consume(&TokenType::LeftParen, "Verwag '(' na 'terwyl'.")?;
        let condition = self.expression()?;
        self.consume(&TokenType::RightParen, "Verwag ')' na voorwaarde.")?;
        self.skip_newlines();

        self.consume(&TokenType::LeftBrace, "Verwag '{' na 'terwyl' voorwaarde.")?;
        let body = Stmt::Block(self.block()?);

        Ok(Stmt::While {
            condition,
            body: Box::new(body),
        })
    }

    fn block(&mut self) -> Result<Vec<Stmt>, String> {
        let mut statements = Vec::new();

        self.skip_newlines();
        while !self.check(&TokenType::RightBrace) && !self.is_at_end() {
            statements.push(self.declaration()?);
            self.skip_newlines();
        }

        self.consume(&TokenType::RightBrace, "Verwag '}' na blok.")?;
        Ok(statements)
    }

    fn expression_statement(&mut self) -> Result<Stmt, String> {
        let expr = self.expression()?;
        self.consume_newline_or_eof()?;
        Ok(Stmt::Expression(expr))
    }

    fn expression(&mut self) -> Result<Expr, String> {
        self.assignment()
    }

    fn assignment(&mut self) -> Result<Expr, String> {
        let expr = self.or()?;

        if self.check(&TokenType::Equal) {
            self.advance();
            let value = self.assignment()?;

            if let Expr::Variable(name) = expr {
                return Ok(Expr::Assign {
                    name,
                    value: Box::new(value),
                });
            }

            return Err("Ongeldige toewysing teiken.".to_string());
        }

        Ok(expr)
    }

    fn or(&mut self) -> Result<Expr, String> {
        let mut expr = self.and()?;

        while self.check(&TokenType::Or) {
            let operator = self.advance().clone();
            let right = self.and()?;
            expr = Expr::Binary {
                left: Box::new(expr),
                operator,
                right: Box::new(right),
            };
        }

        Ok(expr)
    }

    fn and(&mut self) -> Result<Expr, String> {
        let mut expr = self.equality()?;

        while self.check(&TokenType::And) {
            let operator = self.advance().clone();
            let right = self.equality()?;
            expr = Expr::Binary {
                left: Box::new(expr),
                operator,
                right: Box::new(right),
            };
        }

        Ok(expr)
    }

    fn equality(&mut self) -> Result<Expr, String> {
        let mut expr = self.comparison()?;

        while self.check(&TokenType::EqualEqual) || self.check(&TokenType::BangEqual) {
            let operator = self.advance().clone();
            let right = self.comparison()?;
            expr = Expr::Binary {
                left: Box::new(expr),
                operator,
                right: Box::new(right),
            };
        }

        Ok(expr)
    }

    fn comparison(&mut self) -> Result<Expr, String> {
        let mut expr = self.term()?;

        while self.check(&TokenType::Less)
            || self.check(&TokenType::LessEqual)
            || self.check(&TokenType::Greater)
            || self.check(&TokenType::GreaterEqual)
        {
            let operator = self.advance().clone();
            let right = self.term()?;
            expr = Expr::Binary {
                left: Box::new(expr),
                operator,
                right: Box::new(right),
            };
        }

        Ok(expr)
    }

    fn term(&mut self) -> Result<Expr, String> {
        let mut expr = self.factor()?;

        while self.check(&TokenType::Plus) || self.check(&TokenType::Minus) {
            let operator = self.advance().clone();
            let right = self.factor()?;
            expr = Expr::Binary {
                left: Box::new(expr),
                operator,
                right: Box::new(right),
            };
        }

        Ok(expr)
    }

    fn factor(&mut self) -> Result<Expr, String> {
        let mut expr = self.unary()?;

        while self.check(&TokenType::Star)
            || self.check(&TokenType::Slash)
            || self.check(&TokenType::Percent)
        {
            let operator = self.advance().clone();
            let right = self.unary()?;
            expr = Expr::Binary {
                left: Box::new(expr),
                operator,
                right: Box::new(right),
            };
        }

        Ok(expr)
    }

    fn unary(&mut self) -> Result<Expr, String> {
        if self.check(&TokenType::Bang) || self.check(&TokenType::Minus) {
            let operator = self.advance().clone();
            let right = self.unary()?;
            return Ok(Expr::Unary {
                operator,
                right: Box::new(right),
            });
        }

        self.call()
    }

    fn call(&mut self) -> Result<Expr, String> {
        let mut expr = self.primary()?;

        loop {
            if self.check(&TokenType::LeftParen) {
                self.advance();
                expr = self.finish_call(expr)?;
            } else if self.check(&TokenType::LeftBracket) {
                self.advance();
                let index = self.expression()?;
                self.consume(&TokenType::RightBracket, "Verwag ']' na indeks.")?;
                expr = Expr::Index {
                    object: Box::new(expr),
                    index: Box::new(index),
                };
            } else {
                break;
            }
        }

        Ok(expr)
    }

    fn finish_call(&mut self, callee: Expr) -> Result<Expr, String> {
        let mut arguments = Vec::new();

        if !self.check(&TokenType::RightParen) {
            loop {
                if arguments.len() >= 255 {
                    return Err(format!(
                        "Kan nie meer as 255 argumente hê nie. (lyn {})",
                        self.peek().line
                    ));
                }
                arguments.push(self.expression()?);

                if !self.check(&TokenType::Comma) {
                    break;
                }
                self.advance(); // consume comma
            }
        }

        self.consume(&TokenType::RightParen, "Verwag ')' na argumente.")?;

        Ok(Expr::Call {
            callee: Box::new(callee),
            arguments,
        })
    }

    fn primary(&mut self) -> Result<Expr, String> {
        if self.check(&TokenType::Waar) {
            self.advance();
            return Ok(Expr::Literal(Literal::Boolean(true)));
        }

        if self.check(&TokenType::Vals) {
            self.advance();
            return Ok(Expr::Literal(Literal::Boolean(false)));
        }

        if let TokenType::Number(n) = &self.peek().token_type {
            let value = *n;
            self.advance();
            return Ok(Expr::Literal(Literal::Number(value)));
        }

        if let TokenType::Str(s) = &self.peek().token_type {
            let value = s.clone();
            self.advance();
            return Ok(Expr::Literal(Literal::String(value)));
        }

        if let TokenType::Identifier(name) = &self.peek().token_type {
            let name = name.clone();
            self.advance();
            return Ok(Expr::Variable(name));
        }

        if self.check(&TokenType::LeftParen) {
            self.advance();
            let expr = self.expression()?;
            self.consume(&TokenType::RightParen, "Verwag ')' na uitdrukking.")?;
            return Ok(Expr::Grouping(Box::new(expr)));
        }

        // List literal: [a, b, c]
        if self.check(&TokenType::LeftBracket) {
            self.advance();
            let mut elements = Vec::new();

            if !self.check(&TokenType::RightBracket) {
                loop {
                    elements.push(self.expression()?);
                    if !self.check(&TokenType::Comma) {
                        break;
                    }
                    self.advance(); // consume comma
                }
            }

            self.consume(&TokenType::RightBracket, "Verwag ']' na lys elemente.")?;
            return Ok(Expr::List(elements));
        }

        // Lambda expression: fn(params) expr or fn(params) { stmts }
        if self.check(&TokenType::Fn) {
            self.advance();
            return self.lambda();
        }

        // Pattern matching expression: pas(value) { ... }
        if self.check(&TokenType::Pas) {
            self.advance();
            return self.match_expr();
        }

        // Inline if expression: as(condition) then_expr anders else_expr
        if self.check(&TokenType::As) {
            self.advance();
            return self.if_expr();
        }

        Err(format!(
            "Verwag uitdrukking op lyn {}.",
            self.peek().line
        ))
    }

    fn match_expr(&mut self) -> Result<Expr, String> {
        self.consume(&TokenType::LeftParen, "Verwag '(' na 'pas'.")?;
        let value = self.expression()?;
        self.consume(&TokenType::RightParen, "Verwag ')' na waarde.")?;
        self.skip_newlines();
        self.consume(&TokenType::LeftBrace, "Verwag '{' voor pas-gevalle.")?;
        self.skip_newlines();

        let mut arms = Vec::new();

        while !self.check(&TokenType::RightBrace) && !self.is_at_end() {
            self.consume(&TokenType::Geval, "Verwag 'geval' in pas-uitdrukking.")?;
            let pattern = self.parse_pattern()?;
            self.consume(&TokenType::Arrow, "Verwag '=>' na patroon.")?;
            let body = self.expression()?;
            self.skip_newlines();

            arms.push(MatchArm {
                pattern,
                body: Box::new(body),
            });
        }

        self.consume(&TokenType::RightBrace, "Verwag '}' na pas-gevalle.")?;

        if arms.is_empty() {
            return Err("Pas-uitdrukking moet ten minste een geval hê.".to_string());
        }

        Ok(Expr::Match {
            value: Box::new(value),
            arms,
        })
    }

    fn if_expr(&mut self) -> Result<Expr, String> {
        self.consume(&TokenType::LeftParen, "Verwag '(' na 'as'.")?;
        let condition = self.expression()?;
        self.consume(&TokenType::RightParen, "Verwag ')' na voorwaarde.")?;

        let then_branch = self.expression()?;

        self.consume(&TokenType::Anders, "Verwag 'anders' in as-uitdrukking.")?;

        let else_branch = self.expression()?;

        Ok(Expr::IfExpr {
            condition: Box::new(condition),
            then_branch: Box::new(then_branch),
            else_branch: Box::new(else_branch),
        })
    }

    fn parse_pattern(&mut self) -> Result<Pattern, String> {
        // Wildcard: _
        if self.check(&TokenType::Underscore) {
            self.advance();
            return Ok(Pattern::Wildcard);
        }

        // Boolean literals
        if self.check(&TokenType::Waar) {
            self.advance();
            return Ok(Pattern::Literal(Literal::Boolean(true)));
        }

        if self.check(&TokenType::Vals) {
            self.advance();
            return Ok(Pattern::Literal(Literal::Boolean(false)));
        }

        // Number literal
        if let TokenType::Number(n) = &self.peek().token_type {
            let value = *n;
            self.advance();
            return Ok(Pattern::Literal(Literal::Number(value)));
        }

        // String literal
        if let TokenType::Str(s) = &self.peek().token_type {
            let value = s.clone();
            self.advance();
            return Ok(Pattern::Literal(Literal::String(value)));
        }

        // Identifier - could be a variable binding or a constructor
        if let TokenType::Identifier(name) = &self.peek().token_type {
            let name = name.clone();
            self.advance();

            // Check if it's a constructor (followed by parens)
            if self.check(&TokenType::LeftParen) {
                self.advance();
                let mut fields = Vec::new();

                if !self.check(&TokenType::RightParen) {
                    loop {
                        fields.push(self.parse_pattern()?);
                        if !self.check(&TokenType::Comma) {
                            break;
                        }
                        self.advance();
                    }
                }

                self.consume(&TokenType::RightParen, "Verwag ')' na konstruktor patrone.")?;
                return Ok(Pattern::Constructor { name, fields });
            }

            // Check if it's a unit constructor (uppercase first letter convention)
            // For now, we treat any identifier that starts with uppercase as a constructor
            if name.chars().next().map(|c| c.is_uppercase()).unwrap_or(false) {
                // Could be a unit constructor - we'll treat it as a constructor with no fields
                return Ok(Pattern::Constructor {
                    name,
                    fields: Vec::new(),
                });
            }

            // Otherwise it's a variable binding
            return Ok(Pattern::Variable(name));
        }

        Err(format!(
            "Verwag patroon op lyn {}.",
            self.peek().line
        ))
    }

    fn lambda(&mut self) -> Result<Expr, String> {
        self.consume(&TokenType::LeftParen, "Verwag '(' na 'fn'.")?;
        let mut params = Vec::new();

        if !self.check(&TokenType::RightParen) {
            loop {
                if params.len() >= 255 {
                    return Err(format!(
                        "Kan nie meer as 255 parameters hê nie. (lyn {})",
                        self.peek().line
                    ));
                }
                params.push(self.consume_identifier("Verwag parameter naam.")?);

                if !self.check(&TokenType::Comma) {
                    break;
                }
                self.advance(); // consume comma
            }
        }

        self.consume(&TokenType::RightParen, "Verwag ')' na parameters.")?;

        // Check if body is a block or an expression
        let body = if self.check(&TokenType::LeftBrace) {
            self.advance();
            let stmts = self.block()?;
            LambdaBody::Block(stmts)
        } else {
            let expr = self.expression()?;
            LambdaBody::Expr(Box::new(expr))
        };

        Ok(Expr::Lambda { params, body })
    }

    // Helper methods

    fn peek(&self) -> &Token {
        &self.tokens[self.current]
    }

    fn is_at_end(&self) -> bool {
        matches!(self.peek().token_type, TokenType::Eof)
    }

    fn advance(&mut self) -> &Token {
        if !self.is_at_end() {
            self.current += 1;
        }
        &self.tokens[self.current - 1]
    }

    fn check(&self, token_type: &TokenType) -> bool {
        if self.is_at_end() {
            return false;
        }
        std::mem::discriminant(&self.peek().token_type) == std::mem::discriminant(token_type)
    }

    fn consume(&mut self, token_type: &TokenType, message: &str) -> Result<&Token, String> {
        if self.check(token_type) {
            Ok(self.advance())
        } else {
            Err(format!("{} (lyn {})", message, self.peek().line))
        }
    }

    fn consume_identifier(&mut self, message: &str) -> Result<String, String> {
        if let TokenType::Identifier(name) = &self.peek().token_type {
            let name = name.clone();
            self.advance();
            Ok(name)
        } else {
            Err(format!("{} (lyn {})", message, self.peek().line))
        }
    }

    fn consume_newline_or_eof(&mut self) -> Result<(), String> {
        if self.check(&TokenType::Newline) {
            self.advance();
            Ok(())
        } else if self.is_at_end() || self.check(&TokenType::RightBrace) {
            Ok(())
        } else {
            Err(format!(
                "Verwag nuwe lyn na stelling. (lyn {})",
                self.peek().line
            ))
        }
    }

    fn skip_newlines(&mut self) {
        while self.check(&TokenType::Newline) {
            self.advance();
        }
    }
}
