---
description: 'Rspack JavaScript binding 架构中当前的妥协、风险和退出条件。'
---

# JavaScript binding 设计债务

本文记录 JavaScript binding 中已知的架构妥协。它不是 roadmap，也不表示所有问题都应该立即修改。
任何改动都必须保持 webpack 兼容、性能、必要的对象 identity，以及 native/WASI 支持。

历史调研应保存在 `docs/spikes/`，接受的决策应保存在 `docs/adr/`。本文只描述当前状态，状态变化
时必须同步更新。

## 债务登记表

| 领域                                 | 风险 | 当前方向                                                                 |
| ------------------------------------ | ---- | ------------------------------------------------------------------------ |
| Callback-scoped Module 访问          | 高   | 使用可撤销、capability-aware handle 或 owned data 替代项目内 raw pointer |
| Compilation wrapper pointer 生命周期 | 高   | 缩短 pointer 生命周期，通过 owner-controlled context 解析访问            |
| Hook registration 和 callback caches | 中   | 集中管理 invalidation，使 cache ownership 可观察                         |
| Compiler cleanup 和 `unsafeFastDrop` | 中   | 降低显式 close 成本，使其继续作为默认所有权路径                          |
| 分散的类型所有权                     | 中   | 区分生成类型、手写内部类型和公共类型                                     |
| 隐藏的跨边界成本                     | 中   | 文档化并 benchmark 高频 JavaScript API                                   |
| Native 和 WASI 一致性                | 中   | 定义 capability 差异并测试两种 export surface                            |

## Callback-scoped Module 访问

**状态：** 进行中

**影响范围：** `crates/rspack_binding_api/src/module.rs`、module hooks、loader context、
module factory callbacks 和 asset callbacks。

**当前行为：** Binding `Module` 通常通过 `(compiler_id, module_identifier)` 从活跃 compilation
解析。Module 由 `ModuleGraph` 之外的阶段临时持有时，`ModuleObject` 可以携带 raw pointer
fallback。该值通过 JavaScript callback bridge 传递，并使用按 compiler 缓存的 instance 保持
JavaScript identity。

**存在原因：** `buildModule`、`succeedModule`、loaders、module factory callbacks 和 module
execution 可能在 module 进入主 graph 前或位于 graph 外时暴露它。只使用 identifier 会导致这些
callback 中无法提供兼容 webpack 的 `Module` API。

**假设和风险：**

- JavaScript 不会在 native callback scope 之后访问 pointer-backed 对象。
- 不会通过原本共享的 Rust reference 进行 JavaScript mutation。
- 排队的 callback conversion 和 Promise 不会超过 native owner 的生命周期。
- 手写的 `Send` 和 `Sync` 约定始终成立。

这些假设没有被 Rust 类型完整表达，保留的 JavaScript 对象可以破坏它们。

**目标方向：** 对只读 callback 使用 owned snapshot；对需要实时访问或 mutation 的 callback
使用可撤销、带版本和 capability 的 handle。同步成本应放在 JavaScript 边界，而不是普通 Rust
graph traversal。

**退出条件：**

- 删除项目内 Module raw pointer 和手写 `Send`/`Sync`，或将其封装进经过评审的抽象；
- 只读 callback 不能通过 shared reference 修改对象；
- 保留的 JavaScript 对象在撤销后确定性失败；
- module identity 和 loader 行为保持兼容；
- 生命周期和并发测试覆盖 Promise retention 与 rebuild。

## Compilation wrapper pointer 生命周期

**状态：** 进行中

**影响范围：** `JsCompilation`、`JsCompilationWrapper`、compilation instance caches 和
`COMPILER_REFERENCES`。

**当前行为：** Native wrapper 保存 `CompilationId` 和指向 Rust `Compilation` 的 non-null
pointer。Rust 和 JavaScript cache 为活跃 compilation 保持一个 wrapper 和一个公共 facade。
Cleanup 会删除旧 cache entry。

**存在原因：** Compilation API 范围广且可修改。复制完整 compilation 既不现实也不兼容，而反复
重建 wrapper 会破坏对象 identity 并增加转换成本。

**风险：** Safety 依赖 cleanup、compiler lifetime 和 callback sequencing。Wrapper 表示出来的
生命周期和可传递性比底层 borrow 更长。

**目标方向：** 通过 owner-controlled compiler 或 compilation context 访问，提供显式
invalidation，并使用不能返回 borrowed reference 的 closure-based access。

**退出条件：**

- 旧 watch compilation 不会意外解析为新的 compilation；
- 所有访问都会验证 owner 和 compilation generation；
- wrapper 不再需要无约束的 pointer lifetime；
- compiler close 和垃圾回收具有确定的失败行为。

## Hook registration 和 callback caches

**状态：** 进行中，且对性能敏感

**影响范围：** `packages/rspack/src/Compiler.ts`、
`crates/rspack_binding_api/src/plugins/interceptor.rs` 和 `JsHooksAdapterPlugin`。

**当前行为：** JavaScript 暴露 register functions。Native hook interceptor 按 stage 查询 taps，
跳过未使用的 hook kind，并为高频 hook 缓存 tap list。部分 invalidation 在 JavaScript tap 执行后
触发。

