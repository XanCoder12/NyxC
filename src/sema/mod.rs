pub mod symbols;
pub mod types;

use crate::ast::*;
use crate::error::CompileError;
use self::symbols::{FnSymbol, Symbol, SymbolTable};
use self::types::is_assignable;

pub struct SemanticAnalyzer {
    pub symbols: SymbolTable,
    pub has_sys_import: bool,
}

impl SemanticAnalyzer {
    pub fn new() -> Self {
        let mut symbols = SymbolTable::new();

        // Built-in NyxaraOS syscall helpers
        symbols.register_fn(FnSymbol {
            name: "sys::exit".into(),
            param_types: vec![Type::I32],
            ret_type: Type::Void,
            is_extern: true,
        });
        symbols.register_fn(FnSymbol {
            name: "sys::write".into(),
            param_types: vec![Type::I32, Type::Pointer(Box::new(Type::U8)), Type::I32],
            ret_type: Type::I32,
            is_extern: true,
        });
        symbols.register_fn(FnSymbol {
            name: "sys::read".into(),
            param_types: vec![Type::I32, Type::Pointer(Box::new(Type::U8)), Type::I32],
            ret_type: Type::I32,
            is_extern: true,
        });
        symbols.register_fn(FnSymbol {
            name: "sys::fork".into(),
            param_types: vec![],
            ret_type: Type::I32,
            is_extern: true,
        });
        symbols.register_fn(FnSymbol {
            name: "sys::waitpid".into(),
            param_types: vec![Type::I32, Type::Pointer(Box::new(Type::I32)), Type::I32],
            ret_type: Type::I32,
            is_extern: true,
        });
        symbols.register_fn(FnSymbol {
            name: "sys::execve".into(),
            param_types: vec![
                Type::Pointer(Box::new(Type::U8)),
                Type::Pointer(Box::new(Type::Pointer(Box::new(Type::U8)))),
                Type::Pointer(Box::new(Type::Pointer(Box::new(Type::U8)))),
            ],
            ret_type: Type::I32,
            is_extern: true,
        });
        symbols.register_fn(FnSymbol {
            name: "sys::getpid".into(),
            param_types: vec![],
            ret_type: Type::I32,
            is_extern: true,
        });

        // Compiler built-in: length of a NUL-terminated string, inlined by codegen
        symbols.register_fn(FnSymbol {
            name: "str_len".into(),
            param_types: vec![Type::Pointer(Box::new(Type::U8))],
            ret_type: Type::I32,
            is_extern: false,
        });

        Self {
            symbols,
            has_sys_import: false,
        }
    }

    pub fn analyze_program(&mut self, program: &Program) -> Result<(), CompileError> {
        // First pass: register imports and function signatures
        for item in &program.items {
            match item {
                Item::Import(imp) => {
                    if imp.path == "nyx/sys" {
                        self.has_sys_import = true;
                    }
                }
                Item::Fn(f) => {
                    let fn_sym = FnSymbol {
                        name: f.name.clone(),
                        param_types: f.params.iter().map(|p| p.ty.clone()).collect(),
                        ret_type: f.ret_ty.clone(),
                        is_extern: f.is_extern,
                    };
                    self.symbols.register_fn(fn_sym);
                }
                Item::Const(_) => {}
            }
        }

        // Second pass: analyze function bodies
        for item in &program.items {
            if let Item::Fn(f) = item {
                if let Some(body) = &f.body {
                    self.analyze_fn(f, body)?;
                }
            }
        }

        Ok(())
    }

    fn analyze_fn(&mut self, def: &FnDef, body: &[Stmt]) -> Result<(), CompileError> {
        self.symbols.enter_scope();

        // Register parameters with positive offsets: EBP + 8, EBP + 12, etc.
        let mut param_offset = 8;
        for param in &def.params {
            self.symbols.record_fn_var(&def.name, Symbol {
                name: param.name.clone(),
                ty: param.ty.clone(),
                is_mut: true,
                offset: param_offset,
            });
            param_offset += 4;
        }

        // Track local variable offsets (negative from EBP)
        let mut local_offset = -4;
        for stmt in body {
            self.analyze_stmt(stmt, &def.name, &def.ret_ty, &mut local_offset)?;
        }

        self.symbols.exit_scope();
        Ok(())
    }

