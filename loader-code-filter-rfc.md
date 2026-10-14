# RFC: Loader code filter

| _**Changelog**_         | <br>                                                                             |
| ----------------------- | -------------------------------------------------------------------------------- |
| _**Implementation PR**_ | [web-infra-dev/rspack#16073](https://github.com/web-infra-dev/rspack/pull/16073) |

# Summary

为对象形式的 `module.rules[].use[]` 增加实验性 `filter.code`。Rspack 在某个 loader 的 normal handler 即将处理当前输入时，先按源码内容判断是否需要执行该 handler：

```js
{
  test: /\.[jt]sx?$/,
  use: [
    {
      loader: './transform-loader.js',
      filter: { code: /createMiddleware/ },
    },
  ],
}
```

`filter.code` 是性能预过滤条件，只控制 normal，不控制 pitch；它不进入 loader options、loader request、模块 identifier 或 loader cache key。配置作者必须保证：过滤未命中时，跳过 normal 与执行 normal 后原样透传在所有可观察行为上等价。

# Motivation

框架编译器、宏转换等 loader 往往用较宽的文件规则筛选模块，再在 handler 中检查特定源码标记。大多数候选模块不需要转换，却仍会进入 JavaScript loader runtime，甚至查询 loader cache 或分发到 worker。把这项检查前移到调度点，可以在未命中时省去目标 normal handler 的调用及相关跨 runtime 调度。

文件名等静态条件继续由 `module.rules` 处理。源码条件不能在 rule matching 阶段判断：此时尚未执行 pitch、`readResource`，也没有右侧 loader 的输出。提前读取资源会改变虚拟模块、资源 hook 和 pitch 短路的时序。`filter.code` 因此属于某一次 loader 使用，而非整个 rule。

本 RFC 的目标是提供这个调度能力，同时保持不使用它时的现有 loader 链、模块身份和执行语义。性能收益取决于命中率、handler 成本及跨线程成本；不能仅由 matcher 的速度推断端到端构建收益。

# Prior arts

- [Rolldown HookFilter](https://rolldown.rs/reference/Interface.HookFilter) 为 transform hook 提供 `code`、`id`、`moduleType` 等预过滤维度；其 [GeneralHookFilter](https://rolldown.rs/reference/TypeAlias.GeneralHookFilter) 接受单个值、数组或 `include`/`exclude` 对象。Rspack 借用 `code` 条件的值形状，但不照搬整个 hook filter 对象：静态路径筛选已经由 rule 完成。
- [Vite Hook Filters](https://vite.dev/guide/api-plugin#hook-filters) 将预过滤用于减少 Rust 与 JavaScript 间不必要的 hook 调用，并建议在需要兼容旧版本时保留 handler 内的检查。Rspack loader 可以采用相同的兼容回退。
- [esbuild callback filters](https://esbuild.github.io/plugins/#filters) 在进入 JavaScript callback 前原生匹配路径，说明调度层过滤的价值；它匹配的是 path/namespace，不是当前源码。
- [webpack pitching loader](https://webpack.js.org/api/loaders/#pitching-loader) 的 pitch 从左到右、normal 从右到左，且 pitch 可以短路后续流程。Rspack 必须保留这一双阶段语义。

# Glossary

- **当前输入**：目标 normal handler 执行点的 content，可能是 `readResource` 的结果，也可能是右侧 loader 转换后的结果。
- **未命中（miss）**：`filter.code` 对当前输入返回 false；Rspack 跳过该次 normal handler。
- **no-op 契约**：在 miss 时，执行 handler 应与直接透传完全等价，包括 content、source map、additional data、cacheability、dependencies、diagnostics、emitted assets、`buildInfo` 及其他可观察状态。

# User guide

## Rspack 配置

```ts
type RuleSetLoaderCodePattern = string | RegExp;

type RuleSetLoaderCodeFilter =
  | RuleSetLoaderCodePattern
  | RuleSetLoaderCodePattern[]
  | {
      include?: RuleSetLoaderCodePattern | RuleSetLoaderCodePattern[];
      exclude?: RuleSetLoaderCodePattern | RuleSetLoaderCodePattern[];
    };

type RuleSetLoader = string;
type RuleSetLoaderOptions = string | Record<string, any>;

type RuleSetLoaderWithOptions = {
  loader: RuleSetLoader;
  options?: RuleSetLoaderOptions;
  ident?: string;
  parallel?: boolean | { maxWorkers?: number };
  cache?: boolean;
  /** @experimental; only controls the normal handler. */
  filter?: {
    code: RuleSetLoaderCodeFilter;
  };
};
```

单个 pattern 可写 `filter: { code: /createMiddleware/ }`。有多个标记或排除条件时：

```js
export default {
  module: {
    rules: [
      {
        test: /\.[cm]?[jt]sx?$/,
        use: [
          {
            loader: './transform-loader.js',
            options: { mode: 'server' },
            filter: {
              code: {
                include: ['createServerFn', /\bcreateMiddleware\b/],
                exclude: /generated code/,
              },
            },
          },
        ],
      },
    ],
  },
};
```

`filter` 不会传给 `getOptions()`。若需要兼容 webpack 或不支持该字段的 Rspack 版本，loader handler 仍应保留同一条件的 no-op 检查；只在配置中写 `filter.code` 不能为这些环境提供正确性。

## 匹配规则

- string 是区分大小写的子串匹配，不是 glob。
- `RegExp` 每次从 `lastIndex = 0` 开始匹配；`g`/`y` 等有状态 flag 不使结果依赖上一次调用。
- 数组为 OR；`exclude` 任一命中即 miss，优先于 `include`。
- 未提供 `include` 时默认包含；`include: []` 永不命中，`exclude: []` 不排除任何输入。`filter: { code: {} }` 匹配所有有效文本输入。
- 空字符串 content 是有效输入，空 string pattern 能匹配它。content 不存在或 Buffer 不是有效 UTF-8 时一律 miss；有效 UTF-8 Buffer 按文本匹配。
- 不接受函数、`and`/`or`/`not`，也不支持 inline loader request 表达过滤条件。

`filter.code` 检查当前输入而非磁盘原文。例如 `use: [{ loader: './a.js', filter: { code: 'generated' } }, './b.js']` 中，`a.js` 可以匹配到 `b.js` 的 normal 输出。

# Detailed design

## 阶段与 pitch

正常路径的时序为：

```text
NormalModuleFactory 选择、解析 loader
  → pitch（从左到右）
  → readResource / readResource hook
  → normal（从右到左）
      → 对当前输入计算该 loader 的 filter.code
          → miss：标记 normal 已完成，继续左侧 loader
          → hit：原有 loader cache / native / JS / worker 路径
```

pitch 无条件沿用原有行为。配置 `filter.code` 不表示 loader 没有 pitch，也不禁止导出 pitch。若某个 pitch 返回内容，资源读取及其右侧 loader 按现有规则跳过；随后左侧 loader 的 normal 仍可对 pitch 返回的内容计算各自的 filter。不会为已被 pitch 短路跳过的 loader 额外执行 filter。

通用 JS loader 在 pitch 阶段可能必须先加载模块，才能知道它是否导出 pitch。因此“miss 不进入目标 normal runtime”不等于“首次 miss 一定不加载 loader 模块”。

## 配置、binding 与元数据所有权

TypeScript config adapter 校验 `filter` 仅包含必需的 `code`，再把单个 pattern、数组、`include`/`exclude` 规范化为结构化 `RawRuleSetLoaderCodeFilter`。它以 `use.filter !== undefined` 判断是否配置，并保留有效的空 string pattern。缺少 `code`、非法值和未知对象字段在配置阶段报包含完整路径的错误，如 `module.rules[2].use[0].filter.code.include[1]`。

`RegExp` 通过 N-API 的 `RspackRegex` 结构化传递，不经过 JSON 序列化。`RawModuleRuleUse.code_filter` 转为 `ModuleRuleUseLoader.code_filter: Option<Arc<CodeFilter>>`；静态 rule 在配置转换时编译 matcher，并在模块间共享。函数式 `Rule.use` 返回的 use item 走相同转换，但每次动态返回会重新编译，不引入无界的全局 matcher cache。

解析 loader 后，`ResolvedLoader` 分别持有真实目标 loader、loader cache options 和 filter。`NormalModule` 仍保存原有的 `Vec<BoxLoader>`；仅当至少一个 loader 有 filter 时，额外保存与 loader 顺序对齐的：

```rust
type LoaderCodeFilters = Box<[Option<Arc<CodeFilter>>]>;
```

这份元数据独立于 `LoaderRunnerOptions`：后者在未启用 loader cache 时可能整组省略，不能承载 filter。`run_loaders` 按索引将 matcher 附到对应的 `LoaderItem`，不增加第二个 loader，也不通过 `Loader` trait 从 decorator 获取 filter。无 filter 的模块不分配上述数组；normal 调度只检查空 `Option`。

## 匹配与跨 runtime 状态

Rust matcher 直接借用 `Content::String`；Buffer 使用 UTF-8 校验后借用 `&str`，非法编码立即 miss。单个非空 string pattern 使用 `memchr::memmem`，多个 string pattern 使用预编译 Aho-Corasick；空 pattern 为恒匹配。`RegExp` 复用 `RspackRegex`。匹配顺序为先排除，再检查包含。

`LoaderItem` 保存 matcher 与 `NotEvaluated`、`Matched`、`Skipped` 状态。Rust normal loop 在 `start_yielding` 与 `LoaderRunnerPlugin::run_normal_loader` 之前判断，因此 miss 不触发 normal loader cache lookup 或 Rust → JS normal dispatch。

一次 JS yield 可以在同一 batch 中执行多个 loader。主线程 runner 与 worker runner 共用 TypeScript matcher，并在各自的 cache lookup、`loadLoader` 和 worker dispatch 之前判断后续 loader；中间的 miss 只移动 loader index，不强制返回 Rust。Rust 已计算的结果不会在 JS 中重复扫描，JS/worker 的结果会同步回 Rust。JS 对同一 content 视图最多做一次严格 UTF-8 解码，content 改变后失效该视图。

miss 只将该 loader 标记为 normal 已完成；不修改 content、source map、additional data、dependencies、cacheability 或 pitch data。hit 后完全进入原有执行路径，是否实际调用 handler 仍取决于 loader cache 等既有机制。

## 模块身份、缓存与正确性

`filter.code` 是 normal handler 的 no-op precondition assertion，不是用于生成另一种结果的语义开关。因此它不进入 `getOptions()`、目标 loader request、`NormalModule::create_id` 或 loader cache etag。`this.loaders`、`this.request`、`remainingRequest` 和 `previousRequest` 均继续反映真实 loader 链；未使用 `filter` 的配置不变。

这是有意选择的正确性约束：如果某个 handler 在 miss 时仍会增加依赖、发出资产、记录 diagnostics、修改 `buildInfo` 或做出其他可观察改变，那么不能直接为它配置 `filter.code`。否则同一模块在不同 issuer/rule 下去重，或 memory/persistent cache 复用时，结果可能不正确。应先把这些行为移到始终执行的阶段，或放弃预过滤。

filter hit 可沿用目标 loader 的既有 cache 条目；miss 不查询或写入该 loader 的 normal cache。`CodeFilter` 序列化 pattern 并在恢复时重建预编译 matcher，使持久缓存恢复后仍有执行元数据。源码变更由既有 snapshot/invalidation 触发重建，watch rebuild 对新输入重新判断；`cache` 和 `incremental` 的开关彼此独立。

## 验证与性能证据

现有 config cases 覆盖 pattern、空值、Unicode、有效/无效 UTF-8、右侧 loader 输出、pitch、native/JS 混合链及元数据透传；parallel case 覆盖 worker batch；watch 与 compiler cases 覆盖 marker 改变和 legacy/new cache 的持久恢复。英文和中文 `module.rules` 文档同步标记为 experimental。

现有 `xtask/benchmark/benches/groups/loader_code_filter.rs` 仅测 matcher，在 100 个模拟模块上覆盖 0%、10%、100% 命中率及 string/RegExp。它不能证明构建吞吐收益，也不是改动前后的 A/B 结果。仍需增加端到端 miss-heavy loader benchmark，并在相同机器、构建模式和 runner 上比较无 filter、低命中率及高命中率场景。

# Drawbacks

- 这是 Rspack 扩展而非 webpack 配置字段；跨 bundler 使用仍需 handler 内回退检查。
- no-op 契约无法由 Rspack 自动证明，配置错误可能影响依赖、资产或缓存正确性。
- pitch 能力发现可能仍加载 JS loader；对极轻量或高命中率 handler，扫描本身可能抵消收益。
- `NormalModule` 多一个可选字段，启用 filter 时还需保存按 loader 对齐的 matcher 引用；Rust、主线程 JS 与 worker 要维持一致语义。
- filter 不能用于一般的条件转换：若 miss 时 handler 本应改变输出，需要另一种带独立模块身份和缓存语义的 API。

# Unresolved questions & TODOs

- 用端到端 benchmark 测量不同命中率、源码大小、handler 成本、JS/worker/native 链的 break-even point；不把现有 matcher microbenchmark 当成整体收益。
- 评估首次 pitch capability discovery 的成本是否值得通过独立的 loader manifest 等机制优化；v1 不假设“有 filter 的 loader 没有 pitch”。
- 若动态 `Rule.use` 在真实项目中产生大量重复 matcher，再评估有界共享缓存；v1 不增加常驻 cache。
- `filter` 对象为未来的其他维度保留命名空间，但任何新维度都需重新评估取值阶段、pitch 语义、模块身份和缓存，而非仅增加一个字段。

# Alternatives

- **`Rule.testCode`：** rule matching 时尚无当前源码；提前读取会改变 pitch 与 `readResource` 时序，也无法表达“某个 loader 看到右侧转换结果”。
- **builtin wrapper 或透明 `CodeFilteredLoader` decorator：** 前者会改变可见链和 request；后者虽可透明转发，但让执行元数据藏在 `Loader` trait 后，并给无 filter 的调度引入额外 trait 方法调用。直接放入 `LoaderItem` 更符合每次使用的所有权。
- **`codeFilter: ...`：** 少一层嵌套，但把过滤维度写进属性名；未来若增加其他执行阶段的过滤维度，会形成多个平级属性。`filter.code` 更贴近 Rolldown 的结构，并明确 code 是当前唯一支持的维度；文档仍需强调 filter 只控制 normal。
- **loader 导出 `codeFilter`：** 适合不可配置的固有条件，但必须先加载 JS loader 才能读取导出，同一个 loader 也难以按 options 或环境配置不同条件。
- **用 pitch 短路或只在 handler 内检查：** 前者会改变资源读取和右侧 loader 执行，无法匹配当前 normal 输入；后者是必要的兼容回退，却不能避免无效的 normal 调度。
- **将 filter 纳入模块 identifier：** 可把 filter 变成一般语义条件，但会降低模块去重率并改变缓存与 stats 身份。本 RFC 选择更严格的 no-op 契约，保持原身份。

# Refs

- [Rolldown HookFilter](https://rolldown.rs/reference/Interface.HookFilter)
- [Rolldown GeneralHookFilter](https://rolldown.rs/reference/TypeAlias.GeneralHookFilter)
- [Vite Plugin API: Hook Filters](https://vite.dev/guide/api-plugin#hook-filters)
- [webpack Loader Interface: Pitching Loader](https://webpack.js.org/api/loaders/#pitching-loader)
- [esbuild Plugins: Filters](https://esbuild.github.io/plugins/#filters)
- [Rspack module.rules[].use.filter.code](website/docs/en/config/module-rules.mdx#rulesusefiltercode)

# Feedback

1. 对 `filter.code` 采用 no-op 契约、保持模块身份与 loader cache key 不变，约束是否足够明确？
2. normal-only 与 pitch 短路后的行为，是否还需要更醒目的用户提示？
3. 端到端 benchmark 应优先覆盖哪些真实 loader 链和命中率？
