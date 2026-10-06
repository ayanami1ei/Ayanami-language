
use super::*;

const MAX_CONST_DEPTH: usize = 64;
const MAX_CONST_ITERS: u64 = 1_000_000;

pub(crate) enum Flow { Normal, Break, Continue, Return(HirType, HirLiteral) }

impl crate::hir::lower::Ctx {
    pub(crate) fn eval_const_fn(&self, name: Symbol, args: Vec<(HirType, HirLiteral)>, depth: usize) -> Result<(HirType, HirLiteral)> {
        if depth > MAX_CONST_DEPTH {
            return Err(Error::Hir(format!("#[compile_time] recursion limit exceeded at `{}`", name.as_str())));
        }
        let decl = self.const_fns.get(&name).ok_or_else(|| Error::Hir(format!("`{}` is not a #[compile_time] function", name.as_str())))?;
        let Stmt::FnDecl { params, return_type, body, span, .. } = decl else {
            return Err(Error::Hir(format!("`{}` is not a #[compile_time] function", name.as_str())));
        };
        if args.len() != params.len() {
            return Err(Error::Hir(format!("#[compile_time] function `{}` expects {} argument(s), found {}", name.as_str(), params.len(), args.len())));
        }
        let mut env = HashMap::new();
        for ((pname, pty), (mut aty, mut alit)) in params.iter().zip(args.into_iter()) {
            let want = ast_type_to_hir(pty, &self.interfaces);
            super::const_coerce::coerce_literal(&mut aty, &mut alit, &want, span)?;
            env.insert(*pname, (aty, alit));
        }
        match self.exec_const_stmts(&body.stmts, &mut env, depth)? {
            Flow::Return(t, v) => {
                let want = ast_type_to_hir(return_type, &self.interfaces);
                let mut t2 = t;
                let mut v2 = v;
                super::const_coerce::coerce_literal(&mut t2, &mut v2, &want, span)?;
                Ok((want, v2))
            }
            Flow::Normal => {
                if let Some(t) = &body.tail {
                    let (t, v) = self.eval_const_expr(t, &env, depth)?;
                    let want = ast_type_to_hir(return_type, &self.interfaces);
                    let mut t2 = t;
                    let mut v2 = v;
                    super::const_coerce::coerce_literal(&mut t2, &mut v2, &want, span)?;
                    Ok((want, v2))
                } else {
                    Err(Error::Hir(format!("#[compile_time] function `{}` has no return value (at {}:{})", name.as_str(), span.start_line, span.start_col)))
                }
            }
            Flow::Break | Flow::Continue => Err(Error::Hir(format!("break/continue outside loop in #[compile_time] function `{}`", name.as_str()))),
        }
    }

    pub(crate) fn eval_const_block_value(&self, block: &Block, env: &HashMap<Symbol, (HirType, HirLiteral)>, depth: usize) -> Result<(HirType, HirLiteral)> {
        let mut env2 = env.clone();
        match self.exec_const_stmts(&block.stmts, &mut env2, depth)? {
            Flow::Return(t, v) => Ok((t, v)),
            Flow::Normal => {
                if let Some(t) = &block.tail {
                    self.eval_const_expr(t, &env2, depth)
                } else {
                    Err(Error::Hir("const block has no value".into()))
                }
            }
            Flow::Break | Flow::Continue => Err(Error::Hir("break/continue outside loop in const block".into())),
        }
    }

    fn exec_const_stmts(&self, stmts: &[Stmt], env: &mut HashMap<Symbol, (HirType, HirLiteral)>, depth: usize) -> Result<Flow> {
        for stmt in stmts {
            let flow = self.exec_const_stmt(stmt, env, depth)?;
            if !matches!(flow, Flow::Normal) {
                return Ok(flow);
            }
        }
        Ok(Flow::Normal)
    }

