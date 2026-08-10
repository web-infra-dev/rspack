# 嵌套 Loader Chain 重构实施计划

## 1. 背景与结论

当前实现已经能够把连续 `cache: true` loader 合并为一个缓存区间，也能在 mixed cache chain 中运行 JS loader 和 builtin loader。但当前模型只有一个扁平 `LoaderChain`：

```text
LoaderChain
  range
  cache
  execution_kind: Native | JavaScript | Mixed
```

当一个 cache chain 同时包含 JS 和 builtin loader 时，runner 通过 `current_execution_span()` 在运行时重新扫描相邻 loader，临时计算 JS/native 执行区间。这能保证当前执行结果正确，但没有把“缓存边界”和“执行边界”显式组合起来，导致：

- cache chain 内部的 JS 合并没有出现在 planner 结果中；
- `merge_reason` 只能记录 cache 或 JavaScript，不能同时表达两层原因；
- `CacheOnly` 仍可能在运行时隐式合并连续 JS loader，无法独立比较优化；
- 每次 yield 都重新扫描 execution span；
- tracing、测试和 benchmark 无法观察 cache chain 内部的 JS/native 子链。

本次重构将 `LoaderChain` 改为递归 enum，由 `CacheChain` 包含一个或多个显式执行链：

```rust
pub enum LoaderChain {
  CacheChain {
    range: Range<usize>,
    static_fingerprint: String,
    children: Vec<LoaderChain>,
  },
  JsExecutionChain {
    range: Range<usize>,
  },
  NativeExecutionChain {
    range: Range<usize>,
  },
}
```

这里使用递归 enum 表达组合关系，但生产 planner 只生成固定的两层结构：

- `CacheChain` 只能出现在 root；
- `CacheChain.children` 只能包含 `JsExecutionChain` 或 `NativeExecutionChain`；
- `JsExecutionChain` 和 `NativeExecutionChain` 都是叶子；
- 禁止 `CacheChain` 嵌套 `CacheChain`，避免没有语义的任意递归。

## 2. 目标结构

例如 loader 列表为：

```text
0  js-a       cache=true
1  js-b       cache=true
2  builtin-c  cache=true
3  js-d       cache=true
4  js-e       cache=false
5  js-f       cache=false
6  builtin-g  cache=false
```

生产策略应生成：

```text
Loaders
│
├─ CacheChain [0..4]
│  ├─ JsExecutionChain     [0..2]
│  ├─ NativeExecutionChain [2..3]
│  └─ JsExecutionChain     [3..4]
│
├─ JsExecutionChain       [4..6]
│
└─ NativeExecutionChain   [6..7]
```

职责严格分离：

```text
CacheChain
  cache key / lookup / hit replay / miss store
       ↓
ExecutionChain children
  JS chain     -> 一次 Rust → JS → Rust yield
  Native chain -> Rust 内执行
       ↓
扁平 loader_items
  保留全局 loader_index、request、data 和执行状态
```

`LoaderContext.loader_items` 与本轮 `loader_item_states` 通过相同 index 组合成完整 loader 视图。chain tree 只保存 index range 和缓存静态信息，不复制 loader，也不改变 webpack 可观察的 loader 顺序。

## 3. 数据模型与不变量

### 3.1 LoaderChain API

在 `crates/rspack_loader_runner/src/chain.rs` 将当前 struct 替换为 enum，并提供统一访问器：

```rust
impl LoaderChain {
  pub fn range(&self) -> Range<usize>;
  pub fn start(&self) -> usize;
  pub fn end(&self) -> usize;
  pub fn len(&self) -> usize;
  pub fn is_cache(&self) -> bool;
  pub fn execution_kind(&self) -> Option<LoaderExecutionKind>;
  pub fn children(&self) -> &[LoaderChain];
}
```

调整类型：

- `LoaderExecutionKind` 只保留 `Native | JavaScript`，删除 `Mixed`；mixed 是 `CacheChain` 包含不同叶子链的结构属性，不再是某个执行链的类型。
- 删除 `LoaderChainMergeReason`，variant 本身表达 chain 角色；需要 tracing 时记录 `chain_kind=cache|javascript|native`。
- `static_fingerprint` 只属于 `CacheChain`。
- cache lookup/store hook 只接收 `CacheChain`，执行 hook 只接收叶子 execution chain。

