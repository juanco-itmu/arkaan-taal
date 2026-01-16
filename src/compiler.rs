use crate::ast::{Expr, LambdaBody, Literal, Pattern, Stmt};
use crate::bytecode::{Chunk, OpCode};
use crate::token::TokenType;
use crate::value::{Function, TypeConstructorDef, UpvalueDescriptor, Value};
use std::collections::HashSet;
use std::rc::Rc;

// Hidden local variable names used for pattern matching
const MATCH_SCRUTINEE: &str = "$match";
const CTOR_HIDDEN_LOCAL: &str = "$ctor";

#[derive(Debug, Clone)]
struct Local {
    name: String,
    depth: usize,
    is_captured: bool, // True if this local is captured by a closure
}

#[derive(Debug, Clone)]
struct CompilerUpvalue {
    index: usize,
    is_local: bool,
}

#[derive(Debug, Clone, Copy, PartialEq)]
enum FunctionType {
    Script,
    Function,
}

enum VarLocation {
    Local(usize),
    Upvalue(usize),
    Global,
}

struct FunctionCompiler {
    function_name: String,
    function_type: FunctionType,
    chunk: Chunk,
    locals: Vec<Local>,
    upvalues: Vec<CompilerUpvalue>, // Captured variables
    scope_depth: usize,
    arity: usize,
}

impl FunctionCompiler {
    fn new(name: String, function_type: FunctionType, arity: usize) -> Self {
        let mut compiler = FunctionCompiler {
            function_name: name,
            function_type,
            chunk: Chunk::new(),
            locals: Vec::new(),
            upvalues: Vec::new(),
            scope_depth: 0,
            arity,
        };

        // Reserve slot 0 for the function itself (or empty for scripts)
        if function_type == FunctionType::Function {
            compiler.locals.push(Local {
                name: String::new(),
                depth: 0,
                is_captured: false,
            });
        }

        compiler
    }
}

pub struct Compiler {
    current: FunctionCompiler,
    enclosing: Option<Box<Compiler>>,
    functions: Vec<Rc<Chunk>>,
    exported_symbols: HashSet<String>,
}

impl Compiler {
    pub fn new() -> Self {
        Compiler {
            current: FunctionCompiler::new(String::from("<script>"), FunctionType::Script, 0),
            enclosing: None,
            functions: Vec::new(),
            exported_symbols: HashSet::new(),
        }
    }

    pub fn get_exports(&self) -> &HashSet<String> {
        &self.exported_symbols
    }

    pub fn compile(&mut self, statements: Vec<Stmt>) -> Result<(Chunk, Vec<Rc<Chunk>>), String> {
        for stmt in statements {
            self.compile_stmt(stmt)?;
        }
        let nil_idx = self.add_constant(Value::Nil);
        self.emit(OpCode::Constant(nil_idx));
        self.emit(OpCode::Return);

        let main_chunk = self.current.chunk.clone();
        let functions = self.functions.clone();
        Ok((main_chunk, functions))
    }

    fn begin_scope(&mut self) {
        self.current.scope_depth += 1;
    }

    fn end_scope(&mut self) {
        self.current.scope_depth -= 1;

        // Pop locals that are going out of scope
        while !self.current.locals.is_empty()
            && self.current.locals.last().unwrap().depth > self.current.scope_depth
        {
            self.current.locals.pop();
            self.emit(OpCode::Pop);
        }
    }

    fn add_local(&mut self, name: String) -> Result<(), String> {
        // Check for duplicate in current scope
        for local in self.current.locals.iter().rev() {
            if local.depth < self.current.scope_depth {
                break;
            }
            if local.name == name {
                return Err(format!(
                    "Konstante '{}' is reeds in hierdie omvang gedefinieer.",
                    name
                ));
            }
        }

        self.current.locals.push(Local {
            name,
            depth: self.current.scope_depth,
            is_captured: false,
        });
        Ok(())
    }

