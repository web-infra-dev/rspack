# Loader Chain 与 Loader Result Cache 执行计划

## 1. 目标

本次改造包含三个相互关联的目标：

1. 在 loader runner 中引入 `LoaderChain`，一个 chain 包含一个或多个按原顺序排列的 loader，并成为 runner 外层状态机的基本执行单元。
2. 在 `Rule.use` 的 loader 配置中，在 `options`、`parallel` 同一层增加 `cache?: boolean`。`cache: true` 表示该 loader 的 normal 阶段结果可以按“输入代码哈希 + loader options”复用。
3. 将连续的可缓存 loader 合并到同一个 chain，一次计算输入代码哈希并缓存整个转换区间；同时将连续的 JS loader 合并，令 Rust/JS yield 的边界直接由 chain 描述，而不是由 Rust、JS 主线程和 worker 各自扫描 loader 列表推断。

本次不改变 webpack 可观察到的 loader 顺序、pitch/normal 方向、`loaderContext.loaderIndex`、`remainingRequest`、`previousRequest`、`data` 和 raw/string 转换语义。

## 2. 当前执行流与主要问题

当前配置和执行路径为：

```text
Rule.use { loader, options, parallel }
  packages/rspack/src/config/types.ts
       ↓
createRawModuleRuleUses / ident references
  packages/rspack/src/config/adapterRuleUse.ts
       ↓
RawModuleRuleUse -> ModuleRuleUseLoader
  crates/rspack_binding_api/src/raw_options/raw_module/mod.rs
  crates/rspack_core/src/options/module.rs
       ↓
逐个 resolve 为 BoxLoader
  crates/rspack_core/src/normal_module_factory.rs
       ↓
run_loaders(Vec<BoxLoader>)
  crates/rspack_loader_runner/src/runner.rs
       ↓
以 loader_index 为游标逐个 pitch / normal
       ↓
遇到 JS loader 时 should_yield / loader_yield
  crates/rspack_binding_api/src/plugins/js_loader/scheduler.rs
       ↓
JS 主线程和 worker 再次向前/向后扫描连续 loader
  packages/rspack/src/loader-runner/index.ts
  packages/rspack/src/loader-runner/worker.ts
```

主要问题：

- runner 的调度单元、JS yield 单元和未来缓存单元不是同一个概念，边界判断分散在 Rust、JS 主线程和 worker 中。
- 如果用“每个 loader 外包一层 cache loader”的方式实现缓存，连续 loader 会对同一份中间代码重复计算哈希。
- `parallel` 目前通过 ident reference 旁路传递；新增缓存信息如果继续只留在 JS 侧，Rust runner 无法在执行前建立 chain。
- loader 的输出不只有代码，还包括 source map、additional data、依赖集合以及 module/loader context 副作用；只缓存 content 会造成行为不一致。

仓库历史中的 loader cache 原型可以复用其依赖增量回放、loader 文件失效、内存/持久化分层等经验，但不继续采用插入 `builtin:cache-loader` 的结构；缓存应直接成为 `LoaderChain` normal 执行的一部分。

## 3. 目标模型

### 3.1 配置与 resolved loader

公共类型增加：

```ts
type RuleSetLoaderWithOptions = {
  loader: string;
  options?: string | Record<string, any>;
  parallel?: boolean | { maxWorkers?: number };
  cache?: boolean;
  ident?: string;
};
```

绑定和 core 不再只传 `BoxLoader`，而是保留执行规划所需的元信息：

```text
ResolvedLoader
  loader: BoxLoader
  cache: bool
  options_fingerprint: stable bytes/hash
  execution_kind: Native | JavaScript
```

`cache` 默认值为 `false`。inline loader 和字符串形式 loader 没有显式标记时保持不缓存。`execution_kind` 应由 loader/resolver 明确提供，避免长期依赖 `builtin:` 前缀猜测执行域；JS loader 返回 `JavaScript`，native/builtin loader 默认为 `Native`。

JS object options 需要在配置适配阶段生成稳定 fingerprint，再通过 `RawModuleRuleUse` 和 `ModuleRuleUseLoader` 传到 Rust。不能把 `ident` 本身当作 options 内容，也不能直接依赖对象属性插入顺序。第一版仅接受可稳定序列化的 cache options；遇到 function、symbol、循环引用等不稳定值时，在对应 option path 给出明确配置错误，避免产生表面命中但结果错误的缓存。

