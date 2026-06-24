/** @type {import("@rspack/core").Configuration} */
module.exports = {
  externals: {
    fs: 'node-commonjs fs',
    path: 'node-commonjs path',
  },
  target: 'web',
  node: false,
  module: {
    generator: {
      'css/auto': {
        exportsOnly: false,
      },
    },
    rules: [
      {
        test: /\.png$/,
        type: 'asset',
      },
      {
        test: /\.css/,
        type: 'css/auto',
      },
    ],
  },
};
