use crate::token::Span;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Type {
    I8,
    I16,
    I32,
    U8,
    U16,
    U32,
    Bool,
    Void,
    Pointer(Box<Type>),
}

impl Type {
    pub fn size_in_bytes(&self) -> usize {
        match self {
            Type::I8 | Type::U8 | Type::Bool => 1,
            Type::I16 | Type::U16 => 2,
            Type::I32 | Type::U32 | Type::Pointer(_) => 4, // 32-bit x86 target
            Type::Void => 0,
        }
    }

    pub fn name(&self) -> String {
        match self {
            Type::I8 => "i8".into(),
            Type::I16 => "i16".into(),
            Type::I32 => "i32".into(),
            Type::U8 => "u8".into(),
            Type::U16 => "u16".into(),
            Type::U32 => "u32".into(),
            Type::Bool => "bool".into(),
            Type::Void => "void".into(),
            Type::Pointer(inner) => format!("*{}", inner.name()),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BinaryOp {
    Add,
    Sub,
    Mul,
    Div,
    Mod,
    Eq,
    Ne,
    Lt,
    Le,
    Gt,
    Ge,
    And,
    Or,
    BitAnd,
    BitOr,
    BitXor,
    Shl,
    Shr,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum UnaryOp {
    Neg,
    Not,
    BitNot,
    Deref,
    AddrOf,
}

#[derive(Debug, Clone, PartialEq)]
pub enum Expr {
    IntLit(i64, Span),
    StringLit(String, Span),
    BoolLit(bool, Span),
    Ident(String, Span),
    Path(Vec<String>, Span),
    Binary(BinaryOp, Box<Expr>, Box<Expr>, Span),
    Unary(UnaryOp, Box<Expr>, Span),
    Call {
        callee: Box<Expr>,
        args: Vec<Expr>,
        span: Span,
    },
    Syscall {
        number: Box<Expr>,
        args: Vec<Expr>,
        span: Span,
    },
    Assign {
        target: Box<Expr>,
        value: Box<Expr>,
        span: Span,
    },
}

impl Expr {
    pub fn span(&self) -> Span {
        match self {
            Expr::IntLit(_, s) => *s,
            Expr::StringLit(_, s) => *s,
            Expr::BoolLit(_, s) => *s,
            Expr::Ident(_, s) => *s,
            Expr::Path(_, s) => *s,
            Expr::Binary(_, _, _, s) => *s,
            Expr::Unary(_, _, s) => *s,
            Expr::Call { span, .. } => *span,
            Expr::Syscall { span, .. } => *span,
            Expr::Assign { span, .. } => *span,
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum Stmt {
    Let {
        name: String,
        is_mut: bool,
        ty: Option<Type>,
        init: Option<Expr>,
        span: Span,
    },
    Const {
        name: String,
        ty: Option<Type>,
        init: Expr,
        span: Span,
    },
    Return {
        value: Option<Expr>,
        span: Span,
    },
    Expr(Expr, Span),
    If {
        cond: Expr,
        then_branch: Vec<Stmt>,
        else_branch: Option<Vec<Stmt>>,
        span: Span,
    },
    While {
        cond: Expr,
        body: Vec<Stmt>,
        span: Span,
    },
    For {
        init: Box<Stmt>,
        cond: Expr,
        step: Box<Expr>,
        body: Vec<Stmt>,
        span: Span,
    },
    /// Internal wrappers for `for` loop initialization.
    LetWrapper(Box<Stmt>),
    ExprInit(Box<Expr>, Span),
    Block(Vec<Stmt>, Span),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Param {
    pub name: String,
    pub ty: Type,
    pub span: Span,
}

#[derive(Debug, Clone, PartialEq)]
pub struct FnDef {
    pub name: String,
    pub params: Vec<Param>,
    pub ret_ty: Type,
    pub body: Option<Vec<Stmt>>,
    pub is_extern: bool,
    /// Stack space needed for local variables (computed by sema).
    pub stack_size: i32,
    pub span: Span,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Import {
    pub path: String,
    pub alias: Option<String>,
    pub span: Span,
}

#[derive(Debug, Clone, PartialEq)]
pub enum Item {
    Fn(FnDef),
    Import(Import),
    Const(Stmt),
}

#[derive(Debug, Clone, PartialEq)]
pub struct Program {
    pub items: Vec<Item>,
}
