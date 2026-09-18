/** @type {import("@rspack/core").Configuration} */
export default {
  externals: {
    path: 'node-commonjs path',
  },
  target: 'web',
  mode: 'development',
  module: {
    rules: [
      {
        test: /\.css$/i,
        type: 'css',
      },
    ],
  },
};