### 3.2 结构不变量

planner 构造完成后统一校验以下不变量：

1. root chains 按 range 连续覆盖全部 loader，不重叠、不留空洞。
2. 每个 `CacheChain` 至少包含一个 child。
3. `CacheChain.children` 连续覆盖外层完整 range。
4. child 不允许是 `CacheChain`。
5. `JsExecutionChain` 中所有 loader 的 execution kind 都是 JavaScript。
6. `NativeExecutionChain` 中所有 loader 的 execution kind 都是 Native。
7. `CacheChain` 范围内所有 loader 都是 `cache=true`。
8. 未开启 cache 的 loader 不能被放入 `CacheChain`。

校验放在 planner 边界，debug/test 构建中强校验；executor 不重复推断这些事实。

### 3.3 O(1) chain 定位

不要在每次 pitch/normal/yield 时递归扫描 tree。chain 和定位索引直接保存在 `Loaders`：

```rust
pub struct LoaderChainLocation {
  root_index: usize,
  child_index: Option<usize>,
}
```

`locations[loader_index]` 直接定位：

- 当前 root cache/execution chain；
- 当前叶子 execution chain；
- JS bridge 应接收的明确 range。

由此删除 `LoaderContext::current_execution_span()` 及其运行时左右扫描。

### 3.4 在 Module 上保存预组装的 `Arc<Loaders>`

chain tree 不应等到 `NormalModule::build()` 才重新构造。新增不可变的 `Loaders`，在 `NormalModuleFactory` 完成 loader resolve、排序和配置合并后一次性创建，并通过 `Arc` 保存到 `NormalModule`：

```rust
pub struct Loaders {
  loaders: Vec<BoxLoader>,
  options: Vec<LoaderRunnerOptions>,
  loader_items: OnceLock<Arc<[LoaderItem<RunnerContext>]>>,
  loader_chains: OnceLock<(
    Arc<[LoaderChain]>,
    Arc<[LoaderChainLocation]>,
  )>,
  cache_service: RwLock<Option<Arc<LoaderCacheService>>>,
}
```

`Loaders` 包含每个 module 预组装好的 loader 执行描述，以及指向 compiler-scoped cache service 的共享句柄：

- resolved loader 实例；
- request/path/query/fragment/type 等一次解析结果；
- `cache` 和 options fingerprint；
- 预计算的 cache static fingerprint；
- 完整 chain tree 和 location index。
- `Arc<LoaderCacheService>`；每个 module 持有一个 clone，但底层 L1/persistent backend 仍由同一个 compiler service 共享。

这里的“缓存状态”需要区分静态和动态两部分：

- 静态缓存状态放进 `Loaders`：`cache` 开关、options fingerprint、loader identity、`CacheChain` 结构和 static fingerprint。
- cache service 句柄放进 `Loaders`，service 内部可以通过 interior mutability 保存 compiler 级 L1 entry 和 persistent backend。
- 本轮执行状态不直接保存在共享 `Loaders` 字段上：input hash、active hit/miss、miss snapshot 和正在执行的 cache action 依赖当前 build，应保存在本轮 runner state；最终 entry 写入 `Loaders.cache_service` 指向的共享 service。

因此 `Loaders` 是每个 module 自己的完整预组装执行对象，但其中的 cache service 是 compiler 级共享服务，不是为每个 module 单独创建一份 backend。

`NormalModule` 从：

```rust
loaders: Vec<BoxLoader>,
loader_runner_options: Vec<LoaderRunnerOptions>,
```

改为：

```rust
loaders: Arc<Loaders>,
```

`resource_data: Arc<ResourceData>` 保持 `NormalModule` 上的原字段，不放入 `Loaders`；本次只把原 loader 数组和这个分支新增的 loader/cache/chain 字段收拢进 `Loaders`。

factory 与运行时执行流变为：

