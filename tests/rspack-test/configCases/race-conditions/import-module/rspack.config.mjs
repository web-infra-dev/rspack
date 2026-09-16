import { fileURLToPath } from 'node:url';

/** @type {import("@rspack/core").Configuration} */
export default {
  parallelism: 1,
  mode: 'development',
  module: {
    rules: [
      {
        test: /\.css$/i,
        type: 'javascript/auto',
        use: [fileURLToPath(import.meta.resolve('./loader.js')), 'css-loader'],
      },
    ],
  },
};
