# 跨模块来源作用域（M6）

## 目标

包内自由函数的裸调用必须在**来源包作用域**解析。消费方可以声明同名私有函数，
但不能改变已编译包函数体的解析结果，也不能通过重新实例化泛型源码截获包内调用。

本切片解决：

- 非泛型包函数继续在包自己的 LIR 中解析并编译，私有符号不进入消费方普通符号表；
- 跨包泛型源码调用来源包非泛型 helper 时，优先绑定来源包 helper；
- helper 再调用 helper 的传递闭包保持同一作用域；
- `.lcl` 文件名被改作别名后，作用域仍从包内容恢复。

## 包内表示

打包含泛型函数的模块时，编译器从泛型函数体收集实际调用名，并从顶层非泛型自由
函数表中按名收集依赖闭包。依赖函数克隆到保留命名空间：

```ayanami
namespace __ayanami_scope_<package> {
    #[inline]
    fn helper(...) { ... }
}
```

- `<package>` 来自包自身，不从消费方的 import 文本或文件名推导；
- 依赖函数的 `#[inline]` 使 LIR 发射为 `linkonce_odr`，多模块重复注入可被链接器合并；
- 原始 `.lcl` 仍保留自己的 internal 私有实现，不把私有符号暴露成可调用的公共 API。

## 导入语义

依赖解析阶段从 `.lcl` 的 `[generics]` 源码中找到保留命名空间并注入当前 AST。
泛型源码登记时同时记录来源作用域。以下两个阶段都使用同一查找规则：

1. eager 泛型体检查；
2. 实际单态化后的 HIR 降级。

裸自由函数调用的顺序是：

1. 当前来源作用域中的限定名（如 `__ayanami_scope_lib.helper`）；
2. 普通的当前模块/导入函数表；
3. 泛型特化或明确诊断。

命名空间内函数自身调用 sibling helper 时也进入该命名空间作用域，因此依赖闭包
可以跨多层调用。

## 当前限制

- 来源 bundle 目前复制包内所有声明；后续可收紧为按依赖闭包裁剪；
- 宏展开结果若生成新的裸调用，仍依赖展开后普通 HIR 解析路径；
- 私有类型若直接出现在 C ABI 边界，需要在语言层引入显式 ABI 类型，不能跨包暴露；
- 消费方不能直接使用来源包私有类型或私有自由函数。

## 完整作用域设计（wwy4）

完整 bundle 使用一个统一的来源命名空间，而不是为每个调用名单独猜测：

```ayanami
namespace __ayanami_scope_<package> {
    struct Hidden { ... }
    const OFFSET = ...
    static BASE = ...
    fn helper(...) { ... }
    fn generic_helper[T](...) { ... }
    namespace Deep { ... }
    impl[T] Box[T] { ... }
}
```

依赖收集与符号分类规则：

1. 任何跨包泛型函数或泛型方法体中的裸调用先视为潜在来源依赖；
2. 自由函数、泛型函数、类型、const/static、namespace 与 impl 均进入候选表；
3. 函数依赖沿函数体递归，类型依赖沿字段、签名和字面量递归；
4. 选中的声明整体克隆到保留 namespace，非泛型函数/方法带 `#[inline]`（`linkonce_odr`）；
5. 来源包泛型函数互调时，被调泛型也复制到同一 namespace，避免消费方同名泛型截获。

导入与解析规则：

- `current_module` 是当前函数/impl 的来源 namespace；
- 裸值名先查 `current_module.name`，再查普通表；
- 方法分派先查 `current_module.method`，再查普通方法；
- 类型名在来源函数签名与 body 中按同样规则限定；
- 限定名中的 `current_module` 仅在目标表确实存在时生效，普通本地调用不受影响；
- 消费方同名声明仍可正常使用，但不会改变已注入包体内部的解析。

链接规则保持不变：

- 无依赖的 debug 构建可使用普通外部符号；
- 多模块/依赖构建对包私有实现使用 `internal`；
- 注入的泛型依赖函数/方法使用 `linkonce_odr`，重复包不会产生强符号冲突。

## 验收

`tests/lcl_nsr` 覆盖：

- `from_lib(1) == 11`：消费方同名 helper 不影响普通包函数；
- `helper(1) == 21`：消费方仍能调用自己的私有 helper；
- `generic_from_lib[int](1) == 12`：泛型源码经两级 helper 仍绑定来源包；
- 同一 `.lcl` 改名为 `lib_alias.lcl` 后结果不变。
- `generic_complete[int](7) == 35`：来源包私有类型、const/static、namespace、
  impl 方法、private helper 和来源包泛型互调均绑定来源作用域；
- 消费方可用同名的 `Hidden`/`Holder`、`Deep`、`OFFSET`/`BASE`、
  `helper`/`generic_second`，不会截获包体调用；
- `private_entry.aya` 直接使用来源包私有 `LibOnly` 必须编译失败。

## 实现状态

顶层自由函数依赖闭包、eager/HIR 两阶段来源解析和包别名场景已实现。
wwy4 已将完整包声明 bundle（方法、namespace、类型、const/static、来源包泛型）
纳入作用域，并补充 `.lcl method=` 导入分类、类型/闭包/字段限定和父 namespace
逐层查找。std 全量与主仓回归作为验收门禁。