```text
NormalModuleFactory
  resolve post / inline / normal / pre loaders
       ↓
确定最终扁平顺序和 runner options
       ↓
Loaders::new(..., strategy)
  解析 loader descriptor
  生成 LoaderChain tree
  计算 location index/static fingerprint
  clone compiler LoaderCacheService handle
       ↓
Arc<Loaders> 保存到 NormalModule
       ↓
NormalModule::build / rebuild
  Arc::clone（不重新解析、不重新规划）
       ↓
只创建本次运行的 LoaderItemState / CacheChainState
       ↓
run_loaders_with_preplanned(Loaders 中的共享 items/chains/locations, ...)
  按预组装 plan 一次性运行
```

这可以避免每次 build 重复执行：

- clone 两个平行 Vec 并重新 zip；
- 对每个 loader 重新解析 identifier/query/fragment/type；
- 重新读取 execution kind；
- 重新计算静态 fingerprint；
- 重新规划 chain tree 和 location index。

#### 重构 `LoaderItem`：静态定义与运行时状态分离

当前分支的 `LoaderItem` 同时保存固定定义和可变执行状态。重构后，存放在 `Arc<Loaders>` 中的 `LoaderItem` 只保留 factory 阶段已经确定、后续 build 不会变化的字段：

```rust
pub struct LoaderItem<Context: Send> {
  loader: Arc<dyn Loader<Context>>,
  request: Identifier,
  path: Utf8PathBuf,
  query: Option<String>,
  fragment: Option<String>,
  loader_type: String,
  execution_kind: LoaderExecutionKind,
  cache: bool,
  cache_key: String,
}
```

以下字段从 `LoaderItem` 移除：

- `data`；
- `pitch_executed`；
- `normal_executed`；
- `finish_called`。

它们组成每次 run 新建的 `LoaderItemState`，不再需要 `AtomicBool`：

```rust
#[derive(Default)]
pub struct LoaderItemState {
  data: serde_json::Value,
  pitch_executed: bool,
  normal_executed: bool,
  finish_called: bool,
}

pub enum CacheChainState {
  Pending,
  Bypassed,
  Hit,
  Miss(LoaderChainCacheState),
  Completed,
}

pub struct LoaderContext<Context: Send> {
  loaders: Arc<Loaders>,
  item_states: Box<[LoaderItemState]>,
  cache_chain_states: Box<[CacheChainState]>,
  loader_index: i32,
  // content/dependencies/context 等本轮状态
}
```

loader 访问器通过相同 index 组合 `Loaders.items` 与 `LoaderContext.item_states`。JS bridge 返回时只更新 `LoaderItemState`，不替换或重建 `Loaders.items/chains/locations`。

#### 由 `CacheChain` 控制动态 normal 状态

静态 `CacheChain` 不直接保存可变字段，而是通过 root index 控制本轮对应的 `CacheChainState` 和范围内的 `LoaderItemState`：

```text
CacheChain::enter_normal
  Pending
     ├─ 不满足缓存条件 ──> Bypassed
     ├─ cache hit ──────> Hit
     │                    标记 range 内 normal_executed/finish_called
     │                    回放 content/dependencies/build effects
     └─ cache miss ─────> Miss(snapshot)
                          执行 children
                               ↓
CacheChain::finish_normal
  Miss(snapshot) ──> store ──> Completed
  Bypassed       ────────────> Completed
```

状态所有权约束：

- pitch 始终执行，因此 `pitch_executed` 和 pitch 写入的 `data` 由叶子 execution chain 按 loader 更新，不能从 cache entry 回放；
- cache miss 时，JS/native execution chain 逐个更新 `normal_executed` 和 `finish_called`；
- cache hit 时，由外层 `CacheChain` 一次性更新整个 range 的 normal/finish 状态并移动全局 `loader_index`；
- `cacheable(false)`、pitch short-circuit、错误或不可缓存副作用把当前 root 的 `CacheChainState` 转为 `Bypassed`；
- `LoaderChainCacheState` 只属于当前 run，不能放进 module-owned `Loaders` 或复用于下一次 rebuild；
- 下一次 build 为所有 items/cache roots 重新创建默认 state，但复用同一个静态 `Arc<Loaders>`。

core 生产路径最终收敛为从 `Loaders` 取出预组装结果，再调用 loader-runner crate 的通用入口：

