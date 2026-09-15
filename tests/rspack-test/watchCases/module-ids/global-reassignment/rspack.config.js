/** @type {import('@rspack/core').Configuration} */
module.exports = {
  cache: { type: 'memory' },
  optimization: {
    moduleIds: 'deterministic',
    chunkIds: 'named',
    concatenateModules: false,
    inlineExports: false,
  },
  incremental: true,
};
