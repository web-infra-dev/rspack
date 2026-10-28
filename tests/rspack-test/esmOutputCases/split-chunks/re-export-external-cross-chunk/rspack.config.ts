import { defineConfig } from '@rspack/cli';

export default defineConfig({
  externals: {
    fs: 'module fs',
    path: 'module path',
  },
  optimization: {
    splitChunks: {
      cacheGroups: {
        wrapper: {
          test: /wrapper\.js$/,
          name: 'wrapper-chunk',
        },
      },
    },
  },
});
