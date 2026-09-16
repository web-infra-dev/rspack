import path from 'node:path';

/** @type {import("@rspack/core").Configuration} */
export default {
  resolve: {
    tsConfig: {
      configFile: path.resolve(import.meta.dirname, './tsconfig.json'),
    },
  },
  module: {
    rules: [
      {
        test: /index/,
        loader: './loader.js',
      },
    ],
  },
};
