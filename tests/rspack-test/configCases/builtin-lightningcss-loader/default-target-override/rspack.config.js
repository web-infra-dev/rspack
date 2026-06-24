/** @type {import("@rspack/core").Configuration} */
module.exports = {
  externals: {
    fs: 'node-commonjs fs',
    path: 'node-commonjs path',
  },
  target: ['web', 'browserslist:chrome > 95'],
  module: {
    rules: [
      {
        test: /\.css$/,
        type: 'css/auto',
        use: {
          loader: 'builtin:lightningcss-loader',
          options: {
            targets: 'safari >= 4',
          },
        },
      },
    ],
  },
  node: {
    __dirname: false,
  },
};
