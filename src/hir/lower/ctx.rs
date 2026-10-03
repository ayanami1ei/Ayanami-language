use super::*;

// ============================================================
//  Ctx —— HIR 降级上下文
//  负责管理整个 HIR 降级过程中的状态：
//  - 函数注册与重载解析
//  - 接口与虚函数表
//  - 结构体定义
//  - 泛型函数特化
//  - 变量作用域
// ============================================================

/// HIR（高级中间表示）降级上下文
///
/// 保存降级过程中的全部状态，包括：
/// - 所有已注册的函数签名（支持重载）
/// - 接口定义与虚函数表（vtable）
/// - 结构体字段定义
/// - 泛型函数的 AST 与特化结果
/// - 当前作用域中的变量绑定
pub(crate) struct Ctx {
    /// 所有已注册的函数（按 FnId 索引）
    pub fns: Vec<FnSig>,
    /// 函数名到 FnId 列表的映射（支持重载同名函数）
    pub fn_map: HashMap<Symbol, Vec<FnId>>,
    /// 注册的接口：接口名 → 方法签名
    pub interfaces: HashMap<Symbol, InterfaceReg>,
    /// 虚函数表列表：(具体类型, 接口) → 方法 FnId 列表
    pub vtables: Vec<VtableEntry>,
    /// 具体类型名 → 它实现了哪些接口
    pub type_ifaces: HashMap<Symbol, Vec<Symbol>>,
    /// 结构体定义：结构体名 → 字段列表
    pub struct_defs: HashMap<Symbol, Vec<HirStructField>>,
    /// 泛型结构体参数：结构体名 → [(参数名, 约束接口)]
    pub generic_struct_params: HashMap<Symbol, Vec<(Symbol, Option<Symbol>)>>,
    /// 泛型函数 AST：(函数名, 泛型参数列表, FnDecl 语句)
    pub generic_fns: Vec<(Symbol, Vec<(Symbol, Option<Symbol>)>, Stmt)>,
    /// 降级过程中特化（单态化）产生的泛型函数
    pub specialized_fns: Vec<HirFn>,
    /// Lambda 计数器（生成唯一名称）
    pub lambda_counter: u64,
    /// Lambda 表达式降级产生的匿名函数
    pub lambda_fns: Vec<HirFn>,
    /// 当前正在降级的函数 ID
    pub current_fn: FnId,
    /// 当前函数的局部变量列表
    pub locals: Vec<HirLocal>,
    /// 作用域栈：每层作用域是 变量名 → (VarId, 类型, 是否可变)
    pub scopes: Vec<HashMap<Symbol, (VarId, HirType, bool)>>,
    /// 是否允许裸数组字面量（在 ToShared/ToUnique/ToWeak 内允许）
    pub allow_bare_array: bool,
    /// A3d：表达式降级过程中产生的待插入语句（如 `?` 的早退控制流）
    pub pending_stmts: Vec<HirStmt>,
    /// A5b：包导入的宏表（包 stem → 宏名）
    pub imported_macros: HashMap<Symbol, Vec<Symbol>>,
    /// A5d：导入包的优化注解表（pass 名）
    pub imported_passes: HashMap<Symbol, Vec<Symbol>>,
}

impl Ctx {
    /// 创建新的降级上下文
    pub fn new() -> Self {
        // 内置 String 结构体：字符串字面量无需 import 即可使用
        let mut struct_defs = HashMap::new();
        struct_defs.insert(
            Symbol::intern("String"),
            vec![
                HirStructField { name: Symbol::intern("data"), ty: HirType::Unique(Box::new(HirType::Array(Box::new(HirType::Char)))) },
                HirStructField { name: Symbol::intern("len"), ty: HirType::Int },
            ],
        );
        Self {
            fns: Vec::new(),
            fn_map: HashMap::new(),
            interfaces: HashMap::new(),
            vtables: Vec::new(),
            type_ifaces: HashMap::new(),
            struct_defs,
            generic_struct_params: HashMap::new(),
            generic_fns: Vec::new(),
            specialized_fns: Vec::new(),
            lambda_counter: 0,
            lambda_fns: Vec::new(),
            current_fn: FnId(0),
            locals: Vec::new(),
            scopes: Vec::new(),
            allow_bare_array: false,
            pending_stmts: Vec::new(),
            imported_macros: HashMap::new(),
            imported_passes: HashMap::new(),
        }
    }

