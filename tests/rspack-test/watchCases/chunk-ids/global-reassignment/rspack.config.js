/** @type {import('@rspack/core').Configuration} */
module.exports = {
  entry: ['./index.js', './trigger.js'],
  cache: { type: 'memory' },
  optimization: {
    moduleIds: 'named',
    chunkIds: 'deterministic',
    concatenateModules: false,
    inlineExports: false,
  },
  incremental: true,
};
