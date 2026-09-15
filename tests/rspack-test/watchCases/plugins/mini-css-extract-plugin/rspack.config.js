var { CssExtractRspackPlugin: MCEP } = require('@rspack/core');

/** @type {import("@rspack/core").Configuration} */
module.exports = {
  externals: {
    fs: 'node-commonjs fs',
    path: 'node-commonjs path',
  },
  module: {
    rules: [
      {
        test: /\.css$/,
        use: [MCEP.loader, 'css-loader'],
        type: 'javascript/auto',
      },
    ],
  },
  output: {
    publicPath: '',
  },
  target: 'web',
  node: {
    __dirname: false,
  },
  plugins: [new MCEP()],
};
