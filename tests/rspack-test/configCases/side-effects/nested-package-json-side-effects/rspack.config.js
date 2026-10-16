/** @type {import('@rspack/core').Configuration} */
module.exports = {
  mode: 'production',
  optimization: {
    minimize: false,
    sideEffects: true,
    usedExports: true,
  },
};