    // ----------------------------------------------------------------
    //  作用域管理 —— 进入/退出作用域、变量绑定与查找
    // ----------------------------------------------------------------

    /// 进入新作用域（创建新的变量映射表）
    pub fn push_scope(&mut self) { self.scopes.push(HashMap::new()); }

    /// 退出当前作用域（丢弃该层所有变量绑定）
    pub fn pop_scope(&mut self) { self.scopes.pop(); }

    /// 在当前作用域中绑定变量
    pub fn bind_var(&mut self, name: Symbol, id: VarId, ty: HirType, mutable: bool) {
        if let Some(scope) = self.scopes.last_mut() { scope.insert(name, (id, ty, mutable)); }
    }

    /// 按变量名查找绑定，从内层作用域向外层搜索
    pub fn lookup_var(&self, name: &Symbol) -> Option<(VarId, HirType, bool)> {
        for scope in self.scopes.iter().rev() {
            if let Some(v) = scope.get(name) { return Some(v.clone()); }
        }
        None
    }

    /// 查找结构体中某字段的索引位置
    pub fn find_field_index(&self, struct_ty: &HirType, field: &Symbol, span: &Span) -> Result<usize> {
        let type_name = match struct_ty {
            HirType::Named(n) => *n,
            HirType::Unique(inner) | HirType::Ref(inner, _) => {
                return self.find_field_index(inner, field, span);
            }
            _ => return Err(Error::Hir(format!("类型 {} 没有字段 `{}` (位置 {}:{})",
                hir_type_display(struct_ty), field, span.start_line, span.start_col))),
        };
        self.find_field_index_by_name(&type_name, field, span)
    }

    /// 按类型名查找字段索引（含泛型回退与替换）
    fn find_field_index_by_name(&self, type_name: &Symbol, field: &Symbol, span: &Span) -> Result<usize> {
        if let Some(fields) = self.struct_defs.get(type_name) {
            return fields.iter().position(|f| f.name == *field)
                .ok_or_else(|| Error::Hir(format!("结构体 `{}` 没有字段 `{}` (位置 {}:{})", type_name, field, span.start_line, span.start_col)));
        }
        let base = strip_generic_name(type_name);
        if base != *type_name {
            if let Some(fields) = self.struct_defs.get(&base) {
                // 从类型名中提取泛型替换 e.g. LinkedListNode<int> → T=int
                return fields.iter().position(|f| f.name == *field)
                    .ok_or_else(|| Error::Hir(format!("结构体 `{}` 没有字段 `{}` (位置 {}:{})", type_name, field, span.start_line, span.start_col)));
            }
        }
        Err(Error::Hir(format!("未知结构体 `{}` (位置 {}:{})", type_name, span.start_line, span.start_col)))
    }

    /// 查找结构体中某字段的类型
    pub fn find_field_type(&self, struct_ty: &HirType, field: &Symbol, span: &Span) -> Result<HirType> {
        let type_name = match struct_ty {
            HirType::Named(n) => *n,
            HirType::Unique(inner) | HirType::Ref(inner, _) => {
                let inner_name = match inner.as_ref() {
                    HirType::Named(n) => *n,
                    _ => return Err(Error::Hir(format!("类型 {} 没有字段 `{}` (位置 {}:{})",
                        hir_type_display(struct_ty), field, span.start_line, span.start_col))),
                };
                inner_name
            }
            _ => return Err(Error::Hir(format!("类型 {} 没有字段 `{}` (位置 {}:{})",
                hir_type_display(struct_ty), field, span.start_line, span.start_col))),
        };
        self.find_field_type_by_name(&type_name, field, span)
    }

