'use strict';

// Mirrors https://github.com/web-infra-dev/rspack/issues/9023 (webpack #19134):
// a UMD micro-frontend bundle with multiple closure-bound externals where
// `lazyCompilation` defers the entry's dependents. Before the fix, the first
// activation throws because the initial UMD wrapper didn't reserve the closure
// identifiers for externals that weren't yet referenced by the inactive proxy.
/** @type {import("@rspack/core").Configuration} */
module.exports = {
  output: {
    library: {
      name: 'demo',
      type: 'umd',
    },
  },
  externals: {
    fs: 'fs',
    path: 'path',
  },
  externalsType: 'umd',
  lazyCompilation: {
    entries: false,
  },
};
