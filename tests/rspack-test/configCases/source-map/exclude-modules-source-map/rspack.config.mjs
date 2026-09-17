import { rspack as webpack } from '@rspack/core';

/** @type {import("@rspack/core").Configuration} */
export default {
  node: {
    __dirname: false,
    __filename: false,
  },
  entry: {
    bundle0: ['./index.js'],
    bundle1: ['./test1.js', './test2.js'],
  },
  output: {
    filename: '[name].js',
  },
  plugins: [
    new webpack.EvalSourceMapDevToolPlugin({
      include: /\.js$/,
      exclude: /test2\.js/,
      module: true,
      columns: false,
    }),
  ],
};
