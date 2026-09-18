import { fileURLToPath } from 'node:url';

/** @type {import("@rspack/core").Configuration} */
export default {
  target: 'node',
  entry: './index',
  devtool: 'source-map',
  module: {
    rules: [
      {
        test: /\.js$/,
        extractSourceMap: true,
        loader: fileURLToPath(import.meta.resolve('./babel-loader.mjs')),
      },
    ],
  },
};
