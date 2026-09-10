import rspack from '@rspack/core';

/** @type {import("@rspack/core").Configuration} */
export default {
  entry: './src/index.js',
  context: import.meta.dirname,
  mode: 'development',
  plugins: [new rspack.HtmlRspackPlugin()],
  module: {
    rules: [
      {
        test: /\.js$/,
        exclude: [/node_modules/],
        include: [/src/],
        loader: './loader.cjs',
      },
    ],
  },
  devServer: {
    hot: true,
  },
};
