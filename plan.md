# `new URL` async entry 执行计划

> 语义说明：`agent/fix-css-entry-extra-js` 的实际行为是纯 CSS entry 不输出额外 JavaScript。本文按“纯 CSS async entry 与 configured entry 一致，没有真实 JavaScript 内容时不输出 `.js`”规划。

## 目标执行流

```text
JavaScript Parser
  解析静态 new URL(request, import.meta.url)
       ↓
创建 URLDependency
       ↓
放入 AsyncDependenciesBlock
       ↓
设置 GroupOptions::Entrypoint
  独立 runtime + async entry
       ↓
NormalModuleFactory
  按普通 module rules 决定 JS / CSS / WASM / Asset
       ↓
BuildChunkGraph
  建立 async entrypoint 和实际输出 chunk
       ↓
Code Generation
  根据目标模块的真实 SourceType 选择输出文件
       ↓
替换 new URL
  JS → js chunk
  CSS-only → css chunk
  WASM → wasm asset
  asset/resource → emitted asset
  asset/inline → data URL
       ↓
Render Manifest
  只输出真实存在的资源；纯 CSS entry 不生成额外 JS
```

## 1. 以 CSS-only 修复为基线

保留 `agent/fix-css-entry-extra-js` 中的两类修改：

- `crates/rspack_plugin_css/src/parser_and_generator/mod.rs` 根据真实 incoming connection 判断是否需要 `SourceType::JavaScript`。
- `crates/rspack_plugin_javascript/src/plugin/impl_plugin_for_js_plugin.rs` 不再因为 chunk 是 entry/runtime chunk 就无条件生成 JavaScript。

后续将 `NewUrl` async entry connection 视为和 configured `Entry` 一样的输出入口。

## 2. 取消 URL 对 asset 的实现假设

检查并调整 `packages/rspack/src/config/defaults.ts` 中的 `dependency: "url" -> asset/resource` 默认规则：

- 按需求移除通用 `asset/resource` fallback，让目标走普通 module rules。
- 可以保留 `data:` 对应的 `asset/inline` 特例。
- 原有依赖默认 asset 行为的测试补充显式 asset rule。

这是潜在 breaking change。如果仍需兼容旧行为，则保留默认规则作为配置层 fallback，但 URL parser、dependency 和 renderer 内部不得再假设目标一定是 asset。

## 3. 始终为静态本地 URL 创建 async entry

修改 `crates/rspack_plugin_javascript/src/parser_plugin/url_plugin.rs`：

- 静态可解析的本地 `new URL()` 创建 `URLDependency`。
- 将 dependency 放入 `AsyncDependenciesBlock`，不再直接放入普通 dependencies。
- 设置 `GroupOptions::Entrypoint(EntryOptions)`，使用稳定且唯一的 runtime 标识。
- 复用 worker async entry 的构建方式，避免新增一套 chunk graph 流程。
- `new URL(import.meta.url)`、`webpackIgnore`、hash-only URL、远程协议 URL 保持现状。
- 动态表达式继续使用 `NewUrlContext`；“总是创建 async entry”的范围限定为静态可解析请求。

## 4. 保持 inner graph、tree shaking 和 executeModule 正确

由于 dependency 从 module dependencies 移入 block：

- 将 inner graph 对 URL 的跟踪从数组下标改为 `DependencyId`，finalize 时同时扫描普通 dependencies 和 block dependencies。
- async entry 使用独立 runtime 时，正确计算 `used_by_exports`，避免未使用的 URL 留下孤立 entry。
- 在 module executor 中只跟进 block 内的 `NewUrl` dependency，使 loader `importModule` 仍能得到 URL codegen 和 emitted asset；不能同步展开 dynamic import 或 worker block。

涉及文件：

- `crates/rspack_plugin_javascript/src/parser_plugin/inner_graph/state.rs`
- `crates/rspack_plugin_javascript/src/parser_plugin/inner_graph/plugin.rs`
- `crates/rspack_plugin_javascript/src/visitors/dependency/parser/mod.rs`
- `crates/rspack_core/src/compilation/build_module_graph/module_executor/execute.rs`

## 5. 统一各模块类型的 entry source type

