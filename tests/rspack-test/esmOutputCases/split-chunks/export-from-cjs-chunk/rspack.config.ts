import { defineConfig } from '@rspack/cli';

export default defineConfig({
  entry: {
    main: './index.js',
  },
  optimization: {
    sideEffects: true,
    splitChunks: {
      cacheGroups: {
        test: {
          test: /cjs\.js$/,
          name: 'cjs-chunk',
        },
      },
    },
  },
});
