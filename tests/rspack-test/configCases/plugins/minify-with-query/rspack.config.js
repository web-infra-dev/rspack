/**@type {import("@rspack/core").Configuration}*/
module.exports = {
  externals: {
    fs: 'node-commonjs fs',
    path: 'node-commonjs path',
  },
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
  // experiments: { css: true },
  output: {
    filename: 'bundle0.js?hash=[contenthash]',
    cssFilename: 'bundle0.css?hash=[contenthash]',
  },
  optimization: {
    minimize: true,
  },
};