```rust
run_loaders_with_preplanned(
  loader_items: Arc<[LoaderItem<RunnerContext>]>,
  loader_chains: Arc<[LoaderChain]>,
  loader_chain_locations: Arc<[LoaderChainLocation]>,
  resource_data,
  plugin,
  context,
  fs,
)
```

删除生产路径中的平行参数 `Vec<BoxLoader> + Vec<LoaderRunnerOptions>`，也不再在 `run_loaders` 内调用 `LoaderItem::new` 或 `plan_loader_chains`。

cache lookup/store 直接从 `Loaders.cache_service()` 取得服务，不需要再给 `NormalModule` 增加独立的 `loader_cache_service` 字段。

#### 组装时机和 late mutation

`Loaders::new` 必须发生在以下操作全部完成之后：

1. module rules/function-form `use` 已求值；
2. post/inline/normal/pre loader 已 resolve；
3. `matchResource` 和 inline loader 顺序已经确定；
4. 所有能够改变最终 loader 列表的 factory hook 已结束。

当前 `before_loaders` hook 接收 `&mut NormalModule`，但没有公开的 loader mutation API。此次改造应明确 `Loaders` 在 module 创建后默认不可变。如果未来或内部 hook 需要替换 loader，必须调用统一入口：

```rust
NormalModule::replace_loaders(resolved_loaders)
  -> Arc::new(Loaders::new(...))
```

禁止直接修改 items 而不重建 chains/locations。不要对已经共享的 `Arc<Loaders>` 原地写入，也不要只更新 loader items 而保留旧 chain tree。

#### Cacheable/持久化模块缓存

`NormalModule` 当前参与 cacheable 序列化。`Arc<Loaders>` 接入时需要明确区分可序列化的 loader/chain 字段和不可序列化的 compiler service：

- `Loaders.items` 中的 loader trait object 沿用现有 `cacheable_dyn` 能力；
- chains 和 locations 只包含 range、variant、fingerprint 和 index，可安全序列化；
- `LoaderCacheService` 包含当前 compiler 的内存表、文件系统和 persistent 配置，不能作为 module cache payload 序列化，也不能从旧 compiler 恢复到新 compiler；
- `Loaders.cache_service` 必须采用 skip/rebind 语义：新 module 在 factory 中绑定，反序列化 module 在进入当前 compilation 后绑定当前 compiler 的 service；
- rebind 只能替换 service handle，不能重算 items/chains/locations；
- 反序列化后校验 chain 不变量；
- 调整 cache schema/version，避免旧 `Vec<BoxLoader> + Vec<LoaderRunnerOptions>` 与新布局混用。

不引入第二个中间容器类型。`Loaders` 序列化原始 loader trait objects 与 options，并跳过 `OnceLock` 中的运行时 items/plan 以及 cache service。正常 factory 路径会预填两个 `OnceLock`；持久化恢复的 module 因为不再经过 factory，会在第一次 build 时重建一次 items/plan 并绑定当前 compiler service，后续 rebuild 继续复用。这样不会在每次 build 重算，也不会把旧 compiler service 恢复到新 compiler。

```text
NormalModule
  loaders: Arc<Loaders>
             ├─ items
             ├─ chains
             ├─ locations
             └─ Arc<LoaderCacheService>
```

#### Strategy 与 benchmark

生产 `NormalModule` 保存一个按默认 strategy 生成的 `Arc<Loaders>`。测试和 benchmark 需要对比 strategy 时，在计时区间外分别构造多个 `Loaders::new_with_strategy(...)`；不要在 runner 热路径临时重新规划，否则会把 planner 成本混入 executor 对比，也违背预组装目标。

#### 方案边界

这个方案总体可行，但需要接受两个边界：

1. `Arc` 只能消除同一 module 多次 build/rebuild 的重复规划；不同 module 即使 loader 列表相同，仍各自持有一个 `Loaders`。跨 module 去重需要额外的 factory-level interner，不属于本次改造。
2. loader 数通常较少，规划本身是 O(N)；性能收益必须通过 rebuild/大规模模块 benchmark 验证。该结构的首要收益是让 chain 成为稳定的 module 元数据，并简化 runner，而不是预设规划成本一定显著。

