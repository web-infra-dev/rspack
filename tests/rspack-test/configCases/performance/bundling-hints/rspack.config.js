/** @type {import('@rspack/core').Configuration} */
module.exports = {
  mode: 'production',
  devtool: 'inline-source-map',
  performance: {
    all: true,
    hints: 'warning',
  },
  module: {
    rules: [{ test: /big\.svg$/, type: 'asset/inline' }],
  },
  optimization: { minimize: false },
};
