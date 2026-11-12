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
          chunks() {
            throw new Error('CHUNKS_FUNCTION_WITH_ERROR');
          },
          test: /\.js/,
        },
      },
    },
  },
});
