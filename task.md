# Rust 直接调度 parallel JS loader 执行计划

## 目标执行流

```text
NormalModule::build
  crates/rspack_core/src/normal_module.rs
       ↓
rspack_loader_runner::run_loaders
  Box<LoaderContext<RunnerContext>> 保持唯一所有权
       ↓
JsLoaderRspackPlugin::loader_yield
  crates/rspack_binding_api/src/plugins/js_loader/scheduler.rs
       ├─ parallel = false
       │    在主线程边界组装原有 JsLoaderContext DTO
       │    调用同一套 JS runLoaders
       │
       └─ parallel = true
            Rust 直接把完整 LoaderContext move 进通用 MPMC
                 ↓
            rspack_tasks::WorkerJob + async-channel（unbounded）
                 ↓
            任意空闲 JS worker 主动 recv
              context = task.takeContext()
              result = await runLoaders(context)
              task.complete(result) / task.fail(error)
                 ↓
            同一个 LoaderContext 返回 Rust loader runner
```

parallel task 从 Rust loader runner 直接进入 Rust queue，不经过 JS 主线程分发。JS 主线程只负责 worker 生命周期，以及必须保持 isolate affinity 的 JS 对象和函数调用。

worker pool 是进程级单例：首次出现 parallel loader 时启动，进程内所有 compiler 共享；worker 在空闲时继续阻塞于 Rust MPMC queue，主线程只监听退出并按 slot 重启，不按 build acquire/release。

## 数据与所有权边界

### LoaderContext

- channel 直接传完整的 Rust `LoaderContext<RunnerContext>`，不拆成 `JsLoaderContextState`，也不先在主线程构造 worker DTO。
- loader runner 作用域内的 `LoaderCompilation` 随 build task 进入 `RunnerContext`，直接访问当前 native compilation；通用 `Compilation` 模块保持 main 实现，不在 binding 建全局 compilation id-to-pointer 注册表。
- `run_loaders`、yield hook、scheduler 和 worker payload 之间移动同一个 `Box<LoaderContext>`；完成或失败时必须把所有权返回。
- 到实际执行 isolate 的 N-API 边界后，复用原有转换逻辑组装 `JsLoaderContext`；main 和 worker 使用同一个 DTO、同一个 `LoaderObject` 和同一个 `runLoaders`。
- `_compilation` 由 worker task 基于仍然存活的 native `Compilation` 创建本 isolate 的 JS facade；`getPath`、diagnostic 和 `importModule` 等继续复用已有 binding 实现，不通过 compiler reference 反查，也不改通用 compilation 模块。
- JS 返回后复用原有 `merge_loader_context` 写回 content、source map、additional data handle、dependency、loader data 和执行状态。

### NormalModule

- `RunnerContext.module` 在整个 loader 执行期间拥有原来的 `Box<NormalModule>`；`WorkerTask` 持有完整 LoaderContext，JS `_module` 只在该执行窗口内通过 main 的 `ModuleObject::with_ptr` 访问它。
- 主线程和 worker 都复用同一个 `TryFrom<&mut LoaderContext>` 与同一套 `NormalModule` JS binding，不维护 worker module DTO。
- 不为 parallel 构造 `Box::new(module.clone())`，也不把 `NormalModule` 的普通字段全面改成 accessor。
- `ModuleObject::with_ptr` 不复制或移动 `NormalModule`；worker 被终止时 `WorkerJob::drop` 直接归还仍拥有 Box 的 LoaderContext。通用 module/compilation binding 保持 main 实现。

### Loader options 与函数

- options 原对象保存在 JS 主线程的 compiler-owned map，Rust 只在单个 loader metadata 中透传不透明 `u32` handle。
- 不在 `RawOptions` 增加 `__jsReferenceHandles`，Rust 不接收也不管理整张 handle map。
- main loader 通过 handle 取得原对象；worker 通过 MessagePort 向 main 请求一次，普通值使用 structured clone。
- `compiler.options.loader` 与 loader options 共用同一个 JS bridge envelope；Rust payload 和 `JsLoaderContext` 不保存序列化的 `loader_context_extensions`。
- options 中的函数保留在创建它的 isolate；worker 使用现有 `SharedArrayBuffer + Atomics.wait/notify` 桥，在 main 上加锁模拟同步调用。
- 函数桥传递 loader context 作为 `this` 时只结构化克隆用户扩展字段，不枚举或复制 native-backed `_compiler`、`_compilation`、`_module`。
- rule ident 仍由 JS RuleSet map 解析，native inline loader 只保留 `??ident`，不把 ident-to-handle 表传给 Rust。

### Loader additional data

- 连续的 parallel JS loader 在同一个 worker `runLoaders` 循环中直接传递 `additionalData`，中间不经过 Rust 或 main。
- worker 批次结束或下一项不能 parallel 时，通过现有 MessagePort bridge 把值 structured clone 到 main registry；Rust 只保存 `u32` handle。
- Rust 使用 `rspack_napi::MainThreadJsValueHandle` 封装 `u32`；公共 N-API init 方法把 main-isolate TSFN 注册到全局，通用 handle 内部用 `Arc` 保证最后一个 clone Drop 时只调用一次 release，js-loader 只接收和透传 handle。
- main/native/另一个 worker 继续执行时通过 handle 从 main 取得原值或 worker-local clone；Buffer、Map 等类型保持 loader API 语义。

### parallel 配置

