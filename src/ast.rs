use crate::types::Type;
use std::collections::HashMap;

/// Abstract Syntax Tree nodes for the Boba language
#[derive(Debug, Clone, PartialEq)]
pub enum Expr {
    // Literals
    IntLiteral(i64),
    FloatLiteral(f64),
    StringLiteral(String),
    BoolLiteral(bool),
    NullLiteral,
    
    // Collections
    List(Vec<Expr>),
    Map(Vec<(Expr, Expr)>),
    
    // Variables
    Identifier(String),
    VarDeclaration(String, Box<Expr>),
    
    // Operations
    BinaryOp {
        left: Box<Expr>,
        operator: BinaryOperator,
        right: Box<Expr>,
    },
    UnaryOp {
        operator: UnaryOperator,
        expr: Box<Expr>,
    },
    
    // Control flow
    If {
        condition: Box<Expr>,
        then_branch: Vec<Expr>,
        else_if_branches: Vec<(Expr, Vec<Expr>)>,
        else_branch: Option<Vec<Expr>>,
    },
    Loop {
        init: Option<Box<Expr>>,
        condition: Option<Box<Expr>>,
        update: Option<Box<Expr>>,
        body: Vec<Expr>,
    },
    Continue,
    Break,
    Return(Vec<Expr>),
    
    // Function
    FunctionDeclaration {
        name: String,
        params: Vec<(String, Type)>,
        return_types: Vec<Type>,
        body: Vec<Expr>,
    },
    FunctionCall {
        name: String,
        args: Vec<Expr>,
    },
    
    // Built-in functions
    Output(Vec<Expr>),
    OutputFormatted(Box<Expr>),
    OutputAddress(Box<Expr>),
    Input(Box<Expr>),
    InputFormatted(Box<Expr>),
    
    // Type operations
    TypeConversion {
        expr: Box<Expr>,
        target_type: Type,
    },
    TypeCheck {
        expr: Box<Expr>,
        check_type: Type,
        is_negated: bool,
    },
}

#[derive(Debug, Clone, PartialEq)]
pub enum BinaryOperator {
    Add,
    Subtract,
    Multiply,
    Divide,
    Modulo,
    Equal,
    NotEqual,
    LessThan,
    LessThanOrEqual,
    GreaterThan,
    GreaterThanOrEqual,
    And,
    Or,
}

#[derive(Debug, Clone, PartialEq)]
pub enum UnaryOperator {
    Negate,
    Not,
    AddressOf,
}

#[derive(Debug, Clone)]
pub struct Program {
    pub functions: HashMap<String, FunctionDef>,
    pub main_block: Vec<Expr>,
}

#[derive(Debug, Clone)]
pub struct FunctionDef {
    pub name: String,
    pub params: Vec<(String, Type)>,
    pub return_types: Vec<Type>,
    pub body: Vec<Expr>,
}
