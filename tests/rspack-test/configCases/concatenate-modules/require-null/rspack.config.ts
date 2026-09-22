import { defineConfig } from '@rspack/cli';

export default defineConfig({
  mode: 'production',
  entry: './index.js',
  output: {
    filename: '[name].js',
  },
  target: 'node',
  optimization: {
    providedExports: true,
    usedExports: true,
    concatenateModules: true,
    sideEffects: true,
    minimize: false,
    splitChunks: {
      cacheGroups: {
        lib: {
          name: 'lib',
          chunks: 'all',
          test: /[\\/]lib[\\/]/,
          minSize: 0,
        },
      },
    },
  },
});
