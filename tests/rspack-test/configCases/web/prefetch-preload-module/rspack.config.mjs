/** @type {import("@rspack/core").Configuration} */
export default {
  entry: './index.mjs',
  name: 'esm',
  target: 'web',
  output: {
    publicPath: '',
    module: true,
    filename: 'bundle0.mjs',
    chunkFilename: '[name].mjs',
    chunkFormat: 'module',
    crossOriginLoading: 'anonymous',
  },
  module: {
    rules: [
      {
        test: /\.css$/,
        type: 'css/auto',
      },
    ],
  },
  performance: {
    hints: false,
  },
  optimization: {
    minimize: false,
  },
};
