/** @type {import("@rspack/core").Configuration} */
export default {
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
