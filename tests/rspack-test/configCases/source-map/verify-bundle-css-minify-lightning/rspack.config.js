const { rspack } = require('@rspack/core');

/** @type {import("@rspack/core").Configuration} */
module.exports = {
  target: 'web',
  node: false,
  module: {
    generator: {
      'css/auto': {
        exportsOnly: false,
      },
    },
    rules: [
      {
        test: /\.css$/,
        type: 'css/auto',
      },
    ],
  },
  devtool: 'source-map',
  optimization: {
    minimize: true,
    minimizer: [new rspack.LightningCssMinimizerRspackPlugin()],
  },
  externals: [
    {
      '@rspack/test-tools/helper/util/checkSourceMap':
        'commonjs @rspack/test-tools/helper/util/checkSourceMap',
    },
    'source-map',
  ],
  externalsType: 'commonjs',
};
