# Arena Rope Source 设计

状态：Proposal

## 1. 背景

当前 `rspack_sources` 使用多种 `Source` 组成不可变 Source 图：

- `RawStringSource` / `RawBufferSource` 保存叶子内容；
- `OriginalSource` / `SourceMapSource` 提供 source map；
- `ConcatSource` 保存有序子 Source；
- `ReplaceSource` 在 `Vec<Replacement>` 中保存基于原始 UTF-8 byte offset 的编辑；
- `CachedSource` 在 Source 图稳定后缓存 size、rope chunks、hash 和 source map。

`ReplaceSource` 对按 offset 递增产生的 replacement 已有 O(1) append 快路径，但乱序编辑需要
二分查找后执行 `Vec::insert`，会移动后续元素。更重要的是，auto public path、static URL、
worker URL 和 SRI 等功能把 placeholder 编码成特殊字符串，后续需要物化并扫描完整 Source，
还存在用户源码与魔法字符串冲突的可能。

本设计引入基于 `Vec<Node>` arena、使用下标连接的平衡二叉 Rope，并把 replacement 和
placeholder 表示成类型化节点。目标不是只替换 `Vec<Replacement>`，而是建立一层结构化的
Source IR，使组合、编辑、placeholder resolve 和最终流式输出共享同一种表示。

## 2. 目标

1. Placeholder 是类型化节点，不以特殊字符串出现在源码内容中。
2. 通过稳定 ID O(1) 查找和解析 placeholder，不再扫描完整 Source。
3. 乱序 replacement 插入不移动已有节点，复杂度从 O(R) 降到 O(log R)。
4. 保留有序遍历和流式 `rope()` / `to_writer()` / source-map streaming。
5. 在合理范围内统一 `RawStringSource`、`ConcatSource` 和 `ReplaceSource` 的内部表示。
6. 保持现有 UTF-8 byte offset、UTF-16 source-map column、hash、equality 和 persistent cache
   语义。
7. 小 Source 和严格递增 replacement 不应出现不可接受的性能回退。

## 3. 非目标

- 不实现 CRDT 或多方并发编辑。
- 第一阶段不移除 `OriginalSource`、`SourceMapSource`、`RawBufferSource` 或 `CachedSource`。
- 不允许 unresolved placeholder 通过现有 `Source` API 泄漏到 emit、hash 或 source map 阶段。
- 不在第一阶段修改第三方 JS hook，使其理解结构化 placeholder。
- 不因为树的平衡形态改变 canonical hash 或 equality。

## 4. 总体执行流

```text
Parser / dependency templates
  基于原始源码 byte range 产生 Text / Replacement / Placeholder
       ↓
TemplateRopeBuilder
  Vec<Node> arena + 平衡二叉树
  NodeId 使用 NonZeroU32
       ↓
Edit normalization
  保留 (start, end, enforce, insertion_order) 语义
  处理相同位置及重叠 replacement
       ↓
Placeholder resolver
  PlaceholderKey -> PlaceholderId -> resolved value
       ↓
freeze
  检查无 unresolved placeholder
  生成 canonical output rope 和节点 summary
       ↓
RopeSource
  Source::rope / size / map / hash / to_writer
       ↓
CachedSource
  在稳定图边界缓存
```

构建态和冻结态必须是不同类型。只有冻结态实现 `Source`。

## 5. Node ID 与 arena

### 5.1 NodeId

节点 ID 必须使用 `NonZeroU32`：

```rust
use std::num::NonZeroU32;

#[repr(transparent)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct NodeId(NonZeroU32);

impl NodeId {
  fn from_index(index: usize) -> Option<Self> {
    let index = u32::try_from(index).ok()?;
    let raw = index.checked_add(1)?;
    Some(Self(NonZeroU32::new(raw)?))
  }

  fn index(self) -> usize {
    (self.0.get() - 1) as usize
  }
}
```

约束：

- `NodeId(1)` 对应 `nodes[0]`；
- `Option<NodeId>` 利用 niche optimization，预期与 `u32` 同为 4 bytes；
- arena 最多保存 `u32::MAX` 个节点；
- builder 生命周期内 NodeId 不复用，避免 stale ID；
- 如果 freeze 时 compact arena，必须返回 old-to-new remap，并重建所有 NodeId 加速索引；
- NodeId 是单个 arena 内的地址，不得跨 arena 使用或进入 canonical persistent identity。

