import { EntryPlugin } from '@rspack/core';

/** @type {import("@rspack/core").Configuration} */
export default () => ({
  devtool: false,
  mode: 'development',
  entry: {
    main: {
      import: './index.js',
      dependOn: 'shared',
    },
    shared: './common.js',
  },
  output: {
    module: true,
    filename: '[name].mjs',
    library: {
      type: 'module',
    },
  },
  target: ['web', 'es2020'],
  optimization: {
    minimize: false,
    runtimeChunk: false,
    splitChunks: {
      cacheGroups: {
        separate: {
          test: /separate/,
          chunks: 'all',
          filename: 'separate.mjs',
          enforce: true,
        },
      },
    },
  },
  plugins: [new EntryPlugin(import.meta.dirname, './separate.js', 'main')],
});