### 3.2 LoaderChain

在 `crates/rspack_loader_runner/src/chain.rs` 新增结构化模型，chain 持有 loader 在扁平列表中的范围，而不使用 `$` 等字符串分隔符编码：

```text
LoaderChain
  range: [start, end)
  cache_mode: Disabled | WholeChain
  execution_kind: Native | JavaScript | Mixed
  merge_reason: Singleton | Cache | JavaScript
  static_fingerprint: ordered(loader identity + options fingerprint)
```

`LoaderContext` 继续保存扁平 `loader_items` 和全局 `loader_index`，新增内部 `loader_chains`、`chain_index` 和 chain 内部进度。这样 loader 看到的 `loaders`、`loaderIndex` 和各类 request 字符串不发生变化；chain 只是 runner 的内部调度视图。

pitch 阶段按 chain、chain 内按 loader 从左向右执行；normal 阶段按 chain、chain 内按 loader 从右向左执行。pitch 返回内容时仍跳转到当前 loader 左侧的 normal 流程。

### 3.3 独立的 chain 规划/合并逻辑

合并逻辑必须与执行器分离，建议提供以下纯函数：

```text
create_singleton_chains(loaders)
merge_cacheable_chains(chains)
merge_javascript_chains(chains)
plan_loader_chains(loaders, strategy)
```

`strategy` 至少支持内部 A/B 组合：

- `None`：每个 loader 一个 chain，作为功能和性能基线。
- `CacheOnly`：只合并连续 `cache: true` loader。
- `JavaScriptOnly`：只合并连续 JS loader。
- `CacheAndJavaScript`：生产路径，同时应用两类合并。

合并规则：

1. 先把连续 `cache: true` loader 合并为最大区间，允许区间内同时存在 native 和 JS loader；这是一次 cache lookup/store 的语义边界。
2. 再合并其余连续 JS loader。JS 合并不能跨越已有的缓存边界，否则会把 `cache: false` loader 纳入缓存结果，或拆掉连续 cache loader 的单次哈希边界。
3. 未合并的 native loader 保持单元素 chain。
4. 合并只能改变分组，不能改变 loader 顺序和 request 字符串。

对于 mixed cache chain，外层仍把整个区间视为一个缓存单元；chain 内执行器根据连续的 native/JS span 调度。这样可以满足“连续 cache loader 只算一次 key”，同时不要求 Rust loader 在 JS worker 中运行。

## 4. 缓存语义

### 4.1 Key

按需求，动态 key 的核心为：

```text
input_hash = hash(input content bytes)
options_hash = hash(按 chain normal 执行顺序排列的每个 loader options fingerprint)
cache_key = versioned namespace(chain loader identities) + input_hash + options_hash
```

- loader identity 放在 cache namespace/静态 fingerprint 中，防止两个不同 loader 使用相同 options 和输入时冲突。
- `input_hash` 在进入一个 cache chain 时只计算一次；chain 中有多少个 loader 都不重复计算。
- `options_hash` 和 loader identity 尽量在 chain 构造时预计算，每个模块执行时只组合 input hash。
- cache 格式包含 Rspack 版本和格式版本；loader 实现文件加入 build dependencies，并通过 snapshot/内容时间戳使 loader 代码变化后失效。
- 按需求不默认加入 `resourcePath`，因此 `cache: true` 同时是一项纯度声明：normal 输出应只由输入代码和 options 决定。文档中必须明确依赖 `resourcePath`、环境变量、时间或外部隐式状态的 loader 不应开启该选项；显式注册的依赖仍参与 entry 有效性检查。

source map 和 opaque additional data 也可能影响后续 loader。第一版采用保守策略：输入存在无法稳定 fingerprint 的 source map/additional data 时跳过该 chain 的缓存，而不是生成不完整 key；后续可以在不改变 chain API 的前提下扩展稳定序列化。

### 4.2 Lookup、执行和 store

缓存只作用于 normal 阶段；所有 pitch 函数始终执行，以保留短路、`data` 初始化和 pitch 副作用语义。