### 5.2 Arena

```rust
struct RopeArena {
  nodes: Vec<Node>,
}

struct TemplateRopeBuilder {
  arena: RopeArena,
  root: Option<NodeId>,
  placeholders: PlaceholderTable,
  next_insertion_order: u32,
}
```

新增节点始终追加到 `nodes`。树旋转只修改节点间的 NodeId，不移动 `Node`。第一版使用 AVL；
它不依赖随机 priority，构造结果更容易测试和调试。如果 benchmark 证明旋转成本显著，再评估
weight-balanced tree 或块状叶子。

## 6. 节点模型

目标节点模型：

```rust
enum Node {
  Branch(BranchNode),
  Text(TextNode),
  ChildSource(ChildSourceNode),
  OriginalSlice(OriginalSliceNode),
  Replacement(ReplacementNode),
  Placeholder(PlaceholderNode),
}

struct BranchNode {
  left: NodeId,
  right: NodeId,
  height: u8,
  summary: NodeSummary,
}

struct ReplacementNode {
  original_start: u32,
  original_end: u32,
  enforce: ReplacementEnforce,
  insertion_order: u32,
  name: Option<Cow<'static, str>>,
  content: NodeId,
  summary: NodeSummary,
}

struct PlaceholderNode {
  id: PlaceholderId,
}

struct TextNode {
  value: Cow<'static, str>,
  summary: NodeSummary,
}

struct ChildSourceNode {
  source: BoxSource,
  summary: NodeSummary,
}

struct OriginalSliceNode {
  source: BoxSource,
  start: u32,
  end: u32,
  summary: NodeSummary,
}
```

节点分工：

- `Branch` 表示 concat，不拥有文本；
- `Text` 保存生成文本；
- `ChildSource` 暂时复用现有 `BoxSource`，使 `OriginalSource` 和 `SourceMapSource` 可以作为
  mapping-aware 叶子；
- `OriginalSlice` 表示原始 Source 的一个 byte range；
- `Replacement` 是带原始 range、顺序和 name 元数据的一元包装节点，输出其 `content`；
- `Placeholder` 保存紧凑 `PlaceholderId`，不保存可与源码冲突的 marker 字符串。

`Replacement` 和 `Placeholder` 必须在 Debug、hash、persistent cache 中保留类型身份，不能过早
退化为普通字符串。

只有 `Branch` 组成平衡二叉 Rope 的有序骨架；`Replacement` 是一元包装节点，其他节点是叶子。
因此底层严格说是 arena source graph，而不是所有节点都具有左右孩子的传统二叉树。第一版禁止
结构循环，且从 output root 出发同一个结构节点只能出现一次；共享值通过 placeholder slot 或
`Arc<BoxSource>` 表达，不通过多个父节点共享同一个可变 NodeId。

### 6.1 OriginalSlice

将 `ReplaceSource` 完全规范化成输出 Rope 时，需要把未替换的原始内容表示成
`OriginalSlice { source, start, end }`。这要求增加 mapping-aware slice streaming，不能简单调用
`source()` 后做字符串切片，否则会丢失 source map。

第一阶段可以不生成 `OriginalSlice`：保留现有 `ReplaceSource` 作为 compatibility leaf，只先验证
Rope concat 和 placeholder。只有 mapping-aware slice 正确后，才把 replacement 完全降为 Rope
节点。

## 7. Placeholder 模型

### 7.1 稳定语义 ID

Placeholder 查找不应直接依赖 NodeId。树旋转不会改变 NodeId，但删除、compact、clone 和同一
placeholder 多次出现都会让 `PlaceholderKey -> NodeId` 难以维护。

使用两级索引：

