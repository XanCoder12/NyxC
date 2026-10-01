use crate::ast::*;
use crate::sema::SemanticAnalyzer;

pub struct X86GasCodegen<'a> {
    analyzer: &'a SemanticAnalyzer,
    strings: Vec<(String, usize)>, // (content, label_id)
    label_counter: usize,
    current_fn_end_label: Option<String>,
    current_fn_name: Option<String>,
}

impl<'a> X86GasCodegen<'a> {
    pub fn new(analyzer: &'a SemanticAnalyzer) -> Self {
        Self {
            analyzer,
            strings: Vec::new(),
            label_counter: 0,
            current_fn_end_label: None,
            current_fn_name: None,
        }
    }

    fn lookup_var(&self, name: &str) -> Option<&crate::sema::symbols::Symbol> {
        if let Some(fn_name) = &self.current_fn_name {
            if let Some(sym) = self.analyzer.symbols.lookup_fn_var(fn_name, name) {
                return Some(sym);
            }
        }
        self.analyzer.symbols.lookup_var(name)
    }

    fn new_label(&mut self, prefix: &str) -> String {
        self.label_counter += 1;
        format!(".L_{}_{}", prefix, self.label_counter)
    }

    fn add_string_literal(&mut self, s: &str) -> usize {
        for (existing, id) in &self.strings {
            if existing == s {
                return *id;
            }
        }
        let id = self.strings.len();
        self.strings.push((s.to_string(), id));
        id
    }

    pub fn generate(&mut self, program: &Program) -> Result<String, String> {
        let mut text_section = String::new();
        let mut has_main = false;
        let mut has_start = false;

        for item in &program.items {
            if let Item::Fn(f) = item {
                if f.name == "main" {
                    has_main = true;
                }
                if f.name == "_start" {
                    has_start = true;
                }
                if let Some(body) = &f.body {
                    text_section.push_str(&self.generate_fn(f, body)?);
                }
            }
        }

        let mut output = String::new();
        output.push_str(".intel_syntax noprefix\n\n");

        // Read-only data section for string literals
        if !self.strings.is_empty() {
            output.push_str(".section .rodata\n");
            for (s, id) in &self.strings {
                output.push_str(&format!(".LC{}:\n", id));
                output.push_str("    .string ");
                output.push_str(&self.format_asm_string(s));
                output.push('\n');
            }
            output.push('\n');
        }

        // Code section
        output.push_str(".section .text\n");
        output.push_str(&crate::codegen::runtime::generate_runtime_glue(has_main, has_start));
        output.push_str(&text_section);

        Ok(output)
    }

    fn format_asm_string(&self, s: &str) -> String {
        let mut res = String::from("\"");
        for ch in s.chars() {
            match ch {
                '\n' => res.push_str("\\n"),
                '\t' => res.push_str("\\t"),
                '\r' => res.push_str("\\r"),
                '\0' => res.push_str("\\0"),
                '\\' => res.push_str("\\\\"),
                '"' => res.push_str("\\\""),
                _ => res.push(ch),
            }
        }
        res.push('"');
        res
    }

    fn generate_fn(&mut self, def: &FnDef, body: &[Stmt]) -> Result<String, String> {
        let mut asm = String::new();
        let fn_label = def.name.clone();
        let end_label = format!(".L_end_{}", fn_label);
        self.current_fn_end_label = Some(end_label.clone());
        self.current_fn_name = Some(def.name.clone());

        asm.push_str(&format!(".globl {}\n", fn_label));
        asm.push_str(&format!("{}:\n", fn_label));
        asm.push_str("    push ebp\n");
        asm.push_str("    mov ebp, esp\n");
        asm.push_str("    sub esp, 256\n"); // Pre-allocate local stack frame

        for stmt in body {
            asm.push_str(&self.generate_stmt(stmt)?);
        }

        asm.push_str(&format!("{}:\n", end_label));
        asm.push_str("    mov esp, ebp\n");
        asm.push_str("    pop ebp\n");
        asm.push_str("    ret\n\n");

        self.current_fn_end_label = None;
        self.current_fn_name = None;
        Ok(asm)
    }

