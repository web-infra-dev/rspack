# Arena Rope Source 执行计划

本计划对应 [design.md](design.md)。每个阶段都必须能独立 benchmark、验证或回滚，避免同时改写
placeholder、replacement、source map 和 persistent cache。

## Phase 0：建立基线和观测

目标：确认真实 workload，避免只针对理论最坏情况优化。

工作项：

1. 在 `ReplaceSource::add_replacement` 的 profiling/benchmark 构建中记录：
   - replacement 总数；
   - append fast-path 次数；
   - out-of-order 次数；
   - `Vec::insert` 移动的元素总数；
   - 单个 Source 的最大 replacement 数；
   - 单调 run 数量。
2. 记录 placeholder 路径的：
   - `source()` 物化次数和 bytes；
   - scan 次数和 bytes；
   - placeholder 数量；
   - resolve 后再次物化次数。
3. 扩充 `xtask/benchmark` workload：
   - replacement 数量：16 / 128 / 1k / 6k；
   - 顺序：ascending / descending / random / 3～10 monotonic runs；
   - placeholder：0 / 1 / 100 / 1k；
   - source map：off / line-only / columns；
   - direct Source 和 `CachedSource` 两种生命周期。

涉及文件：

- `crates/rspack_sources/src/replace_source.rs`
- `xtask/benchmark/benches/groups/rspack_sources_complex_replace_source.rs`
- 新增或扩展 `xtask/benchmark/benches/groups/rspack_sources_*.rs`

完成标准：

- 有当前实现的 CPU、allocation 和 peak retained memory 基线；
- 能区分 replacement 插入成本、最终遍历成本和 placeholder scan 成本；
- 临时观测代码不进入普通 release 热路径，或使用 feature/cfg 控制。

## Phase 1：实现 NodeId 和最小 arena

目标：实现不改变现有 Source 行为的基础容器。

工作项：

1. 新增 `NodeId(NonZeroU32)`：
   - `index + 1` 编码；
   - checked conversion；
   - `Option<NodeId>` size assertion；
   - Debug 输出同时显示 raw ID 和 index。
2. 实现 append-only `RopeArena`：
   - `alloc(node) -> Result<NodeId>`；
   - checked `get/get_mut`；
   - builder 生命周期不复用槽位。
3. 实现 `Text`、`ChildSource` 和 `Branch` 节点。
4. 实现 AVL rotation 和 summary 更新。
5. 实现非递归中序 iterator，避免极端树深导致栈溢出。

涉及文件：

- `crates/rspack_sources/src/rope/node_id.rs`
- `crates/rspack_sources/src/rope/arena.rs`
- `crates/rspack_sources/src/rope/summary.rs`
- `crates/rspack_sources/src/rope/builder.rs`

验证：

- 在现有专用测试 crate 或 benchmark correctness setup 中验证顺序遍历；
- fuzz/property 测试比较 Rope 拼接与 `String` 拼接；
- 检查空、单节点、最大合法 NodeId conversion 和 arena overflow error；
- 不新增普通 crate-local inline `#[test]`。

完成标准：

- arena 不含裸指针；
- NodeId 稳定；
- 树高符合 AVL 上界；
- 相同逻辑序列不受插入和平衡形态影响。

## Phase 2：实现冻结态 RopeSource

目标：让 Rope 作为 concat/string composition 的 `Source` 实现运行，但不处理 placeholder 和
replacement。

工作项：

1. 增加 `RopeSource`，实现：
   - `source()`；
   - `rope()`；
   - `buffer()`；
   - `size()`；
   - `to_writer()`；
   - structural hash/equality；
   - `StreamChunks`。
2. `ChildSource` mapping 组合复用 `ConcatSourceChunks` 的 source/name index remap 逻辑。
3. 分别实现 column 和 line-only map 路径。
4. 增加有序 bulk build 和单叶子 fast path。
5. 与 `ConcatSource` 做 differential benchmark，不替换生产调用点。

涉及文件：

- `crates/rspack_sources/src/rope_source.rs`
- `crates/rspack_sources/src/rope/stream_chunks.rs`
- `crates/rspack_sources/src/concat_source.rs`
- `crates/rspack_sources/src/lib.rs`

验证：

- 对相同 child Source 序列比较 `source/size/buffer/to_writer/map/hash`；
- 覆盖 ASCII、非 ASCII、多行、尾随换行、无 mapping 和嵌套 source map；
- benchmark 小节点、数百节点和 chunk 级拼接。

完成标准：

- 无 source map 行为差异；
- 无 bundle-sized retained flat string；
- concat-heavy benchmark 至少不明显回退，才能进入下一阶段。

## Phase 3：实现 typed placeholder 和构建/冻结边界

目标：验证方案最确定的收益，不依赖 replacement Rope 化。

工作项：

1. 实现：
   - `PlaceholderKey`；
   - `PlaceholderId`；
   - lazy `PlaceholderTable`；
   - `FxHashMap<PlaceholderKey, PlaceholderId>`；
   - slot value table；
   - 可重建 occurrence index。
