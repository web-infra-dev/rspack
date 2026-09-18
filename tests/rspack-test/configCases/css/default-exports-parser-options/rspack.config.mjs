/** @type {import("@rspack/core").Configuration} */
export default {
  target: 'node',
  mode: 'development',
  devtool: false,
  module: {
    rules: [
      {
        test: /\.css/,
        parser: {
          namedExports: false,
        },
        type: 'css/module',
      },
    ],
  },
};