    fn generate_stmt(&mut self, stmt: &Stmt) -> Result<String, String> {
        let mut asm = String::new();

        match stmt {
            Stmt::Let { name, init, .. } => {
                if let Some(init_expr) = init {
                    asm.push_str(&self.generate_expr(init_expr)?);
                    if let Some(sym) = self.lookup_var(name) {
                        asm.push_str(&format!("    mov [ebp + ({})], eax\n", sym.offset));
                    }
                }
            }
            Stmt::Return { value, .. } => {
                if let Some(expr) = value {
                    asm.push_str(&self.generate_expr(expr)?);
                }
                if let Some(end_label) = &self.current_fn_end_label {
                    asm.push_str(&format!("    jmp {}\n", end_label));
                }
            }
            Stmt::Expr(expr, _) => {
                asm.push_str(&self.generate_expr(expr)?);
            }
            Stmt::If { cond, then_branch, else_branch, .. } => {
                let else_lbl = self.new_label("else");
                let end_lbl = self.new_label("endif");

                asm.push_str(&self.generate_expr(cond)?);
                asm.push_str("    test eax, eax\n");
                asm.push_str(&format!("    jz {}\n", if else_branch.is_some() { &else_lbl } else { &end_lbl }));

                for s in then_branch {
                    asm.push_str(&self.generate_stmt(s)?);
                }

                if let Some(else_stmts) = else_branch {
                    asm.push_str(&format!("    jmp {}\n", end_lbl));
                    asm.push_str(&format!("{}:\n", else_lbl));
                    for s in else_stmts {
                        asm.push_str(&self.generate_stmt(s)?);
                    }
                }

                asm.push_str(&format!("{}:\n", end_lbl));
            }
            Stmt::While { cond, body, .. } => {
                let loop_start = self.new_label("while_start");
                let loop_end = self.new_label("while_end");

                asm.push_str(&format!("{}:\n", loop_start));
                asm.push_str(&self.generate_expr(cond)?);
                asm.push_str("    test eax, eax\n");
                asm.push_str(&format!("    jz {}\n", loop_end));

                for s in body {
                    asm.push_str(&self.generate_stmt(s)?);
                }

                asm.push_str(&format!("    jmp {}\n", loop_start));
                asm.push_str(&format!("{}:\n", loop_end));
            }
            Stmt::Block(stmts, _) => {
                for s in stmts {
                    asm.push_str(&self.generate_stmt(s)?);
                }
            }
            Stmt::Const { .. } => {}
        }

        Ok(asm)
    }

