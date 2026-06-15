# Ayanami 编译器文法驱动重构方案

用 **文法（grammar）** 定义整个语言。框架从 `.grammar` 文件自动推导出 parser、AST/HIR/LIR 节点、lowering 和代码生成。**零 Rust 代码可完成全部流程**，自定义语义只需在 grammar 中添加声明式变换规则。

```
.grammar 文件（唯一输入）
    │
    ▼
 框架（build.rs 生成）
    │
    ├── Lexer        ← 从 @lexer 推导 DFA/关键字表
    ├── Parser       ← 从 @ast 产生式推导 LL(k) 递归下降
    ├── AST → HIR    ← 从 @hir 产生式推导树变换（自动状态机）
    ├── HIR → MIR    ← 从 @mir 产生式推导
    ├── MIR → LIR    ← 从 @lir 产生式推导
    └── LIR → LLVM   ← 从 @emit 推导代码模板
```

---

## 0. 核心理念：文法即编译器

传统编译器工作流：

```
文法（人脑）→ 手写 parser → 手写 enum → 手写 lowering → 手写 emit
                                                      ↑
                                              每步都是大 match
```

本框架工作流：

```
.grammar 文件 → 框架自动推导全部 ↓
                   ↓
  parser    节点定义     lowering      emit
  ← 递归下降  ← struct   ← 树自动机   ← LLVM IR 模板
```

框架不依赖任何 Rust 函数引用。lowering 是 **树自动机（tree automaton）**：框架从 `@ast` 和 `@hir` 的节点结构差异中自动推导变换规则。只有当自动推导无法覆盖的语义（如虚表生成、泛型特化）才需要用户写 Rust pass。

---

## 1. 文法格式

### 1.1 基础文法

```
// mylang.grammar

@lexer {
    keyword: "fn", "let", "return", "if", "true", "false"
    operator: "+" (prec:5, assoc:left)
              "*" (prec:6, assoc:left)
              "(" (prefix, group)
              "{" (prefix, group)
    punct: ";", ",", ":", "->"
    token:  identifier→Ident
            number→IntLiteral
}

@tokens {
    IntLiteral: i64
    Ident: Symbol
}

@ast {
    Program  = Stmt*
    Stmt     = FnDecl | VarDecl | ExprStmt
    FnDecl   = "fn" Ident "(" FnParamList? ")" Block
    FnParamList = FnParam ("," FnParam)*
    FnParam  = Ident ":" Type
    VarDecl  = "let" Ident ":" Type "=" Expr ";"
    Block    = "{" Stmt* "}"
    Expr     = BinaryExpr | CallExpr | Ident | IntLiteral
    BinaryExpr = Expr operator Expr        // 优先级/结合性从 @lexer 读
    CallExpr = Ident "(" ExprList? ")"
}

// ─── 以下三个区块框架可自动推导 ───

@hir {
    // 框架自动比较 @ast 和 @hir 的结构差异，推导变换规则
    // 如果 @hir 产生式名与 @ast 相同且结构兼容，自动直通
    HirFnDecl   = FnDecl       // 结构相同，框架自动直通
    HirVarDecl  = VarDecl      // 同上
    HirBlock    = Block        // 同上
    HirBinaryOp = BinaryExpr   // 同上

    // @hir 独有节点（AST 中无对应）
    HirIdent        = Ident
    HirIntLiteral   = IntLiteral
    HirCall         = CallExpr
    // 框架自动生成：递归下降取子节点，逐层变换
}

@mir {
    // MIR 更接近线性指令，框架从 hir 与控制流结构推导
    // 基本块、phi 节点等由框架自动插入
}

@lir {
    // LIR 指令集；框架从 @mir 推导指令选择
}
```

### 1.2 变换规则声明（grammar 级，不写 Rust）

对于框架无法自动直通的 lowering，用 **标注（annotation）** 描述：

