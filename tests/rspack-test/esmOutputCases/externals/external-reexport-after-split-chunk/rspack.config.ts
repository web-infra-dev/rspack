import { defineConfig } from '@rspack/cli';

export default defineConfig({
  target: 'node',
  externalsType: 'module',
  externals: {
    './ext.mjs': './ext.mjs',
    './side.mjs': './side.mjs',
  },
  optimization: {
    minimize: false,
    splitChunks: {
      chunks: 'all',
      cacheGroups: {
        setup: {
          test: /setup\.mjs$/,
          name: 'setup',
          enforce: true,
        },
      },
    },
  },
});
