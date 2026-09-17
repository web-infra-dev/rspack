/** @type {import('@rspack/core').Configuration} */
export default {
  cache: { type: 'memory' },
  optimization: {
    moduleIds: 'deterministic',
    chunkIds: 'named',
    concatenateModules: false,
    inlineExports: false,
  },
  incremental: true,
};