    fn resolve_local(&self, name: &str) -> Option<usize> {
        for (i, local) in self.current.locals.iter().enumerate().rev() {
            if local.name == name {
                return Some(i);
            }
        }
        None
    }

    fn resolve_upvalue(&mut self, name: &str) -> Option<usize> {
        // Check if there's an enclosing compiler
        if self.enclosing.is_none() {
            return None;
        }

        // Try to resolve as a local in the enclosing scope
        let enclosing = self.enclosing.as_mut().unwrap();
        if let Some(local_idx) = enclosing.resolve_local(name) {
            // Mark the local as captured
            enclosing.current.locals[local_idx].is_captured = true;
            return Some(self.add_upvalue(local_idx, true));
        }

        // Try to resolve as an upvalue in the enclosing scope (for nested closures)
        if let Some(upvalue_idx) = enclosing.resolve_upvalue(name) {
            return Some(self.add_upvalue(upvalue_idx, false));
        }

        None
    }

    fn add_upvalue(&mut self, index: usize, is_local: bool) -> usize {
        // Check if we already have this upvalue
        for (i, upvalue) in self.current.upvalues.iter().enumerate() {
            if upvalue.index == index && upvalue.is_local == is_local {
                return i;
            }
        }

        // Add new upvalue
        self.current
            .upvalues
            .push(CompilerUpvalue { index, is_local });
        self.current.upvalues.len() - 1
    }

    fn resolve_variable(&mut self, name: &str) -> VarLocation {
        if let Some(slot) = self.resolve_local(name) {
            VarLocation::Local(slot)
        } else if let Some(upvalue) = self.resolve_upvalue(name) {
            VarLocation::Upvalue(upvalue)
        } else {
            VarLocation::Global
        }
    }

    /// Compiles a callable (function or lambda) with shared setup/teardown logic.
    /// The `compile_body` closure handles the specific body compilation.
    fn compile_callable<F>(
        &mut self,
        name: String,
        params: Vec<String>,
        compile_body: F,
    ) -> Result<(Rc<Chunk>, usize, Vec<UpvalueDescriptor>), String>
    where
        F: FnOnce(&mut Self) -> Result<(), String>,
    {
        let arity = params.len();

        // Save current compiler state
        let old_current = std::mem::replace(
            &mut self.current,
            FunctionCompiler::new(name, FunctionType::Function, arity),
        );
        let old_enclosing = self.enclosing.take();

        // Create enclosing chain
        self.enclosing = Some(Box::new(Compiler {
            current: old_current,
            enclosing: old_enclosing,
            functions: Vec::new(),
            exported_symbols: HashSet::new(),
        }));

        // Begin function scope
        self.begin_scope();

        // Bind parameters as locals
        for param in params {
            self.add_local(param)?;
        }

        // Compile body using provided closure
        compile_body(self)?;

        // Get the compiled function chunk and upvalue info
        let function_chunk = self.current.chunk.clone();
        let upvalues = self.extract_upvalues();

        // Restore compiler state
        if let Some(enclosing) = self.enclosing.take() {
            self.current = enclosing.current;
            self.enclosing = enclosing.enclosing;
        }

        // Store the function chunk and return it
        let chunk = Rc::new(function_chunk);
        self.functions.push(Rc::clone(&chunk));

        Ok((chunk, arity, upvalues))
    }