    fn generate_expr(&mut self, expr: &Expr) -> Result<String, String> {
        let mut asm = String::new();

        match expr {
            Expr::IntLit(n, _) => {
                asm.push_str(&format!("    mov eax, {}\n", n));
            }
            Expr::BoolLit(b, _) => {
                asm.push_str(&format!("    mov eax, {}\n", if *b { 1 } else { 0 }));
            }
            Expr::StringLit(s, _) => {
                let str_id = self.add_string_literal(s);
                asm.push_str(&format!("    lea eax, [.LC{}]\n", str_id));
            }
            Expr::Ident(name, span) => {
                if let Some(sym) = self.lookup_var(name) {
                    asm.push_str(&format!("    mov eax, [ebp + ({})]\n", sym.offset));
                } else {
                    return Err(format!("Undefined variable '{}' at {}:{}", name, span.line, span.col));
                }
            }
            Expr::Path(parts, span) => {
                let full_name = parts.join("::");
                return Err(format!("Direct path expression '{}' not allowed as value at {}:{}", full_name, span.line, span.col));
            }
            Expr::Assign { target, value, span } => {
                asm.push_str(&self.generate_expr(value)?);
                match &**target {
                    Expr::Ident(name, _) => {
                        if let Some(sym) = self.lookup_var(name) {
                            asm.push_str(&format!("    mov [ebp + ({})], eax\n", sym.offset));
                        } else {
                            return Err(format!("Undefined assignment target '{}'", name));
                        }
                    }
                    Expr::Unary(UnaryOp::Deref, ptr_expr, _) => {
                        asm.push_str("    push eax\n");
                        asm.push_str(&self.generate_expr(ptr_expr)?);
                        asm.push_str("    mov edx, eax\n");
                        asm.push_str("    pop eax\n");
                        asm.push_str("    mov [edx], eax\n");
                    }
                    _ => return Err(format!("Invalid assignment target at {}:{}", span.line, span.col)),
                }
            }
            Expr::Unary(op, inner, _) => {
                asm.push_str(&self.generate_expr(inner)?);
                match op {
                    UnaryOp::Neg => asm.push_str("    neg eax\n"),
                    UnaryOp::Not => {
                        asm.push_str("    test eax, eax\n");
                        asm.push_str("    sete al\n");
                        asm.push_str("    movzx eax, al\n");
                    }
                    UnaryOp::BitNot => asm.push_str("    not eax\n"),
                    UnaryOp::Deref => asm.push_str("    mov eax, [eax]\n"),
                    UnaryOp::AddrOf => {
                        if let Expr::Ident(name, _) = &**inner {
                            if let Some(sym) = self.lookup_var(name) {
                                asm.push_str(&format!("    lea eax, [ebp + ({})]\n", sym.offset));
                            }
                        }
                    }
                }
            }
            Expr::Binary(op, left, right, _) => {
                asm.push_str(&self.generate_expr(left)?);
                asm.push_str("    push eax\n");
                asm.push_str(&self.generate_expr(right)?);
                asm.push_str("    mov ebx, eax\n");
                asm.push_str("    pop eax\n");

                match op {
                    BinaryOp::Add => asm.push_str("    add eax, ebx\n"),
                    BinaryOp::Sub => asm.push_str("    sub eax, ebx\n"),
                    BinaryOp::Mul => asm.push_str("    imul eax, ebx\n"),
                    BinaryOp::Div => {
                        asm.push_str("    cdq\n");
                        asm.push_str("    idiv ebx\n");
                    }
                    BinaryOp::Mod => {
                        asm.push_str("    cdq\n");
                        asm.push_str("    idiv ebx\n");
                        asm.push_str("    mov eax, edx\n");
                    }
                    BinaryOp::Eq => {
                        asm.push_str("    cmp eax, ebx\n");
                        asm.push_str("    sete al\n");
                        asm.push_str("    movzx eax, al\n");
                    }
                    BinaryOp::Ne => {
                        asm.push_str("    cmp eax, ebx\n");
                        asm.push_str("    setne al\n");
                        asm.push_str("    movzx eax, al\n");
                    }
                    BinaryOp::Lt => {
                        asm.push_str("    cmp eax, ebx\n");
                        asm.push_str("    setl al\n");
                        asm.push_str("    movzx eax, al\n");
                    }
                    BinaryOp::Le => {
                        asm.push_str("    cmp eax, ebx\n");
                        asm.push_str("    setle al\n");
                        asm.push_str("    movzx eax, al\n");
                    }
                    BinaryOp::Gt => {
                        asm.push_str("    cmp eax, ebx\n");
                        asm.push_str("    setg al\n");
                        asm.push_str("    movzx eax, al\n");
                    }
                    BinaryOp::Ge => {
                        asm.push_str("    cmp eax, ebx\n");
                        asm.push_str("    setge al\n");
                        asm.push_str("    movzx eax, al\n");
                    }
                    BinaryOp::BitAnd | BinaryOp::And => asm.push_str("    and eax, ebx\n"),
                    BinaryOp::BitOr | BinaryOp::Or => asm.push_str("    or eax, ebx\n"),
                    BinaryOp::BitXor => asm.push_str("    xor eax, ebx\n"),
                    BinaryOp::Shl => {
                        asm.push_str("    mov ecx, ebx\n");
                        asm.push_str("    shl eax, cl\n");
                    }
                    BinaryOp::Shr => {
                        asm.push_str("    mov ecx, ebx\n");
                        asm.push_str("    shr eax, cl\n");
                    }
                }
            }
            Expr::Call { callee, args, .. } => {
                // Push arguments right-to-left
                for arg in args.iter().rev() {
                    asm.push_str(&self.generate_expr(arg)?);
                    asm.push_str("    push eax\n");
                }

                let target_label = match &**callee {
                    Expr::Ident(name, _) => name.clone(),
                    Expr::Path(parts, _) => parts.join("_"), // sys::write -> sys_write
                    _ => return Err("Indirect function call not supported yet".into()),
                };

                asm.push_str(&format!("    call {}\n", target_label));
                if !args.is_empty() {
                    asm.push_str(&format!("    add esp, {}\n", args.len() * 4));
                }
            }
            Expr::Syscall { number, args, .. } => {
                // Intrinsic direct syscall
                let registers = ["ebx", "ecx", "edx", "esi"];
                for (i, arg) in args.iter().enumerate().take(registers.len()) {
                    asm.push_str(&self.generate_expr(arg)?);
                    asm.push_str(&format!("    push eax # save {}\n", registers[i]));
                }
                // Syscall number into eax
                asm.push_str(&self.generate_expr(number)?);
                // Pop saved registers into target registers in reverse
                for (i, _) in args.iter().enumerate().take(registers.len()).rev() {
                    asm.push_str(&format!("    pop {}\n", registers[i]));
                }
                asm.push_str("    int 0x80\n");
            }
        }

        Ok(asm)
    }
}
