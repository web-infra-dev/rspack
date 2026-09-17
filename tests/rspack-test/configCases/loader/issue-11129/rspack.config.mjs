import path from 'node:path';

/** @type {import("@rspack/core").Configuration} */
export default {
  module: {
    rules: [
      {
        test: path.resolve(import.meta.dirname, 'lib.js'),
        resourceQuery: /inline/,
        use: 'exports-loader?type=commonjs&exports=single|lamejs',
      },
      {
        test: path.resolve(import.meta.dirname, 'lib.js'),
        resourceQuery: /object/,
        use: {
          loader: 'exports-loader',
          options: {
            type: 'commonjs',
            exports: 'single|lamejs',
          },
        },
      },
    ],
  },
};
