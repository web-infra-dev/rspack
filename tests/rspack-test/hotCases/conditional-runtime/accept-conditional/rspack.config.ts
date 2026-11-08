import { defineConfig } from '@rspack/cli';

export default defineConfig({
  optimization: {
    sideEffects: true,
    usedExports: true,
    innerGraph: true,
    splitChunks: {
      cacheGroups: {
        forceMerge: {
          test: /shared/,
          enforce: true,
          name: 'shared',
          chunks: 'all',
        },
      },
    },
  },
  module: {
    rules: [
      {
        test: /dep/,
        sideEffects: false,
      },
    ],
  },
});
