# @rspack/core

## 0.1.11

### Patch Changes

- 5d4333f: fix: rebuild loses error messages
- 2bb2bcd6: add extractComments option
- 1d76fec: refactor: remove dependency parent module identifier
- 8d0cba49: fix rebuild panic when tap watchRun hook
- fdaad48: fix: call devServer.invalidate() panic
- b452de5: fix: entry startup for target node
- 488071c: feat: add build meta exports type
- bce8523: db9a03e7: add compilation processWarnings hook
  db9a03e7: feat: ignoreWarnings option
- 5b33c94d: fix: hash chunk_loading_global
- 1c83bbd: fix: add correct chunk group runtime for multiply entry single runtime
- 4e25fafa: fix: react refresh usage finder
- bc52026: fix: already split module should connect chunk group at multiply entry
- 04daf09: feat: add optimization.realContentHash
- 3abb2fc: feat: support inline match resource
- c567571: fix: add entry startup to fix async script chunk loading
- f4c1f8d: refactor: external module for http request
- 6ec9e1c: fix: css url generate with auto public path
- Updated dependencies [f609f7a4]
- Updated dependencies [ada51e2]
- Updated dependencies [2bb2bcd6]
- Updated dependencies [ff85ce8]
- Updated dependencies [e764de62]
- Updated dependencies [c0e550b]
- Updated dependencies [088220b3]
- Updated dependencies [abf788a0]
- Updated dependencies [2a6a778]
- Updated dependencies [e5d33eb]
- Updated dependencies [04daf09]
- Updated dependencies [3abb2fc]
- Updated dependencies [2d9e5c2]
- Updated dependencies [f4c1f8d]
  - @rspack/binding@0.1.11
  - @rspack/dev-client@0.1.11

## 0.1.9

### Patch Changes

- 7c26a2d: feat: compiler.getCache
- 820c029: perf: compose loaders on the native side
- bc65893: feat: add moduleAssets for stats
- 6cc3076: feat: external module render with node-commonjs & module
- 7342a47: feat: support externals function
- c1720f9: add optimizeModules hook
- 0f1d3be: fix: use webworker instead of web-worker in AvailableTarget type
- e2647bb: feat: require.resolve and require.resolveWeak for string literal
- 057829b: fix: source map lost when enable builtins.banner
- ef7d3c5: feat: add ImportScriptsChunkLoadingPlugin
- c162cb6: feat: support banner plugin
- b77b706: fix: undefined reasons with stats.toJson({ reasons: true })
- Updated dependencies [820c029]
- Updated dependencies [0fd6d7a]
- Updated dependencies [bc65893]
- Updated dependencies [61d6e5d]
- Updated dependencies [aee4fdc]
- Updated dependencies [c1720f9]
- Updated dependencies [c951f35]
- Updated dependencies [e2647bb]
- Updated dependencies [057829b]
- Updated dependencies [c162cb6]
- Updated dependencies [b77b706]
- Updated dependencies [7f2cf5e]
  - @rspack/binding@0.1.9
  - @rspack/dev-client@0.1.9

## 0.1.8

### Patch Changes

- a72daa0: Support for provide top-level imports
- ef030d4: fix: extend chunkGroup name from EsmDynamicImportDependency
- f54621d: chore: bump swc 0.74.6
- fdc5fd9: fix: fix crash in multiCompiler
- Updated dependencies [352e563]
- Updated dependencies [9822cef]
- Updated dependencies [27afffc]
- Updated dependencies [fdc5fd9]
- Updated dependencies [d28a9d0]
- Updated dependencies [aa91ce7]
  - @rspack/binding@0.1.8
  - @rspack/dev-client@0.1.8

## 0.1.7

### Patch Changes

