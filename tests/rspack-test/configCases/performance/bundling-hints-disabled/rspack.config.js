/** @type {import('@rspack/core').Configuration} */
module.exports = {
  mode: 'production',
  devtool: false,
  performance: {
    all: false,
    hints: 'warning',
  },
  module: {
    rules: [{ test: /big\.svg$/, type: 'asset/inline' }],
  },
  optimization: { minimize: false },
};