```text
进入 cache chain 的 normal 阶段
  取得当前 content
       ↓
计算一次 input_hash + 组合静态 fingerprint
       ↓
cache hit? ── yes ──> 回放结果/依赖/副作用，标记 chain 内 normal_executed
    │ no
    ↓
按原顺序执行 chain（必要时在 native/JS span 间 yield）
    ↓
chain 完成且允许缓存?
    ├─ no: 直接进入前一个 chain
    └─ yes: 捕获输出与增量，写入 cache，再进入前一个 chain
```

以下情况不写入 entry：执行报错、任一 loader 调用 `cacheable(false)`、出现无法安全保存/回放的上下文副作用、输入或输出 additional data 无法由当前 backend 表示、执行期间依赖快照发生变化。失败和 cache I/O 只表现为 miss，不应让 compilation 失败。

### 4.3 Cache entry

entry 至少保存并在 hit 时回放：

- content（保留 string/buffer 类型）和 source map；
- backend 支持时的 additional data；不支持时该次不落盘，且不能返回缺失数据的 hit；
- file/context/missing/build dependency 相对 chain 执行前的增删 delta；
- parse meta、`cacheable` 状态以及 chain 执行产生的可缓存 diagnostics；
- loader/module context 上可观察的产物，例如 emitted assets/build info delta；若某种副作用暂时无法完整捕获，则该次执行 bypass cache；
- 完整 cache identity、格式版本、写入时间和依赖快照，用于校验碰撞、损坏和失效。

不要缓存 error，也不要在 hit 时重新触发 loader 日志。normal 执行完成后才原子发布 entry，避免并发读取半成品。

### 4.4 Backend 与生命周期

在 `crates/rspack_core/src/loader/loader_cache.rs` 提供 compiler-scoped `LoaderCacheService`：

- L1 使用并发内存 map，支持同一 compiler 内不同模块复用。
- compiler 配置为 persistent cache 时接入现有 persistent storage/snapshot 生命周期，使用独立 versioned scope；memory cache 时只保留 L1。
- 全局 cache disabled 时 `cache: true` 仍保留 chain 规划，但 lookup/store 退化为关闭状态，便于保持单一执行路径。
- 同 key 并发 miss 可选用 single-flight，避免多个模块同时执行同一昂贵 chain；第一阶段先保证原子写和正确性，再单独评估 single-flight 是否值得加入。

## 5. 分阶段实施

### 阶段 A：配置与元信息贯通

1. 在 `packages/rspack/src/config/types.ts` 增加 `cache?: boolean` 和纯度/默认值说明。
2. 修改 `packages/rspack/src/config/adapterRuleUse.ts`：设置默认值、生成稳定 options fingerprint，并把 cache 元信息直接写入 `RawModuleRuleUse`；不要仅放进 JS ident reference。
3. 修改 `crates/rspack_binding_api/src/raw_options/raw_module/mod.rs`、生成的 binding 类型和 `crates/rspack_core/src/options/module.rs`，让静态数组和 function-form `use` 都保留 `cache` 与 fingerprint。
4. 修改 `crates/rspack_core/src/normal_module_factory.rs` 的 resolve 结果，使用 `ResolvedLoader` 保存 loader、cache、fingerprint 和执行域；inline/pre/normal/post loader 拼接后仍保持当前最终顺序。
5. 更新所有手工构造 `ModuleRuleUseLoader` 的 Rust 调用点，默认 `cache: false`。

阶段验收：不开启 `cache` 时生成的 request、loader 顺序和现有用例结果完全不变；类型测试能识别 `cache: true`。

### 阶段 B：引入 chain，但先保持 singleton 行为

1. 在 `rspack_loader_runner` 增加 `LoaderChain`、chain 游标和扁平 loader 兼容访问器。
2. 将 `run_loaders` 外层状态机改成按 chain 推进，但先固定使用 `strategy = None`，确保每个 chain 只有一个 loader。
3. 把 pitch/normal 的 loader 级循环移动到 chain executor；保持 `finish_called`、空返回值、pitch 短路和错误定位行为。
4. tracing 从只记录 loader 增加 chain span，并保留具体 loader 子 span，字段包括 chain 长度、range、execution kind、merge reason。

阶段验收：关闭合并时，现有 loader runner、builtin loader、JS loader 和错误快照无变化；这是之后所有优化的稳定基线。

### 阶段 C：独立实现并启用 JS chain 合并

