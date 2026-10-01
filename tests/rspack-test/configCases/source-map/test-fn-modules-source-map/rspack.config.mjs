import { rspack } from '@rspack/core';

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
    new rspack.EvalSourceMapDevToolPlugin({
      test: (str) => {
        if (str.endsWith('.js')) return true;
        return false;
      },
      exclude: /test2\.js/,
      module: true,
      columns: false,
    }),
  ],
};
