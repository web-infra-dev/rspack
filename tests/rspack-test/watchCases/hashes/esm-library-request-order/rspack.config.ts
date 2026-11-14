import { defineConfig } from '@rspack/cli';

export default defineConfig({
  mode: 'production',
  target: 'node',
  cache: true,
  incremental: true,
  entry: './entry.mjs',
  externalsType: 'module',
  externals: ['./setup.mjs', './value.mjs'],
  output: {
    module: true,
    library: { type: 'modern-module' },
    filename: '[name].[chunkhash:8].mjs',
    chunkFilename: '[name].[chunkhash:8].mjs',
  },
  optimization: {
    minimize: false,
    splitChunks: {
      chunks: 'all',
      cacheGroups: {
        shared: {
          test: /[\\/](barrel|inner)\.mjs$/,
          name: 'shared',
          enforce: true,
        },
      },
    },
  },
});
