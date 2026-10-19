import { rspack } from '@rspack/core';

/** @type { import('@rspack/core').RspackOptions } */
export default {
  context: import.meta.dirname,
  entry: './src/index.js',
  mode: 'development',
  stats: 'none',
  devtool: false,
  module: {
    // Keep the reexport target unmarked, as in a mixed barrel whose leaves
    // cannot be skipped as side-effect-free imports.
    rules: [{ test: /default-barrel\.js$/, sideEffects: false }],
  },
  optimization: {
    usedExports: false,
    sideEffects: 'flag',
  },
  plugins: [new rspack.HtmlRspackPlugin()],
  lazyCompilation: true,
  devServer: { hot: true },
};