## 4. 独立 planner

planner 继续与 executor 分离，并拆成可以单独测试和 benchmark 的纯步骤：

```text
create_loader_groups(loaders, strategy)
       ↓
生成 root cache boundaries
       ↓
partition_execution_chains(root range, strategy)
       ↓
生成 CacheChain children 或 root execution chain
       ↓
build_location_index + validate
       ↓
Loaders.chains + Loaders.locations
```

建议接口：

```rust
pub fn plan_loader_chains<Context: Send>(
  loaders: &[LoaderItem<Context>],
  strategy: LoaderChainStrategy,
) -> (Box<[LoaderChain]>, Box<[LoaderChainLocation]>);

fn plan_root_ranges(...);
fn plan_execution_children(...);
fn build_location_index(...);
fn validate_plan(...);
```

### 4.1 策略语义

保留四种内部策略，但让两类优化真正独立：

| Strategy | cache=true loader | 连续 JS loader |
| --- | --- | --- |
| `None` | 每个 loader 是 singleton `CacheChain` | 每个叶子是 singleton |
| `CacheOnly` | 连续 cache loader 合并为一个 `CacheChain` | children 保持 singleton |
| `JavaScriptOnly` | cache loader 保持 singleton `CacheChain` | 只合并同一 cache boundary 内或连续 uncached JS |
| `CacheAndJavaScript` | 合并连续 cache loader | 每个 root 内再合并连续 JS loader |

即使关闭 cache merge，`cache=true` 的 loader 仍必须有 singleton `CacheChain`，继续保留缓存功能；strategy 只控制是否把多个缓存 loader 合为一次 key/lookup，而不是控制 cache 功能开关。

JS merge 不得跨越两个 cache root，否则会破坏独立 cache lookup/store 的语义边界。

### 4.2 NativeExecutionChain 规则

- `None`、`CacheOnly` 基线中 native loader 保持 singleton，方便严格对比。
- `JavaScriptOnly` 不额外合并 native loader。
- `CacheAndJavaScript` 中 cache root 内相邻 native loader 可以归入同一个 `NativeExecutionChain`，但第一版也可以保持 singleton；是否合并 native 不得影响 yield 次数或 cache key 次数。
- 计划和 executor 均允许 `NativeExecutionChain` 包含一个或多个 loader，为后续 native batch/tracing 留出空间。

## 5. Runner 执行模型

### 5.1 Pitch

pitch 不参与缓存，但按 plan 的叶子 execution chain 调度：

```text
root 从左到右
  execution child 从左到右
    loader 从左到右 pitch
```

- 进入 `JsExecutionChain` 时一次 yield 给 JS。
- 进入 `NativeExecutionChain` 时在 Rust 中运行完整 range。
- pitch 返回 content 后，仍按全局 `loader_index - 1` 进入 normal。
- pitch short-circuit 发生在 `CacheChain` 内部时，只运行当前 loader 左侧部分；该 cache root 本轮 normal 必须 bypass lookup/store，因为不是完整 chain 输入输出。

### 5.2 Normal

normal 按 root 和 child 逆序执行：

```text
root 从右到左
  CacheChain before_normal_chain
    hit  -> 回放并跳过全部 children
    miss -> children 从右到左
              loader 从右到左 normal
            完成后 after_normal_chain/store 一次
```

以 mixed cache chain 为例：

```text
CacheChain [0..4]
  normal order:
    JsExecutionChain [3..4]
      ↓
    NativeExecutionChain [2..3]
      ↓
    JsExecutionChain [0..2]，内部顺序 1 → 0
      ↓
    store CacheChain [0..4]
```

Cache miss 状态保存在当前 run 对应 root index 的 `CacheChainState::Miss` 中，跨多个 child yield 保持到 root 左边界，不能放到某个 JS/native child 或静态 `LoaderItem` 中。这样 cache hook 的调用次数始终是每个 outer cache chain 一次，而不是每个 child 一次。

### 5.3 状态兼容

必须继续保持：