    fn compile_stmt(&mut self, stmt: Stmt) -> Result<(), String> {
        match stmt {
            Stmt::Expression(expr) => {
                self.compile_expr(expr)?;
                self.emit(OpCode::Pop);
            }
            Stmt::Print(expr) => {
                self.compile_expr(expr)?;
                self.emit(OpCode::Print);
            }
            Stmt::VarDecl { name, initializer } => {
                self.compile_expr(initializer)?;

                if self.current.scope_depth > 0 {
                    // Local constant
                    self.add_local(name)?;
                    // Value is already on stack, that's the local
                } else {
                    // Global constant
                    self.emit(OpCode::DefineGlobal(name));
                }
            }
            Stmt::Block(statements) => {
                self.begin_scope();
                for stmt in statements {
                    self.compile_stmt(stmt)?;
                }
                self.end_scope();
            }
            Stmt::If {
                condition,
                then_branch,
                else_branch,
            } => {
                self.compile_expr(condition)?;

                let jump_to_else = self.emit(OpCode::JumpIfFalse(0));
                self.emit(OpCode::Pop);

                self.compile_stmt(*then_branch)?;

                if let Some(else_stmt) = else_branch {
                    let jump_over_else = self.emit(OpCode::Jump(0));

                    let else_start = self.current_offset();
                    self.current.chunk.patch_jump(jump_to_else, else_start);
                    self.emit(OpCode::Pop);

                    self.compile_stmt(*else_stmt)?;

                    let after_else = self.current_offset();
                    self.current.chunk.patch_jump(jump_over_else, after_else);
                } else {
                    let after_if = self.current_offset();
                    self.current.chunk.patch_jump(jump_to_else, after_if);
                    self.emit(OpCode::Pop);
                }
            }
            Stmt::While { condition, body } => {
                let loop_start = self.current_offset();

                self.compile_expr(condition)?;

                let exit_jump = self.emit(OpCode::JumpIfFalse(0));
                self.emit(OpCode::Pop);

                self.compile_stmt(*body)?;

                self.emit(OpCode::Jump(loop_start));

                let after_loop = self.current_offset();
                self.current.chunk.patch_jump(exit_jump, after_loop);
                self.emit(OpCode::Pop);
            }
            Stmt::Return { value } => {
                if self.current.function_type == FunctionType::Script {
                    return Err("Kan nie buite 'n funksie terugkeer nie.".to_string());
                }

                if let Some(expr) = value {
                    // Check for tail call optimization
                    if let Expr::Call { callee, arguments } = expr {
                        self.compile_tail_call(callee, arguments)?;
                    } else {
                        self.compile_expr(expr)?;
                        self.emit(OpCode::Return);
                    }
                } else {
                    let nil_idx = self.add_constant(Value::Nil);
                    self.emit(OpCode::Constant(nil_idx));
                    self.emit(OpCode::Return);
                }
            }
            Stmt::ReturnIf {
                value,
                condition,
                else_value,
            } => {
                if self.current.function_type == FunctionType::Script {
                    return Err("Kan nie buite 'n funksie terugkeer nie.".to_string());
                }

                // Compile condition
                self.compile_expr(condition)?;

                // Jump past return if condition is false
                let skip_jump = self.emit(OpCode::JumpIfFalse(0));
                self.emit(OpCode::Pop); // Pop condition

                // Compile value and return
                if let Expr::Call { callee, arguments } = value {
                    self.compile_tail_call(callee, arguments)?;
                } else {
                    self.compile_expr(value)?;
                    self.emit(OpCode::Return);
                }

                // Patch jump: come here if condition was false
                let after_return = self.current_offset();
                self.current.chunk.patch_jump(skip_jump, after_return);
                self.emit(OpCode::Pop); // Pop condition

                // If there's an else value, return it
                if let Some(else_expr) = else_value {
                    if let Expr::Call { callee, arguments } = else_expr {
                        self.compile_tail_call(callee, arguments)?;
                    } else {
                        self.compile_expr(else_expr)?;
                        self.emit(OpCode::Return);
                    }
                }
                // If no else, execution continues to next statement
            }
            Stmt::TypeDecl { name, constructors } => {
                // For each constructor, create a TypeConstructor value and define it as a global
                for constructor in constructors {
                    let constructor_def = TypeConstructorDef {
                        type_name: name.clone(),
                        constructor_name: constructor.name.clone(),
                        arity: constructor.fields.len(),
                    };

                    let constructor_value = Value::TypeConstructor(Rc::new(constructor_def));
                    let const_idx = self.add_constant(constructor_value);
                    self.emit(OpCode::Constant(const_idx));
                    self.emit(OpCode::DefineGlobal(constructor.name));
                }
            }
            Stmt::Import { path, alias } => {
                // Emit LoadModule instruction which will load and push the module
                self.emit(OpCode::LoadModule(path, alias.clone()));
                // Define the module as a global variable
                self.emit(OpCode::DefineGlobal(alias));
            }
            Stmt::ExportVarDecl { name, initializer } => {
                // Track this symbol as exported
                self.exported_symbols.insert(name.clone());

                // Compile like a regular global constant declaration
                self.compile_expr(initializer)?;
                self.emit(OpCode::DefineGlobal(name));
            }
        }

        Ok(())
    }