1. 实现 `merge_javascript_chains`，先在测试/benchmark 中对比 `None` 与 `JavaScriptOnly`。
2. 修改 `crates/rspack_binding_api/src/plugins/js_loader/context.rs`，向 JS 传递结构化 chain range/chain index，同时继续传完整扁平 loader items。
3. 修改 `crates/rspack_binding_api/src/plugins/js_loader/scheduler.rs`：`should_yield` 只判断当前 chain/span，不再通过当前 loader 前缀重复推断整个连续区间；`no_pitch` 信息按 chain 成员更新。
4. 修改 `packages/rspack/src/loader-runner/index.ts`：JS runner 严格在 Rust 提供的 chain range 内 pitch/normal，删除“扫描到下一个 builtin loader”为止的隐式边界。
5. 修改 `packages/rspack/src/loader-runner/worker.ts`：worker 同样消费明确 range；parallel 与 `maxWorkers` 仍按成员配置执行，不能因 chain 合并把 non-parallel loader 发送到 worker。
6. 命中 JS chain 边界后一次 Rust→JS→Rust round-trip 完成该范围，返回每个成员的 executed/data 状态和最终全局 loader index。

阶段验收：相同 loader 列表下 JS yield 次数等于 JS chain 数，而不是 JS loader 数；pitching、mixed builtin/JS、parallel mixed、raw loader 和无 pitch loader 均保持现有行为。

### 阶段 D：cache chain 与内存缓存

1. 实现 `merge_cacheable_chains`，并在生产 planner 中使用 `CacheAndJavaScript`；cache 合并先于 JS 合并。
2. 在 chain normal executor 前后加入 lookup/store 状态，确保 mixed chain 跨 JS yield 后仍能继续同一个 miss 执行并在左边界只 store 一次。
3. 实现静态 fingerprint 预计算和单次 input content hash；tracing/计数器分别记录 key 计算次数、hit/miss/store/bypass 原因。
4. 实现 compiler-scoped 内存 `LoaderCacheService`，完整捕获并回放 entry 中的输出和 context delta。
5. 尊重 `cacheable(false)`、错误、diagnostics、dependency 与副作用的 bypass 规则。

阶段验收：连续 N 个 cache loader 在一次 normal 流程中只计算一次 input hash、一次 lookup、一次 store；相同输入和 options 的下一次执行跳过全部 N 个 normal loader。

### 阶段 E：持久化与失效

1. 将 loader cache scope 接入现有 persistent cache 生命周期，复用现有 cache directory、readonly、版本和 snapshot 约束，避免创建另一个用户配置入口。
2. entry 落盘采用 versioned envelope、完整 identity 校验、校验和与原子写；损坏或 I/O 错误降级为 miss。
3. 保存 loader 实现文件及显式依赖快照；资源输入通过 input content hash 失效，依赖和 loader 文件通过 snapshot 失效。
4. additional data 或 context 副作用无法稳定持久化的 entry 仅允许停留在 L1，且旧的同 key 磁盘 entry 必须失效，防止错误回退。
5. compiler close 时等待 pending writes，readonly 模式只读不写。

阶段验收：第二个 compiler/process 能命中有效 entry；修改输入、options、loader 文件或依赖后 miss；损坏 entry、并发 writer 和 readonly 不影响构建正确性。

### 阶段 F：默认启用、清理旧判断与文档

1. 所有回归和性能数据通过后启用 `CacheAndJavaScript`，保留 `None`/分项 strategy 仅供测试和 benchmark 调用，不暴露临时环境变量作为正式 API。
2. 删除 Rust/JS/worker 中已经由 chain range 取代的重复连续 loader 扫描和 `$` composed-loader 遗留注释。
3. 更新 loader 配置文档和类型说明，给出适合缓存的纯 loader 示例与不应缓存的 `resourcePath`/时间/隐式外部状态示例。
4. 在 release note 中说明 `cache` 是 opt-in、默认 false、normal-only，且 `cacheable(false)` 会阻止写入。

## 6. 测试矩阵

遵循项目测试约束，优先在现有 runner 下增加 case，不新增顶层 `test.js`，也不添加 crate 内联 Rust unit test。

### 配置和 chain 规划

