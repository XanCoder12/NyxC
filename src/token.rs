#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Span {
    pub line: usize,
    pub col: usize,
}

impl Span {
    pub fn new(line: usize, col: usize) -> Self {
        Self { line, col }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum TokenKind {
    // Keywords
    Fn,
    Let,
    Mut,
    Const,
    Return,
    If,
    Else,
    While,
    For,
    Import,
    Syscall,
    Extern,
    True,
    False,

    // Types
    TypeI8,
    TypeI16,
    TypeI32,
    TypeU8,
    TypeU16,
    TypeU32,
    TypeBool,
    TypeVoid,

    // Literals
    Ident(String),
    IntLit(i64),
    StringLit(String),
    CharLit(u8),

    // Symbols & Operators
    LParen,
    RParen,
    LBrace,
    RBrace,
    LBracket,
    RBracket,
    Comma,
    Colon,
    PathSep,      // ::
    Semicolon,
    Arrow,        // ->
    Newline,

    Plus,
    Minus,
    Star,
    Slash,
    Percent,
    Eq,           // =
    EqEq,         // ==
    NotEq,        // !=
    Lt,           // <
    LtEq,         // <=
    Gt,           // >
    GtEq,         // >=
    AmpAmp,       // &&
    PipePipe,     // ||
    Bang,         // !
    Amp,          // &
    Pipe,         // |
    Caret,        // ^
    Tilde,        // ~
    Shl,          // <<
    Shr,          // >>

    Eof,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Token {
    pub kind: TokenKind,
    pub span: Span,
}

impl Token {
    pub fn new(kind: TokenKind, span: Span) -> Self {
        Self { kind, span }
    }
}