- 5bc1f55: fix Module Export Entry does not respect ordering
- 723a229: feat: support node false
- e649469: fix builtins.html does replace [hash] and [name] in builtins.html.filename
- 45aa2fe: fix: use codegen hash to calculate chunk hashes & fix runtime chunk hash
- 83f309a: fix: initial watching should stale until invalidate happens
- 55d3ea4: feat: enable resolve.fullySpecified for defaultRules
- 84851dc: Syntax support for export destructring binding
- 905cacf: feat: support module.rule[].descriptionData
- 8872af5: fix(css): remove trailing space at classname
- f4eb7c7: feat: support output.chunkLoadingGlobal
- 32f822b: add type and user_request field for module.reason
- Updated dependencies [5bc1f55]
- Updated dependencies [6f23105]
- Updated dependencies [c2bb73c]
- Updated dependencies [e649469]
- Updated dependencies [fff64ea]
- Updated dependencies [dc10d1f]
- Updated dependencies [905cacf]
- Updated dependencies [f4eb7c7]
- Updated dependencies [32f822b]
  - @rspack/binding@0.1.7
  - @rspack/dev-client@0.1.7

## 0.1.6

### Patch Changes

- Updated dependencies [3607f25]
- Updated dependencies [18dcca0]
  - @rspack/dev-client@0.1.6
  - @rspack/binding@0.1.6

## 0.1.5

### Patch Changes

- 125bb94: expose util for compiler.webpack
- 7bfcc7b: fix windows path is considered scheme incorrectly
- 050e4fb: feat: wasm loading types
- 92ee3c1: fix: library amd returning
- e8db1d7: feat: module.rule[].dependency
- c82529b: feat: new url in target node
- Updated dependencies [7bfcc7b]
- Updated dependencies [050e4fb]
- Updated dependencies [e8db1d7]
- Updated dependencies [c82529b]
- Updated dependencies [6c08098]
- Updated dependencies [60e0aec]
- Updated dependencies [33e916e]
  - @rspack/binding@0.1.5
  - @rspack/dev-client@0.1.5

## 0.1.4

### Patch Changes

- 85e47e2: fix: context module request
- fbaeb41: fix: duplicate \_\_webpack_require\_\_.a in async module
- Updated dependencies [fbaeb41]
  - @rspack/binding@0.1.4
  - @rspack/dev-client@0.1.4

## 0.1.3

### Patch Changes

- 62bad72: optimize the initialization of instance
- 9cb8c7e: fix: cjs transfrom preset env
- 4471853: fix: amd should return with iife
- 54cb3fa: fix: use babel mode for mjs
- 1aadf05: feat: Support `new URL("./foo", import.meta.url)`
- b323220: add async-wasm & js-async-module support
- 28b9757: feat: resolve.byDependency
- b0cffba: feat: inline external type syntax
- 9c71512: add finishModules hook
- c49c03c: Support `suspend` and `resume` in Watching
- d04485d: feat: stats for timings and builtAt
- Updated dependencies [4471853]
- Updated dependencies [397b0d7]
- Updated dependencies [b323220]
- Updated dependencies [28b9757]
- Updated dependencies [b6ab7b7]
- Updated dependencies [b0cffba]
- Updated dependencies [9c71512]
  - @rspack/binding@0.1.3
  - @rspack/dev-client@0.1.3

## 0.1.2

### Patch Changes