- Rust 的 `RawModuleRuleUse.parallel`、`ModuleRuleUseLoader.parallel` 和 `LoaderRunnerOptions.parallel` 都是 `bool`。
- `maxWorkers` 只影响 JS worker pool 的创建，不作为 number 进入每个 Rust loader item。
- loader cache 只缓存 loader 可观察结果；`js_options_handle` 必须 skip cache，不能进入持久化 cache key 或 cache value。

## 实施任务

### 1. 通用 unbounded MPMC

- [x] 在 `rspack_tasks` 提供与 loader 无关的 `WorkerJob<T, E>` request/oneshot 封装；binding 直接持有 unbounded channel sender/receiver。
- [x] 所有 worker 竞争同一个 receiver；无 pool id、worker 注册、consumer count 或 callback registry。
- [x] `dispatch` 使用 oneshot 返回完成结果；取消和 task drop 必须唤醒 Rust waiter。
- [x] 当前 payload 只有一种，不携带 task type；未来加入第二类 native task 时把上层 envelope 改成 enum。

### 2. Rust loader runner 直接发起 parallel task

- [x] `LoaderRunnerPlugin::start_yielding` 转移并归还完整 boxed LoaderContext。
- [x] scheduler 根据当前 loader 的布尔 `parallel` 决定 main lane 或 worker lane。
- [x] parallel 分支直接向 channel enqueue，enqueue 前不调用 JS `runLoaders` 或 JS task dispatcher。
- [x] pitch、normal、builtin/JS 混合 chain 返回后继续使用同一个 Rust loader runner 状态机。

### 3. JS worker 主动 recv

- [x] worker 启动后循环 `recv -> takeContext -> runLoaders -> complete/fail`。
- [x] 删除 Tinypool `pool.run`、task `postMessage`、worker callback 注册和销毁协议。
- [x] service 只负责线程 ready/restart 和 options/function bridge，不接触 loader task payload。

### 4. 复用主线程 JsLoaderContext 逻辑

- [x] queue payload 保存完整 Rust LoaderContext。
- [x] main 和 worker 都在各自 N-API 边界组装同一个 `JsLoaderContext` DTO。
- [x] 删除 bulk `JsLoaderContextState`、整套 native field accessor 和重复的 parallel DTO。
- [x] loader `data` 沿用 main 的 `serde_json::Value`；`additionalData` 由 main JS registry 持有，Rust 只透传带 Drop 清理的 handle。
- [x] worker `_module` 复用 main 的 pointer-based `ModuleObject`；Box 始终由 WorkerTask 内的 LoaderContext 持有，不复制 NormalModule。

### 5. 清理 options/cache 的无关改动

- [x] 删除 `RawOptions.__jsReferenceHandles` 及 Rust compiler options 中对应 map。
- [x] options handle 由 JS 创建、保存和解析，Rust 只透传单个 handle。
- [x] 删除 Rust `loader_context_extensions`；worker 通过 options handle 从 JS 主线程取得 `compiler.options.loader`，并复用 main 的 `Object.assign` 逻辑。
- [x] `parallel` 在 Rust 侧改为 bool，worker 数只在 JS 侧处理。
- [x] `js_options_handle` 标记为 skip cache。
- [x] 回滚与本改造无关的 cache metadata、自定义 build meta、file metadata、SWC 和 loader cache 改动。

### 6. 验证

- [x] `cargo check -p rspack_binding_api`
- [x] `pnpm run build:binding:dev`
- [x] `pnpm run build:js`
- [x] `cd tests/rspack-test && pnpm run test -t "configCases/loader-parallel"`
- [x] 清理完成后重新执行 binding/JS build、loader-parallel 和 worker lifecycle 用例。
- [x] 检查 diff 中只剩 direct dispatch、通用 queue、pointer-based module access、必要的 worker bridge 和测试；`compilation/mod.rs` 不包含 loader worker 特判。

按仓库约定，不运行会在 sandbox 卡住的 storage/native watcher 测试。

### 7. 性能对比

改动完成后使用 `/home/jinzhixin/projects/mock-oai` 对比 merge-base main 与 candidate，固定 Node、pnpm、mock-oai commit、worker 数和构建 profile。Babel loader 分别测试：

| parallel | loader cache | main | candidate | candidate vs main |
| -------- | ------------ | ---- | --------- | ----------------- |
| off      | off          |      |           |                   |
| off      | on           |      |           |                   |
| on       | off          |      |           |                   |
| on       | on           |      |           |                   |

每组至少 5 次，cold/warm 分开记录原始样本、median 和 mean。同时确认 SWC loader 没有被配置为 cache，避免把 SWC cache 行为误算成 Babel cache 开销。

## 验收标准

- Rust loader runner 是 parallel task 的唯一 producer。
- JS 主线程不 materialize、serialize 或转发 parallel LoaderContext。
- 所有 JS workers 循环消费同一个进程级 unbounded MPMC。
- channel 传完整 Rust LoaderContext，到 JS 边界才复用原逻辑组装 JsLoaderContext。
- `NormalModule` 不复制；canonical Box 始终由 LoaderContext 持有，JS 仅在 WorkerTask 生命周期内通过 main 的 pointer binding 访问。
- options map 和 ident map 留在 JS；Rust 不持有 `__jsReferenceHandles`。
- main/worker 的 loader context 与 loader runner 实现一致，唯一特殊路径是 isolate-bound Function 的同步桥。
- cache 不保存 `js_options_handle`，parallel 元数据不再混用 worker 数。
