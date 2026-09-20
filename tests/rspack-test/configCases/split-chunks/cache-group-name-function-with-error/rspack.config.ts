import { defineConfig } from '@rspack/cli';

export default defineConfig({
  entry: {
    main: './index',
  },
  output: {
    filename: '[name].js',
  },
  target: 'async-node',
  optimization: {
    splitChunks: {
      chunks: 'all',
      minSize: 0,
      cacheGroups: {
        foo: {
          test: /\.js/,
          name(_module, _chunks) {
            throw new Error('CACHE_GROUP_NAME_FUNCTION_WITH_ERROR');
          },
        },
      },
    },
  },
});
