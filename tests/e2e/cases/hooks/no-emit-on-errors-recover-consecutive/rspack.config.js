import rspack from '@rspack/core';

/** @type {import("@rspack/core").Configuration} */
export default {
  entry: './src/index.js',
  context: import.meta.dirname,
  mode: 'development',
  plugins: [new rspack.HtmlRspackPlugin()],
  optimization: {
    emitOnErrors: false,
  },
  devServer: {
    hot: true,
  },
};