    fn compile_expr(&mut self, expr: Expr) -> Result<(), String> {
        match expr {
            Expr::Literal(lit) => {
                let value = self.literal_to_value(&lit);
                let idx = self.add_constant(value);
                self.emit(OpCode::Constant(idx));
            }
            Expr::Variable(name) => {
                match self.resolve_variable(&name) {
                    VarLocation::Local(slot) => self.emit(OpCode::GetLocal(slot)),
                    VarLocation::Upvalue(idx) => self.emit(OpCode::GetUpvalue(idx)),
                    VarLocation::Global => self.emit(OpCode::GetGlobal(name)),
                };
            }
            Expr::Grouping(inner) => {
                self.compile_expr(*inner)?;
            }
            Expr::Unary { operator, right } => {
                self.compile_expr(*right)?;
                match operator.token_type {
                    TokenType::Minus => self.emit(OpCode::Negate),
                    TokenType::Bang => self.emit(OpCode::Not),
                    _ => return Err("Onbekende unêre operator.".to_string()),
                };
            }
            Expr::Binary {
                left,
                operator,
                right,
            } => match operator.token_type {
                TokenType::And => {
                    self.compile_expr(*left)?;
                    let jump = self.emit(OpCode::JumpIfFalse(0));
                    self.emit(OpCode::Pop);
                    self.compile_expr(*right)?;
                    let after = self.current_offset();
                    self.current.chunk.patch_jump(jump, after);
                }
                TokenType::Or => {
                    self.compile_expr(*left)?;
                    let else_jump = self.emit(OpCode::JumpIfFalse(0));
                    let end_jump = self.emit(OpCode::Jump(0));

                    let else_branch = self.current_offset();
                    self.current.chunk.patch_jump(else_jump, else_branch);
                    self.emit(OpCode::Pop);
                    self.compile_expr(*right)?;

                    let end = self.current_offset();
                    self.current.chunk.patch_jump(end_jump, end);
                }
                _ => {
                    self.compile_expr(*left)?;
                    self.compile_expr(*right)?;

                    match operator.token_type {
                        TokenType::Plus => self.emit(OpCode::Add),
                        TokenType::Minus => self.emit(OpCode::Subtract),
                        TokenType::Star => self.emit(OpCode::Multiply),
                        TokenType::Slash => self.emit(OpCode::Divide),
                        TokenType::Percent => self.emit(OpCode::Modulo),
                        TokenType::EqualEqual => self.emit(OpCode::Equal),
                        TokenType::BangEqual => self.emit(OpCode::NotEqual),
                        TokenType::Less => self.emit(OpCode::Less),
                        TokenType::LessEqual => self.emit(OpCode::LessEqual),
                        TokenType::Greater => self.emit(OpCode::Greater),
                        TokenType::GreaterEqual => self.emit(OpCode::GreaterEqual),
                        _ => return Err("Onbekende binêre operator.".to_string()),
                    };
                }
            },
            Expr::Call { callee, arguments } => {
                // Compile the callee (the function to call)
                self.compile_expr(*callee)?;

                // Compile arguments
                let arg_count = self.compile_arguments(arguments)?;
                self.emit(OpCode::Call(arg_count));
            }
            Expr::Lambda { params, body } => {
                // Compile lambda similar to a function
                let (chunk, arity, upvalues) = self.compile_lambda(params, body)?;

                // Create function value
                let function = Value::Function(Rc::new(Function {
                    name: String::from("<lambda>"),
                    arity,
                    chunk,
                    upvalue_count: upvalues.len(),
                }));

                let const_idx = self.add_constant(function);

                // Emit Closure opcode if there are upvalues, otherwise just Constant
                if upvalues.is_empty() {
                    self.emit(OpCode::Constant(const_idx));
                } else {
                    self.emit(OpCode::Closure(const_idx, upvalues));
                }
            }
            Expr::List(elements) => {
                // Compile each element and push onto stack
                let count = elements.len();
                for elem in elements {
                    self.compile_expr(elem)?;
                }
                // Create list from stack values
                self.emit(OpCode::MakeList(count));
            }
            Expr::Index { object, index } => {
                // Compile the object (list) and index
                self.compile_expr(*object)?;
                self.compile_expr(*index)?;
                self.emit(OpCode::GetIndex);
            }
            Expr::Match { value, arms } => {
                // Begin a scope for the entire match expression
                self.begin_scope();

                // Evaluate the value to match and store as hidden local
                // This ensures pattern bindings have correct stack indices
                self.compile_expr(*value)?;
                self.add_local(String::from(MATCH_SCRUTINEE))?;
                let scrutinee_slot = self.resolve_local(MATCH_SCRUTINEE).unwrap();

                // Track jump addresses
                let mut end_jumps = Vec::new();

                for (i, arm) in arms.iter().enumerate() {
                    let is_last = i == arms.len() - 1;

                    // Get a copy of the scrutinee onto the stack
                    self.emit(OpCode::GetLocal(scrutinee_slot));

                    // Begin a new scope for pattern bindings
                    self.begin_scope();

                    // Compile pattern matching
                    let bindings = self.collect_pattern_bindings(&arm.pattern);
                    let fail_jump = self.compile_pattern(&arm.pattern, !is_last)?;

                    // Compile the body
                    self.compile_expr((*arm.body).clone())?;

                    // Clean up: result is on top, bindings below, scrutinee at bottom
                    // Stack: [scrutinee, bindings..., result]
                    // We want: [result]

                    // Save result to scrutinee slot (overwrites scrutinee)
                    self.emit(OpCode::SetLocal(scrutinee_slot));
                    // Pop the result from top (it's saved in slot 0)
                    self.emit(OpCode::Pop);
                    // Pop each binding manually (can't use end_scope() - need precise stack control)
                    for _ in 0..bindings {
                        self.emit(OpCode::Pop);
                        self.current.locals.pop();
                    }
                    self.current.scope_depth -= 1;
                    // Stack is now [result] in the scrutinee slot position

                    // Jump to end after successful match
                    end_jumps.push(self.emit(OpCode::Jump(0)));

                    // Patch the fail jump to come here (next arm)
                    // When pattern fails, we need to clean up the stack:
                    // - Pop the boolean from CheckConstructor (if it was a constructor pattern)
                    // - Pop the scrutinee copy that we pushed for this arm
                    if let Some(fail_addr) = fail_jump {
                        let next_arm = self.current_offset();
                        self.current.chunk.patch_jump(fail_addr, next_arm);
                        // Pop the boolean result from CheckConstructor or Equal
                        self.emit(OpCode::Pop);
                        // Pop the scrutinee copy for this failed arm
                        self.emit(OpCode::Pop);
                    }
                }

                // Patch all end jumps to come here
                let end = self.current_offset();
                for jump in end_jumps {
                    self.current.chunk.patch_jump(jump, end);
                }

                // End the outer scope manually (can't use end_scope() - result is in scrutinee slot)
                // The result overwrote the scrutinee, so just clean up locals tracking
                self.current.locals.pop();
                self.current.scope_depth -= 1;
            }
            Expr::IfExpr {
                condition,
                then_branch,
                else_branch,
            } => {
                // Compile condition
                self.compile_expr(*condition)?;

                // Jump to else if false
                let else_jump = self.emit(OpCode::JumpIfFalse(0));
                self.emit(OpCode::Pop); // Pop condition

                // Compile then branch
                self.compile_expr(*then_branch)?;

                // Jump past else branch
                let end_jump = self.emit(OpCode::Jump(0));

                // Patch else jump
                let else_offset = self.current_offset();
                self.current.chunk.patch_jump(else_jump, else_offset);
                self.emit(OpCode::Pop); // Pop condition

                // Compile else branch
                self.compile_expr(*else_branch)?;

                // Patch end jump
                let end_offset = self.current_offset();
                self.current.chunk.patch_jump(end_jump, end_offset);
            }
            Expr::MemberAccess { object, member } => {
                // Compile the object (module)
                self.compile_expr(*object)?;
                // Emit GetMember instruction
                self.emit(OpCode::GetMember(member));
            }
        }

        Ok(())
    }