```
// 方案 A：字段映射标注
@hir {
    HirEnumMatch = AST.MatchExpr {
        // 声明从 AST 字段到 HIR 字段的映射
        matched_expr: matched          // 同名直通
        arms:         @map(arms, arm → HirMatchArm)
        tag_ty:       @derive("int")   // 注入常量
    }
}

// 方案 B：模式-替换规则
@transform {
    // 将 AST BinaryExpr 拆成 HIR 具体操作
    BinaryExpr(op:"+", lhs, rhs)  → HirAdd(lhs, rhs)
    BinaryExpr(op:"-", lhs, rhs)  → HirSub(lhs, rhs)
    BinaryExpr(op:"==", lhs, rhs) → HirEq(lhs, rhs)
}

// 方案 C：条件守卫
@transform {
    CallExpr(name:"print", args)  → HirPrintCall(args)
    CallExpr(name, args)          → HirCall(resolve(name), args)
}
```

### 1.3 自定义 Rust pass（逃生舱）

框架自动推导无法覆盖时，用户写 Rust 函数：

```
// grammar 中引用 Rust pass
@hir {
    HirVTable  →  lower:"build_vtable"     // 只在必要时用
}

// src/passes/build_vtable.rs
#[pass]
fn build_vtable(ctx: &mut HirCtx, defs: &[ImplDef]) -> Result<Vec<VTable>, Error> {
    // 自定义逻辑，但 Error 仍然自动获得 span
}
```

**默认不写 Rust**。Rust pass 只用于：泛型特化、虚表构造、目标相关优化。

---

## 2. 自动状态机推导（核心）

### 2.1 树自动机原理

框架将 lowering 建模为 **树 transducer**：

```
输入树（AST）          输出树（HIR）
    │                      │
    └── 框架自动推导 ──────┘
        规则：同名同构 → 直通
        规则：同名异构 → 字段映射
        规则：新名     → 模式匹配
```

例：`@ast` 有 `BinaryExpr { op, lhs, rhs }`，`@hir` 有 `HirAdd { lhs, rhs }`、`HirSub { lhs, rhs }`。框架自动推导：

```
输入: BinaryExpr { op: "+", lhs: X, rhs: Y }
                    ↓ 匹配 transform 规则
输出: HirAdd { lhs: lower(X), rhs: lower(Y) }
```

树 transducer 的迁移函数完全从 grammar 推导，不需要用户提供。

### 2.2 状态机与节点的关系

```
文法文件（.grammar）
    │
    ├── @ast 产生式 →  AST 节点类型 + 树自动机输入状态
    ├── @hir 产生式 →  HIR 节点类型 + 树自动机输出状态
    ├── @transform  →  树自动机迁移规则
    └── @pipeline   →  状态机执行顺序

树自动机执行：
    parse(ast_root)
    → state_machine(ast_root, { rules from @transform })
    → hir_root
```

每个 lowering pass 是状态机的一个 **阶段（phase）**，各阶段串联形成完整管线。

### 2.3 自动错误检测

树 transducer 在编译时（build.rs）可检测：

- **未覆盖的产生式**：AST 节点没有对应 HIR 节点或 transform 规则 → 编译时报错
- **不可达的 HIR 节点**：HIR 节点没有 AST 来源 → 可能多余
- **歧义规则**：同一 AST 模式匹配多条 transform → 按优先级 / 报错

这些检测在 grammar 编译期完成，不等到运行时。

---

## 3. Pipeline 架构

### 3.1 阶段划分

所有阶段由 grammar 声明，框架自动生成对应代码。

```
@pipeline {
    // 每个 phase 是一个状态机阶段

    phase: "parse" → lex, parse_ast
        // 输入：源码字符串
        // 输出：AST（树）
        // 方法：lexer 产 token → parser 按产生式递归下降

    phase: "hir" → lower_to_hir
        // 输入：AST 树
        // 输出：HIR 树
        // 方法：树 transducer（@transform 规则驱动）

    phase: "canonicalize" → resolve_names, type_check, specialize
        // 输入：HIR 树
        // 输出：规范化 HIR 树
        // 方法：树自动机 + 内置变换（改名解析、泛型展开）

    phase: "lir" → lower_to_lir
        // 输入：HIR 树
        // 输出：LIR 指令列表（线性）
        // 方法：树 → 线性序列

    phase: "emit" → emit_llvm
        // 输入：LIR 指令列表
        // 输出：LLVM IR 字符串
        // 方法：每条 LIR 指令 → 对应 @emit 模板
}
```

