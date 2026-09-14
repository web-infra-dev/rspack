const { DefinePlugin } = require('@rspack/core');
module.exports = [0, 1].map((index) => ({
  mode: 'production',
  target: 'web',
  experiments: { chunkArrayLoading: true, css: true },
  output: {
    filename: 'main-' + index + '.js',
    chunkFilename: '[name].js',
    cssChunkFilename: '[name].css',
  },
  module: { rules: [{ test: /\.css$/, type: 'css' }] },
  plugins: [
    new DefinePlugin({
      FAIL_CSS: JSON.stringify(index === 1),
      CASE_INDEX: JSON.stringify(index),
    }),
  ],
  optimization: {
    minimize: false,
    splitChunks: {
      minSize: 0,
      cacheGroups: {
        dep: {
          test: /dep\.js$/,
          name: 'dep-' + index,
          enforce: true,
          chunks: 'async',
        },
        lazy: {
          test: /lazy\.js$/,
          name: 'js-group-' + index,
          enforce: true,
          chunks: 'async',
        },
        css: {
          test: /\.css$/,
          name: 'css-group-' + index,
          enforce: true,
          chunks: 'async',
        },
      },
    },
  },
}));