    /// 按类型名查找字段类型（含泛型回退与替换）
    fn find_field_type_by_name(&self, type_name: &Symbol, field: &Symbol, span: &Span) -> Result<HirType> {
        if let Some(fields) = self.struct_defs.get(type_name) {
            return fields.iter().find(|f| f.name == *field)
                .map(|f| f.ty.clone())
                .ok_or_else(|| Error::Hir(format!("结构体 `{}` 没有字段 `{}` (位置 {}:{})", type_name, field, span.start_line, span.start_col)));
        }
        let base = strip_generic_name(type_name);
        if base != *type_name {
            if let Some(fields) = self.struct_defs.get(&base) {
                let subst = self.build_generic_subst(type_name, &base);
                return fields.iter().find(|f| f.name == *field)
                    .map(|f| substitute_hir_type(&f.ty, &subst))
                    .ok_or_else(|| Error::Hir(format!("结构体 `{}` 没有字段 `{}` (位置 {}:{})", type_name, field, span.start_line, span.start_col)));
            }
        }
        Err(Error::Hir(format!("未知结构体 `{}` (位置 {}:{})", type_name, span.start_line, span.start_col)))
    }

    /// A3d：取枚举 `_data_X` 变体结构体的首个载荷字段类型（处理泛型替换）。
    pub fn variant_payload_type(&self, enum_ty: &HirType, data_field: &Symbol, span: &Span) -> Result<HirType> {
        let var_ty = self.find_field_type(enum_ty, data_field, span)?;
        let var_name = match &var_ty {
            HirType::Named(n) => *n,
            _ => return Ok(var_ty),
        };
        let fields = self.struct_defs.get(&var_name)
            .ok_or_else(|| Error::Hir(format!("未知变体结构体 `{}` (位置 {}:{})", var_name, span.start_line, span.start_col)))?;
        let first = match fields.first() {
            Some(f) => f.ty.clone(),
            None => return Ok(HirType::Void),
        };
        let enum_name = match enum_ty {
            HirType::Named(n) => *n,
            _ => return Ok(first),
        };
        let base = strip_generic_name(&enum_name);
        let subst = self.build_generic_subst(&enum_name, &base);
        Ok(substitute_hir_type(&first, &subst))
    }

    /// 从完整类型名（含泛型参数）构建替换映射
    pub(crate) fn build_generic_subst(&self, type_name: &Symbol, base: &Symbol) -> HashMap<Symbol, HirType> {
        let mut subst = HashMap::new();
        let s = type_name.as_str();
        let b = base.as_str();
        if let Some(start) = s.find('<') {
            if &s[..start] == b {
                let inner = s[start..].trim_start_matches('<').trim_end_matches('>');
                let inner_parts: Vec<&str> = inner.split(',').collect();
                let gp = self.collected_generic_params(base);
                for ((gp_name, _), val_str) in gp.iter().zip(inner_parts.iter()) {
                    let hir_ty = sig_str_to_hir(val_str.trim());
                    subst.insert(*gp_name, hir_ty);
                }
            }
        }
        subst
    }

    /// 获取结构体的泛型参数列表（如有）
    pub fn collected_generic_params(&self, type_name: &Symbol) -> Vec<(Symbol, Option<Symbol>)> {
        self.generic_struct_params.get(type_name).cloned().unwrap_or_default()
    }

    /// 注册或查找变量：如已存在则返回已有的，否则创建新变量
    pub fn register_or_lookup(&mut self, name: Symbol, inferred_ty: HirType) -> (VarId, HirType, bool) {
        if let Some(existing) = self.lookup_var(&name) { return existing; }
        let id = VarId(self.locals.len());
        self.locals.push(HirLocal { name, ty: inferred_ty.clone(), mutable: false });
        self.bind_var(name, id, inferred_ty.clone(), false);
        (id, inferred_ty, false)
    }

    /// 更新变量的类型（用于泛型推导后更新变量类型）
    pub fn update_var_type(&mut self, var_id: VarId, new_ty: HirType) {
        if let Some(local) = self.locals.get_mut(var_id.0) {
            local.ty = new_ty.clone();
        }
        for scope in self.scopes.iter_mut() {
            for (_, (id, ty, _mutable)) in scope.iter_mut() {
                if *id == var_id {
                    *ty = new_ty;
                    return;
                }
            }
        }
    }

    /// Check if a type is an enum (has _tag field as first field)
    pub fn is_enum_type(&self, type_name: &Symbol) -> bool {
        self.struct_defs.get(type_name)
            .and_then(|fields| fields.first())
            .map(|f| f.name.as_str() == "_tag")
            .unwrap_or(false)
    }
}
