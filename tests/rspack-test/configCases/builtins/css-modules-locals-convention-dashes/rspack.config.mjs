/** @type {import("@rspack/core").Configuration} */
export default {
  externals: {
    path: 'node-commonjs path',
  },
  module: {
    rules: [
      {
        test: /\.css$/,
        type: 'css/module',
        generator: {
          exportsConvention: 'dashes',
        },
      },
    ],
  },
};
