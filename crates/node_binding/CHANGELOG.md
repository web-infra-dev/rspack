# @rspack/binding

## 0.1.11

### Patch Changes

- f609f7a4: fix(resolve): relative path resolve bug
- ada51e2: fix: circular dependencies rebuild panic
- 2bb2bcd6: add extractComments option
- ff85ce8: feat(new_split_chunks): support `splitChunks.name`
- e764de62: add comments when module code generation
- c0e550b: bump napi cli
- 088220b3: feat(new_split_chunks): support splitChunks.minSize
- abf788a0: fix: should parse non-module file correctly
- 2a6a778: feat(new_split_chunks): support `maxInitialRequests` and `maxAsyncRequests` internally
- e5d33eb: feat(new_split_chunks): support `reuseExistingChunk`
- 04daf09: feat: add optimization.realContentHash
- 3abb2fc: feat: support inline match resource
- 2d9e5c2: feat(new_split_chunks): support `splitChunks.{cacheGroup}.minSize`
- f4c1f8d: refactor: external module for http request

## 0.1.9

### Patch Changes

- 820c029: perf: compose loaders on the native side
- 0fd6d7a: feat: support loader-runner and pitching
- bc65893: feat: add moduleAssets for stats
- 61d6e5d: feat: support inline loader
- aee4fdc: fix: react refresh runtime inject for nest function call
- c1720f9: add optimizeModules hook
- c951f35: feat: impl new loader-runner
- e2647bb: feat: require.resolve and require.resolveWeak for string literal
- 057829b: fix: source map lost when enable builtins.banner
- c162cb6: feat: support banner plugin
- b77b706: fix: undefined reasons with stats.toJson({ reasons: true })
- 7f2cf5e: feat: support `Rule.enforce`

## 0.1.8

### Patch Changes

- 352e563: fix(html): html inject public path incorrect
- 9822cef: fix templete literal in dynamic import
- 27afffc: fix: panic hook failed to restore after being suppressed
- fdc5fd9: fix: fix crash in multiCompiler
- d28a9d0: fix(react-refresh): lazily access module.exports to prevent unexpected/potential TDZ error
- aa91ce7: fix(progress): should not display empty progress bar

## 0.1.7

### Patch Changes

- 5bc1f55: fix Module Export Entry does not respect ordering
- 6f23105: feat(html): support templateContent in `builtins.html`
- c2bb73c: fix: fix typing of raw cross origin loading
- e649469: fix builtins.html does replace [hash] and [name] in builtins.html.filename
- fff64ea: react refresh with context
- dc10d1f: fix: draw progress bar to stdout instead of stderr
- 905cacf: feat: support module.rule[].descriptionData
- f4eb7c7: feat: support output.chunkLoadingGlobal
- 32f822b: add type and user_request field for module.reason

## 0.1.6

### Patch Changes

- 18dcca0: feat(rspack_plugin_dev_friendly_split_chunks): reduce MAX_MODULES_PER_CHUNK and add MAX_SIZE_PER_CHUNK condition #2578

## 0.1.5

### Patch Changes

- 7bfcc7b: fix windows path is considered scheme incorrectly
- 050e4fb: feat: wasm loading types
- e8db1d7: feat: module.rule[].dependency
- c82529b: feat: new url in target node
- 6c08098: fix: add more hash logic
- 60e0aec: fix require.cache undefined
- 33e916e: feat: built-in support for ProvidePlugin

## 0.1.4

### Patch Changes

- fbaeb41: fix: duplicate \_\_webpack_require\_\_.a in async module

## 0.1.3

### Patch Changes

- 4471853: fix: amd should return with iife
- 397b0d7: feat(runtime): support set **webpack_public_path** runtime
- b323220: add async-wasm & js-async-module support
- 28b9757: feat: resolve.byDependency
- b6ab7b7: fix: relay visitor should add unresolved mark to `require`
- b0cffba: feat: inline external type syntax
- 9c71512: add finishModules hook

## 0.1.2

### Patch Changes

- 68c4df8: feat: module.hot.accept/decline support arrary dependencies
- a43a849: fix: cheap-module-source-map too slow
- f562fa9: feat: add simple ignore options for copy-plugin
- 97dc0dd: fix regex external not working
- b4c5ed5: fix: css url rewrite for complex http url within ~
- 36ffd12: fix: Tailwind CSS HMR not work
- a6ac7da: feat: function type RuleSetCondition
- 48eff5e: feat: add normalModuleFactory and resolveForScheme hook
- 8f8e025: align optimizeChunkModules
- 7fde34f: fix: node mode interop
- 2c01586: fix: fix identifier of missing module
- 6d9eb8b: fix: contenthash should care about ChunkGraphModule#id
- 2e9d331: feat(runtime): support node.\_\_filename polyfill
- a54179d: fix(html): builtins.html minify not work
- fbc315d: add percent symbol in progress plugin message
- c4c20a3: add cross-env

