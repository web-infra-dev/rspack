/** @type {import("@rspack/core").Configuration} */
module.exports = {
  externals: {
    'node:fs': 'node-commonjs node:fs',
    'node:path': 'node-commonjs node:path',
  },
  target: 'web',
  node: false,
  module: {
    rules: [
      {
        test: /\.css$/,
        type: 'css/auto',
        use: 'builtin:lightningcss-loader',
      },
    ],
  },
};