    fn exec_const_stmt(&self, stmt: &Stmt, env: &mut HashMap<Symbol, (HirType, HirLiteral)>, depth: usize) -> Result<Flow> {
        match stmt {
            Stmt::Assign { name, value, .. } => {
                let (t, v) = self.eval_const_expr(value, env, depth)?;
                env.insert(*name, (t, v));
                Ok(Flow::Normal)
            }
            Stmt::Return { value, span } => {
                if let Some(v) = value {
                    let (t, v) = self.eval_const_expr(v, env, depth)?;
                    Ok(Flow::Return(t, v))
                } else {
                    Err(Error::Hir(format!("const fn must return a value (at {}:{})", span.start_line, span.start_col)))
                }
            }
            Stmt::If { cond, then_block, elifs, else_block, .. } => {
                let (t, v) = self.eval_const_expr(cond, env, depth)?;
                if t != HirType::Bool {
                    return Err(Error::Hir("if condition must be bool in const evaluation".into()));
                }
                if let HirLiteral::Bool(true) = v {
                    let mut env2 = env.clone();
                    let flow = self.exec_const_stmts(&then_block.stmts, &mut env2, depth)?;
                    if matches!(flow, Flow::Normal) {
                        *env = env2;
                    }
                    return Ok(flow);
                }
                for (c, b) in elifs {
                    let (t, v) = self.eval_const_expr(c, env, depth)?;
                    if t != HirType::Bool {
                        return Err(Error::Hir("elif condition must be bool in const evaluation".into()));
                    }
                    if let HirLiteral::Bool(true) = v {
                        let mut env2 = env.clone();
                        let flow = self.exec_const_stmts(&b.stmts, &mut env2, depth)?;
                        if matches!(flow, Flow::Normal) {
                            *env = env2;
                        }
                        return Ok(flow);
                    }
                }
                if let Some(b) = else_block {
                    let mut env2 = env.clone();
                    let flow = self.exec_const_stmts(&b.stmts, &mut env2, depth)?;
                    if matches!(flow, Flow::Normal) {
                        *env = env2;
                    }
                    return Ok(flow);
                }
                Ok(Flow::Normal)
            }
            Stmt::While { cond, body, .. } => {
                let mut iter = 0u64;
                loop {
                    iter += 1;
                    if iter > MAX_CONST_ITERS {
                        return Err(Error::Hir("const evaluation iteration limit exceeded".into()));
                    }
                    let (t, v) = self.eval_const_expr(cond, env, depth)?;
                    if t != HirType::Bool {
                        return Err(Error::Hir("while condition must be bool in const evaluation".into()));
                    }
                    if let HirLiteral::Bool(false) = v {
                        break;
                    }
                    let mut env2 = env.clone();
                    let flow = self.exec_const_stmts(&body.stmts, &mut env2, depth)?;
                    match flow {
                        Flow::Normal | Flow::Continue => {
                            *env = env2;
                        }
                        Flow::Break => {
                            *env = env2;
                            break;
                        }
                        Flow::Return(t, v) => return Ok(Flow::Return(t, v)),
                    }
                }
                Ok(Flow::Normal)
            }
            Stmt::For { iterator, start, end, step, body, .. } => {
                let (start_t, start_v) = self.eval_const_expr(start, env, depth)?;
                if start_t != HirType::Int {
                    return Err(Error::Hir("for start must be int in const evaluation".into()));
                }
                let (end_t, end_v) = self.eval_const_expr(end, env, depth)?;
                if end_t != HirType::Int {
                    return Err(Error::Hir("for end must be int in const evaluation".into()));
                }
                let step_val = if let Some(s) = step {
                    let (s_t, s_v) = self.eval_const_expr(s, env, depth)?;
                    if s_t != HirType::Int {
                        return Err(Error::Hir("for step must be int in const evaluation".into()));
                    }
                    match s_v {
                        HirLiteral::Int(i) => i,
                        _ => return Err(Error::Hir("for step must be int in const evaluation".into())),
                    }
                } else {
                    1
                };
                if step_val == 0 {
                    return Err(Error::Hir("for step must be non-zero int".into()));
                }
                let start_i = match start_v {
                    HirLiteral::Int(i) => i,
                    _ => return Err(Error::Hir("for start must be int in const evaluation".into())),
                };
                let end_i = match end_v {
                    HirLiteral::Int(i) => i,
                    _ => return Err(Error::Hir("for end must be int in const evaluation".into())),
                };
                let mut iter = 0u64;
                let mut i = start_i;
                loop {
                    iter += 1;
                    if iter > MAX_CONST_ITERS {
                        return Err(Error::Hir("const evaluation iteration limit exceeded".into()));
                    }
                    let cond = if step_val > 0 { i < end_i } else { i > end_i };
                    if !cond {
                        break;
                    }
                    let mut env2 = env.clone();
                    env2.insert(*iterator, (HirType::Int, HirLiteral::Int(i)));
                    let flow = self.exec_const_stmts(&body.stmts, &mut env2, depth)?;
                    match flow {
                        Flow::Normal | Flow::Continue => {
                            *env = env2;
                        }
                        Flow::Break => {
                            *env = env2;
                            break;
                        }
                        Flow::Return(t, v) => return Ok(Flow::Return(t, v)),
                    }
                    i += step_val;
                }
                Ok(Flow::Normal)
            }
            Stmt::ExprStmt { expr, .. } => {
                self.eval_const_expr(expr, env, depth)?;
                Ok(Flow::Normal)
            }
            Stmt::Break { .. } => Ok(Flow::Break),
            Stmt::Continue { .. } => Ok(Flow::Continue),
            Stmt::Attributed { stmt, .. } => self.exec_const_stmt(stmt, env, depth),
            _ => Err(Error::Hir(format!("statement not supported in const evaluation (at {}:{})", stmt.span().start_line, stmt.span().start_col))),
        }
    }
}
