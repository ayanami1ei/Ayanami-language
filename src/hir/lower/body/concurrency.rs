//! M7：跨线程调用边界与结构递归的 Send 检查（docs/concurrency-model.md）。

use std::collections::HashSet;

use super::*;

impl crate::hir::lower::Ctx {
    /// 对声明 `#[cross_thread]` 的被调函数检查闭包实参的捕获环境。
    pub(super) fn check_cross_thread_call(
        &self,
        fn_id: FnId,
        args: &[HirNodeBox],
        span: &Span,
    ) -> Result<()> {
        if !self.fns[fn_id.0].effects.has_effect("cross_thread") {
            return Ok(());
        }
        let params = &self.fns[fn_id.0].params;
        for (arg, (_, param_ty)) in args.iter().zip(params.iter()) {
            if !matches!(param_ty, HirType::Closure(..)) {
                continue;
            }
            let Some(captures) = self.closure_capture_types(arg) else {
                if self.in_unsafe() {
                    continue;
                }
                return Err(Error::Hir(format!(
                    "thread entry closure provenance is not statically Send; pass a closure literal/local directly or use explicit `unsafe` (at {}:{})",
                    span.start_line, span.start_col
                )));
            };
            for ty in captures {
                if !self.is_send_capture_type(&ty, &mut HashSet::new()) {
                    if self.in_unsafe() {
                        break;
                    }
                    return Err(Error::Hir(format!(
                        "thread entry captures non-Send `{}`; move an owned value or use explicit `unsafe` (at {}:{})",
                        hir_type_display(&ty), span.start_line, span.start_col
                    )));
                }
            }
        }
        Ok(())
    }

    /// 记录闭包局部值的环境字段类型；非闭包赋值会清除同槽旧记录。
    pub(super) fn record_closure_captures(&mut self, var: VarId, value: &HirNodeBox) {
        let key = (self.current_fn, var);
        match self.closure_capture_types(value) {
            Some(types) => { self.closure_captures.insert(key, types); }
            None => { self.closure_captures.remove(&key); }
        }
    }

    fn closure_capture_types(&self, node: &HirNodeBox) -> Option<Vec<HirType>> {
        if let Some(mfp) = node.as_mfp_cloned() {
            return match &mfp.ty {
                HirType::Closure(_, _, false, _) => Some(Vec::new()),
                HirType::Closure(_, _, true, _) => {
                    let env = mfp.value.as_struct_cloned()?;
                    Some(env.fields.iter().map(|(_, v)| expr_type(v)).collect())
                }
                _ => None,
            };
        }
        if let Some(inner) = node.as_move().or_else(|| node.as_clone()) {
            return self.closure_capture_types(inner);
        }
        node.as_local().and_then(|var| {
            self.closure_captures.get(&(self.current_fn, var)).cloned()
        })
    }

    fn is_send_capture_type(&self, ty: &HirType, seen: &mut HashSet<Symbol>) -> bool {
        match ty {
            HirType::Ref(..) | HirType::RawPtr(..) | HirType::FatPtr { .. } => false,
            HirType::Unique(inner) | HirType::Array(inner) => {
                self.is_send_capture_type(inner, seen)
            }
            HirType::ArraySized(inner, _) => self.is_send_capture_type(inner, seen),
            HirType::Named(name) => {
                if !seen.insert(*name) {
                    return true;
                }
                let send = self.struct_defs.get(name).map_or(true, |fields| {
                    fields.iter().all(|field| self.is_send_capture_type(&field.ty, seen))
                });
                seen.remove(name);
                send
            }
            // 拥有闭包类型本身不携带捕获字段表；直接闭包值由外层环境递归检查。
            HirType::Closure(_, _, owns_env, _) => !owns_env,
            _ => true,
        }
    }
}