    /// Count how many stack slots a pattern will occupy (including hidden locals)
    fn collect_pattern_bindings(&self, pattern: &Pattern) -> usize {
        match pattern {
            Pattern::Wildcard => 0,
            Pattern::Variable(_) => 1,
            Pattern::Literal(_) => 0,
            Pattern::Constructor { fields, .. } => {
                // Count the hidden $ctor local plus all field bindings
                let field_bindings: usize = fields
                    .iter()
                    .map(|p| self.collect_pattern_bindings(p))
                    .sum();
                if fields.is_empty() {
                    0 // No hidden local for zero-field constructors
                } else {
                    1 + field_bindings // 1 for $ctor + field bindings
                }
            }
        }
    }

    /// Compile a pattern match check. Returns jump address if pattern can fail.
    ///
    /// This function expects the value to match on top of the stack.
    /// After matching:
    /// - For Variable: the value remains on stack as a new local
    /// - For Wildcard: the value is popped
    /// - For Literal: the value is popped
    /// - For Constructor: the constructor is popped, but field bindings remain
    fn compile_pattern(
        &mut self,
        pattern: &Pattern,
        can_fail: bool,
    ) -> Result<Option<usize>, String> {
        match pattern {
            Pattern::Wildcard => {
                // Always matches, pop the value
                self.emit(OpCode::Pop);
                Ok(None)
            }
            Pattern::Variable(name) => {
                // Bind the value to a local constant
                // The value is on top of stack and becomes the local's storage
                self.add_local(name.clone())?;
                Ok(None)
            }
            Pattern::Literal(lit) => {
                // Duplicate scrutinee so we don't consume it during comparison
                self.emit(OpCode::Dup);
                // Compare with literal
                let const_value = self.literal_to_value(lit);
                let const_idx = self.add_constant(const_value);
                self.emit(OpCode::Constant(const_idx));
                self.emit(OpCode::Equal);
                // Stack now has: [..., scrutinee, bool]

                let fail_jump = if can_fail {
                    Some(self.emit(OpCode::JumpIfFalse(0)))
                } else {
                    None
                };
                self.emit(OpCode::Pop); // Pop the boolean result
                self.emit(OpCode::Pop); // Pop the scrutinee (literal patterns don't bind)
                Ok(fail_jump)
            }
            Pattern::Constructor { name, fields } => {
                // Check if value is this constructor with correct arity
                self.emit(OpCode::CheckConstructor(name.clone(), fields.len()));

                let fail_jump = if can_fail {
                    Some(self.emit(OpCode::JumpIfFalse(0)))
                } else {
                    None
                };
                self.emit(OpCode::Pop); // Pop the boolean result

                let num_fields = fields.len();

                if num_fields == 0 {
                    // No fields, just pop the ADT
                    self.emit(OpCode::Pop);
                } else {
                    // For multi-field constructors, save the ADT as a hidden local
                    // This ensures field extractions use correct stack indexing
                    self.add_local(String::from(CTOR_HIDDEN_LOCAL))?;
                    let ctor_slot = self.resolve_local(CTOR_HIDDEN_LOCAL).unwrap();

                    // Extract each field value and process its pattern
                    for (i, field_pattern) in fields.iter().enumerate() {
                        // Get the ADT from the hidden local
                        self.emit(OpCode::GetLocal(ctor_slot));
                        // Get the field value (leaves ADT copy on stack, pushes field)
                        self.emit(OpCode::GetFieldPop(i));

                        // Recursively compile the field pattern
                        self.compile_pattern(field_pattern, false)?;
                    }

                    // Pop the hidden ADT local
                    // The field bindings are now proper locals above it
                    // We need to remove $ctor from locals but keep the field bindings
                    // Actually, just leave it - the scope management will handle it
                    // But we do need to pop it from the stack eventually...
                    // Actually no - GetLocal doesn't remove from stack, it copies.
                    // The ADT is still sitting in its slot. We need to remove it.

                    // The $ctor local is still at ctor_slot on the stack
                    // We need to pop it but keep the field values above it
                    // This requires swapping - but we don't have that.
                    // Alternative: leave it and let scope cleanup handle it
                    // But that messes up the binding count.

                    // Actually, let's track that we added this hidden local
                    // and handle it in the cleanup. For now, just keep it.
                }

                Ok(fail_jump)
            }
        }
    }

