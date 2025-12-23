use crate::token::Token;

#[derive(Debug, Clone)]
pub enum Expr {
    Binary {
        left: Box<Expr>,
        operator: Token,
        right: Box<Expr>,
    },
    Unary {
        operator: Token,
        right: Box<Expr>,
    },
    Literal(Literal),
    Variable(String),
    Grouping(Box<Expr>),
    Assign {
        name: String,
        value: Box<Expr>,
    },
    // Function call
    Call {
        callee: Box<Expr>,
        arguments: Vec<Expr>,
    },
    // Lambda expression: fn(params) expr or fn(params) { stmts }
    Lambda {
        params: Vec<String>,
        body: LambdaBody,
    },
    // List literal: [a, b, c]
    List(Vec<Expr>),
    // Index expression: list[index]
    Index {
        object: Box<Expr>,
        index: Box<Expr>,
    },
    // Pattern matching expression: pas(value) { geval Pattern => expr ... }
    Match {
        value: Box<Expr>,
        arms: Vec<MatchArm>,
    },
    // Inline if expression: as(condition) then_expr anders else_expr
    IfExpr {
        condition: Box<Expr>,
        then_branch: Box<Expr>,
        else_branch: Box<Expr>,
    },
}

/// Represents a type constructor definition
#[derive(Debug, Clone)]
pub struct TypeConstructor {
    pub name: String,
    pub fields: Vec<String>,  // Field names (can be empty for unit constructors)
}

/// Represents a pattern for pattern matching
#[derive(Debug, Clone)]
pub enum Pattern {
    /// Wildcard pattern: _
    Wildcard,
    /// Variable binding: x
    Variable(String),
    /// Literal pattern: 42, "hello", waar, vals
    Literal(Literal),
    /// Constructor pattern: Sommige(x), Kons(h, t)
    Constructor {
        name: String,
        fields: Vec<Pattern>,
    },
}

/// A single match arm: geval Pattern => body
#[derive(Debug, Clone)]
pub struct MatchArm {
    pub pattern: Pattern,
    pub body: Box<Expr>,
}

#[derive(Debug, Clone)]
pub enum LambdaBody {
    Expr(Box<Expr>),
    Block(Vec<Stmt>),
}

#[derive(Debug, Clone)]
pub enum Literal {
    Number(f64),
    Boolean(bool),
    String(String),
    Nil,
}

#[derive(Debug, Clone)]
pub enum Stmt {
    Expression(Expr),
    Print(Expr),
    VarDecl {
        name: String,
        initializer: Expr,
        is_mutable: bool,  // true for 'stel', false for 'laat'
    },
    Block(Vec<Stmt>),
    If {
        condition: Expr,
        then_branch: Box<Stmt>,
        else_branch: Option<Box<Stmt>>,
    },
    While {
        condition: Expr,
        body: Box<Stmt>,
    },
    // Function declaration
    FunDecl {
        name: String,
        params: Vec<String>,
        body: Vec<Stmt>,
    },
    // Return statement
    Return {
        value: Option<Expr>,
    },
    // Type declaration (ADT)
    TypeDecl {
        name: String,
        constructors: Vec<TypeConstructor>,
    },
}
