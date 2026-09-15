/** @type {import("@rspack/core").Configuration} */
module.exports = {
  externals: {
    fs: 'node-commonjs fs',
  },
  target: ['async-node', 'browserslist:node > 16'],
  module: {
    rules: [
      {
        test: /\.js$/,
        use: {
          loader: 'builtin:swc-loader',
          options: {
            env: {
              targets: 'node >= 4',
            },
          },
        },
      },
    ],
  },
};