    fn compile_lambda(
        &mut self,
        params: Vec<String>,
        body: LambdaBody,
    ) -> Result<(Rc<Chunk>, usize, Vec<UpvalueDescriptor>), String> {
        self.compile_callable(String::from("<lambda>"), params, |compiler| {
            match body {
                LambdaBody::Expr(expr) => {
                    // Single expression - implicit return
                    // Check for tail call optimization
                    if let Expr::Call { callee, arguments } = *expr {
                        compiler.compile_tail_call(callee, arguments)?;
                    } else {
                        compiler.compile_expr(*expr)?;
                        compiler.emit(OpCode::Return);
                    }
                }
                LambdaBody::Block(stmts) => {
                    // Block body - like a function
                    for stmt in stmts {
                        compiler.compile_stmt(stmt)?;
                    }
                    // Implicit nil return
                    let nil_idx = compiler.add_constant(Value::Nil);
                    compiler.emit(OpCode::Constant(nil_idx));
                    compiler.emit(OpCode::Return);
                }
            }
            Ok(())
        })
    }

    fn emit(&mut self, op: OpCode) -> usize {
        self.current.chunk.write(op)
    }

    fn add_constant(&mut self, value: Value) -> usize {
        self.current.chunk.add_constant(value)
    }

    fn current_offset(&self) -> usize {
        self.current.chunk.code.len()
    }

    fn literal_to_value(&self, lit: &Literal) -> Value {
        match lit {
            Literal::Number(n) => Value::Number(*n),
            Literal::Boolean(b) => Value::Boolean(*b),
            Literal::String(s) => Value::String(Rc::new(s.clone())),
            Literal::Nil => Value::Nil,
        }
    }

    fn extract_upvalues(&self) -> Vec<UpvalueDescriptor> {
        self.current
            .upvalues
            .iter()
            .map(|u| UpvalueDescriptor {
                index: u.index,
                is_local: u.is_local,
            })
            .collect()
    }

    fn compile_arguments(&mut self, arguments: Vec<Expr>) -> Result<usize, String> {
        let arg_count = arguments.len();
        for arg in arguments {
            self.compile_expr(arg)?;
        }
        Ok(arg_count)
    }

    fn compile_tail_call(&mut self, callee: Box<Expr>, arguments: Vec<Expr>) -> Result<(), String> {
        self.compile_expr(*callee)?;
        let arg_count = self.compile_arguments(arguments)?;
        self.emit(OpCode::TailCall(arg_count));
        Ok(())
    }
}

impl Default for Compiler {
    fn default() -> Self {
        Self::new()
    }
}