### 3.2 内置阶段 vs 自定义阶段

| 阶段 | 框架默认 | 可自定义 |
|------|---------|---------|
| parse | ✅ 从 @ast 生成 | 加钩子（如宏展开） |
| hir | ✅ 从 @transform 推导 | 自定义变换规则 |
| canonicalize | ✅ 内置改名解析 + 泛型 | 自定义类型检查 |
| lir | ✅ 从 @lir 推导 | 自定义指令选择 |
| emit | ✅ 从 @emit 模板生成 | 自定义目标后端 |

### 3.3 框架生成内容

```
build.rs 读取 .grammar 文件，生成：

src/generated/
├── lexer.rs              // TokenKind + 识别表
├── parser.rs             // 递归下降（每产生式一个方法）
├── ast_nodes.rs          // AST 节点 struct（带 span）
├── hir_nodes.rs          // HIR 节点 struct
├── lir_nodes.rs          // LIR 节点
├── tree_transducer.rs    // 从 @transform 推导的树自动机
├── emit_templates.rs     // 从 @emit 推导的 LLVM IR 模板
├── pipeline.rs           // 阶段串联代码
├── visitors.rs           // Visitor trait（可选，供 Rust pass 用）
└── grammar_info.rs       // 文法元数据（诊断、工具用）
```

### 3.4 用户可选的自定义

```
mylang/
├── src/
│   ├── main.rs           // 只是 fn main() { run_pipeline() }
│   └── custom_pass.rs    // 可选：只在自动推导不够时写
└── mylang.grammar       // 唯一必需文件
```

---

## 4. 错误报告

每个节点自带 `Span`，框架在树变换过程中自动传递位置信息。

### 4.1 Span 自动嵌入

框架为 grammar 产生的每个节点自动计算 span：

```
@ast {
    FnDecl = "fn" Ident "(" ... ")" Block
    //         ↑     ↑              ↑
    //        start               end（框架自动计算）
}
```

生成的 `AstFnDecl` 含 `span: Span`，从第一个 token 到最后一个 token 的位置。

### 4.2 错误类型

```rust
pub struct Error {
    pub kind: ErrorKind,
    pub span: Span,                          // 自动填充
    pub secondary_spans: Vec<(Span, String)>, // 关联位置链
    pub hint: Option<String>,                 // 修复建议
}

pub enum ErrorKind {
    UndefinedSymbol(String),
    TypeMismatch { expected: String, found: String },
    WrongArity { expected: usize, found: usize },
    Semantic(String),
    User(String),       // 自定义 pass 用
}
```

### 4.3 自动 span 传播

树 transducer 遍历子节点时，如果子节点报错，框架自动将父节点 span 附加到错误链：

```
error: type mismatch in call at 12:5
  ├─ expected `int`, found `String`
  ├─ while lowering argument at 12:12
  ├─ in call to `foo` at 12:5
  └─ pass: type_check
```

自定义 Rust pass 返回 `Err(Error::new(User("bad")))`，框架自动填入当前节点 span 和 pass 名。

### 4.4 收集模式

一次编译收集所有错误再输出：

```
@pipeline {
    phase: "canonicalize" → { mode: "collect" }
}
```

---

## 5. 与当前 Ayanami 对比

| 方面 | 当前 | 文法驱动 |
|------|------|---------|
| 编写 parser | 手写 ~2000 行 | 从 grammar 生成 |
| 新增关键字 | 改 4 处 | 加一行 `keyword` |
| 新增表达式 | 改 enum + 所有 match | 加一条产生式 |
| 新增 lowering | 改 match + function | grammar 自动推导或加 transform |
| 起步 | 理解 Rust + 编译器架构 | 写 BNF |
| Rust 依赖 | 全部 | 零（可选逃生舱） |
| 编译速度 | 改 enum 全量重编 | 只重新生成 affected |

---

## 6. 实施路线

### Phase 1 — .grammar 解析器 + 代码生成

- 写 `.grammar` 文法的解析器（用本框架 bootstrap）
- build.rs 读取 `.grammar`，生成 lexer + parser 骨架
- 验证：生成能 parse `fn main() {}` 的 parser

### Phase 2 — 节点定义生成