**存在原因：** 为每个 module 或 asset 反复注册和转换 JavaScript function 成本很高。Rspack
还需要兼容 webpack hook stages，且不能在启动时将所有 JavaScript tap 永久安装为 native object。

**风险：** Ownership 和 invalidation 分散在 JavaScript 与 Rust 两侧。Stale cache 可能遗漏
late tap，缓存的线程安全函数也可能保留 compiler 或 compilation closure。

**目标方向：** 为 registration snapshot 增加明确 generation 和 owner，集中 invalidation，并为
cache query 与 callback invocation 提供 counter 或 tracing。

**退出条件：**

- cache invalidation 规则被写成约束并在 rebuild 中测试；
- callback handle 在 close 和 environment cleanup 时释放；
- 添加 hook 不再要求在多个文件中维护重复样板；
- hot-hook overhead 有回归 benchmark。

## Compiler cleanup 和 `unsafeFastDrop`

**状态：** 活跃的内部优化

**影响范围：** JavaScript `Compiler.unsafeFastDrop`、native `JsCompiler` finalization 和
compiler-scoped thread-safe function 管理。

**当前行为：** Native compiler 保存于 `ManuallyDrop`，常规 cleanup 会显式 drop。内部 fast-drop
模式会跳过高成本 drop，并依赖进程退出释放资源。

**存在原因：** 短生命周期 CLI 结束时删除大型 compiler graph，可能明显影响总命令耗时。

**风险：** 只有进程即将退出时才能跳过 destructor。在长期运行进程中使用会保留 native memory
和 callback resources。

**目标方向：** 保持该优化为内部能力，明确其 process-lifetime 假设，并降低或移动 cleanup 工作，
使普通显式所有权路径成本可接受。

**退出条件：**

- 长期运行的 API 用户始终使用确定性 cleanup；
- 测试证明 `close()` 会释放 callbacks 和 native state；
- 不能通过公共 API 意外启用进程退出优化。

## 分散的类型所有权

**状态：** 进行中

**影响范围：** `crates/node_binding/napi-binding.d.ts`、
`crates/node_binding/scripts/banner.d.ts`、`crates/node_binding/binding.d.ts`、Rust `#[napi]`
annotations 和公共 `@rspack/core` types。

**当前行为：** napi-rs 生成大部分内部声明；手写 banner 补充不便生成的类型；`binding.d.ts`
修复 CJS/ESM interop。公共 package 再选择性 re-export 或包装 binding types。

**风险：** 某一层类型看似正确，但可能与 runtime conversion 或公共 wrapper 不一致。内部生成
类型也可能被误认为受支持的公共 API。

**目标方向：** 明确三种类型所有权，并分别提供 type tests：

- Rust 生成的内部 binding surface；
- 手写内部补充；
- 公共 `@rspack/core` API。

**退出条件：**

- 不直接修改生成文件；
- 每个公共 binding-backed API 都有唯一权威公共类型；
- 适用时，type tests 覆盖 CJS、ESM、native 和 WASI 入口。

## 隐藏的跨边界成本

**状态：** 文档不足

**影响 API：** Graph getters、module/chunk iteration、stats conversion、assets、sources、
自定义文件系统、loaders 和高频 hooks。

**当前行为：** JavaScript 语法无法显示操作属于纯 JavaScript、native lookup、collection
materialization、source conversion，还是 Rust 到 JavaScript callback。

**风险：** Plugin 可能意外将线性的 native phase 变为数千次跨语言调用或重复 graph scan。

**目标方向：** 按 boundary behavior 分类公共 API，在 reference page 中增加 implementation
notes，并 benchmark 有代表性的 API，而不只 benchmark 完整构建。

**退出条件：**

- 高频 API 记录 lifetime、materialization、mutation 和预期成本；
- 热点 binding 操作具有 microbenchmark 或 tracing；
- 新公共 API 包含 boundary-cost review。

## Native 和 WASI 一致性

**状态：** 进行中

**影响范围：** Native platform packages、`rspack.wasi.cjs`、browser wrappers、worker
adapters 和条件 Rust features。

**当前行为：** 同一份生成 API 被适配到 native Node.js 和 WASI 环境，但 runtime、文件系统、
worker 和 feature 行为可能不同。

**风险：** 新 export 可能在 native 编译和运行正常，却在 emnapi/WASI 中不受支持、低效或错误。

**目标方向：** 维护明确的平台 capability matrix，并在 native 和 WASI CI 中检查生成的 export
list。

**退出条件：**

- 不支持的 API 显式失败或有意省略；
- Browser API 文档记录平台差异；
- 新 boundary type 测试 WASI conversion 和 worker behavior。

## 更新登记表

当某项妥协被删除或改变：

1. 先更新当前架构文档。
2. 将长期有效的原因记录到 ADR。
3. 更新或删除本文中的对应债务。
4. 如果约束或修改配方变化，更新 `.agents/BINDING.md`。
5. 增加或更新能够证明退出条件的测试。
