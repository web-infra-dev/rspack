import { DefinePlugin } from '@rspack/core';
export default {
  target: 'node',
  entry: { main: './index.js', helper: './helper.ts' },
  output: { filename: '[name].js', library: { type: 'commonjs2' } },
  optimization: { minimize: false },
  resolve: { extensions: ['.ts', '.js'] },
  module: {
    rules: [
      {
        test: /\.ts$/,
        loader: 'builtin:swc-loader',
        options: { jsc: { parser: { syntax: 'typescript' } } },
      },
    ],
  },
  plugins: [new DefinePlugin({ IS_BROWSER: 'false' })],
};
