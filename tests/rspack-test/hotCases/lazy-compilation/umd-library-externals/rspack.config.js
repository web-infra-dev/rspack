'use strict';

// A UMD library bundle with a closure-bound external (`util`) where
// `lazyCompilation` defers the entry's dependents. Before the fix, the first
// activation throws because the initial UMD wrapper didn't reserve the closure
// identifier for the `util` external that the inactive proxy never referenced.
/** @type {import("@rspack/core").Configuration} */
module.exports = {
  output: {
    library: {
      name: 'TestLib',
      type: 'umd',
    },
  },
  externals: {
    util: 'util',
  },
  externalsType: 'umd',
  lazyCompilation: {
    entries: false,
  },
};