- `loader_index` 是扁平列表中的全局 index；
- `pitch_executed`、`normal_executed`、`finish_called` 仍属于单个 loader 的本轮 `LoaderItemState`；
- `remainingRequest`、`currentRequest`、`previousRequest` 基于完整扁平 loader list；
- raw/string 转换发生在相邻 loader 之间，不因 chain 边界改变；
- cache hit 标记 outer chain 内全部 normal loader 为 executed/finished；
- error 仍定位到具体 loader，而不是只显示 outer cache chain。

## 6. JS 主线程与 worker 调度

### 6.1 Rust → JS bridge

`JsLoaderContext` 不再接收动态 `current_execution_span()`，直接读取 plan 中的 `JsExecutionChain.range`。

内部字段建议从：

```text
loaderChainStart / loaderChainEnd
```

重命名为：

```text
executionChainStart / executionChainEnd
```

避免 JS 侧把它误认为包含 cache root 的完整 range。若 binding 兼容成本较高，可以先保留字段名，但注释必须明确它始终表示叶子 `JsExecutionChain`。

### 6.2 JS runner

`packages/rspack/src/loader-runner/index.ts`：

- pitching 只执行 `[executionChainStart, executionChainEnd)`；
- normal 只逆序执行同一范围；
- JS runner 不再判断相邻 loader 是否 builtin；planner 已保证范围内全部是 JS loader；
- 一次 yield 可以完成整个 JS child，并返回最终 `loader_index` 和每个成员状态。

### 6.3 Worker

`packages/rspack/src/loader-runner/worker.ts`：

- worker 仍只执行逐成员满足 `parallel` 的 JS loader；
- non-parallel loader 仍回到 JS 主线程；
- 删除用于识别 Rust builtin 的 `BUILTIN_LOADER_PREFIX` 分支，因为 builtin 不可能出现在 `JsExecutionChain`；
- `maxWorkers`、raw/source map/additional data 和同步请求行为保持逐 loader 语义。

## 7. Cache service 适配

缓存 key 和 entry 格式不需要重新设计。core runner 从 `Arc<Loaders>` 取得其 compiler-scoped service handle，再让 cache API 只接受 `CacheChain`：

```rust
before_normal_chain(context, cache_chain)
after_normal_chain(context, cache_chain, state)
```

调整点：

1. `static_fingerprint` 从 enum 的 `CacheChain` variant 读取。
2. input content 只在进入 outer cache chain 时 hash 一次。
3. cache hit 一次标记并跳过 outer range 内全部 child/loader。
4. cache miss 在执行完全部 child 后 store 一次。
5. dependency、parse meta、assets、build info 和 additional data delta 仍以 outer range 的执行前后状态捕获。
6. pitch short-circuit 进入 outer range 中部时 bypass 整个 cache chain，不能对子链产生部分 entry。
7. global cache disabled 时保留 tree 规划，只让 cache action 返回 `Disabled`。

缓存 identity 继续按 outer cache chain 的 normal 顺序包含所有 loader identity、type 和 options fingerprint。children 的划分不得进入 cache key，否则仅切换 JS merge strategy 会造成无意义的缓存失效。

## 8. 分阶段实施

### 阶段 A：在 factory 引入预组装 `Loaders` 和 enum plan

1. 将 `LoaderChain` struct 改成 enum，并增加 location index 和结构校验。
2. 新增单一的 `Loaders` struct，收拢当前 module 上的 loader 数组、runner options、本分支新增的静态字段、chains、locations 和 cache service。
3. 在 `NormalModuleFactory` 得到 post/inline/normal/pre 的最终顺序后调用一次 `Loaders::new`，并绑定 compiler 的 `Arc<LoaderCacheService>`。
4. `NormalModule` 改为只保存 `Arc<Loaders>`，删除平行的 `loaders` 与 `loader_runner_options` 字段。
5. 重写 `plan_loader_chains`，为四种 strategy 生成完整 tree，并在 factory 组装阶段调用。
6. 保留当前 executor，通过临时适配器读取 `Loaders` 中的定义和 tree，先保证代码可编译。
7. 在 tracing/test-only 输出中打印 root 与 children，确认 mixed cache chain 的 shape。

