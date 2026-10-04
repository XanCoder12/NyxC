use crate::token::Span;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CompileError {
    pub code: &'static str,
    pub message: String,
    pub span: Option<Span>,
}

impl CompileError {
    pub fn new(code: &'static str, message: impl Into<String>, span: Option<Span>) -> Self {
        Self {
            code,
            message: message.into(),
            span,
        }
    }
}

pub fn render(file: &str, source: &str, err: &CompileError) -> String {
    let mut out = String::new();
    out.push_str(&format!("error[{}]: {}\n", err.code, err.message));

    if let Some(s) = err.span {
        out.push_str(&format!("  --> {}:{}:{}\n", file, s.line, s.col));

        let lines: Vec<&str> = source.lines().collect();
        if s.line >= 1 && s.line <= lines.len() {
            let width = s.line.to_string().len();
            let line = lines[s.line - 1];
            out.push_str(&format!("{:>w$} |\n", " ", w = width));
            out.push_str(&format!("{:>w$} | {}\n", s.line, line, w = width));
            let caret = s.col.saturating_sub(1).min(line.chars().count());
            out.push_str(&format!(
                "{pad:>w$} | {spaces}^",
                pad = " ",
                w = width,
                spaces = " ".repeat(caret)
            ));
        }
    }

    out
}