分别调整：

- CSS：`Entry` 和 `NewUrl` connection 都不因为被 JavaScript 模块引用而自动增加 JavaScript source；确实需要 JavaScript exports 的 CSS 模式继续保留 JavaScript。
- Asset：URL entry 直接使用 `CodeGenerationDataUrl` 或 `CodeGenerationDataFilename`，不生成仅用于导出 URL 的 JavaScript wrapper。
- WASM：只有 `Entry`/`NewUrl` connection 时输出 WASM，不生成额外 JavaScript glue；被正常 JavaScript import 时仍保留 glue。
- Mixed entry、library entry 和真正具有 JavaScript/runtime 内容的 entry 继续输出 JavaScript。

最终 configured CSS entry 与 async URL CSS entry 遵循同一判定：没有真实 JavaScript source 就没有 `.js` 文件。

主要涉及：

- `crates/rspack_plugin_css/src/parser_and_generator/mod.rs`
- `crates/rspack_plugin_asset/src/lib.rs`
- `crates/rspack_plugin_wasm/src/parser_and_generator.rs`
- `crates/rspack_plugin_javascript/src/plugin/impl_plugin_for_js_plugin.rs`

## 6. 按真实输出文件替换 URL

重构以下文件：

- `crates/rspack_plugin_javascript/src/dependency/url/mod.rs`
- `crates/rspack_plugin_javascript/src/plugin/url_plugin.rs`

实现方式：

- codegen 阶段写 dependency placeholder，因为此时 chunk id、hash 和最终 filename 还未完全确定。
- render 阶段通过 parent block 找到 async entrypoint，再按目标 source type 解析实际文件：
  - `CodeGenerationDataUrl`
  - `CodeGenerationDataFilename`
  - CSS chunk filename
  - JavaScript chunk filename
  - WASM filename
- 完整支持默认、`relative`、`new-url-relative` 三种 parser mode，以及 string/relative/auto `publicPath`。
- ESM library 等绕过普通 module render 的路径也调用同一个 placeholder resolver。
- 检查 splitChunks 场景，保证 URL 指向的 entry 文件能够独立使用，或者能够正确加载其依赖 chunk。

如果需要从任意 runtime 获取 runtime-independent 的 asset codegen metadata，在 `crates/rspack_core/src/artifacts/code_generation_results.rs` 增加安全的可选查询 API，避免 inactive async runtime 导致 panic。

## 7. 测试与验收

新增 `tests/rspack-test/configCases/url/async-entry-module-types`，覆盖：

- module graph 中存在 `AsyncDependenciesBlock` 和 async entrypoint。
- JavaScript、CSS、WASM、asset/resource、asset/inline target。
- CSS-only、WASM-only、asset-only 不产生额外 JavaScript。
- JavaScript target 产生可执行 JavaScript entry。
- 默认、`relative`、`new-url-relative` URL mode。
- 未使用 URL 的 tree shaking。
- ignored、远程 URL、动态 context。
- 多个 URL、相同 target、splitChunks、增量重建。
- loader `importModule`。

同时回归：

- `configCases/asset-modules/only-entry`
- `configCases/asset-modules/entry-with-runtimeChunk`
- `configCases/tree-shaking/new-url`
- `configCases/rstest/new-url-wasm`
- builtin new-url source/inline snapshots
- ESM output new-url snapshots
- CSS runtime 和 no-extra-runtime-in-js cases

建议验证命令：

```bash
pnpm run build:binding:dev

cd tests/rspack-test
pnpm run test -t "configCases/url/async-entry-module-types"
pnpm run test -t "configCases/asset-modules/only-entry"
pnpm run test -t "configCases/asset-modules/entry-with-runtimeChunk"
pnpm run test -t "configCases/tree-shaking/new-url"
pnpm run test -t "configCases/rstest/new-url-wasm"
```

最后运行 Rust tests、相关 unit tests 和格式检查。`codex/poc-unified-entry-rendering` 可以作为实现参考，但其中基于 `GET_CHUNK_SCRIPT_FILENAME` 或按模块类型零散补丁的部分应收敛成统一的“async entry 实际输出文件”解析逻辑。