验收：factory 对每个 module 只组装一次 `Loaders`；同一 module rebuild 不再解析 loader 或调用 planner；同一 loader list 在四种 strategy 下得到预期 tree。

### 阶段 B：执行器切换到显式 leaf chain

1. runner 入口和 `LoaderContext` 直接接收 `Arc<Loaders>`，通过其 items/chains/locations 增加 `current_root_chain()`、`current_execution_chain()` 和 O(1) location 查询。
2. pitch/normal runner 以 root + leaf range 推进。
3. 将 `LoaderItem` 拆成 `Loaders.items` 中的静态部分和每次 run 的 `LoaderItemState`，删除静态 item 上的 `AtomicBool`/`data`。
4. 为每个 cache root 创建 `CacheChainState`，把 lookup/hit/miss/bypass/store 生命周期固定在 outer cache root。
5. JS bridge 只回写 `LoaderItemState`，禁止修改共享 items/chains/locations；cache hit 由 `CacheChain` 批量更新其 range。
6. 删除生产路径的 loader/options clone+zip、`LoaderItem::new` 和 `plan_loader_chains` 调用。
7. 删除 `Mixed`、`merge_reason` 和旧 chain range 分支。
8. 保留完整扁平 loader 观察语义并验证 pitch short-circuit。

验收：使用 `strategy=None` 时与当前逐 loader 行为完全一致；mixed cache chain 的 hit/miss 各只触发一次 cache hook。

### 阶段 C：JS bridge 消费显式 JsExecutionChain

1. Rust scheduler 只在当前叶子是 `JsExecutionChain` 时 yield。
2. binding 传递叶子的明确 range，不再调用 `current_execution_span()`。
3. JS 主线程和 worker 严格消费该 range。
4. 删除 Rust 和 worker 中的相邻 execution-kind/builtin 扫描。
5. 更新生成的 N-API 类型。

验收：一次 `JsExecutionChain` 只产生一次 Rust/JS round-trip；builtin 永远不会进入 JS range。

### 阶段 D：缓存、测试和可观测性收尾

1. factory 创建 `Loaders` 时 clone compiler-scoped `LoaderCacheService` handle；确认所有 module 共享同一底层 service，而不是各自创建 backend。
2. cache service API 限定为 `CacheChain`，确认 fingerprint 不包含 child shape。
3. 实现 module cache 反序列化后的 service rebind；不能恢复旧 compiler service，允许在首次 build 惰性重建一次被跳过的 items/chains/locations。
4. tracing 分别记录 outer cache chain 和 leaf execution chain。
5. 增加 strategy A/B 指标：factory plan 次数、rebuild replan 次数、root 数、child 数、JS yield 数、cache hash/lookup/store 数。
6. 更新内部文档和 `plan.md` 对应实现状态。
7. 运行构建、lint 和回归测试。

验收：`CacheOnly` 与 `CacheAndJavaScript` 的 cache 次数相同、JS yield 次数不同；`JavaScriptOnly` 与 `None` 的 cache boundary 相同、JS yield 次数不同。

## 9. 测试矩阵

遵循项目约束，不新增 crate-local inline Rust `#[test]`；优先使用现有 JS case runner，确需直接验证 planner shape 时放入合适的专用测试 crate/integration harness。

### 9.1 Planner shape

- 全 JS、全 native、JS/native/JS 混合。
- 连续 cache、cache/non-cache/cache、单个 cache loader。
- mixed cache root 的 children 为 JS/Native/JS。
- root 和 children range 连续、无重叠、无空洞。
- children 中不存在 `CacheChain`。
- 四种 strategy 对同一 loader list 生成预期不同的 tree。

重点 fixture：

```text
cache JS A
cache JS B
cache builtin C
cache JS D
uncached JS E
uncached JS F
uncached builtin G
```

`CacheAndJavaScript` 期望：

```text
Cache[0..4](JS[0..2], Native[2..3], JS[3..4])
JS[4..6]
Native[6..7]
```

### 9.2 执行顺序

