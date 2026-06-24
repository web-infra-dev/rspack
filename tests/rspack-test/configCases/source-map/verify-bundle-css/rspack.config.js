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
  externals: [
    {
      fs: 'node-commonjs fs',
      path: 'node-commonjs path',
    },
    {
      '@rspack/test-tools/helper/util/checkSourceMap':
        'commonjs @rspack/test-tools/helper/util/checkSourceMap',
    },
    'source-map',
  ],
  externalsType: 'commonjs',
};
