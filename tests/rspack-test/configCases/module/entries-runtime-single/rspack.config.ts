import { defineConfig } from '@rspack/cli';
import { EntryPlugin } from '@rspack/core';

export default defineConfig(() => ({
  devtool: false,
  mode: 'development',
  entry: {
    main: {
      import: './index.js',
    },
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
    runtimeChunk: 'single',
    splitChunks: {
      cacheGroups: {
        separate: {
          test: /separate/,
          chunks: 'all',
          filename: 'separate.mjs',
          enforce: true,
        },
        common: {
          test: /common/,
          chunks: 'all',
          filename: 'common.mjs',
          enforce: true,
        },
      },
    },
  },
  plugins: [new EntryPlugin(import.meta.dirname, './separate.js', 'main')],
}));