    fn analyze_stmt(
        &mut self,
        stmt: &Stmt,
        fn_name: &str,
        expected_ret: &Type,
        local_offset: &mut i32,
    ) -> Result<(), CompileError> {
        match stmt {
            Stmt::Let { name, is_mut, ty, init, span } => {
                let init_ty = if let Some(expr) = init {
                    Some(self.type_of_expr(expr)?)
                } else {
                    None
                };

                let var_ty = match (ty, &init_ty) {
                    (Some(explicit_ty), Some(expr_ty)) => {
                        if !is_assignable(explicit_ty, expr_ty) {
                            return Err(CompileError::new(
                                "SEM_003",
                                format!(
                                    "type mismatch: cannot assign {} to {}",
                                    expr_ty.name(),
                                    explicit_ty.name()
                                ),
                                Some(*span),
                            ));
                        }
                        explicit_ty.clone()
                    }
                    (Some(explicit_ty), None) => explicit_ty.clone(),
                    (None, Some(expr_ty)) => expr_ty.clone(),
                    (None, None) => {
                        return Err(CompileError::new(
                            "SEM_009",
                            format!(
                                "variable '{}' needs an explicit type or an initializer",
                                name
                            ),
                            Some(*span),
                        ));
                    }
                };

                self.symbols.record_fn_var(fn_name, Symbol {
                    name: name.clone(),
                    ty: var_ty,
                    is_mut: *is_mut,
                    offset: *local_offset,
                });
                *local_offset -= 4;
            }
            Stmt::Return { value, span } => {
                let val_ty = if let Some(expr) = value {
                    self.type_of_expr(expr)?
                } else {
                    Type::Void
                };

                if !is_assignable(expected_ret, &val_ty) {
                    return Err(CompileError::new(
                        "SEM_005",
                        format!(
                            "invalid return type: expected {}, got {}",
                            expected_ret.name(),
                            val_ty.name()
                        ),
                        Some(*span),
                    ));
                }
            }
            Stmt::Expr(expr, _) => {
                self.type_of_expr(expr)?;
            }
            Stmt::If { cond, then_branch, else_branch, span } => {
                let cond_ty = self.type_of_expr(cond)?;
                if !matches!(cond_ty, Type::Bool | Type::I32 | Type::U32 | Type::Pointer(_)) {
                    return Err(CompileError::new(
                        "SEM_008",
                        format!(
                            "if condition must be a bool, integer, or pointer, found {}",
                            cond_ty.name()
                        ),
                        Some(*span),
                    ));
                }
                for s in then_branch {
                    self.analyze_stmt(s, fn_name, expected_ret, local_offset)?;
                }
                if let Some(else_stmts) = else_branch {
                    for s in else_stmts {
                        self.analyze_stmt(s, fn_name, expected_ret, local_offset)?;
                    }
                }
            }
            Stmt::While { cond, body, span } => {
                let cond_ty = self.type_of_expr(cond)?;
                if !matches!(cond_ty, Type::Bool | Type::I32 | Type::U32 | Type::Pointer(_)) {
                    return Err(CompileError::new(
                        "SEM_008",
                        format!(
                            "while condition must be a bool, integer, or pointer, found {}",
                            cond_ty.name()
                        ),
                        Some(*span),
                    ));
                }
                for s in body {
                    self.analyze_stmt(s, fn_name, expected_ret, local_offset)?;
                }
            }
            Stmt::Block(stmts, _) => {
                self.symbols.enter_scope();
                for s in stmts {
                    self.analyze_stmt(s, fn_name, expected_ret, local_offset)?;
                }
                self.symbols.exit_scope();
            }
            Stmt::Const { .. } => {}
        }
        Ok(())
    }