- pitch 按 0 → N，normal 按 N → 0。
- JS child 内多个 loader 一次 yield，但保持逐 loader raw/string 转换。
- JS/native/JS mixed cache miss 的输出与 singleton baseline 相同。
- mixed cache hit 跳过全部 JS/native normal loader。
- pitch 在 cache root 中间短路时不 lookup/store 部分 cache entry。
- error、empty return、`cacheable(false)` 和 diagnostics 行为不变。

### 9.3 Strategy 独立性

- `None`：每个执行 leaf singleton，每个 cache loader singleton cache root。
- `CacheOnly`：连续 cache 合并，但内部连续 JS 仍是 singleton children。
- `JavaScriptOnly`：cache boundary 不合并，允许每个 boundary 内 JS 合并。
- `CacheAndJavaScript`：外层 cache 和内层 JS 都合并。
- 四种策略输出、dependency、source map 和 additional data 完全一致。

### 9.4 JS/worker

- 连续 JS child 的主线程执行。
- parallel true/false 在同一 JS child 中切换。
- worker 不再需要 builtin 前缀判断。
- `maxWorkers`、无 pitch loader、raw buffer、source map、additional data。
- 复跑现有 loader-parallel cases。

### 9.5 Cache invalidation

- mixed chain memory hit 和 persistent hit。
- 修改输入、任一 child loader options、任一 loader 文件或显式依赖后 miss。
- child 划分策略改变但 outer loader identity/input/options 不变时 cache key 保持一致。
- cache hit 后依赖和 build side effects 正确回放。

### 9.6 `Loaders` 生命周期

- factory 为一个 module 只调用一次 `Loaders::new` 和 planner。
- 同一 module 的 watch rebuild 复用同一个 `Loaders` 和 chain 结果，但所有 `LoaderItemState`/`CacheChainState` 都重新创建。
- cache hit 由 `CacheChain` 一次性标记范围内的 normal/finish 状态；pitch/data 保持本轮执行结果。
- 两个 module 分别拥有自己的 `Arc<Loaders>`，但其 `cache_service` 指向同一个 compiler-scoped service。
- 相同 cache key 可以跨 module 命中，证明没有误建 module-local backend。
- module cache 序列化不包含 compiler service；反序列化后绑定当前 compiler service。
- 反序列化后的首次 build 最多重新解析/规划一次，之后的 rebuild 必须复用同一 plan。
- late loader replacement 只能通过统一 API 完成，并生成新的 `Arc<Loaders>` 和 plan。

## 10. 验证命令

JS 和 Rust 都会修改，完成后依次执行：

```text
pnpm run build:cli:dev
pnpm run test:rs
pnpm run test:unit
cargo lint
```

另外单独运行：

- loader-chain watch cases；
- loader-chain persistent cache case；
- loader-parallel config/runtime cases；
- planner strategy 对比用例。

按仓库约束，在 sandbox 中跳过 storage 和 native watcher 会卡住的测试；对应部分交给非 sandbox CI。

## 11. 完成标准

- `LoaderChain` 已改为 `CacheChain | JsExecutionChain | NativeExecutionChain` enum。
- `NormalModule` 只保存一个 factory 预组装的 `Arc<Loaders>`，不再分别保存 loader 数组和 runner options。
- `Loaders` 包含 loader items、chains、locations 和 compiler-scoped cache service handle；不同 module 不共享组装结果，但共享底层 cache service。
- `LoaderItem` 的固定定义与 `LoaderItemState` 已分离，module-owned items 不包含 `data` 或 executed/finish 状态。
- module rebuild 不重新解析 loader、不重新计算 static fingerprint，也不重新规划 chain；只初始化本轮 `LoaderItemState` 和 `CacheChainState`。
- mixed cache chain 显式包含 JS/native children，不再用 `execution_kind=Mixed` 表示。
- runner、JS bridge 和 worker 不再动态扫描 execution span。
- cache lookup/store 只发生在 outer `CacheChain`，JS yield 只发生在 `JsExecutionChain`。
- 四种 strategy 能独立启停 cache 合并和 JS 合并，并可用指标验证。
- pitch/normal 顺序、loader context、parallel、缓存结果和失效行为与 singleton baseline 一致。
- 所有新增与相关回归测试通过，文档与生成 binding 类型同步更新。