- `cache` 缺省/false/true，object、string 和 function-form `use`。
- 不稳定 options 给出包含配置路径的错误。
- singleton、连续 cache、由 non-cache 分隔、连续 JS、mixed native/JS 的 chain shape 通过 tracing/test-only inspection 验证。
- `None` 与 `CacheAndJavaScript` 对同一 fixture 产生相同输出、request 和 loader context 观察值。

### 缓存 key 和结果

- 相同代码 + 相同 options 命中；代码变化 miss；options 变化 miss；不同 loader identity 不冲突。
- 两个或更多连续 cache loader 只执行一次 chain lookup，并全部在 hit 时跳过。
- 相同内容的不同资源可以共享 entry；依赖 `resourcePath` 的反例在文档中明确为不满足 cache contract。
- string/buffer、source map、可支持的 additional data、空返回 loader。
- `cacheable(false)`、throw/callback error、diagnostic、无法回放的 emit/side effect 均不写入错误 entry。

### pitch、JS yield 和 parallel

- 有/无 pitch、pitch 返回 content、pitch data 传到 normal、pitch 短路跨 chain。
- 连续 JS loader 一次 yield；JS/native/JS 形成明确边界；cache mixed chain 可在内部 yield 后完成同一个 store。
- parallel true/false 混合、不同 `maxWorkers`、worker loader context、raw buffer、source map 和 additional data。
- 复跑现有 `tests/rspack-test/configCases/loader-parallel*`，确保 worker 行为没有退化。

### watch/persistent invalidation

- 仅修改 resource code、loader options、loader 文件、file/context/missing/build dependency。
- cache hit 时依赖增删 delta 被正确回放，watch 下一轮仍能收到失效信号。
- 第二 compiler 命中、格式版本变化 miss、损坏 entry miss、并发写原子性、readonly。

本地 sandbox 中按项目约束跳过 storage 和 native watcher 容易卡住的测试；对应场景交给非 sandbox CI。JS+Rust 修改完成后先执行 `pnpm run build:cli:dev`，再运行新增的过滤 case、`pnpm run test:unit`、`pnpm run test:rs` 和 `cargo lint`。

## 7. 性能对比与可观测性

为独立比较合并收益，使用相同 executor，只替换 planner strategy，采集：

- chain 数、平均/最大 chain 长度；
- Rust→JS yield/round-trip 次数；
- worker dispatch 次数；
- input hash 计算次数和总字节数；
- cache lookup/hit/miss/store/bypass 数；
- loader normal 实际执行次数；
- 冷构建、同 compiler 重复构建、watch rebuild、第二 process persistent warm build 耗时。

至少对以下四组结果做对比：`None`、`JavaScriptOnly`、`CacheOnly`、`CacheAndJavaScript`。性能 fixture 应包含大量小 JS loader、多个连续昂贵 cache loader、cache/native/JS 混排以及低命中场景，防止只优化理想高命中路径。

验收标准：

- 未标记 cache 的项目输出和行为零变化；JS chain 合并只减少调度开销。
- 连续 cache loader 的 input hash 次数从 loader 数降为 chain 数。
- warm hit 路径不执行 chain 内 normal loader，并正确回放所有可观察结果。
- miss 路径的额外 fingerprint/hash/快照成本可量化，且 `CacheAndJavaScript` 相对 singleton 基线有稳定收益或至少无显著回退。

## 8. 主要风险与处理

- **错误共享跨资源结果**：严格文档化 input/options 纯度契约；loader identity 隔离 namespace；显式依赖参与失效。
- **options fingerprint 不稳定**：配置阶段 canonicalize；不支持的值直接报错，不静默退化为 ident 或 `[object Object]`。
- **source map/additional data 不完整**：无法稳定 fingerprint/保存时 bypass，不产生部分 entry。
- **pitch 与 normal 状态错位**：pitch 不缓存；保留扁平 loader 状态和全局 index；先以 singleton chain 完成等价重构。
- **JS/worker 边界改变 parallel 语义**：chain 只确定可执行范围，worker eligibility 和 pool options 仍逐成员检查。
- **副作用丢失**：以 chain 前后 delta 捕获并回放；暂不支持的副作用使 entry 不可缓存。
- **合并优化难以归因**：planner 独立、strategy 可注入、统一 tracing 指标，确保可以单独关闭 cache merge 或 JS merge 做对比。
