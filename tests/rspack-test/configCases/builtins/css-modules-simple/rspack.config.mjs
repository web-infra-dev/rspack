/** @type {import("@rspack/core").Configuration} */
export default {
  externals: {
    path: 'node-commonjs path',
  },
  module: {
    rules: [
      {
        test: /\.module\.css$/,
        type: 'css/module',
      },
    ],
  },
};
