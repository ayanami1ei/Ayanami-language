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
    pub generic_struct_params: HashMap<Symbol, Vec<(Symbol, Vec<Symbol>)>>,
    /// 泛型函数 AST：(函数名, 泛型参数列表, FnDecl 语句)
    pub generic_fns: Vec<(Symbol, Vec<(Symbol, Vec<Symbol>)>, Stmt)>,
    /// 降级过程中特化（单态化）产生的泛型函数
    pub specialized_fns: Vec<HirFn>,
    /// M1.7：合成的外部运行时助手（溢出检查等）
    pub synth_externs: Vec<HirFn>,
    /// M1.7：溢出助手去重（名称 → FnId）
    pub ovf_helpers: HashMap<Symbol, FnId>,
    /// M6.1：编译期常量（名 → (类型, 值)），使用点内联为 SConst
    pub consts: HashMap<Symbol, (HirType, HirLiteral)>,
    /// M6.1b：可导出常量（规范化名 → (类型, 值)）；顶层裸名、命名空间限定名
    pub const_exports: Vec<(Symbol, HirType, HirLiteral)>,
    /// M6.2：全局变量（名 → 定义），可寻址
    pub statics: HashMap<Symbol, crate::hir::HirStatic>,
    /// M6.3：`const fn` AST（名 → FnDecl），常量上下文编译期求值
    pub const_fns: HashMap<Symbol, Stmt>,
    /// Lambda / for-in 迭代器临时变量计数器（生成唯一卫生名）
    pub lambda_counter: u64,
    pub for_iter_counter: u64,
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
    /// A5d：导入包的只读检查注解表（check 名）
    pub imported_checks: HashMap<Symbol, Vec<Symbol>>,
    /// 泛型特化产生的函数实例（重载解析的隐式转换回退不参与，避免串型）
    pub specialized_ids: std::collections::HashSet<crate::hir::ty::FnId>,
    /// 后续用法推断出的泛型实参：变量名 → 实参类型
    pub usage_hints: HashMap<Symbol, Vec<Type>>,
    /// A5c-2：导入包导出的函数宏（宏名 → lcl 路径，可多个用于歧义报错）
    pub imported_macro_lcls: HashMap<Symbol, Vec<String>>,
    /// A5c-2：宏展开递归深度
    pub macro_depth: usize,
    /// M2：当前正在降级的 lambda 的捕获环境（捕获名 → env 字段）
    pub lambda_env: Option<crate::hir::lower::helpers::LambdaEnv>,
    /// M2：静态闭包 trampoline 缓存 + extern "C" 声明（无体）函数 id（实参借用）
    pub closure_tramps: HashMap<Symbol, (Symbol, Symbol)>,
    pub extern_fn_ids: std::collections::HashSet<FnId>,
    /// M5：unsafe 块嵌套深度
    pub unsafe_depth: usize,
    /// M5：当前函数是否为 `unsafe fn`（函数体整体处于 unsafe 上下文）
    pub cur_fn_unsafe: bool,
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
            synth_externs: Vec::new(),
            ovf_helpers: HashMap::new(),
            consts: HashMap::new(),
            const_exports: Vec::new(),
            statics: HashMap::new(),
            const_fns: HashMap::new(),
            lambda_counter: 0, for_iter_counter: 0,
            lambda_fns: Vec::new(),
            current_fn: FnId(0),
            locals: Vec::new(),
            scopes: Vec::new(),
            allow_bare_array: false,
            pending_stmts: Vec::new(),
            imported_macros: HashMap::new(),
            imported_passes: HashMap::new(),
            imported_checks: HashMap::new(),
            specialized_ids: std::collections::HashSet::new(),
            usage_hints: HashMap::new(),
            imported_macro_lcls: HashMap::new(),
            macro_depth: 0,
            lambda_env: None,
            closure_tramps: HashMap::new(),
            extern_fn_ids: std::collections::HashSet::new(),
            unsafe_depth: 0,
            cur_fn_unsafe: false,
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

    /// 注册或查找变量：如已存在则返回已有的，否则创建新变量
    pub fn register_or_lookup(&mut self, name: Symbol, inferred_ty: HirType) -> (VarId, HirType, bool) {
        if let Some(existing) = self.lookup_var(&name) { return existing; }
        let id = VarId(self.locals.len());
        self.locals.push(HirLocal { name, ty: inferred_ty.clone(), mutable: false, is_result: false });
        self.bind_var(name, id, inferred_ty.clone(), false);
        (id, inferred_ty, false)
    }

    /// Check if a type is an enum (has _tag field as first field)
    /// 接收者类型是否已知（本地定义/导入/接口）；用于“是否缺少 import”提示
    pub(crate) fn receiver_type_known(&self, ty: &HirType) -> bool {
        match strip_ownership_ref(ty) {
            HirType::Named(n) => {
                let base = crate::hir::lower::strip_generic_name(n);
                self.struct_defs.contains_key(n)
                    || self.struct_defs.contains_key(&base)
                    || self.is_enum_type(n)
                    || self.type_ifaces.contains_key(n)
            }
            _ => true,
        }
    }

    pub fn is_enum_type(&self, type_name: &Symbol) -> bool {
        self.struct_defs.get(type_name).and_then(|f| f.first())
            .map(|f| f.name.as_str() == "_tag").unwrap_or(false)
    }
}
