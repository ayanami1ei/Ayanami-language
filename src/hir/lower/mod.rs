pub mod body;
pub mod helpers;
pub(crate) use helpers::*;

use std::collections::HashMap;
use crate::intern::Symbol;
use crate::parser::ast::*;
use crate::span::Span;
use super::ir::*;

/// 从类型名中剥离泛型参数
/// 例如 "LinkedListNode<T>" → "LinkedListNode"，也处理 "LinkedListNode[T]"
pub(super) fn strip_generic_name(name: &Symbol) -> Symbol {
    let s = name.as_str();
    let pos = s.find('<').or_else(|| s.find('['));
    if let Some(p) = pos {
        Symbol::intern(&s[..p])
    } else {
        *name
    }
}

// ============================================================
//  类型定义：HIR 降级过程中使用的内部数据结构
// ============================================================

/// 函数签名 —— 用于函数重载解析和虚函数表构建
#[derive(Clone)]
pub(crate) struct FnSig {
    /// 函数名称
    pub name: Symbol,
    /// 参数列表：(参数名, 参数类型)
    pub params: Vec<(Symbol, HirType)>,
    /// 返回值类型
    pub return_type: HirType,
}

/// 接口注册信息 —— 记录接口的泛型参数和方法签名
#[derive(Clone)]
pub(crate) struct InterfaceReg {
    /// 泛型参数列表：(参数名, 约束接口名)
    pub generic_params: Vec<(Symbol, Option<Symbol>)>,
    /// 接口中定义的方法列表
    pub methods: Vec<HirInterfaceMethod>,
}

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
    /// 当前正在降级的函数 ID
    pub current_fn: FnId,
    /// 当前函数的局部变量列表
    pub locals: Vec<HirLocal>,
    /// 作用域栈：每层作用域是 变量名 → (VarId, 类型, 是否可变)
    pub scopes: Vec<HashMap<Symbol, (VarId, HirType, bool)>>,
    /// 是否允许裸数组字面量（在 ToShared/ToUnique/ToWeak 内允许）
    pub allow_bare_array: bool,
}

impl Ctx {
    /// 创建新的降级上下文
    pub fn new() -> Self {
        Self {
            fns: Vec::new(),
            fn_map: HashMap::new(),
            interfaces: HashMap::new(),
            vtables: Vec::new(),
            type_ifaces: HashMap::new(),
            struct_defs: HashMap::new(),
            generic_struct_params: HashMap::new(),
            generic_fns: Vec::new(),
            specialized_fns: Vec::new(),
            current_fn: FnId(0),
            locals: Vec::new(),
            scopes: Vec::new(),
            allow_bare_array: false,
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
    pub fn find_field_index(&self, struct_ty: &HirType, field: &Symbol, span: &Span) -> Result<usize, String> {
        let type_name = match struct_ty {
            HirType::Named(n) => *n,
            HirType::Shared(inner) | HirType::Unique(inner) | HirType::Weak(inner) => {
                return self.find_field_index(inner, field, span);
            }
            _ => return Err(format!("类型 {} 没有字段 `{}` (位置 {}:{})",
                hir_type_display(struct_ty), field, span.start_line, span.start_col)),
        };
        let base_name = strip_generic_name(&type_name);
        let raw = type_name.as_str();
        let fields = self.struct_defs.get(&base_name)
            .ok_or_else(|| format!("未知结构体 `{}` (位置 {}:{})", type_name, span.start_line, span.start_col))?;
        fields.iter().position(|f| f.name == *field)
            .ok_or_else(|| format!("结构体 `{}` 没有字段 `{}` (位置 {}:{})", type_name, field, span.start_line, span.start_col))
    }

    /// 查找结构体中某字段的类型
    pub fn find_field_type(&self, struct_ty: &HirType, field: &Symbol, span: &Span) -> Result<HirType, String> {
        let type_name = match struct_ty {
            HirType::Named(n) => *n,
            HirType::Shared(inner) | HirType::Unique(inner) | HirType::Weak(inner) => {
                let inner_name = match inner.as_ref() {
                    HirType::Named(n) => *n,
                    _ => return Err(format!("类型 {} 没有字段 `{}` (位置 {}:{})",
                        hir_type_display(struct_ty), field, span.start_line, span.start_col)),
                };
                inner_name
            }
            _ => return Err(format!("类型 {} 没有字段 `{}` (位置 {}:{})",
                hir_type_display(struct_ty), field, span.start_line, span.start_col)),
        };
        let base_name = strip_generic_name(&type_name);
        let fields = self.struct_defs.get(&base_name)
            .ok_or_else(|| format!("未知结构体 `{}` (位置 {}:{})", type_name, span.start_line, span.start_col))?;
        fields.iter().find(|f| f.name == *field)
            .map(|f| f.ty.clone())
            .ok_or_else(|| format!("结构体 `{}` 没有字段 `{}` (位置 {}:{})", type_name, field, span.start_line, span.start_col))
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
}

// ============================================================
//  入口函数 —— lower_program
//  执行完整的 AST → HIR 降级流程
// ============================================================

/// 将抽象语法树（AST）降级为高级中间表示（HIR）
///
/// 执行三个阶段：
/// 1. 收集阶段（collect_fns）：注册所有函数和接口签名
/// 2. 构建阶段（build_vtables）：验证实现签名并构建虚函数表
/// 3. 降级阶段（lower_items）：递归处理所有语句/表达式，生成 HIR 节点
///
/// 同时收集过程中产生的特化泛型函数，以及从其他模块导入的函数签名。
pub fn lower_program(program: &Program) -> Result<HirProgram, String> {
    let mut ctx = Ctx::new();
    ctx.collect_fns(&program.stmts)?;
    ctx.build_vtables()?;
    let mut items = ctx.lower_items(&program.stmts)?;

    // 收集已定义函数的 ID，找出哪些是外部导入的
    let defined_ids: std::collections::HashSet<_> = items.iter().filter_map(|item| {
        if let HirItem::Fn(f) = item { Some(f.fn_id) } else { None }
    }).collect();
    let imported_fns: Vec<ImportedFnSig> = ctx.fns.iter().enumerate()
        .filter(|(i, _)| !defined_ids.contains(&FnId(*i)))
        .map(|(i, sig)| ImportedFnSig {
            fn_id: FnId(i), name: sig.name,
            params: sig.params.clone(), return_type: sig.return_type.clone(),
        })
        .collect();

    // 追加降级过程中特化的泛型函数
    for f in ctx.specialized_fns.drain(..) { items.push(HirItem::Fn(f)); }

    Ok(HirProgram { items, vtables: ctx.vtables.clone(), struct_defs: ctx.struct_defs.clone(), imported_fns })
}