```rust
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
enum PlaceholderKey {
  AutoPublicPath,
  StaticUrl(DependencyId),
  WorkerUrl(DependencyId),
  Sri(ChunkUkey),
  AssetFilename(ModuleIdentifier),
}

#[repr(transparent)]
struct PlaceholderId(NonZeroU32);

struct PlaceholderTable {
  ids: FxHashMap<PlaceholderKey, PlaceholderId>,
  slots: Vec<PlaceholderSlot>,
  occurrences: Vec<SmallVec<[NodeId; 1]>>,
}

struct PlaceholderSlot {
  key: PlaceholderKey,
  value: Option<ResolvedPlaceholder>,
}
```

执行流：

```text
PlaceholderKey
  通过 FxHashMap 注册/查找
       ↓
PlaceholderId
  直接索引 slots[id - 1]
       ↓
ResolvedPlaceholder
  Text 或 BoxSource
```

`occurrences` 只用于按 ID 找到所有节点并做增量 summary 更新；它是可重建加速索引，不参与
canonical hash/equality。若 resolve 集中发生在 freeze 前，可以不增量更新祖先，而是在所有 slot
填充后一次自底向上计算 summary。

### 7.2 Resolve 规则

- 一个 PlaceholderId 可以出现多次，所有出现位置共享 resolved value；
- resolve 同一个 ID 为不同值必须报错；
- freeze 时存在 unresolved slot 必须报错，并输出 placeholder kind 和关联实体；
- placeholder value 可以是 text 或已冻结 `BoxSource`，不能包含指向当前 builder 的循环引用；
- resolver 不允许通过扫描 Source 文本发现 placeholder；
- 用户源码中的任意字符串都只能形成 `Text`/`ChildSource`，不会形成 `Placeholder`。

### 7.3 生命周期

```rust
struct TemplateRopeSource {
  // 允许 unresolved placeholder，不实现 Source
}

struct RopeSource {
  // 所有 placeholder 已解析，结构不可变，实现 Source
}
```

`TemplateRopeSource::freeze()` 负责 resolve 校验、normalization、summary 计算和索引清理。
`CachedSource` 只能包装 `RopeSource`，不能包装构建态。

## 8. Replacement 语义

现有 replacement 使用原始 inner Source 的 UTF-8 byte offset，范围是半开区间 `[start, end)`。
后续 replacement 不受之前 replacement 输出长度影响。Rope 不能把每次编辑按“当前输出 offset”
立即应用，否则会改变现有语义。

构建阶段的排序键保持：

```text
(start, end, enforce, insertion_order)
```

其中 `enforce` 顺序为 `Pre < Normal < Post`。

执行流：

```text
dependency template 调用 insert/replace
  保存原始 byte range + content NodeId
       ↓
Edit tree
  按现有 Replacement key 保持有序
       ↓
normalize
  处理相同位置、重叠范围和超出 EOF 的 replacement
       ↓
Output rope
  OriginalSlice 与 Replacement 节点交替
```

`insert(start, content)` 仍等价于 `replace(start, start, content)`。相同 range 的输出顺序必须与
现有实现一致。迁移前应把当前 overlapping、enforce 和 EOF 行为整理成兼容性用例。

### 8.1 为什么仍需 normalization

Edit tree 解决的是乱序插入时的节点搬移，不直接解决重叠编辑。当前 `ReplaceSource::rope()` 和
source-map streaming 在顺序消费 replacement 时维护 `replacement_end`。新实现必须复用同一
状态机或给出逐例等价证明。

### 8.2 有序 append 快路径

已知大型 fixture 中约 6,116 个 replacement 严格递增，现有 `Vec` 全部走 O(1) push。新实现必须
保留类似快路径：

- builder 记录最大 key；
- 新 key 不小于最大 key 时，使用 right-spine append/bulk build；
- 大批有序节点优先构建平衡子树后与现有 root merge，而不是逐节点执行 AVL 插入；
- freeze 后的中序布局可选 compact 为 traversal-friendly arena。

否则虽然随机插入变快，主流有序输入可能退化。

## 9. NodeSummary

冻结态每个节点缓存：

```rust
struct NodeSummary {
  bytes: usize,
  generated_line: u32,
  generated_column: u32,
  is_ascii: bool,
}
```

语义与当前 `GeneratedInfo` 一致：空内容结束于 line 1 / column 0，column 使用 UTF-16 code
units。Branch summary 由左右节点组合。

构建态 placeholder 长度可能未知，使用独立的 `BuildSummary`：

