# Compilation 所有权与尚未迁移的字段

## Compiler 内部的所有权容器

`Compiler.compilation` 使用 `compiler/compilation.rs` 中的 `CompilationCell`，
内部为 `Cell<Option<Either<Arc<Compilation>, UniqueArc<Compilation>>>>`。
`None` 只供同步转换时移出非 Copy 的指针使用，不跨 await 或插件调用。

该类型只出现在 Compiler 的字段上。hook 和 pass 保持原来的
`&Compilation` / `&mut Compilation` 签名，没有每个 hook/pass 的写 guard。
转换要求 `&mut CompilationCell`。原生 Cell 不支持 Sync；封装类型的 Sync 实现
依赖其字段私有、所有状态修改都要求独占借用这一不变量。共享访问只读，不能
增加通过 `&self` 切换状态的方法。emit 后台任务只接收 Arc，不借用整个 Compiler。

`share()` 发布 Arc；任务全部完成后 `try_make_unique()` 回收 UniqueArc。
如果 reader 尚未释放，则返回错误并保留共享状态，可以重试。
独占修改只允许在 UniqueArc 状态执行，不会在 DerefMut 中隐式切换所有权。

## 已接入的调度边界

`src/compiler/mod.rs::emit_assets` 已接入真实的 owned Arc 调度：

```text
Compiler 持有 UniqueArc<Compilation>
  emit hook：&mut Compilation
       ↓ share()
Arc<Compilation>
  写文件任务：持有 owned Arc，读取 assets
  assetEmitted hook：&Compilation
  emitted_assets：仍使用既有 DashSet
       ↓ 等待所有任务，释放 reader，try_make_unique()
UniqueArc<Compilation>
  传播任务错误 / afterEmit hook：&mut Compilation
```

任务返回错误时也先恢复独占所有权。任务 scope 的取消行为沿用既有
`rspack_parallel::scope` 契约，没有新增异步取消时自动恢复的承诺。
build/rebuild 入口会尝试恢复独占所有权，保留 reader 时返回错误。

build/rebuild 替换分配内的 Compilation 值，保持地址不变。Weak 在 UniqueArc
存在期间不能 upgrade，在共享阶段可以 upgrade；它引用当前值而非历史快照。
增量恢复的旧值仍为 `Box<Compilation>`；module executor 的临时值保持独占。

## 保持原样、需要独占写入的字段

下面的字段没有改成 QCell、锁或其他内部可变容器。取得 UniqueArc 后原有
`&mut Compilation` 算法可以继续使用；持有共享 Arc 时不能直接修改它们。

| 字段 | 主要修改点 | 限制 |
| --- | --- | --- |
| `assets`、`assets_related_in`、`diagnostics` | `Compilation::{emit_asset,update_asset,delete_asset,rename_asset,par_rename_assets}`；processAssets plugins | asset 操作会联动 related 索引、诊断，删除/改名还影响 chunk files，不能只合并 assets map |
| `entries`、`global_entry` | `add_entry`、`add_include`；make / finishMake | entry 更新联动 module graph，仍需整体独占阶段 |
| `dependency_factories`、`dependency_templates`、`value_cache_versions` | thisCompilation / compilation hooks | 普通 map；注册仍通过独占借用 |
| `build_chunk_graph_artifact` | build/optimize chunks、module concatenation、SourceMap/RealContentHash/HMR | 包含 chunk graph、chunks、groups、entrypoints、named maps，以及 files/auxiliary_files；不能通过共享 Arc 直接改 |
| 所有 `StealCell<…>` artifacts | `src/utils/steal_cell.rs`；build_module_graph、各个优化与生成 pass | StealCell 实际为 `Option<T>`，`steal`/`try_write` 需要 `&mut self`；被取走期间读也会失败，并不提供内部可变性 |
| `code_generation_results` | code generation、HMR、binding code-generation wrappers | BindingCell 的裸指针访问不是读写同步；不能把它当作可并发修改的 cell |
| `runtime_modules`、`runtime_modules_hash`、`runtime_modules_code_generation_source` | runtime requirements、create hash、HMR | 普通 map，生成/哈希/源码更新继续在独占阶段 |
| `code_generated_modules`、`build_time_executed_modules` | code generation、module executor 汇总、HMR | 普通 set，线程本地结果仍按现有路径汇总 |
| `file_dependencies`、`context_dependencies`、`missing_dependencies`、`build_dependencies` | build graph 汇总；Copy/Html/DTS；binding add*Dependencies；store_cache_metadata | 普通集合；既有任务本地收集不等于 Compilation 上的集合支持共享写 |
| `module_executor` | build_module_graph、finish_modules、rebuild | take/放回及任务停止回收仍需要独占；后台任务自己拥有 artifacts |
| `minimize_persistent_cache`、`source_map_dev_tool_plugin_cache`、`module_build_cache` | Swc minimizer、SourceMap、cache pass hooks | Option 的 take/放回需要独占，不改变缓存的所有权设计 |
| `records`、`hash`、`hot_index`、`use_source_map_dev_tool_plugin_cache` | HMR、create hash、rebuild、SourceMap 注册 | 普通值，继续独占修改 |
| `modified_files`、`removed_files`、`is_rebuild` | 创建/替换 Compilation | 在替换前准备，不改成共享写 |

`StealCell` 覆盖的字段包括 module graph / exports info、async / defer modules、
module/chunk IDs、side effects、依赖诊断、module/chunk hashes、runtime requirements、
runtime proxy metadata、chunk render 及相应缓存 artifacts、circular_modules。
这些类型的定义仍保留在 `src/compilation/mod.rs`。

已有的 `emitted_assets`（DashSet）、`logging`、`import_var_map`、`in_finish_make`
（AtomicBool）、`module_static_cache` 和 `incremental` 的锁/原子状态保持原样。
这些机制只保护各自字段，不授予其他字段的修改权。

## 尚未完成的 binding / worker 迁移

- `JsCompilation::{as_ref,as_mut}` 仍通过 `NonNull<Compilation>` 返回引用，未改为
  Weak，也不计入 Arc reader 数量。`with_compilation_mut` 会先尝试恢复独占
  所有权，有 reader 时返回错误；直接 `as_mut` 路径仍依赖既有 hook/loader 窗口约束。
- `crates/rspack_binding_api/src/compilation/mod.rs` 中的 asset API、
  add*Dependencies、exports info 和 module graph 访问涉及上述字段，保持字段
  类型及其行为不变。BindingCell 和 compiler-context artifact 指针也没有改成锁。
- loader `emitFile` 通过 module build-info 收集资产，再由 create_module_assets
  合并；loader 直接调用 `_compilation.emitAsset` 则仍走 Compilation 直接写路径。
  后者若改为任务本地缓冲，需要单独决定读取刚写入资产时的语义。
- pass 内原有并行任务仍使用 scoped `&Compilation` 或独立 artifacts；尚未
  给这些任务分发 owned Arc。现有 `PassExt` 仍接收 `&mut Compilation`，没有逐 pass
  动态读写状态检测。要分发 owned Arc，需要把拥有者交给内部调度器，业务 hook
  仍然使用原来的引用参数。
- module executor 的独立 task loop 跨越 finishMake，直到
  `module_executor::after_build_module_graph` 才停止。它目前复制配置并持有自己的
  artifacts；不能机械地替换成长期持有主 Compilation 的 Arc，否则中间的
  finishMake 等独占修改阶段无法回收 UniqueArc。

现有 JS 裸指针访问尚未纳入指针类型表达的独占/共享边界，也没有解除共享期间
修改上述普通字段的限制。
