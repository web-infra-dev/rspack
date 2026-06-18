/** @type {import("@rspack/core").Configuration} */
module.exports = {
  target: 'node',
  experiments: { outputModule: true },
  output: { module: true, chunkFormat: 'module', library: { type: 'module' } },
  module: {
    parser: { javascript: { createRequire: true, requireResolve: false } },
  },
  // Keep modules separate so the export boundary is real (the created require is observed
  // only across the dep -> index module edge).
  optimization: { moduleIds: 'named', concatenateModules: false },
};
