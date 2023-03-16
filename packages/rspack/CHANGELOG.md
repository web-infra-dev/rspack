# @rspack/core

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