    pub fn type_of_expr(&self, expr: &Expr) -> Result<Type, CompileError> {
        match expr {
            Expr::IntLit(_, _) => Ok(Type::I32),
            Expr::StringLit(_, _) => Ok(Type::Pointer(Box::new(Type::U8))),
            Expr::BoolLit(_, _) => Ok(Type::Bool),
            Expr::Ident(name, span) => {
                if let Some(sym) = self.symbols.lookup_var(name) {
                    Ok(sym.ty.clone())
                } else {
                    Err(CompileError::new(
                        "SEM_001",
                        format!("cannot find value '{}' in this scope", name),
                        Some(*span),
                    ))
                }
            }
            Expr::Path(parts, span) => {
                let full_name = parts.join("::");
                if let Some(fn_sym) = self.symbols.lookup_fn(&full_name) {
                    Ok(fn_sym.ret_type.clone())
                } else {
                    Err(CompileError::new(
                        "SEM_002",
                        format!("cannot find symbol '{}' in this scope", full_name),
                        Some(*span),
                    ))
                }
            }
            Expr::Binary(op, left, right, span) => {
                let lty = self.type_of_expr(left)?;
                let rty = self.type_of_expr(right)?;

                match op {
                    BinaryOp::Eq | BinaryOp::Ne | BinaryOp::Lt | BinaryOp::Le | BinaryOp::Gt | BinaryOp::Ge => {
                        Ok(Type::Bool)
                    }
                    BinaryOp::And | BinaryOp::Or => Ok(Type::Bool),
                    _ => {
                        if !is_assignable(&lty, &rty) {
                            return Err(CompileError::new(
                                "SEM_003",
                                format!(
                                    "type mismatch in binary operation: {} and {}",
                                    lty.name(),
                                    rty.name()
                                ),
                                Some(*span),
                            ));
                        }
                        Ok(lty)
                    }
                }
            }
            Expr::Unary(op, inner, span) => {
                let ty = self.type_of_expr(inner)?;
                match op {
                    UnaryOp::Neg | UnaryOp::BitNot => Ok(ty),
                    UnaryOp::Not => Ok(Type::Bool),
                    UnaryOp::AddrOf => Ok(Type::Pointer(Box::new(ty))),
                    UnaryOp::Deref => match ty {
                        Type::Pointer(inner_ty) => Ok(*inner_ty),
                        _ => Err(CompileError::new(
                            "SEM_010",
                            format!("cannot dereference a non-pointer value of type {}", ty.name()),
                            Some(*span),
                        )),
                    },
                }
            }
            Expr::Call { callee, args, span } => {
                let fn_name = match &**callee {
                    Expr::Ident(name, _) => name.clone(),
                    Expr::Path(parts, _) => parts.join("::"),
                    _ => {
                        return Err(CompileError::new(
                            "SEM_011",
                            "function call target must be an identifier or a path",
                            Some(*span),
                        ))
                    }
                };

                if let Some(fn_sym) = self.symbols.lookup_fn(&fn_name) {
                    if fn_sym.param_types.len() != args.len() {
                        return Err(CompileError::new(
                            "SEM_006",
                            format!(
                                "function '{}' takes {} arguments ({} provided)",
                                fn_name,
                                fn_sym.param_types.len(),
                                args.len()
                            ),
                            Some(*span),
                        ));
                    }
                    for (arg, param_ty) in args.iter().zip(&fn_sym.param_types) {
                        let arg_ty = self.type_of_expr(arg)?;
                        if !is_assignable(param_ty, &arg_ty) {
                            return Err(CompileError::new(
                                "SEM_007",
                                format!(
                                    "argument type mismatch in call to '{}': expected {}, got {}",
                                    fn_name,
                                    param_ty.name(),
                                    arg_ty.name()
                                ),
                                Some(*span),
                            ));
                        }
                    }
                    Ok(fn_sym.ret_type.clone())
                } else {
                    Err(CompileError::new(
                        "SEM_002",
                        format!("cannot find function '{}' in this scope", fn_name),
                        Some(*span),
                    ))
                }
            }
            Expr::Syscall { number, args, .. } => {
                self.type_of_expr(number)?;
                for arg in args {
                    self.type_of_expr(arg)?;
                }
                Ok(Type::I32)
            }
            Expr::Assign { target, value, span } => {
                let target_ty = self.type_of_expr(target)?;
                let val_ty = self.type_of_expr(value)?;
                if !is_assignable(&target_ty, &val_ty) {
                    return Err(CompileError::new(
                        "SEM_012",
                        format!(
                            "cannot assign {} to {}",
                            val_ty.name(),
                            target_ty.name()
                        ),
                        Some(*span),
                    ));
                }
                Ok(target_ty)
            }
        }
    }
}