2. 增加 `TemplateRopeSource`，明确不实现 `Source`。
3. 实现 register、append placeholder、resolve、duplicate resolve 校验和 unresolved error。
4. 实现 `freeze() -> Result<RopeSource>`：
   - 检查所有 slot；
   - 检测循环引用；
   - 计算 summary；
   - 清理 builder-only index。
5. 构建态和冻结态使用不同 Debug、hash 和 persistent cache tag。

涉及文件：

- `crates/rspack_sources/src/rope/placeholder.rs`
- `crates/rspack_sources/src/rope/builder.rs`
- `crates/rspack_sources/src/rope_source.rs`
- `crates/rspack_error` 中适当的错误上下文接口（如需要）

验证：

- 用户文本与旧 marker 完全相同也不会被 resolve；
- 同一 placeholder 多次出现只 resolve 一次；
- unresolved、conflicting resolve、cycle 提供可操作错误；
- resolve 后 `source/size/map/hash` 稳定。

完成标准：

- placeholder 路径不需要文本 marker；
- freeze 前后类型边界由 Rust API 强制；
- 没有 `CachedSource<unresolved>` 的可能。

## Phase 4：迁移第一批 placeholder 热点

目标：移除明确存在的全量物化和扫描。

迁移顺序：

1. JavaScript auto public path：
   - `crates/rspack_plugin_javascript/src/runtime.rs`
2. JavaScript static URL / worker URL：
   - `crates/rspack_plugin_javascript/src/plugin/url_plugin.rs`
   - `crates/rspack_plugin_javascript/src/dependency/url/mod.rs`
   - `crates/rspack_plugin_javascript/src/dependency/worker/mod.rs`
3. CSS auto public path：
   - `crates/rspack_plugin_css/src/plugin/impl_plugin_for_css_plugin.rs`
   - `crates/rspack_plugin_css/src/dependency/url.rs`

每个迁移项执行流：

```text
Dependency template
  写 PlaceholderKey，不写 marker 字符串
       ↓
Code generation data
  保留 resolver 所需 DependencyId / output context
       ↓
内部 render phase
  resolve PlaceholderId
       ↓
freeze
  生成 BoxSource
       ↓
现有 public/plugin hooks
```

验证：

- 为旧 marker 同名用户源码添加现有 integration harness 下的 case；
- 比较 output 和 source map snapshot；
- 确认迁移路径不再调用 `source()` 搜索 marker；
- 测量 CPU、allocation 和 peak memory。

完成标准：

- 三条路径不再扫描 marker；
- 兼容现有 hooks；
- benchmark 显示收益，或至少证明正确性收益的成本可接受。

## Phase 5：实现 replacement edit tree

目标：将 replacement 元数据放入 arena tree，优化乱序插入，同时保留原始 offset 语义。

工作项：

1. 增加 `Replacement` 节点，键严格为：
   - `start`；
   - `end`；
   - `enforce`；
   - `insertion_order`。
2. 实现 `insert()` 等价于 `[start, start)` replacement。
3. 支持：
   - append/right-spine fast path；
   - AVL random insert；
   - batch sorted bulk build；
   - in-order edit iterator。
4. 暂时让遍历 iterator 驱动现有 ReplaceSource rendering/source-map state machine，先只替换
   replacement 容器，不马上引入 `OriginalSlice`。
5. 调整 `replacements()` API：
   - 避免承诺返回 `&[Replacement]`；
   - 提供有序 iterator；
   - persistent cache 使用 iterator 序列化。

涉及文件：

- `crates/rspack_sources/src/rope/replacement.rs`
- `crates/rspack_sources/src/replace_source.rs`
- `crates/rspack_sources/src/cacheable.rs`

验证：

- differential 比较旧 Vec 和新 tree 的顺序与输出；
- 覆盖 ascending / descending / random / monotonic runs；
- 覆盖 overlapping、同 range、Pre/Normal/Post 和 EOF 之外 replacement；
- 比较 source map full-column 和 line-only；
- 对现有 6k ascending fixture 设置明确的回退阈值。

完成标准：

- 所有 replacement 语义等价；
- random/descending 显著改善；
- ascending case 未超过约定回退预算；
- 没有按读取次数重复 materialize 排序 Vec。

## Phase 6：将 replacement 规范化为 Rope 节点

目标：完成 `OriginalSlice + Replacement(content)` 输出图，逐步移除 compatibility leaf。

工作项：

1. 实现 mapping-aware `OriginalSlice`：
   - UTF-8 byte range 校验；
   - 跨 child rope span 切片；
   - UTF-16 generated column 修正；
   - source/name index remap；
   - line-only 专用路径。
2. 把顺序 edit iterator normalize 成：
   - 未修改 `OriginalSlice`；
   - `Replacement` wrapper；
   - `Text` / `ChildSource` / `Placeholder` content。
3. 与当前 `replacement_end` overlap 状态机逐例比较。
4. freeze 后按中序 compact arena并重建 occurrence index，评估 traversal locality。

涉及文件：

- `crates/rspack_sources/src/rope/stream_chunks.rs`
- `crates/rspack_sources/src/rope/replacement.rs`
- `crates/rspack_sources/src/replace_source.rs`
- `crates/rspack_sources/src/helpers.rs`
- `crates/rspack_sources/src/with_utf16.rs`

