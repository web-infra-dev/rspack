/** @type {import("@rspack/core").Configuration} */
module.exports = {
  target: 'node',
  experiments: { outputModule: true },
  output: {
    module: true,
    chunkFormat: 'module',
    library: { type: 'module' },
    importMetaName: '__custom_import_meta',
  },
  module: {
    parser: { javascript: { createRequire: true, requireResolve: false } },
  },
  optimization: { moduleIds: 'named', concatenateModules: false },
};