- 68c4df8: feat: module.hot.accept/decline support arrary dependencies
- 2486b2a: fix: returning exports from runtime for libraries
- f562fa9: feat: add simple ignore options for copy-plugin
- b4c5ed5: fix: css url rewrite for complex http url within ~
- eb7051f: fix: export without specifier
- cd011c1: feat: stats.outputPath
- 7431442: fix(builtins.html): should parse `builtins.html.template` as path
- a6ac7da: feat: function type RuleSetCondition
- 48eff5e: feat: add normalModuleFactory and resolveForScheme hook
- 8f8e025: align optimizeChunkModules
- 7fde34f: fix: node mode interop
- 2934f8f: compilation.hooks.additionalAssets
- a0994a8: fix(cli): rspack build --watch not work ([#2280](https://github.com/web-infra-dev/rspack/issues/2280))
- 2e9d331: feat(runtime): support node.\_\_filename polyfill
- Updated dependencies [68c4df8]
- Updated dependencies [a43a849]
- Updated dependencies [f562fa9]
- Updated dependencies [97dc0dd]
- Updated dependencies [b4c5ed5]
- Updated dependencies [36ffd12]
- Updated dependencies [a6ac7da]
- Updated dependencies [48eff5e]
- Updated dependencies [8f8e025]
- Updated dependencies [7fde34f]
- Updated dependencies [bfb1738]
- Updated dependencies [2c01586]
- Updated dependencies [6d9eb8b]
- Updated dependencies [2e9d331]
- Updated dependencies [a54179d]
- Updated dependencies [fbc315d]
- Updated dependencies [c4c20a3]
  - @rspack/binding@0.1.2
  - @rspack/dev-client@0.1.2

## 0.1.1

### Patch Changes

- 6f8706f0: fix: postcss modules warning when using postcss-loader
- 0e32353d: fix(resolve): upgrade the version of nodejs_resolver to fix the error which will resolve failed when target is relative by tsconfig/baseUrl
- d2072dd4: fix: commonjs loading with extra output dir
- 706207a4: feat: loader api add dependency
- 62fca585: feat: config validate strategy
- 7b3167ca: feat: externalsPresets.node
- e463ebf0: refactor: runtime module
- f204c108: feat: array type of externals
- bb22416a: feat: stats for chunkRelations and chunkModules
- 98854e36: fix: runtime module cacheable
- a004765a: Avoid reporting warning the export when src module format is none esm
- a004268f: fix: library umd code excute
- 2d2c9624: fix: fix webpack-sources interoperation
- cdf6a52a: fix: normalize identifier for externalsType umd
- Updated dependencies [6f8706f0]
- Updated dependencies [0e32353d]
- Updated dependencies [23048514]
- Updated dependencies [7b3167ca]
- Updated dependencies [f204c108]
- Updated dependencies [bb22416a]
- Updated dependencies [44dc1e8b]
- Updated dependencies [a004765a]
- Updated dependencies [2f7ffb57]
- Updated dependencies [2d2c9624]
- Updated dependencies [6bdc0840]
  - @rspack/binding@0.1.1
  - @rspack/dev-client@0.1.1

## 0.1.0

### Minor Changes

- 2ba87f3a: chore: let's rspack

### Patch Changes

- 6514137b: fix: remove unnecessary logs
- 95c3593b: fix: postinstall script is wrongly configured
- Updated dependencies [2ba87f3a]
  - @rspack/binding@0.1.0
  - @rspack/dev-client@0.1.0

## 0.0.26

### Patch Changes

- c81fd863: fix: umd library name template string
- c81fd863: feat: json schema check for RspackOptions
  - @rspack/binding@0.0.26
  - @rspack/dev-client@0.0.26

## 0.0.25

### Patch Changes

- 6d6e65e4: feat: update packages repository config
- Updated dependencies [6d6e65e4]
  - @rspack/binding@0.0.25
  - @rspack/dev-client@0.0.25

## 0.0.24

### Patch Changes

- 3cc27e32a: feat: add publicPath on stats
- 3495d3d72: feat: add stats.namedChunkGroups
- c456aed7e: fix: use poll for default watching method
- abf34dc3d: feat: normalize for RuleSetUse
- e2466248d: feat: add stats.assetsByChunkName
- a22149e98: fix: css modules ident-name leading digits
- e9bf3de8b: feat: set builtins.progress default value to true while dev
- a528a8e06: feat: more logical conditions matcher
- 58b77bafc: feat: add plugin-import
- 6316c28e7: feat: add swc relay plugin
- Updated dependencies [3495d3d72]
- Updated dependencies [e2466248d]
- Updated dependencies [a22149e98]
- Updated dependencies [58b77bafc]
- Updated dependencies [b101fd41e]
  - @rspack/binding@0.0.24
  - @rspack/dev-client@0.0.24

## 0.0.23

### Patch Changes

- b67418968: chore: 🤖 use module path instead of module id in diagnositc
- f3e0d8287: feat: support module.rule.oneOf
- 962f8a251: fix: should create different module with different module rule
- 327b600d6: fix wrong fallback value of sideEffects in production mode
- 1c7ab6dfd: feat: rspack-cli and devServer support multiCompiler
- 6b5555ee1: refactor: refactor builtins.preset_env option
- 766c94042: fix rust test
- 2315cad48: fix: lost chunk loading runtime when target is browserlist
- e5d628ce3: feat: commonjs require context module
- 2d395d6b0: fix: rspack postcss loader options.postcssOptions.plugin
- 035c15953: basic implementation of compilation.hooks.optimizeChunkModules
- 82aa28d6b: feat: add copy-plugin
- b694d4a58: feat: add backtrace if either napi stack or message is failed to extract
- c1f19b817: align webpack config optimization.sideEffects
- Updated dependencies [b67418968]
- Updated dependencies [962f8a251]
- Updated dependencies [17bf167f6]
- Updated dependencies [766c94042]
- Updated dependencies [39f8a9c42]
- Updated dependencies [e64506a51]
- Updated dependencies [26e66549e]
- Updated dependencies [c98bf5768]
- Updated dependencies [035c15953]
- Updated dependencies [82aa28d6b]
- Updated dependencies [b694d4a58]
- Updated dependencies [60fb4c5bf]
- Updated dependencies [c1f19b817]
  - @rspack/binding@0.0.23
  - @rspack/dev-client@0.0.23

## 0.0.22

### Patch Changes

- 361f9a539: fix: relative source map url path
- 59edc2cb4: fix watch options
- b77074dfa: feat: support multiple configuration
- c65ca69eb: feat: environment, afterEnvironment, afterPlugins, and afterResolvers hooks
- 4de9eea6c: feat(close: #1654): support optimization.runtimeChunk
- 0bc720c7e: chore: add minify pureFuncs & dropConsole options
- ac02a096e: fix: getRspackMemoryAssets failed to get index.html when request path is /
  feat: extends webpack-dev-server
- 8a4cb9a38: feat: more js api for webpack plugin compatibility
- 9c90398a8: feat(core): improve mode config typing
- 290bf7fb5: feat: add devServer.historyApiFallback options
- 86ed12184: fix: add readonly type for compilation.assets and compilation.entrypoints
- 126b2160e: fix: failed to apply loader when loader is esModule
- 792304dd9: feat: support rule specific parser.dataUrlCondition.maxSize and generator.filename
- 6d4f3e627: feat: missing module
- d88ffa666: feat: support devServer.client.webSocketUrl
- 3fcfa7462: use callback in close of watch
- cb93bb421: fix: not show module reasons for default stats toString
- 53acb67c7: feat: support copy-webpack-plugin@5
- 6b95cf27f: feat: support rule.issuer.not
- 2f4db99e0: feat: processAssets hook stage
- 82ae10cad: fix: css url rewrite with output.cssFilename
  fix: css url rewrite with data url
- 5e722adf1: feat: add emit and afterEmit hooks
- b6e9a1b5e: Supports `optimization.removeAvailableModules`
- 6722de813: fix: css url rewrite with ~
- 58465b81b: feat(packages/rspack): loader context support compiler and compilation
- 70586d79e: fix: `delete compilation.assets[filename]` should keep deleted asset info
- Updated dependencies [e402226e5]
- Updated dependencies [59edc2cb4]
- Updated dependencies [549796acc]
- Updated dependencies [51916f548]
- Updated dependencies [da069320e]
- Updated dependencies [8a4cb9a38]
- Updated dependencies [cd7736377]
- Updated dependencies [97eaa8208]
- Updated dependencies [d88ffa666]
- Updated dependencies [b5343d3f4]
- Updated dependencies [10db0a2d8]
- Updated dependencies [e845df7da]
- Updated dependencies [cb93bb421]
- Updated dependencies [faef6fc00]
- Updated dependencies [2f4db99e0]
- Updated dependencies [820b5a79b]
- Updated dependencies [5e722adf1]
- Updated dependencies [278e89cc1]
- Updated dependencies [b6e9a1b5e]
- Updated dependencies [0e1a42d46]
- Updated dependencies [0269ff40d]
- Updated dependencies [8dc513ac3]
- Updated dependencies [58bef147b]
- Updated dependencies [70586d79e]
  - @rspack/binding@0.0.22
  - @rspack/dev-client@0.0.22

## 0.0.21

### Patch Changes

- fix watch
- Updated dependencies
  - @rspack/binding@0.0.21
  - @rspack/dev-client@0.0.21

## 0.0.20

### Patch Changes

- fix load extra css chunk
- Updated dependencies
  - @rspack/binding@0.0.20
  - @rspack/dev-client@0.0.20

## 0.0.19

### Patch Changes

- db66ae2e: feat: add hash as return value of stats.toJson
- 882093b8: support module.resolve
- Updated dependencies [db66ae2e]
- Updated dependencies [882093b8]
  - @rspack/binding@0.0.19
  - @rspack/dev-client@0.0.19

## 0.0.18

### Patch Changes

- bump version
- Updated dependencies
  - @rspack/binding@0.0.18
  - @rspack/dev-client@0.0.18

## 0.0.17

### Patch Changes

- upgrade
- Updated dependencies
  - @rspack/binding@0.0.17
  - @rspack/dev-client@0.0.17

## 0.0.16

### Patch Changes

- support optional dependency
- Updated dependencies
  - @rspack/binding@0.0.16
  - @rspack/dev-client@0.0.16

## 0.0.15

### Patch Changes

- bump version
- Updated dependencies
  - @rspack/binding@0.0.15
  - @rspack/dev-client@0.0.15

## 0.0.14

### Patch Changes

- bump version
- 11e87c61: fix less resolve bug
- Updated dependencies
- Updated dependencies [11e87c61]
  - @rspack/binding@0.0.14
  - @rspack/dev-client@0.0.14

## 0.0.13

### Patch Changes

- 3701a8bf: fix less resolve bug
- Updated dependencies [3701a8bf]
- Updated dependencies
  - @rspack/binding@0.0.13
  - @rspack/dev-client@0.0.13

## 0.0.12

### Patch Changes

- @rspack/binding@0.0.12
- @rspack/dev-client@0.0.12

## 0.0.11

### Patch Changes

- 2eca9ade: add ~ alias support
  - @rspack/binding@0.0.11
  - @rspack/dev-client@0.0.11

## 0.0.10

### Patch Changes

- 062e692d: fix parse url failed
  - @rspack/binding@0.0.10
  - @rspack/dev-client@0.0.10

## 0.0.9

### Patch Changes

- Updated dependencies [69becfa5]
  - @rspack/binding@0.0.9
  - @rspack/plugin-postcss@0.0.9
  - @rspack/plugin-less@0.0.9
  - @rspack/dev-client@0.0.9

## 0.0.8

### Patch Changes

- 589b99bb: bump to 0.0.8
- Updated dependencies [589b99bb]
  - @rspack/binding@0.0.8
  - @rspack/dev-client@0.0.8
  - @rspack/plugin-less@0.0.8
  - @rspack/plugin-postcss@0.0.8

## 0.0.6

### Patch Changes

- e6d0926a: unify version
- Updated dependencies [e6d0926a]
  - @rspack/binding@0.0.6
  - @rspack/plugin-postcss@0.0.6
  - @rspack/plugin-less@0.0.6
  - @rspack/dev-client@0.0.6

## 0.0.4

### Patch Changes

- d466288: add process_assets hook
- Updated dependencies [d466288]
  - @rspack/binding@0.0.5
  - @rspack/dev-server@0.0.4
  - @rspack/plugin-postcss@0.0.4

## 0.0.3

### Patch Changes

- 536f6f70: fix broken lib
- Updated dependencies [536f6f70]
  - @rspack/binding@0.0.4
  - @rspack/dev-server@0.0.3
  - @rspack/plugin-postcss@0.0.3

## 0.0.2

### Patch Changes

- bd57e818: first release
- Updated dependencies [bd57e818]
  - @rspack/binding@0.0.2
  - rspack-dev-server@0.0.2
  - rspack-plugin-postcss@0.0.2
