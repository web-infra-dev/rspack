import { defineConfig } from '@rspack/cli';

export default defineConfig({
  mode: 'development',
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
    runtimeChunk: true,
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
});