完成标准：

- `ReplaceSource` 可以作为 Rope builder 的兼容 facade；
- 不 flatten inner Source；
- source map differential 全部通过；
- replacement 与 placeholder 可以组合而无需 marker 字符串。

## Phase 7：迁移 SRI 与流式 hash

目标：利用结构化 placeholder 和 Rope traversal 消除 SRI 的扫描与最终字符串物化。

工作项：

1. 把 SRI placeholder 改为 typed key；
2. 明确 chunk 间 SRI dependency graph 和 resolve 顺序；
3. 检测 placeholder cycle；
4. 使用 `to_writer` 风格 traversal 直接更新 integrity hasher；
5. 只在最终 asset API 需要时物化字符串。

涉及文件：

- `crates/rspack_plugin_sri/src/asset.rs`
- `crates/rspack_plugin_sri/src/runtime.rs`
- `crates/rspack_plugin_sri/src/util.rs`

完成标准：

- 不使用 regex 查找 SRI placeholder；
- 不为 integrity hash 调用完整 `source()`；
- chunk dependency cycle 有明确诊断；
- hot update warning 和 asset info 行为保持兼容。

## Phase 8：Source 组合层统一

目标：根据前面 benchmark 决定替换范围，不预设必须删除所有旧类型。

候选迁移：

1. `RawStringSource` 可直接成为 Text 单叶子 fast path；
2. `ConcatSource` 内部改用 Rope Branch/bulk build；
3. `ReplaceSource` 改为 Rope facade；
4. `OriginalSource` / `SourceMapSource` 长期保留为 `ChildSource`；
5. `RawBufferSource` 保持 binary 专用实现；
6. `CachedSource` 保持 freeze 后缓存层。

决策门槛：

- 只有 memory、CPU、source-map 和 cache benchmark 综合改善才移除旧实现；
- 单叶子和小 concat 明显回退时保留 specialized representation；
- 不为“类型数量更少”牺牲 binary correctness 或 mapping streaming。

## Phase 9：Persistent cache、文档与清理

工作项：

1. 定义 canonical logical node serialization；
2. 确保 NodeId、AVL topology、capacity 和 occurrence index 不入 cache identity；
3. 反序列化时 bulk-build 平衡 Rope；
4. 增加 cache version/tag，区分旧 Source、unresolved template 和 frozen Rope；
5. 更新 `.agents/RSPACK_SOURCES.md`；
6. 删除已迁移 marker regex、marker 常量和无用 code-generation flags；
7. 保留兼容 facade，按 deprecation/内部迁移节奏清理。

涉及文件：

- `crates/rspack_sources/src/cacheable.rs`
- `crates/rspack_sources/src/rope/cacheable.rs`
- `.agents/RSPACK_SOURCES.md`

完成标准：

- 相同逻辑 Rope 的不同平衡形态得到相同 hash 和 cache representation；
- persistent cache restore 后 graph identity、output 和 map 一致；
- 文档覆盖 ownership、freeze、placeholder 和 source-map 不变量。

## 全程验证矩阵

每个涉及 Rust 行为的阶段至少执行与修改范围相称的验证：

```text
Rust implementation change
       ↓
pnpm run build:binding:dev
       ↓
targeted rspack_sources / plugin tests
       ↓
pnpm run test:rs
       ↓
pnpm run lint:rs + cargo lint
       ↓
targeted benchmark comparison
```

如果变更跨 JS/TS，再改用 `pnpm run build:cli:dev` 并补 `pnpm run test:unit`。按项目约束跳过
storage 和 native watcher 测试，除非任务明确要求。

重点 correctness 维度：

- ASCII / BMP / astral Unicode；
- UTF-8 replacement boundary；
- UTF-16 source-map column；
- 空内容和尾随换行；
- overlapping replacement；
- 相同 offset 的 Pre/Normal/Post；
- placeholder 重复出现、未解析、冲突和 cycle；
- line-only / column source map；
- hash/equality/cache roundtrip；
- binary Source 不进入 text Rope。

## Benchmark 决策指标

至少报告：

- build/edit insertion time；
- freeze/normalization time；
- first `source()` time；
- repeated `source()` through `CachedSource`；
- `size()`；
- `to_writer()`；
- full-column 和 line-only map；
- allocations 和 allocated bytes；
- peak retained bytes；
- arena node count、tree height 和 bytes/node；
- placeholder scan bytes eliminated；
- ascending workload regression；
- random/descending workload improvement。

最终是否默认启用 Rope，必须以真实 Rspack compilation workload 为主要依据，synthetic reverse/random
只用于识别复杂度边界。

## 回滚策略

- Phase 1～4 作为新增类型和少量 placeholder 迁移，可逐路径回退到 marker + `ReplaceSource`；
- Phase 5 保留旧 Vec 容器作为 differential/reference implementation，直到 benchmark 和正确性稳定；
- Phase 6 通过 compatibility facade 隔离，不要求调用方一次性迁移；
- Source 类型删除只在 Phase 8 单独进行；
- persistent cache format 切换放在最后，避免前期原型锁定格式。
