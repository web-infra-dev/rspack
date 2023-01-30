# @rspack/core

## 1.0.0

### Minor Changes

- 4de9eea6: feat(close: #1654): support optimization.runtimeChunk

### Patch Changes

- 59edc2cb: fix watch options
- b77074df: feat: support multiple configuration
- c65ca69e: feat: environment, afterEnvironment, afterPlugins, and afterResolvers hooks
- 8a4cb9a3: feat: more js api for webpack plugin compatibility
- 9c90398a: feat(core): improve mode config typing
- 290bf7fb: feat: add devServer.historyApiFallback options
- 86ed1218: fix: add readonly type for compilation.assets and compilation.entrypoints
- 792304dd: feat: support rule specific parser.dataUrlCondition.maxSize and generator.filename
- 6d4f3e62: feat: missing module
- 3fcfa746: use callback in close of watch
- cb93bb42: fix: not show module reasons for default stats toString
- 2f4db99e: feat: processAssets hook stage
- 5e722adf: feat: add emit and afterEmit hooks
- 70586d79: fix: `delete compilation.assets[filename]` should keep deleted asset info
- Updated dependencies [e402226e]
- Updated dependencies [59edc2cb]
- Updated dependencies [549796ac]
- Updated dependencies [51916f54]
- Updated dependencies [da069320]
- Updated dependencies [8a4cb9a3]
- Updated dependencies [cd773637]
- Updated dependencies [b5343d3f]
- Updated dependencies [10db0a2d]
- Updated dependencies [e845df7d]
- Updated dependencies [cb93bb42]
- Updated dependencies [faef6fc0]
- Updated dependencies [2f4db99e]
- Updated dependencies [820b5a79]
- Updated dependencies [5e722adf]
- Updated dependencies [278e89cc]
- Updated dependencies [0269ff40]
- Updated dependencies [8dc513ac]
- Updated dependencies [58bef147]
- Updated dependencies [70586d79]
  - @rspack/binding@1.0.0
  - @rspack/dev-client@1.0.0

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
