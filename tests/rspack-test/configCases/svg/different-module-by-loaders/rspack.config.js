/** @type {import("@rspack/core").Configuration} */
module.exports = {
  externals: {
    fs: 'node-commonjs fs',
    path: 'node-commonjs path',
  },
  target: 'web',
  node: {
    __dirname: false,
    __filename: false,
  },
  module: {
    generator: {
      'css/auto': {
        exportsOnly: false,
      },
    },
    rules: [
      {
        test: /\.svg$/i,
        issuer: { not: [/\.css$/] },
        use: [{ loader: 'file-loader', options: { name: '[name].[ext]' } }],
        type: 'javascript/auto',
      },
      {
        test: /\.svg$/,
        issuer: { not: [/\.js$/] },
        type: 'asset/inline',
      },
      {
        test: /\.css/,
        type: 'css/auto',
      },
    ],
  },
};