```rust
struct BuildSummary {
  bytes: Option<usize>,
  generated: Option<GeneratedInfo>,
  is_ascii: Option<bool>,
}
```

不要用虚构 marker 的长度临时填充 summary。freeze 后不得存在 unknown。

## 10. Source API 与 source map

`RopeSource` 实现现有 `Source`：

- `source()`：按 summary 精确分配一次，顺序遍历叶子；
- `rope()`：按顺序借用 Text 和 ChildSource spans；
- `buffer()`：文本模式转 bytes；binary Source 保留独立路径；
- `size()`：读取 root summary，O(1)；
- `to_writer()`：顺序遍历并直接写入；
- `map()` / `stream_chunks()`：组合 child-local mapping 并平移 generated position；
- `update_hash()`：hash canonical logical nodes，不 hash arena 下标和树平衡形态。

Replacement content 的 mapping repair 必须等价于当前 `ReplaceSourceChunks`。`OriginalSlice`
必须按 UTF-8 byte boundary 切内容，并在 source map 层正确处理 UTF-16 column。

禁止为了实现 Rope source map 而：

- 先 flatten 全部文本；
- 收集全部 mapping 再统一转换；
- 为每个 mapping clone source/name；
- 先生成 full-column map 再丢弃 column 实现 line-only map。

## 11. Source 类型统一边界

### 11.1 第一阶段可以统一

| 现有类型 | Rope 表示 |
| --- | --- |
| `RawStringSource` | `Text` |
| `ConcatSource` | `Branch` |
| Placeholder marker | `Placeholder` |
| 已规范化 replacement content | `Replacement(content)` |

### 11.2 第一阶段保留为叶子或包装层

| 现有类型 | 原因 |
| --- | --- |
| `OriginalSource` | 拥有 token/line mapping 生成逻辑 |
| `SourceMapSource` | 拥有 outer/inner map 组合逻辑 |
| `RawBufferSource` | 必须无损保留任意二进制数据 |
| `CachedSource` | 是 freeze 后缓存策略，不是内容节点 |
| `ReplaceSource` | mapping-aware slice 完成前作为 compatibility leaf |

长期可以让 `OriginalSource` 和 `SourceMapSource` 作为 `ChildSource` 叶子长期存在；统一组合层并不
要求消灭所有专用叶子类型。

## 12. Hook 与兼容性边界

当前 `RenderSource` 只暴露 `BoxSource`。任意插件都可能调用 `source()`、修改字符串并返回新的
`RawSource`，这种操作无法保留 unresolved placeholder 元数据。

第一阶段采用保守边界：

```text
内部 codegen
  TemplateRopeSource
       ↓
内部 placeholder resolve + freeze
       ↓
BoxSource / RopeSource
       ↓
现有 render hooks 和第三方插件
```

需要跨多个内部 render hook 才能解析的 placeholder，可以把内部 hook 参数升级为结构化 builder，
但必须在进入用户可见 hook 前 freeze。不能让第三方插件看到 unresolved placeholder，除非未来
显式发布新的 hook API。

## 13. Hash、Equality 与 persistent cache

树旋转、插入顺序和 arena compact 不能改变 canonical identity。

Canonical hash 按逻辑节点序列计算：

```text
Text(type + bytes)
ChildSource(type + child structural hash)
Replacement(type + original range + enforce + insertion order + name + content)
Placeholder(type + key)                  // 构建态结构 hash
Resolved placeholder(type + final value) // 冻结态 hash
```

不参与 hash：

- NodeId；
- root/left/right；
- AVL height；
- arena capacity；
- occurrence index；
- 偶然的平衡形态。

Persistent cache 优先序列化 canonical logical sequence，而不是原样序列化平衡树。反序列化后可批量
构建平衡 Rope。构建态 cache 与冻结态 cache 必须使用不同 tag，不能把 unresolved structural hash
误当最终 content hash。

## 14. Ownership 与并发

