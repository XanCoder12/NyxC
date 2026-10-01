pub mod symbols;
pub mod types;

use crate::ast::*;
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

        Self {
            symbols,
            has_sys_import: false,
        }
    }

    pub fn analyze_program(&mut self, program: &Program) -> Result<(), String> {
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

    fn analyze_fn(&mut self, def: &FnDef, body: &[Stmt]) -> Result<(), String> {
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
    ) -> Result<(), String> {
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
                            return Err(format!(
                                "Type mismatch at {}:{}: cannot assign {:?} to {:?}",
                                span.line, span.col, expr_ty, explicit_ty
                            ));
                        }
                        explicit_ty.clone()
                    }
                    (Some(explicit_ty), None) => explicit_ty.clone(),
                    (None, Some(expr_ty)) => expr_ty.clone(),
                    (None, None) => {
                        return Err(format!(
                            "Variable '{}' at {}:{} requires explicit type or initializer",
                            name, span.line, span.col
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
                    return Err(format!(
                        "Invalid return type at {}:{}: expected {:?}, got {:?}",
                        span.line, span.col, expected_ret, val_ty
                    ));
                }
            }
            Stmt::Expr(expr, _) => {
                self.type_of_expr(expr)?;
            }
            Stmt::If { cond, then_branch, else_branch, .. } => {
                let cond_ty = self.type_of_expr(cond)?;
                if !matches!(cond_ty, Type::Bool | Type::I32 | Type::U32 | Type::Pointer(_)) {
                    return Err("If condition must evaluate to a boolean or integer".into());
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
            Stmt::While { cond, body, .. } => {
                let cond_ty = self.type_of_expr(cond)?;
                if !matches!(cond_ty, Type::Bool | Type::I32 | Type::U32 | Type::Pointer(_)) {
                    return Err("While condition must evaluate to a boolean or integer".into());
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

    pub fn type_of_expr(&self, expr: &Expr) -> Result<Type, String> {
        match expr {
            Expr::IntLit(_, _) => Ok(Type::I32),
            Expr::StringLit(_, _) => Ok(Type::Pointer(Box::new(Type::U8))),
            Expr::BoolLit(_, _) => Ok(Type::Bool),
            Expr::Ident(name, span) => {
                if let Some(sym) = self.symbols.lookup_var(name) {
                    Ok(sym.ty.clone())
                } else {
                    Err(format!("Undefined identifier '{}' at {}:{}", name, span.line, span.col))
                }
            }
            Expr::Path(parts, span) => {
                let full_name = parts.join("::");
                if let Some(fn_sym) = self.symbols.lookup_fn(&full_name) {
                    Ok(fn_sym.ret_type.clone())
                } else {
                    Err(format!("Undefined symbol '{}' at {}:{}", full_name, span.line, span.col))
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
                            return Err(format!(
                                "Type mismatch in binary operation at {}:{}: {:?} and {:?}",
                                span.line, span.col, lty, rty
                            ));
                        }
                        Ok(lty)
                    }
                }
            }
            Expr::Unary(op, inner, _) => {
                let ty = self.type_of_expr(inner)?;
                match op {
                    UnaryOp::Neg | UnaryOp::BitNot => Ok(ty),
                    UnaryOp::Not => Ok(Type::Bool),
                    UnaryOp::AddrOf => Ok(Type::Pointer(Box::new(ty))),
                    UnaryOp::Deref => match ty {
                        Type::Pointer(inner_ty) => Ok(*inner_ty),
                        _ => Err("Cannot dereference non-pointer type".into()),
                    },
                }
            }
            Expr::Call { callee, args, span } => {
                let fn_name = match &**callee {
                    Expr::Ident(name, _) => name.clone(),
                    Expr::Path(parts, _) => parts.join("::"),
                    _ => return Err(format!("Unsupported function call target at {}:{}", span.line, span.col)),
                };

                if let Some(fn_sym) = self.symbols.lookup_fn(&fn_name) {
                    if fn_sym.param_types.len() != args.len() {
                        return Err(format!(
                            "Function '{}' expects {} arguments, but got {} at {}:{}",
                            fn_name,
                            fn_sym.param_types.len(),
                            args.len(),
                            span.line,
                            span.col
                        ));
                    }
                    for (arg, param_ty) in args.iter().zip(&fn_sym.param_types) {
                        let arg_ty = self.type_of_expr(arg)?;
                        if !is_assignable(param_ty, &arg_ty) {
                            return Err(format!(
                                "Argument type mismatch in call to '{}' at {}:{}: expected {:?}, got {:?}",
                                fn_name, span.line, span.col, param_ty, arg_ty
                            ));
                        }
                    }
                    Ok(fn_sym.ret_type.clone())
                } else {
                    Err(format!("Undefined function '{}' at {}:{}", fn_name, span.line, span.col))
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
                    return Err(format!(
                        "Cannot assign type {:?} to {:?} at {}:{}",
                        val_ty, target_ty, span.line, span.col
                    ));
                }
                Ok(target_ty)
            }
        }
    }
}
