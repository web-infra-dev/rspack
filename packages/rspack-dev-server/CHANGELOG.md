# rspack-dev-server

## 0.1.11

### Patch Changes

- cafa227d: fix cannot reading noEmitAssets by always getting the first compiler's noEmitAssets
- Updated dependencies [cafa227d]
  - @rspack/dev-server@0.1.11
  - @rspack/dev-middleware@0.1.11
  - @rspack/dev-client@0.1.11

## 0.1.9

### Patch Changes

- @rspack/dev-middleware@0.1.9
- @rspack/dev-server@0.1.9
- @rspack/dev-client@0.1.9

## 0.1.8

### Patch Changes

- @rspack/dev-middleware@0.1.8
- @rspack/dev-server@0.1.8
- @rspack/dev-client@0.1.8

## 0.1.7

### Patch Changes

- Updated dependencies [fff64ea]
  - @rspack/dev-client@0.1.7
  - @rspack/dev-middleware@0.1.7
  - @rspack/dev-server@0.1.7

## 0.1.6

### Patch Changes

- 3607f25: Remove custom dev client and use webpack-dev-server/client direclty
- Updated dependencies [3607f25]
  - @rspack/dev-client@0.1.6
  - @rspack/dev-server@0.1.6
  - @rspack/dev-middleware@0.1.6

## 0.1.5

### Patch Changes

- @rspack/dev-middleware@0.1.5
- @rspack/dev-server@0.1.5
- @rspack/dev-client@0.1.5

## 0.1.4

### Patch Changes

- @rspack/dev-middleware@0.1.4
- @rspack/dev-server@0.1.4
- @rspack/dev-client@0.1.4

## 0.1.3

### Patch Changes

- @rspack/dev-middleware@0.1.3
- @rspack/dev-server@0.1.3
- @rspack/dev-client@0.1.3

## 0.1.2

### Patch Changes

- d63e3fd: upgrade webpack-dev-server & webpack-dev-middleware
- bfb1738: fix: react refresh runtime injection
- d63e3fd: upgrade webpack-dev-server
- bb4e6a6: use super isWebTarget in dev-server
- Updated dependencies [d63e3fd]
- Updated dependencies [a6ac7da]
- Updated dependencies [deaca70]
- Updated dependencies [bfb1738]
- Updated dependencies [d63e3fd]
- Updated dependencies [bb4e6a6]
  - @rspack/dev-middleware@0.1.2
  - @rspack/dev-server@0.1.2
  - @rspack/dev-client@0.1.2

## 0.1.1

### Patch Changes

- a004765a: Avoid reporting warning the export when src module format is none esm
- Updated dependencies [f5f661a2]
- Updated dependencies [a004765a]
  - @rspack/dev-middleware@0.1.1
  - @rspack/dev-client@0.1.1
  - @rspack/dev-server@0.1.1

## 0.1.0

### Minor Changes

- 2ba87f3a: chore: let's rspack

### Patch Changes

- Updated dependencies [2ba87f3a]
  - @rspack/dev-client@0.1.0
  - @rspack/dev-middleware@0.1.0
  - @rspack/dev-server@0.1.0

## 0.0.26

### Patch Changes

- @rspack/dev-middleware@0.0.26
- @rspack/dev-server@0.0.26
- @rspack/dev-client@0.0.26

## 0.0.25

### Patch Changes

- 6d6e65e4: feat: update packages repository config
- Updated dependencies [6d6e65e4]
  - @rspack/dev-client@0.0.25
  - @rspack/dev-middleware@0.0.25
  - @rspack/dev-server@0.0.25

## 0.0.24

### Patch Changes

- 58b77bafc: feat: add plugin-import
- 4f432286b: fix: devServer.hot should be true by default when used by rspack api
- Updated dependencies [58b77bafc]
- Updated dependencies [4f432286b]
  - @rspack/dev-server@0.0.24
  - @rspack/dev-middleware@0.0.24
  - @rspack/dev-client@0.0.24

## 0.0.23

### Patch Changes

- b67418968: chore: 🤖 use module path instead of module id in diagnositc
- 962f8a251: fix: should create different module with different module rule
- 1c7ab6dfd: feat: rspack-cli and devServer support multiCompiler
- 766c94042: fix rust test
- 035c15953: basic implementation of compilation.hooks.optimizeChunkModules
- 82aa28d6b: feat: add copy-plugin
- c1f19b817: align webpack config optimization.sideEffects
- Updated dependencies [b67418968]
- Updated dependencies [962f8a251]
- Updated dependencies [1c7ab6dfd]
- Updated dependencies [766c94042]
- Updated dependencies [035c15953]
- Updated dependencies [82aa28d6b]
- Updated dependencies [60fb4c5bf]
- Updated dependencies [c1f19b817]
  - @rspack/dev-client@0.0.23
  - @rspack/dev-middleware@0.0.23
  - @rspack/dev-server@0.0.23

## 0.0.22

### Patch Changes

