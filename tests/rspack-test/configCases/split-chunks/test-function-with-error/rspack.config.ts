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
        splitLib2: {
          test() {
            throw new Error('TEST_FUNCTION_WITH_ERROR');
          },
        },
      },
    },
  },
});
