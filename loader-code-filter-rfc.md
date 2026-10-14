# RFC: Loader code filter

| _**Changelog**_         | <br>                                                                             |
| ----------------------- | -------------------------------------------------------------------------------- |
| _**Implementation PR**_ | [web-infra-dev/rspack#16073](https://github.com/web-infra-dev/rspack/pull/16073) |

# Summary

为 `module.rules[].use` 的对象形式增加实验性的 `codeFilter` 配置，使 Rspack 能在 loader 的 normal handler 执行前，根据该位置的当前输入代码判断是否需要调用 loader。匹配值沿用 Rolldown Hook Filter 的 `code` 匹配语义；Rspack 的 rule 已提供路径等静态条件，因此字段直接表明它只过滤当前输入代码。

```ts
module: {
  rules: [
    {
      test: /\.[jt]sx?$/,
      use: [
        {
          loader: require.resolve('./transform-loader'),
          options: { mode: 'server' },
          codeFilter: /createMiddleware/,
        },
      ],
    },
  ],
}
```

`codeFilter` 只过滤 normal handler，不过滤 pitch。它是 loader normal handler 的执行前置条件：用户必须保证在 filter miss 时，该 handler 原本也应当是无可观察副作用的 no-op。基于这一约束，filter 不参与 loader options、loader request、`NormalModule` identifier 或 loader cache etag。

该设计的目标是把 transform 中常见的源码正则预检查前移到 Rspack 调度层，在不改变资源读取、pitch、loader 链和缓存身份的前提下，减少无效的 Rust → JavaScript 调用及 worker dispatch。

# Motivation

框架编译器、宏转换和代码注入类 loader 通常只能通过较宽的文件条件筛选候选模块，例如 `test: /\.[jt]sx?$/`。真正需要转换的文件往往只占其中一小部分，因此 handler 内通常还有一次源码检查：

```js
module.exports = function transformLoader(source) {
  if (!/\bcreateServerFn\b/.test(source)) {
    return source;
  }

  return expensiveTransform(source);
};
```

当前检查发生在 JavaScript handler 内。即使最终什么都不做，Rspack 仍可能需要：

1. 从 Rust loader runner yield 到 JavaScript；
2. 进入 JavaScript loader batch；
3. 查询 loader cache；
4. 调用 handler，或向 worker 分发任务；
5. 将未变化的内容和执行状态同步回 Rust。

在大型项目中，“候选文件多、实际命中文件少”是常见场景。例如 TanStack Start 的编译器需要扫描 JavaScript 和 TypeScript 文件，但通常只有包含 server function、middleware 等标记的模块才需要进入编译器。将源码预检查移动到调度层，可以让稳定态 miss 在进入目标 normal runtime 前结束。

本 RFC 的设计目标如下：

- 过滤当前 loader 执行点的输入，而不是磁盘上的原始源码；
- 保留 webpack-compatible pitch 顺序和短路行为；
- miss 时不调用 normal handler，不查询或写入该 loader 的 normal cache，也不 dispatch worker；
- 不增加真实或伪 builtin loader，不改变用户可观察的 loader 链；
- 不改变未使用 `codeFilter` 的模块身份、缓存和热路径行为；
- 不把 filter 混入 `options`，避免 loader 通过 `getOptions()` 观察到 Rspack 调度元数据；
- 静态 matcher 只编译一次，并尽可能在 Rust 中零拷贝匹配；
- 为 Rsbuild `api.transform` 提供可直接使用的底层能力。

预期收益主要来自减少跨 runtime 调用和无效 transform 调度，而不是正则匹配本身。具体收益依赖命中率、handler 成本、是否使用 worker 以及 loader 链组成，必须通过 benchmark 验证，RFC 不预设固定百分比。

# Prior arts

## Rolldown 和 Vite hook filters

Rolldown 的 Hook Filters 允许 `transform` 根据 `id`、`moduleType` 和 `code` 在 Rust 侧预过滤，从而避免不必要的 Rust → JavaScript hook 调用。Vite 6.3+ 和 Rollup 4.38+ 也支持相同的 object hook 形式。

```js
transform: {
  filter: {
    code: {
      include: ['createServerFn', /createMiddleware/],
    },
  },
  handler(code, id) {
    return transform(code, id)
  },
}
```

本 RFC 沿用 Rolldown `code` 条件的匹配语义：`codeFilter` 使用 `GeneralHookFilter<string | RegExp>` 风格的直接值、数组或 `include`/`exclude` 对象；数组表示 OR、`exclude` 优先、缺少 `include` 时默认包含。差异在于 Rspack loader 还存在独立的 pitch 阶段，因此 `codeFilter` 必须明确限定为 normal-only。

## webpack 和 Rspack loader

webpack loader 和 Rspack loader 都包含两个方向相反的阶段：pitch 从左到右执行，normal 从右到左执行。pitch 可以通过返回结果短路剩余 pitch、资源读取和右侧 loader 的 normal 执行。

loader 导出的 `raw`、`pitch` 等能力属于 loader 本身，但 Rspack 在读取源码前必须先加载 JavaScript loader，才能发现这些导出。code filter 与 `raw` 不同：它依赖当前输入内容，只有进入 normal 阶段后才能可靠判断。

## esbuild callback filters

esbuild 要求插件在注册 `onResolve`、`onLoad` 等 callback 时提供正则 filter，并在进入 JavaScript callback 前由原生侧完成路径匹配。这证明了“在调度层预过滤 callback”能够显著减少插件边界开销，但 esbuild 的 filter 主要匹配 path/namespace，并不等价于本 RFC 的当前源码匹配。

# Glossary

- **Rule matching**：NormalModuleFactory 根据 `test`、`resource`、`issuer` 等不依赖当前源码的条件选择 loader 的阶段。
- **Pitch**：loader 从左到右执行的前置阶段；可以写入 `this.data`，也可以返回内容并短路后续流程。
- **Normal handler**：loader 默认导出函数，从右到左执行，接收右侧 loader 的输出。
- **当前输入（current input）**：某个 normal handler 即将执行时的 `content`，可能是 `readResource` 的结果，也可能已经被其右侧 loader 转换。
- **Code filter**：根据当前输入文本判断是否进入 normal handler 的执行前置条件。
- **No-op**：不改变 content、source map、additional data、cacheability、dependencies、diagnostics、emitted assets、`buildInfo` 或任何其他可观察状态。
- **Runtime boundary**：Rust、Node.js 主线程和 loader worker 之间的调用或数据同步边界。

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

type RuleSetLoaderWithOptions = {
  loader: RuleSetLoader;
  options?: RuleSetLoaderOptions;
  ident?: string;
  parallel?: boolean | { maxWorkers?: number };
  cache?: boolean;

  /** Filters the normal handler using its current input code. @experimental */
  codeFilter?: RuleSetLoaderCodeFilter;
};
```

示例：

```js
export default {
  module: {
    rules: [
      {
        test: /\.[cm]?[jt]sx?$/,
        use: [
          {
            loader: require.resolve('./server-function-loader.js'),
            codeFilter: {
              include: [/\bcreateServerFn\b/, /\bcreateMiddleware\b/],
              exclude: '/* generated */',
            },
          },
        ],
      },
    ],
  },
};
```

以下规则均针对 `codeFilter`：

- string 使用子串匹配，不是 glob；
- `RegExp` 的匹配不依赖调用前的 `lastIndex`，每次都从 `lastIndex = 0` 开始；
- 数组中任意一项匹配即视为匹配；
- `exclude` 优先于 `include`；
- 未提供 `include` 时默认包含；
- `include: []` 永不匹配，`exclude: []` 不排除任何内容；
- `codeFilter: {}` 等价于匹配所有有效文本输入；
- 空字符串是有效输入；空 pattern 字符串按 JavaScript `String.prototype.includes('')` 的语义匹配；
- string content 和有效 UTF-8 Buffer 可以参与匹配；content 不存在或 Buffer 不是有效 UTF-8 时视为 miss；
- v1 不接受函数，也不支持 `and`、`or`、`not` 组合表达式。

`codeFilter` 检查的是 normal handler 执行点的当前输入：

```js
use: [
  {
    loader: './a-loader.js',
    codeFilter: 'generated-by-b',
  },
  './b-loader.js',
];
```

normal 阶段从右到左执行，因此 `a-loader` 的 code filter 能看到 `b-loader` 的输出。

## 与 pitch 共存

配置 `codeFilter` 不代表 loader 没有 pitch，也不禁止 loader 导出 pitch。pitch 始终按现有语义执行：

```js
module.exports = function normal(source) {
  return transform(source);
};

module.exports.pitch = function pitch(remainingRequest, previousRequest, data) {
  data.startedAt = Date.now();
};
```

上述 loader 的 pitch 对所有匹配 rule 的模块执行；只有 normal handler 受 `codeFilter` 控制。如果 pitch 返回内容并触发短路，则继续沿用现有 loader runner 行为，不额外执行 code filter。

## 正确性约束和兼容回退

`codeFilter` 是性能 prefilter，不应被用来为同一个 loader request 创建两套不同的转换语义。loader 作者或配置作者必须保证：当 filter miss 时，跳过 normal handler 与调用 handler 后直接透传完全等价。

为兼容 webpack、旧版 Rspack 或其他不识别该字段的运行环境，loader 可以保留 handler 内检查：

```js
const codeFilter = /\bcreateServerFn\b/;

module.exports = function loader(source) {
  codeFilter.lastIndex = 0;
  if (!codeFilter.test(source)) return source;
  return transform(source);
};
```

该检查在支持 `codeFilter` 的 Rspack 中只会对命中模块执行；在不支持该配置的环境中仍保证正确性。

如果 handler 在 miss 路径仍会增加 dependencies、发出文件、修改 `buildInfo`、记录 diagnostics 或产生其他副作用，则不能使用 code filter，除非这些行为被移到 pitch 或另一个不受过滤的 loader 中。

## Rsbuild API

Rsbuild 的 `api.transform` 使用 `code` 表示当前 transform 的源码过滤条件；映射到 Rspack loader 配置时使用 `codeFilter`：

```ts
type TransformDescriptor = {
  test?: RuleSetCondition;
  /** @experimental */
  code?: TransformCodeFilter;

  // existing fields...
};
```

```js
api.transform(
  {
    test: /\.[cm]?[jt]sx?$/,
    code: {
      include: [/\bcreateServerFn\b/, /\bcreateMiddleware\b/],
    },
    order: 'pre',
  },
  ({ code }) => transform(code),
);
```

Rsbuild 仅在 `code` 存在时将其原样映射为 Rspack loader use item 的 `codeFilter`；未配置时生成结果保持不变。`raw: true` 仍决定 handler 接收 Buffer，但 filter 只对有效 UTF-8 文本视图进行匹配。

# Detailed design

## API 所属层级

code filter 位于具体的 `Rule.use` item，而不是与 `Rule.test` 同级，原因如下：

1. rule matching 发生在读取资源之前，此时没有源码可匹配；
2. 提前读取资源会改变 `readResource` hook、虚拟模块和 pitch 短路的时序；
3. 同一个 rule 中不同 loader 看到的输入不同，rule 级 code 条件无法表达“当前 loader 位置”；
4. filter 是特定 normal handler 的执行前置条件，而不是整个 module rule 是否成立的条件。

字段采用 `codeFilter`，保留 Rolldown `HookFilter.code` 的匹配值语义，但不照搬其两层结构：

- `codeFilter` 明确指出匹配对象是 normal handler 的当前输入代码，而不是 resource path、loader options 或 pitch；它由 Rspack 消费，不传给 loader `options`；
- code condition 仍支持直接 pattern、pattern 数组和 `include`/`exclude` 三种写法，与 Rolldown `GeneralHookFilter` 一致；
- 不额外提供 `filter: { code: ... }` 别名，以免同一语义存在两套配置结构；
- 不使用 `when.code`，因为 `when` 容易让用户误以为 pitch 和 normal 都受条件控制；
- 不使用 loader use item 顶层的裸 `code`，避免与 loader 输入或 options 混淆。

Rolldown 的 `id` 和 `moduleType` 不直接照搬：resource、resourceQuery、issuer、mimetype 等静态信息已经由 `module.rules` 在资源读取前筛选；将它们加入 loader filter 会重复现有能力，并产生“是否同时过滤 pitch”的阶段歧义。如果未来出现只能在 normal 调度点获得、且有充分实际场景的新维度，应通过独立 RFC 设计明确的字段和组合语义。`codeFilter: {}` 可用，表示匹配所有有效文本输入。

Rolldown 与 Rspack 的职责映射如下：

| Rolldown transform filter | Rspack 对应能力                                              | Rspack loader filter 是否重复提供 |
| ------------------------- | ------------------------------------------------------------ | --------------------------------- |
| `id`                      | `module.rules` 的 `test`、`resource`、`resourceQuery` 等条件 | 否                                |
| `moduleType`              | 在 loader runner 前由 module rule 和 module factory 决定     | 否                                |
| `code`                    | 某个 normal handler 执行点的当前输入                         | 是，使用 `codeFilter`             |

## 执行时序

未发生 pitch 短路时，完整时序为：

```text
NormalModuleFactory 选择并解析 loader
  → pitch（从左到右，行为不变）
  → readResource / readResource hook（行为不变）
  → normal（从右到左）
      → 判断当前 loader 的 codeFilter
          → miss：标记 normal 已完成，移动到左侧 loader
          → hit：loader cache lookup → normal handler / worker dispatch
```

filter 必须在以下操作之前判断：

- normal loader cache lookup；
- normal handler 的 `loadLoader`；
- Rust → JavaScript normal dispatch；
- main thread → worker normal dispatch。

对于通用 JavaScript loader，Rspack 仍需要在 pitch 阶段加载模块以发现其是否导出 pitch。因此，本 RFC 保证 miss 不进入 **normal** runtime，但不保证第一次构建时完全不加载 JavaScript loader 模块。Rspack 已有的 “loader 无 pitch” 信息可以继续用于后续模块的 pitch 调度优化；本 RFC 不新增“默认无 pitch”的假设。

pitch 短路保持现有行为：如果某个 pitch 返回 content，loader runner 直接转入该 loader 左侧的 normal 链。被现有 pitch 语义跳过的 normal handler 不需要再判断 filter。

## 每次 loader 使用的执行元数据

实现上不注册 `builtin:code-filter-loader`，不包装目标 loader，也不向用户链中插入第二个 loader。NormalModuleFactory 解析目标 loader 后，在 `ResolvedLoader` 中分别保存真实目标 loader、loader cache options 和已编译 code filter。`NormalModule` 继续保存原有的 `Vec<BoxLoader>`；仅当至少一个 loader 配有 code filter 时，另外保存按 loader 索引排列的可选表：

```rust
type LoaderCodeFilters = Box<[Option<Arc<CodeFilter>>]>;
```

这避免了为未启用 loader cache 的模块构造一整组 `LoaderRunnerOptions`。无 code filter 的模块不分配此表；静态 rule 的已编译 matcher 通过 `Arc` 在模块间共享。进入 loader runner 时，按照索引将 filter 移入对应 `LoaderItem`；执行状态 `NotEvaluated`、`Matched` 或 `Skipped` 也由该 item 保存。filter 不属于 `Loader` trait，目标 loader 的 `identifier()`、`pitch()`、`run()`、`type()`、`cache_version()` 和 `execution_kind()` 均保持原样。

因此 `this.loaders`、`this.request`、`currentRequest`、`remainingRequest`、`previousRequest` 和 `getOptions()` 均继续展示真实目标 loader。实际匹配逻辑集中在 `rspack_loader_runner::loader`：`LoaderItem` 暴露一个 normal precondition 检查；`runner.rs` 的 normal loop 只调用该检查，并在 `Skipped` 时设置 `normalExecuted` 后继续。

这一检查必须位于 `start_yielding` 之前，否则当前 loader 是 JavaScript loader 时仍会先跨越 Rust → JavaScript 边界。它也必须位于 `LoaderRunnerPlugin::run_normal_loader` 之前，否则 miss 仍会进入 loader cache。

## 配置适配和 binding

TypeScript config adapter 将 `codeFilter` 规范化为结构化 binding 类型，RegExp 通过现有 `RspackRegex` N-API 能力传递，不能使用 JSON 序列化。

概念上的 Rust 类型如下：

```rust
enum CodeFilterPattern {
  String(String),
  RegExp(RspackRegex),
}

struct CodeFilter {
  include: Option<Vec<CodeFilterPattern>>,
  exclude: Vec<CodeFilterPattern>,
}
```

`RawModuleRuleUse`、`ModuleRuleUseLoader` 和 `ResolvedLoader` 分别携带原始、已规范化和已编译的 filter。filter 独立于仅在 `cache: true` 时保留的 `LoaderRunnerOptions`；即使 `cache: false`，可选表仍会传到 loader runner。

静态 rule 的 matcher 在配置转换或 loader resolve 后编译一次，并通过 `Arc` 在模块间共享。函数形式的 `Rule.use` 也允许返回 `codeFilter`；v1 对每次动态返回的 loader item 编译 matcher，不引入无界的跨模块 matcher cache。

配置错误应携带完整 option path，例如 `module.rules[2].use[0].codeFilter.include[1]`。函数、逻辑条件对象或其他不支持的值应在配置阶段报错，而不是静默忽略。

## Rust matcher

matcher 不复制当前源码：

- `Content::String` 直接借用 `&str`；
- `Content::Buffer` 先使用 SIMD UTF-8 校验，成功后借用 `&str`，失败直接 miss；
- 单个 string pattern 使用 `memchr::memmem`；
- 同一 include 或 exclude 中存在多个 string pattern 时，使用预编译的 Aho-Corasick matcher；
- RegExp 在配置阶段编译为 `RspackRegex` 并跨模块复用。

执行顺序固定为：

1. 检查 exclude；任一命中立即返回 miss；
2. include 不存在时返回 hit；
3. include 存在时，任一 pattern 命中返回 hit；
4. 否则返回 miss。

当 filter 同时包含 string 和 RegExp 时，string matcher 和 RegExp matcher 分别预编译；短路顺序不构成公共 API，用户不能依赖哪个 pattern 先执行。

## JavaScript batch 和 worker

Rspack 的一次 JavaScript yield 可以连续执行多个 JS loader。仅在 Rust normal loop 检查当前 loader 不足以覆盖 batch 中后续 loader，因此 binding 还需要向 JavaScript loader item 同步：

- 规范化后的 code filter 引用；
- filter 是否已由 Rust 评估；
- normal 是否已经跳过。

主线程和 worker 共用同一个 TypeScript matcher。JavaScript normal loop 在 cache lookup、`loadLoader` 和 worker dispatch 前检查 filter。Rust 已评估的 loader 不重复扫描；JavaScript 评估结果在 batch 返回时同步回 Rust。

同一份 content 在没有被 loader 更新时最多创建一次 UTF-8 文本视图。任何 normal handler 执行结束后，不论返回值是否与输入为同一对象，都使该文本视图失效，以兼容 loader 原地修改 Buffer 的情况。

filter miss 不应强制结束当前 JavaScript batch。这样连续 JS loader 中间出现 miss 时，只在 JavaScript 内移动 `loaderIndex`，不会增加 JS → Rust → JS 往返。

## 状态保持

filter miss 等价于该 normal handler 未产生任何结果：

- content 保持不变；
- source map 保持不变且不触发惰性解析；
- additional data 保持不变；
- cacheable 状态保持不变；
- dependencies 不 reset、不 merge，也不新增；
- pitch 写入的 `this.data` 保留；
- loader context request 字段和 loader 数组保持不变；
- 不触发 normal handler 的 diagnostics、logging、asset emission 或 `buildInfo` 写入。

miss 后将该 item 的 `normalExecuted` 设置为 `true`，确保主线程、worker 和 Rust 恢复执行时不会再次判断或调用。

## Module identity 和缓存

filter 被定义为 normal handler 的 precondition assertion，而不是会改变转换结果的 loader option。因此：

- 不修改 `Loader::identifier()`；
- 不向 request 添加 query、fragment、NUL suffix 或 digest；
- 不修改 `NormalModule::create_id`；
- 不把 filter 放入 `getOptions()`；
- 不把 filter fingerprint 放入 loader cache etag。

这依赖前文的正确性约束：所有合法配置都必须保证 miss 时执行 handler 与跳过 handler 可观察等价。若配置违反这一约束，模块去重、memory cache 或 persistent cache 可以复用另一条路径的结果，Rspack 不保证输出正确。

loader cache 只保存“目标 loader 在给定 content、options、version 和 dependencies 下的输出”。filter hit 后可以复用现有条目；filter miss 不查询也不写入条目。由于 filter 不改变 target 的输出函数，同一条 cache entry 可以被不同但等价的 prefilter 复用。

源码发生变化时，现有 snapshot 和 module invalidation 负责触发模块重建；重建的 normal 阶段对新 content 重新判断 filter。cache 与 incremental 仍是两条独立机制，本功能不得让其中一个隐式启用或禁用另一个。

## 性能特征

无 filter 的热路径只增加一次可预测的空 `Option` 检查。filter 的主要成本是一次文本扫描；主要收益是避免目标 normal runtime。

需要增加两类 benchmark：

1. matcher microbenchmark：单 string、多 string、RegExp、混合 include/exclude、ASCII 和 Unicode、不同源码大小；
2. loader integration benchmark：0%、10%、100% 命中率，分别覆盖主线程 JS loader、parallel loader、native loader 和混合链。

验收标准：

- 无 filter 的 benchmark 无可辨识回退；
- miss 不进入目标 normal handler、normal loader cache 或 worker dispatch；
- 连续 JS loader 中的 miss 不增加 Rust/JS 往返；
- 100% 命中不因 filter 引入额外 runtime boundary；
- 性能结论使用同一机器、同一构建模式的 A/B 数据，不用单次构建时间代替 benchmark。

## 测试计划

### 匹配语义

- string、RegExp、数组和混合 pattern；
- include、exclude、exclude 优先；
- 缺少 include、空对象、`include: []`、`exclude: []`；
- 空 pattern、空 content、Unicode；
- 有效 UTF-8 Buffer 和非法 UTF-8 Buffer；
- global/sticky RegExp 以及非零 `lastIndex`；
- 非法配置的 option path 和错误信息。

### Loader 语义

- hit 调用 normal，miss 不调用 normal；
- hit 和 miss 都保留 pitch；
- pitch 短路与现有行为一致；
- 左侧 filter 能看到右侧 loader 的输出；
- miss 保持 source map、additional data、dependencies、cacheability 和 pitch data；
- `this.loaders`、request 系列字段和 `getOptions()` 不包含 code filter 元数据；
- raw loader 命中时仍接收 Buffer；
- inline loader request 不获得配置 filter，但与 rule loader 混用时顺序正确。

### Runtime 和缓存

- 主线程 JS loader；
- parallel worker loader；
- native builtin loader；
- JS/native 混合链和 JS batch 中间 miss；
- loader cache hit/miss，并确认 filter miss 不 lookup/store；
- memory cache、persistent cache、incremental 的独立组合；
- watch rebuild 中新增或删除 marker；
- 同一资源由不同 issuer 命中等价 prefilter 时模块身份保持稳定；
- 函数形式 `Rule.use` 返回带 `codeFilter` 的 loader。

### Rsbuild 和真实消费方

- `api.transform({ code })` 只调用命中 handler；
- `raw`、`order`、`environment` 和其他 descriptor 条件保持现有行为；
- 不配置 `codeFilter` 时生成的 Rspack rule 不变；
- TanStack Start 普通源码 miss；
- 右侧 transform 生成虚拟源码后，编译器 transform 的 code filter 能命中；
- watch 和 persistent cache 下 server function metadata 保持正确。

## 文档和发布

- API 标记为 `@experimental`，不增加全局 experiments 开关；
- 同步更新 Rspack 中英文 `module.rules` 文档和 Rsbuild 中英文 `api.transform` 文档；
- 文档必须说明 normal-only、当前输入、UTF-8、pitch 和 no-op assertion；
- v1 不支持 inline loader request 表达 filter；
- v1 只返回是否执行，不暴露 matched pattern 或 pattern index。

# Drawbacks

- 这是 Rspack 扩展，不属于 webpack 配置规范；直接把相同配置交给 webpack 可能报未知字段或忽略该字段，loader 仍需保留兼容检查。
- “只过滤 normal、不过滤 pitch”需要用户理解 loader 的双阶段模型，存在学习成本。
- 对通用 JavaScript loader，第一次 pitch capability discovery 仍可能加载 loader 模块，因此 miss 不能消除所有 loader loading 成本。
- Rust、JavaScript 主线程和 worker 需要保持 matcher 语义及执行状态一致，实现和测试复杂度高于单纯在 handler 内检查。
- 同一 loader 可能同时保留配置 filter 和 handler 内 fallback，存在 pattern 漂移风险；框架应从同一个 pattern 定义生成两者。
- filter 不参与 module identity 的正确性依赖 no-op assertion。Rspack 无法自动证明 handler 在 miss 路径没有依赖、资产或其他副作用。
- RegExp 扫描仍有 CPU 成本；对高命中率或极轻量 handler，filter 可能没有收益，甚至略有回退。
- 未来若增加其他过滤维度，必须重新评估它们在 pitch、normal、缓存和 identity 中的语义，不能假设它们可以直接复用 `codeFilter`。

# Unresolved questions & TODOs

- 用真实大型项目 benchmark 确认 0%、10%、100% 命中率下的收益和 break-even point。
- 评估是否需要未来的 loader manifest 或显式 `pitch: false` 元数据，以便在首次构建时也跳过无 pitch JS loader 的加载；该能力不属于 v1。
- 确定动态 `Rule.use` matcher cache 的容量和淘汰策略，防止用户按资源生成无限多个 pattern。
- 根据真实动态 `Rule.use` 使用情况，评估后续是否值得增加有界 matcher cache；v1 直接编译动态返回的 filter，避免引入隐藏的常驻缓存。
- Changelog 和 implementation PR 创建后回填本文顶部链接。

# Alternatives

## `Rule.testCode`

```js
{
  test: /\.js$/,
  testCode: /createServerFn/,
  use: ['./transform-loader'],
}
```

不采用。rule matching 时源码尚未读取；提前读取会改变 pitch 短路、`readResource` hook 和虚拟模块时序。此外 rule 级条件无法看到 loader 链中间结果，也无法只过滤某一个 loader。

## `builtin:code-filter-loader`

在目标 loader 前插入 builtin wrapper 可以复用现有 loader 链模型，但它会成为真实 loader item，改变 module identifier、request 字段、pitch/normal 索引和用户对 `this.loaders` 的观察。为了隐藏 wrapper 又需要 identity request 与 execution request 双轨模型，复杂度和兼容风险更高。

## `rspack.experiments.withCodeFilter()` helper

helper 可以隐藏内部协议，但用户需要学习额外工厂 API，函数形式 `Rule.use`、配置合并工具和 Rsbuild chain API 也需要特殊适配。`codeFilter` 直接属于 loader use item，组合性更好。

## Loader 导出 `codeFilter`

```js
export const codeFilter = /createServerFn/;
```

这种形式把 filter 与 loader 实现绑定，适合不可配置的固有 precondition，也能降低配置与 handler pattern 漂移的概率。但 Rspack 必须先加载 JS loader 才能读取导出，无法优化首次模块加载和 pitch discovery；同一个 loader 的不同实例也无法按 options 或环境配置不同 pattern。后续可以单独 RFC 讨论 loader manifest，但不替代本 RFC 的 per-use 配置。

## 利用 pitch 短路

由 pitch 读取源码并在 miss 时返回透传结果会绕过标准 `readResource` 时序，并可能跳过右侧 loader、source map 和 additional data 处理。它还要求每个 loader 自己重新实现资源读取，不能正确表达“匹配右侧 loader 的输出”。因此不采用。

## 只在 handler 内检查

这是当前兼容方案，语义最简单，但无法避免 Rust/JS、worker 和 handler 调用开销。它应继续作为跨 bundler fallback，而不是 Rspack 的性能实现。

## 将 filter 纳入 module identifier

可以为 filter 生成稳定 fingerprint 并追加到 module identity，从而允许 filter 成为一般的语义条件。但这会让不改变转换结果的 prefilter 产生不同 module，降低去重率，并改变 stats、cache key 和调试输出。本 RFC 选择更严格的 no-op assertion，以换取完全稳定的 loader request 和 module identity。若未来需要一般性的条件转换，应设计独立 API，而不是放宽本 RFC 的 prefilter 契约。

## 直接使用 `filter` 简写

```js
{
  loader: "./transform-loader.js",
  filter: /foo/,
}
```

该形式更短，但无法从字段名直接看出 pattern 匹配的是 resource 还是当前代码；`codeFilter` 保持简短，同时明确了匹配对象。

## 使用 `filter.code`

```js
{
  loader: './transform-loader.js',
  filter: { code: /foo/ },
}
```

它与 Rolldown/Vite Hook Filter 结构更接近，也为其他维度预留了命名空间。但 Rspack 已通过 module rule 提供多数静态维度，v1 只有当前代码这一项。额外嵌套会增加配置和类型复杂度，并暗示未来可以直接添加其他维度；本 RFC 改用语义明确的 `codeFilter`。

## Loader use item 顶层使用 `code`

```js
{
  loader: "./transform-loader.js",
  code: /foo/,
}
```

这种形式虽然简短，但 `code` 脱离过滤上下文后容易被理解为输入值或 loader option，因此不采用。

# Refs

- [Rolldown `HookFilter` interface](https://rolldown.rs/reference/Interface.HookFilter)
- [Rolldown `GeneralHookFilter` type](https://rolldown.rs/reference/TypeAlias.GeneralHookFilter)
- [Rolldown Plugin Hook Filters](https://rolldown.rs/apis/plugin-api/hook-filters)
- [Vite Plugin API - Hook Filters](https://vite.dev/guide/api-plugin#hook-filters)
- [webpack Loader Interface - Pitching Loader](https://webpack.js.org/api/loaders/#pitching-loader)
- [webpack Loader Interface - Raw Loader](https://webpack.js.org/api/loaders/#raw-loader)
- [Rspack - Writing loaders](https://rspack.rs/api/loader-api/writing-loaders)
- [Rspack - Loader](https://rspack.rs/guide/features/loader)
- [esbuild Plugin API - Filters](https://esbuild.github.io/plugins/#filters)

# Feedback

重点希望获得以下反馈：

1. 是否接受将 `codeFilter` 定义为 no-op precondition assertion，并因此保持 module identity 和 cache etag 不变；
2. normal-only 语义是否足够清晰，是否需要在类型名或 diagnostics 中进一步强调 pitch 不受影响；
3. Rspack loader use item 的 `codeFilter` 与 Rsbuild `api.transform` 的 `code` 命名差异是否足够清晰；
4. v1 是否需要支持 loader export 形式，还是应留给后续 loader manifest RFC；
5. benchmark 和真实框架接入还需要覆盖哪些典型 loader 链。
