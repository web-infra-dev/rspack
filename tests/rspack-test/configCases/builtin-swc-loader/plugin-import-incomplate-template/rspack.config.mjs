/** @type {import("@rspack/core").Configuration} */
export default {
  module: {
    rules: [
      {
        test: /\.js$/,
        loader: 'builtin:swc-loader',
        options: {
          transformImport: [
            {
              libraryName: './lib',
              customName: './lib/{{ member }',
            },
          ],
        },
      },
    ],
  },
};