## 0.1.1

### Patch Changes

- 6f8706f0: fix: postcss modules warning when using postcss-loader
- 0e32353d: fix(resolve): upgrade the version of nodejs_resolver to fix the error which will resolve failed when target is relative by tsconfig/baseUrl
- 23048514: fix: incremental rebuild not work
- 7b3167ca: feat: externalsPresets.node
- f204c108: feat: array type of externals
- bb22416a: feat: stats for chunkRelations and chunkModules
- 44dc1e8b: fix: fix react fast-refresh module id on windows
- a004765a: Avoid reporting warning the export when src module format is none esm
- 2f7ffb57: feat: support guided panic info with backtrace
- 2d2c9624: fix: fix webpack-sources interoperation
- 6bdc0840: pref(sourcemap): upgrade rspack_sources to improve the performance of line sourcemap'

## 0.1.0

### Minor Changes

- 2ba87f3a: chore: let's rspack

## 0.0.26

## 0.0.25

### Patch Changes

- 6d6e65e4: feat: update packages repository config

## 0.0.24

### Patch Changes

- 3495d3d72: feat: add stats.namedChunkGroups
- e2466248d: feat: add stats.assetsByChunkName
- a22149e98: fix: css modules ident-name leading digits
- 58b77bafc: feat: add plugin-import
- b101fd41e: fix: update swc_emotion to avoid single line comment inside template string break the styles

## 0.0.23

### Patch Changes

- b67418968: chore: 🤖 use module path instead of module id in diagnositc
- 962f8a251: fix: should create different module with different module rule
- 17bf167f6: Fix unstable order of generated files from chunks
- 766c94042: fix rust test
- 39f8a9c42: fix: wrong line number lead by CachedSource
- e64506a51: fix side effects pattern match
- 26e66549e: feat(crates/rspack): catch panics caused by spawned threads
- c98bf5768: feat: trace a symbol in symbol graph
- 035c15953: basic implementation of compilation.hooks.optimizeChunkModules
- 82aa28d6b: feat: add copy-plugin
- b694d4a58: feat: add backtrace if either napi stack or message is failed to extract
- c1f19b817: align webpack config optimization.sideEffects

## 0.0.22

### Patch Changes

- e402226e5: refactor: dependency code generation for JavaScript
- 59edc2cb4: fix watch options
- 549796acc: more webpack test case
- 51916f548: fix: recoverable error generation
- da069320e: Align more code with webpack for bundle splitting
- 8a4cb9a38: feat: more js api for webpack plugin compatibility
- cd7736377: bump swc core
- 97eaa8208: feat: Port \`findGraphRoots\` in Webpack.
- b5343d3f4: chore: remove top line breaks in css
- 10db0a2d8: Bump napi to fix memory error
- e845df7da: port RemoveEmptyChunksPlugin of Webpack
- cb93bb421: fix: not show module reasons for default stats toString
- faef6fc00: Should normalize SplitChunks options correctly
- 2f4db99e0: feat: processAssets hook stage
- 820b5a79b: fix: dead loop in rspack-sources when columns is false
- 5e722adf1: feat: add emit and afterEmit hooks
- 278e89cc1: fix: string type sideEffects
- b6e9a1b5e: Supports `optimization.removeAvailableModules`
- 0e1a42d46: fix: missing query symbol in contextify for requests with query
- 0269ff40d: fix: avoid **webpack_require**.m not defined
- 8dc513ac3: fix: make module type recoverable
- 58bef147b: fix(crates/rspack_core): should expect shutdown on some occasions
- 70586d79e: fix: `delete compilation.assets[filename]` should keep deleted asset info

## 0.0.21

### Patch Changes

- fix watch

## 0.0.20

### Patch Changes

- fix load extra css chunk

## 0.0.19

### Patch Changes

- db66ae2e: feat: add hash as return value of stats.toJson
- 882093b8: support module.resolve

## 0.0.18

### Patch Changes

- bump version

## 0.0.17

### Patch Changes

- upgrade

## 0.0.16

### Patch Changes

- support optional dependency

## 0.0.15

### Patch Changes

- bump version

## 0.0.14

### Patch Changes

- bump version
- 11e87c61: fix less resolve bug

## 0.0.13

### Patch Changes

- 3701a8bf: fix less resolve bug
- optional strategy

## 0.0.12

## 0.0.11

## 0.0.10

## 0.0.9

### Patch Changes

- 69becfa5: linux binary use release binary

## 0.0.8

### Patch Changes

- 589b99bb: bump to 0.0.8

## 0.0.6

### Patch Changes

- e6d0926a: unify version

## 0.0.5

### Patch Changes

- d466288: add process_assets hook

## 0.0.4

### Patch Changes

- 536f6f70: fix broken lib

## 0.0.2

### Patch Changes

- bd57e818: first release