- Builder 使用 `&mut self`，不需要内部锁；
- freeze 后 arena 和 placeholder value 不再改变，通过 `Arc` 共享；
- `RopeSource` 必须保持 `Send + Sync`；
- Text 借用必须由 Rope arena 或 ChildSource owner 保活；
- 如果缓存 `&'static str` view，必须像当前 `CachedSource` 一样明确 owner-retention 和 drop order；
- 不在单个 Rope stream 内引入 rayon/tokio；并行仍在 module/asset 层进行。

## 15. 性能模型与风险

### 15.1 预期收益

- placeholder 不再需要 `source()` 全量物化和 regex/match scan；
- 用户源码不会与 placeholder marker 冲突；
- placeholder resolve 通过 HashMap + Vec slot 完成；
- 随机 replacement 插入不移动已有元素；
- `size()` 可由 root summary O(1) 返回；
- emit 和 SRI hash 可以直接顺序流式处理；
- 多层 `ConcatSource` 可以压缩成一个 arena。

### 15.2 可能回退

- 小 Source 增加 Node tag、NodeId、height 和 summary 开销；
- 严格递增 replacement 从连续 `Vec::push` 变成树维护；
- 中序遍历会按 NodeId 跳转，cache locality 弱于排序 `Vec<Replacement>`；
- source-map slicing 和 replacement repair 实现复杂；
- persistent cache 格式和 hook 边界迁移范围较大；
- hash table 不应为完全没有 placeholder 的 Source 分配。

为减少小对象成本：

- 无 placeholder、无 concat、无 replacement 的单叶子 Source 不建立 Rope；
- PlaceholderTable 延迟分配；
- builder 对少量节点可使用 inline/small representation，超过阈值后升级 arena；
- 有序 replacement 使用 bulk-build 快路径；
- freeze 可按中序 compact，改善读阶段 locality。

## 16. 正确性不变量

1. `NodeId` 必须是有效的 `NonZeroU32`，且只属于一个 arena。
2. 所有 replacement offset 是原始 Source 的 UTF-8 byte offset 和有效字符边界。
3. source-map generated/original column 使用 UTF-16 code units。
4. freeze 后没有 unresolved placeholder。
5. `size() == buffer().len() == to_writer()` 写出长度。
6. `source()` 与 `rope()` 顺序拼接结果一致。
7. 同 range replacement 的 `Pre/Normal/Post/insertion_order` 与现有实现一致。
8. Tree topology、NodeId 和 compact 不影响 hash/equality。
9. Binary content 不经过 lossy text Rope。
10. `CachedSource` 只包装不可变、已解析的 Source 图。

## 17. 建议文件布局

```text
crates/rspack_sources/src/
  rope/
    mod.rs
    node_id.rs
    arena.rs
    builder.rs
    placeholder.rs
    summary.rs
    stream_chunks.rs
    cacheable.rs
  rope_source.rs
```

兼容和迁移相关文件：

- `crates/rspack_sources/src/source.rs`
- `crates/rspack_sources/src/replace_source.rs`
- `crates/rspack_sources/src/concat_source.rs`
- `crates/rspack_sources/src/cached_source.rs`
- `crates/rspack_sources/src/cacheable.rs`
- `crates/rspack_plugin_javascript/src/runtime.rs`
- `crates/rspack_plugin_javascript/src/plugin/url_plugin.rs`
- `crates/rspack_plugin_css/src/plugin/impl_plugin_for_css_plugin.rs`
- `crates/rspack_plugin_sri/src/asset.rs`
- `xtask/benchmark/benches/groups/rspack_sources*.rs`

## 18. 决策摘要

- 采用 `Vec<Node>` arena 和基于下标的平衡二叉 Rope；
- `NodeId` 使用 `NonZeroU32`，0 留给 `Option<NodeId>::None`；
- Placeholder 是结构化节点，使用稳定 `PlaceholderId` 和 value table；
- `PlaceholderKey -> PlaceholderId` 是 canonical lookup，NodeId occurrence 是可重建加速索引；
- replacement 保持原始 offset 语义，经过 normalization 后形成输出 Rope；
- unresolved builder 不实现 `Source`，freeze 后的 `RopeSource` 才实现；
- 第一阶段统一字符串、concat 和 placeholder，mapping-aware Source 继续作为叶子；
- 是否完全替代 `ReplaceSource`，由 source-map 正确性和 benchmark 结果决定。
