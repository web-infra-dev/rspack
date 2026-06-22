const { TsCheckerRspackPlugin } = require('ts-checker-rspack-plugin');

module.exports = {
  mode: 'development',
  entry: {
    output: './index.ts',
  },
  module: {
    rules: [
      {
        test: /\.tsx?$/,
        loader: 'builtin:swc-loader',
        options: {
          detectSyntax: 'auto',
        },
      },
    ],
  },
  resolve: {
    extensions: ['.ts', '.js', '.json'],
  },
  plugins: [new TsCheckerRspackPlugin({ async: false })],
};
