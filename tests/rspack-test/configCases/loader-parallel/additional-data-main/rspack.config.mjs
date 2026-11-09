import { createRequire } from 'node:module';
const require = createRequire(import.meta.url);
const path = require('path');

/** @type {import("@rspack/core").RspackOptions} */
export default {
  context: import.meta.dirname,
  module: {
    rules: [
      {
        test: path.join(import.meta.dirname, 'a.js'),
        use: [
          { loader: './main-loader.js' },
          { loader: './parallel-loader.js', parallel: true, options: {} },
        ],
      },
    ],
  },
};
