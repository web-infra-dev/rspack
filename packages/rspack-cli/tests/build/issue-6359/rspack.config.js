const path = require('path');
module.exports = /** @type {import('@rspack/cli').Configuration} */ {
  mode: 'production',
  entry: './entry.js',
  output: {
    clean: true,
    path: path.resolve(__dirname, 'dist'),
  },
  plugins: [
    {
      apply(compiler) {
        new compiler.webpack.DefinePlugin({
          DEFINE_ME: JSON.stringify(
            `WEBPACK_SERVE=${process.env.WEBPACK_SERVE ?? '<EMPTY>'}`,
          ),
        }).apply(compiler);
      },
    },
  ],
  devServer: {
    devMiddleware: {
      writeToDisk: true,
    },
  },
};