- 59edc2cb4: fix watch options
- ac02a096e: fix: getRspackMemoryAssets failed to get index.html when request path is /
  feat: extends webpack-dev-server
- 290bf7fb5: feat: add devServer.historyApiFallback options
- d88ffa666: feat: support devServer.client.webSocketUrl
- Updated dependencies [ce31cd029]
- Updated dependencies [59edc2cb4]
- Updated dependencies [ac02a096e]
- Updated dependencies [290bf7fb5]
- Updated dependencies [d88ffa666]
- Updated dependencies [0269ff40d]
  - @rspack/dev-middleware@0.0.22
  - @rspack/dev-client@0.0.22
  - @rspack/dev-server@0.0.22

## 0.0.21

### Patch Changes

- fix watch
- Updated dependencies
  - @rspack/core@0.0.21
  - @rspack/dev-client@0.0.21
  - @rspack/dev-middleware@0.0.21
  - @rspack/dev-server@0.0.21

## 0.0.20

### Patch Changes

- fix load extra css chunk
- Updated dependencies
  - @rspack/core@0.0.20
  - @rspack/dev-client@0.0.20
  - @rspack/dev-middleware@0.0.20
  - @rspack/dev-server@0.0.20

## 0.0.19

### Patch Changes

- 882093b8: support module.resolve
- Updated dependencies [db66ae2e]
- Updated dependencies [882093b8]
  - @rspack/core@0.0.19
  - @rspack/dev-client@0.0.19
  - @rspack/dev-middleware@0.0.19
  - @rspack/dev-server@0.0.19

## 0.0.18

### Patch Changes

- bump version
- Updated dependencies
  - @rspack/core@0.0.18
  - @rspack/dev-client@0.0.18
  - @rspack/dev-middleware@0.0.18
  - @rspack/dev-server@0.0.18

## 0.0.17

### Patch Changes

- upgrade
- Updated dependencies
  - @rspack/core@0.0.17
  - @rspack/dev-client@0.0.17
  - @rspack/dev-middleware@0.0.17
  - @rspack/dev-server@0.0.17

## 0.0.16

### Patch Changes

- support optional dependency
- Updated dependencies
  - @rspack/core@0.0.16
  - @rspack/dev-client@0.0.16
  - @rspack/dev-middleware@0.0.16
  - @rspack/dev-server@0.0.16

## 0.0.15

### Patch Changes

- bump version
- Updated dependencies
  - @rspack/core@0.0.15
  - @rspack/dev-client@0.0.15
  - @rspack/dev-middleware@0.0.15
  - @rspack/dev-server@0.0.15

## 0.0.14

### Patch Changes

- bump version
- 11e87c61: fix less resolve bug
- Updated dependencies
- Updated dependencies [11e87c61]
  - @rspack/core@0.0.14
  - @rspack/dev-client@0.0.14
  - @rspack/dev-middleware@0.0.14
  - @rspack/dev-server@0.0.14

## 0.0.13

### Patch Changes

- 3701a8bf: fix less resolve bug
- Updated dependencies [3701a8bf]
  - @rspack/core@0.0.13
  - @rspack/dev-client@0.0.13
  - @rspack/dev-middleware@0.0.13
  - @rspack/dev-server@0.0.13

## 0.0.12

### Patch Changes

- @rspack/core@0.0.12
- @rspack/dev-client@0.0.12
- @rspack/dev-middleware@0.0.12
- @rspack/dev-server@0.0.12

## 0.0.11

### Patch Changes

- Updated dependencies [2eca9ade]
  - @rspack/core@0.0.11
  - @rspack/dev-middleware@0.0.11
  - @rspack/dev-server@0.0.11
  - @rspack/dev-client@0.0.11

## 0.0.10

### Patch Changes

- Updated dependencies [062e692d]
  - @rspack/core@0.0.10
  - @rspack/dev-middleware@0.0.10
  - @rspack/dev-server@0.0.10
  - @rspack/dev-client@0.0.10

## 0.0.9

### Patch Changes

- @rspack/core@0.0.9
- @rspack/dev-middleware@0.0.9
- @rspack/dev-server@0.0.9
- @rspack/dev-client@0.0.9

## 0.0.8

### Patch Changes

- 589b99bb: bump to 0.0.8
- Updated dependencies [589b99bb]
  - @rspack/core@0.0.8
  - @rspack/dev-client@0.0.8
  - @rspack/dev-middleware@0.0.8
  - @rspack/dev-server@0.0.8

## 0.0.6

### Patch Changes

- e6d0926a: unify version
- Updated dependencies [e6d0926a]
  - @rspack/core@0.0.6
  - @rspack/dev-server@0.0.6
  - @rspack/dev-middleware@0.0.6
  - @rspack/dev-client@0.0.6

## 0.0.4

### Patch Changes

- d466288: add process_assets hook

## 0.0.3

### Patch Changes

- 536f6f70: fix broken lib

## 0.0.2

### Patch Changes

- bd57e818: first release
