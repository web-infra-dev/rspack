/** @type {import("@rspack/core").Configuration} */
module.exports = {
  target: 'node',
  experiments: { outputModule: true },
  output: { module: true, chunkFormat: 'module', library: { type: 'module' } },
  module: {
    parser: { javascript: { createRequire: true, requireResolve: false } },
  },
  optimization: { moduleIds: 'named', concatenateModules: false },
};