- `@ast` / `@hir` / `@lir` 区块生成对应 struct + span
- `@tokens` 区块生成 Token 枚举
- 框架自动推导同名产生式的直通 lowering

### Phase 3 — 树 transducer

- 实现从 `@transform` 规则生成树自动机
- 支持字段映射、模式匹配、条件守卫
- 自动检测未覆盖产生式和歧义规则

### Phase 4 — 内置阶段

- 改名解析（scope tree）
- 类型检查（根据 interface 约束）
- 泛型特化（从调用点推导具体类型）

### Phase 5 — LIR + Emit

- `@lir` 区块生成 LIR 指令定义
- `@emit` 区块生成 LLVM IR 模板
- 支持多目标（x86_64 / wasm）

### Phase 6 — 迁移 Ayanami 自身

- 将 Ayanami 现有语法写成 `.grammar`
- 用 Rust pass 逃生舱填补自动推导无法覆盖的部分（虚表、enum dispatch）
- 逐步淘汰手写代码

---

## 7. 风险与缓解

| 风险 | 缓解 |
|------|------|
| 自动推导不够智能 | 提供 `@transform` 标注 + Rust 逃生舱 |
| 生成代码膨胀 | 只生成用到的产生式；build.rs 裁剪 |
| 调试困难 | 支持 `--trace-tree` 输出树变换步骤 |
| 文法冲突 | build.rs 阶段检测：未覆盖、歧义、不可达 |
| 错误信息下降 | 树 transducer 保留 span 链；collect 模式多错误输出 |

---

## 8. 示例：完整语言定义（无需 Rust）

```
// minic.grammar — 小型 C 子集

@lexer {
    keyword: "int", "return", "if", "while", "fn"
    operator: "+" (prec:4), "*"(prec:5), "("(group, prefix)
    punct: ";", "{", "}", ","
    token: number→IntLiteral, identifier→Ident
}

@tokens {
    IntLiteral: i64
    Ident: Symbol
}

@ast {
    Program     = Stmt*
    Stmt        = VarDecl | ReturnStmt | IfStmt | WhileStmt | Block | ExprStmt
    VarDecl     = "int" Ident ("=" Expr)? ";"
    ReturnStmt  = "return" Expr ";"
    IfStmt      = "if" "(" Expr ")" Stmt ("else" Stmt)?
    WhileStmt   = "while" "(" Expr ")" Stmt
    Block       = "{" Stmt* "}"
    Expr        = BinaryExpr | CallExpr | Ident | IntLiteral
    BinaryExpr  = Expr operator Expr
    CallExpr    = Ident "(" ExprList? ")"
    ExprStmt    = Expr ";"
}

@hir {
    HirProgram   = Program
    HirVarDecl   = VarDecl
    HirReturn    = ReturnStmt
    HirIf        = IfStmt
    HirWhile     = WhileStmt
    HirBlock     = Block
    HirCall      = CallExpr
}

@transform {
    // 将 binary op 拆成具体操作
    BinaryExpr(op:"+", l, r) → HirAdd(l, r)
    BinaryExpr(op:"*", l, r) → HirMul(l, r)
    BinaryExpr(op, l, r)     → error("unsupported operator")
}

@emit {
    HirAdd(l, r) → "%d = add i64 %l, %r"
    HirMul(l, r) → "%d = mul i64 %l, %r"
    HirReturn(e) → "ret i64 %e"
    HirCall(f, args) → "%d = call i64 @%f(%args)"
}

@pipeline {
    phase: "parse"  → lex, parse
    phase: "hir"    → lower
    phase: "emit"   → codegen
}
```

这个文件是所有信息的唯一来源。框架据此：
- 生成 lexer（识别 `int`、`return`、`+` 等）
- 生成 parser（`parse_program`、`parse_stmt`、`parse_expr` 等）
- 生成 AST 节点（`AstVarDecl`、`AstBinaryExpr` 等）
- 生成 HIR 节点（`HirAdd`、`HirMul` 等）
- 生成树 transducer（`BinaryExpr → HirAdd / HirMul` 的自动机）
- 生成 emit 模板（HIR → LLVM IR 字符串的映射）

用户完全不写 Rust 就得到了一个能编译 C 子集的完整编译器。
